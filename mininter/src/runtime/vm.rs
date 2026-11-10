#[cfg(feature = "rand")]
use rand::rngs::StdRng;

use crate::compiler::Op;
use crate::error::RuntimeError;
use crate::runtime::intrinsics;
use crate::runtime::value::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Registration info returned by the `#[api]` proc macro.
pub struct ApiFunction {
    pub name: &'static str,
    pub func: fn(&[Value]) -> Result<Value, RuntimeError>,
}

struct CallFrame {
    function: Rc<Function>,
    ip: usize,
    stack_offset: usize,
    is_method: bool,
}

pub struct Vm {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
    /// Stack of `self` values for method calls.
    self_stack: Vec<Value>,
    /// Prototype maps for built-in types. Shared with globals["string"] etc.
    pub string_proto: Rc<RefCell<HashMap<String, Value>>>,
    pub list_proto: Rc<RefCell<HashMap<String, Value>>>,
    pub map_proto: Rc<RefCell<HashMap<String, Value>>>,
    #[cfg(feature = "rand")]
    rng: Rc<RefCell<rand::rngs::StdRng>>,
}

impl Vm {
    pub fn new() -> Self {
        let string_proto = Rc::new(RefCell::new(intrinsics::make_string_proto()));
        let list_proto = Rc::new(RefCell::new(intrinsics::make_list_proto()));
        let map_proto = Rc::new(RefCell::new(intrinsics::make_map_proto()));

        let mut globals = HashMap::new();
        globals.insert("string".to_string(), Value::Map(string_proto.clone()));
        globals.insert("list".to_string(), Value::Map(list_proto.clone()));
        globals.insert("map".to_string(), Value::Map(map_proto.clone()));

        Vm {
            stack: Vec::with_capacity(256),
            globals,
            frames: Vec::new(),
            self_stack: Vec::new(),
            string_proto,
            list_proto,
            map_proto,
            #[cfg(feature = "rand")]
            rng: Rc::new(RefCell::new(rand::make_rng())),
        }
    }

    #[cfg(feature = "rand")]
    pub fn rng(&mut self) -> Rc<RefCell<StdRng>> {
        Rc::clone(&self.rng)
    }

    /// Register a native function by name.
    pub fn register_native(
        &mut self,
        name: &str,
        func: impl Fn(&[Value]) -> Result<Value, RuntimeError> + 'static,
        max_args: usize,
    ) {
        self.globals.insert(
            name.to_string(),
            Value::NativeFunction(Rc::new(NativeFunction {
                name: name.to_string(),
                func: Rc::new(func),
                max_args,
            })),
        );
    }

    /// Register a function from the `#[api]` macro.
    pub fn register(&mut self, api: ApiFunction) {
        let name = api.name.to_string();
        let func = api.func;
        self.globals.insert(
            name.clone(),
            Value::NativeFunction(Rc::new(NativeFunction {
                name,
                func: Rc::new(move |args| func(args)),
                max_args: usize::MAX,
            })),
        );
    }

    /// Run a compiled top-level function.
    pub fn run(&mut self, function: Function) -> Result<Value, RuntimeError> {
        let func = Rc::new(function);
        self.stack.push(Value::Null); // slot 0 sentinel
        self.frames.push(CallFrame {
            function: func,
            ip: 0,
            stack_offset: 0,
            is_method: false,
        });
        self.execute()
    }

    // ── Main dispatch loop ───────────────────────────────────────

    fn execute(&mut self) -> Result<Value, RuntimeError> {
        loop {
            let op_byte = self.read_byte();
            let op = Op::from_byte(op_byte)
                .ok_or_else(|| RuntimeError::Custom(format!("unknown opcode {op_byte}")))?;

            match op {
                Op::Constant => {
                    let idx = self.read_byte() as usize;
                    let val = self.current_constant(idx).clone();
                    self.push(val);
                }
                Op::Null => self.push(Value::Null),
                Op::True => self.push(Value::Bool(true)),
                Op::False => self.push(Value::Bool(false)),
                Op::Pop => {
                    self.pop()?;
                }

                Op::GetGlobal => {
                    let idx = self.read_byte() as usize;
                    let name = self.current_constant(idx).as_string()?.to_string();
                    let val = self
                        .globals
                        .get(&name)
                        .cloned()
                        .ok_or_else(|| RuntimeError::UndefinedVariable(name))?;
                    self.push(val);
                }
                Op::SetGlobal => {
                    let idx = self.read_byte() as usize;
                    let name = self.current_constant(idx).as_string()?.to_string();
                    let val = self.peek(0)?.clone();
                    self.globals.insert(name, val);
                }
                Op::GetLocal => {
                    let slot = self.read_byte() as usize;
                    let offset = self.current_frame().stack_offset;
                    let val = self.stack[offset + slot].clone();
                    self.push(val);
                }
                Op::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let offset = self.current_frame().stack_offset;
                    let val = self.peek(0)?.clone();
                    self.stack[offset + slot] = val;
                }

                // Arithmetic
                Op::Add => self.binary_add()?,
                Op::Sub => self.binary_sub()?,
                Op::Mul => self.binary_mul()?,
                Op::Div => self.binary_div()?,
                Op::Mod => self.binary_mod()?,
                Op::Power => self.binary_power()?,
                Op::Negate => {
                    let val = self.pop()?;
                    self.push(Value::Number(-val.as_number()?));
                }

                // Logic & comparison
                Op::Not => {
                    let val = self.pop()?;
                    self.push(Value::Bool(!val.is_truthy()));
                }
                Op::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a.equals(&b)));
                }
                Op::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(!a.equals(&b)));
                }
                Op::Less => {
                    self.compare(|a, b| a < b)?;
                }
                Op::Greater => {
                    self.compare(|a, b| a > b)?;
                }
                Op::LessEq => {
                    self.compare(|a, b| a <= b)?;
                }
                Op::GreaterEq => {
                    self.compare(|a, b| a >= b)?;
                }
                Op::Isa => {
                    let parent = self.pop()?;
                    let child = self.pop()?;
                    self.push(Value::Bool(self.check_isa(&child, &parent)));
                }

                // Control flow
                Op::Jump => {
                    let offset = self.read_u16();
                    self.current_frame_mut().ip += offset;
                }
                Op::JumpIfFalse => {
                    let offset = self.read_u16();
                    if !self.peek(0)?.is_truthy() {
                        self.current_frame_mut().ip += offset;
                    }
                }
                Op::Loop => {
                    let offset = self.read_u16();
                    self.current_frame_mut().ip -= offset;
                }

                // Functions
                Op::Call => {
                    let arg_count = self.read_byte() as usize;
                    self.call_value(arg_count)?;
                }
                Op::Return => {
                    let result = self.pop()?;
                    let frame = self.frames.pop().unwrap();
                    if frame.is_method {
                        self.self_stack.pop();
                    }
                    self.stack.truncate(frame.stack_offset);
                    if self.frames.is_empty() {
                        return Ok(result);
                    }
                    self.push(result);
                }

                // Collections
                Op::NewList => {
                    let count = self.read_byte() as usize;
                    let start = self.stack.len() - count;
                    let items: Vec<Value> = self.stack.drain(start..).collect();
                    self.push(Value::List(Rc::new(RefCell::new(items))));
                }
                Op::NewMap => {
                    let pair_count = self.read_byte() as usize;
                    let start = self.stack.len() - pair_count * 2;
                    let entries: Vec<Value> = self.stack.drain(start..).collect();
                    let mut map = HashMap::new();
                    for chunk in entries.chunks(2) {
                        let key = chunk[0].to_display_string();
                        let val = chunk[1].clone();
                        map.insert(key, val);
                    }
                    self.push(Value::Map(Rc::new(RefCell::new(map))));
                }
                Op::GetIndex => {
                    let index = self.pop()?;
                    let object = self.pop()?;
                    self.push(self.index_get(&object, &index)?);
                }
                Op::SetIndex => {
                    let index = self.pop()?;
                    let object = self.pop()?;
                    let value = self.pop()?;
                    self.index_set(&object, &index, &value)?;
                }

                // Objects & members
                Op::GetField => {
                    let idx = self.read_byte() as usize;
                    let field = self.current_constant(idx).as_string()?.to_string();
                    let object = self.pop()?;
                    let result = self.get_field(&object, &field)?;
                    self.push(result);
                }
                Op::SetField => {
                    let idx = self.read_byte() as usize;
                    let field = self.current_constant(idx).as_string()?.to_string();
                    let object = self.pop()?;
                    let value = self.pop()?;
                    match &object {
                        Value::Map(map) => {
                            map.borrow_mut().insert(field, value);
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "map".into(),
                                got: object.type_name().into(),
                            });
                        }
                    }
                }
                Op::CallMethod => {
                    let idx = self.read_byte() as usize;
                    let arg_count = self.read_byte() as usize;
                    let field = self.current_constant(idx).as_string()?.to_string();
                    self.call_method(&field, arg_count)?;
                }
                Op::NewObject => {
                    let parent = self.pop()?;
                    match &parent {
                        Value::Map(_) => {
                            let mut new_map = HashMap::new();
                            new_map.insert("__isa".to_string(), parent);
                            self.push(Value::Map(Rc::new(RefCell::new(new_map))));
                        }
                        _ => {
                            return Err(RuntimeError::TypeError {
                                expected: "map".into(),
                                got: parent.type_name().into(),
                            });
                        }
                    }
                }
                Op::GetSelf => {
                    let self_val = self.self_stack.last().cloned().unwrap_or(Value::Null);
                    self.push(self_val);
                }
                Op::ResolveCall => {
                    // Auto-invoke: if top of stack is callable, call with 0 args
                    let top = self.peek(0)?.clone();
                    if top.is_callable() {
                        self.call_value(0)?;
                    }
                }
                Op::Module => {
                    let fn_idx = self.read_byte() as usize;
                    let name_idx = self.read_byte() as usize;
                    let module_fn = match self.current_constant(fn_idx) {
                        Value::Function(f) => f.clone(),
                        _ => return Err(RuntimeError::Custom("invalid import".into())),
                    };
                    let mod_name = self.current_constant(name_idx).as_string()?.to_string();

                    // Save state
                    let saved_stack = std::mem::take(&mut self.stack);
                    let saved_frames = std::mem::take(&mut self.frames);
                    let saved_self_stack = std::mem::take(&mut self.self_stack);
                    let saved_globals = self.globals.clone();

                    // Run module (globals has builtins via clone)
                    self.run((*module_fn).clone())?;

                    // Collect new or changed
                    let mut module_map = HashMap::new();
                    for (k, v) in &self.globals {
                        match saved_globals.get(k) {
                            Some(old) if old.equals(v) => {}
                            _ => {
                                module_map.insert(k.clone(), v.clone());
                            }
                        }
                    }

                    // Full restore
                    self.globals = saved_globals;
                    self.stack = saved_stack;
                    self.frames = saved_frames;
                    self.self_stack = saved_self_stack;

                    // Bind module
                    self.globals
                        .insert(mod_name, Value::Map(Rc::new(RefCell::new(module_map))));
                }
            }
        }
    }

    // ── Field access (prototype-based) ──────────────────────────

    fn get_field(&self, object: &Value, field: &str) -> Result<Value, RuntimeError> {
        match object {
            Value::String(_) => {
                let proto = self.string_proto.borrow();
                if let Some(val) = proto.get(field) {
                    return Ok(self.maybe_bind(val, object));
                }
                Err(RuntimeError::UndefinedField(field.into()))
            }
            Value::List(_) => {
                let proto = self.list_proto.borrow();
                if let Some(val) = proto.get(field) {
                    return Ok(self.maybe_bind(val, object));
                }
                Err(RuntimeError::UndefinedField(field.into()))
            }
            Value::Map(map) => {
                // 1. Check the map itself
                if let Some(val) = map.borrow().get(field).cloned() {
                    return Ok(self.maybe_bind(&val, object));
                }
                // 2. Walk __isa chain
                if let Some(val) = intrinsics::isa_chain_lookup_skip_self(&map.borrow(), field) {
                    return Ok(self.maybe_bind(&val, object));
                }
                // 3. Fall back to map prototype
                let proto = self.map_proto.borrow();
                if let Some(val) = proto.get(field) {
                    return Ok(self.maybe_bind(val, object));
                }
                Err(RuntimeError::UndefinedField(field.into()))
            }
            _ => Err(RuntimeError::TypeError {
                expected: "string, list, or map".into(),
                got: object.type_name().into(),
            }),
        }
    }

    /// If `val` is callable, wrap it in a BoundMethod with the given receiver.
    /// Otherwise return the value as-is.
    fn maybe_bind(&self, val: &Value, receiver: &Value) -> Value {
        if val.is_callable() {
            Value::BoundMethod {
                func: Box::new(val.clone()),
                self_val: Box::new(receiver.clone()),
            }
        } else {
            val.clone()
        }
    }

    // ── Call helpers ─────────────────────────────────────────────

    fn call_value(&mut self, arg_count: usize) -> Result<(), RuntimeError> {
        let callee_idx = self
            .stack
            .len()
            .checked_sub(1 + arg_count)
            .ok_or(RuntimeError::StackUnderflow)?;
        let callee = self.stack[callee_idx].clone();

        match callee {
            Value::Function(func) => {
                self.check_arity(&func, arg_count)?;
                // Fill defaults for missing optional args
                for i in arg_count..func.max_arity {
                    self.push(func.defaults[i - func.arity].clone());
                }
                self.frames.push(CallFrame {
                    function: func,
                    ip: 0,
                    stack_offset: callee_idx,
                    is_method: false,
                });
                Ok(())
            }
            Value::NativeFunction(native) => {
                let args_start = callee_idx + 1;
                let args: Vec<Value> = self.stack[args_start..].to_vec();
                if args.len() > native.max_args {
                    return Err(RuntimeError::WrongArgCount {
                        expected: native.max_args,
                        got: args.len(),
                    });
                }
                let result = (native.func)(&args)?;
                self.stack.truncate(callee_idx);
                self.push(result);
                Ok(())
            }
            Value::BoundMethod { func, self_val } => {
                self.call_bound_method(*func, *self_val, arg_count, callee_idx)
            }
            _ => Err(RuntimeError::NotCallable(callee.type_name().into())),
        }
    }

    fn call_bound_method(
        &mut self,
        func: Value,
        self_val: Value,
        arg_count: usize,
        callee_idx: usize,
    ) -> Result<(), RuntimeError> {
        match func {
            Value::NativeFunction(native) => {
                // Collect explicit args, prepend self
                let args: Vec<Value> = self.stack[callee_idx + 1..].to_vec();
                if args.len() + 1 > native.max_args {
                    // +1 because self counts
                    return Err(RuntimeError::WrongArgCount {
                        expected: native.max_args - 1,
                        got: args.len(),
                    });
                }
                self.stack.truncate(callee_idx);
                let mut method_args = vec![self_val];
                method_args.extend(args);
                let result = (native.func)(&method_args)?;
                self.push(result);
                Ok(())
            }
            Value::Function(user_func) => {
                self.check_arity(&user_func, arg_count)?;
                // Fill defaults
                for i in arg_count..user_func.max_arity {
                    self.push(user_func.defaults[i - user_func.arity].clone());
                }
                self.self_stack.push(self_val);
                self.stack[callee_idx] = Value::Function(user_func.clone());
                self.frames.push(CallFrame {
                    function: user_func,
                    ip: 0,
                    stack_offset: callee_idx,
                    is_method: true,
                });
                Ok(())
            }
            _ => Err(RuntimeError::NotCallable(
                "bound method with non-callable inner".into(),
            )),
        }
    }

    fn call_method(&mut self, field: &str, arg_count: usize) -> Result<(), RuntimeError> {
        // Stack: [..., object, arg0, arg1, ..., argN-1]
        let obj_idx = self.stack.len() - arg_count - 1;
        let object = self.stack[obj_idx].clone();

        let method = self.find_method(&object, field)?;

        match method {
            Value::NativeFunction(native) => {
                let args: Vec<Value> = self.stack[obj_idx + 1..].to_vec();
                if args.len() + 1 > native.max_args {
                    return Err(RuntimeError::WrongArgCount {
                        expected: native.max_args - 1,
                        got: args.len(),
                    });
                }
                self.stack.truncate(obj_idx);
                let mut method_args = vec![object];
                method_args.extend(args);
                let result = (native.func)(&method_args)?;
                self.push(result);
                Ok(())
            }
            Value::Function(user_func) => {
                self.check_arity(&user_func, arg_count)?;
                for i in arg_count..user_func.max_arity {
                    self.push(user_func.defaults[i - user_func.arity].clone());
                }
                self.self_stack.push(object);
                self.stack[obj_idx] = Value::Function(user_func.clone());
                self.frames.push(CallFrame {
                    function: user_func,
                    ip: 0,
                    stack_offset: obj_idx,
                    is_method: true,
                });
                Ok(())
            }
            _ => Err(RuntimeError::NotCallable(method.type_name().into())),
        }
    }

    /// Look up a method by name for the given object, checking prototypes.
    fn find_method(&self, object: &Value, name: &str) -> Result<Value, RuntimeError> {
        match object {
            Value::String(_) => self
                .string_proto
                .borrow()
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::UndefinedField(name.into())),
            Value::List(_) => self
                .list_proto
                .borrow()
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::UndefinedField(name.into())),
            Value::Map(map) => {
                // Check map itself + __isa chain
                if let Some(val) = intrinsics::isa_chain_lookup(&map.borrow(), name) {
                    return Ok(val);
                }
                // Fall back to map proto
                self.map_proto
                    .borrow()
                    .get(name)
                    .cloned()
                    .ok_or_else(|| RuntimeError::UndefinedField(name.into()))
            }
            _ => Err(RuntimeError::TypeError {
                expected: "string, list, or map".into(),
                got: object.type_name().into(),
            }),
        }
    }

    fn check_arity(&self, func: &Function, arg_count: usize) -> Result<(), RuntimeError> {
        if arg_count < func.arity || arg_count > func.max_arity {
            Err(RuntimeError::WrongArgCount {
                expected: func.arity,
                got: arg_count,
            })
        } else {
            Ok(())
        }
    }

    // ── Arithmetic helpers ──────────────────────────────────────

    fn binary_add(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x + y),
            (Value::String(x), Value::String(y)) => Value::String(Rc::new(format!("{x}{y}"))),
            (Value::String(x), _) => {
                Value::String(Rc::new(format!("{x}{}", b.to_display_string())))
            }
            (_, Value::String(y)) => {
                Value::String(Rc::new(format!("{}{y}", a.to_display_string())))
            }
            (Value::List(x), Value::List(y)) => {
                let mut v = x.borrow().clone();
                v.extend(y.borrow().iter().cloned());
                Value::List(Rc::new(RefCell::new(v)))
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "number or string".into(),
                    got: format!("{} + {}", a.type_name(), b.type_name()),
                });
            }
        };
        self.push(result);
        Ok(())
    }

    fn binary_sub(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x - y),
            (Value::String(x), Value::String(y)) => {
                let s = x.as_ref();
                if s.ends_with(y.as_ref()) {
                    Value::String(Rc::new(s[..s.len() - y.len()].to_string()))
                } else {
                    Value::String(Rc::new(s.to_string()))
                }
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "number or string".into(),
                    got: format!("{} - {}", a.type_name(), b.type_name()),
                });
            }
        };
        self.push(result);
        Ok(())
    }

    fn binary_mul(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => Value::Number(x * y),
            (Value::String(s), Value::Number(n)) | (Value::Number(n), Value::String(s)) => {
                let count = (*n as usize).max(0);
                Value::String(Rc::new(s.as_ref().repeat(count)))
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "number or string*number".into(),
                    got: format!("{} * {}", a.type_name(), b.type_name()),
                });
            }
        };
        self.push(result);
        Ok(())
    }

    fn binary_div(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?.as_number()?;
        let a = self.pop()?.as_number()?;
        if b == 0.0 {
            return Err(RuntimeError::DivisionByZero);
        }
        self.push(Value::Number(a / b));
        Ok(())
    }

    fn binary_mod(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?.as_number()?;
        let a = self.pop()?.as_number()?;
        if b == 0.0 {
            return Err(RuntimeError::DivisionByZero);
        }
        self.push(Value::Number(a % b));
        Ok(())
    }

    fn binary_power(&mut self) -> Result<(), RuntimeError> {
        let b = self.pop()?.as_number()?;
        let a = self.pop()?.as_number()?;
        self.push(Value::Number(a.powf(b)));
        Ok(())
    }

    fn compare(&mut self, f: fn(f64, f64) -> bool) -> Result<(), RuntimeError> {
        let b = self.pop()?;
        let a = self.pop()?;
        let result = match (&a, &b) {
            (Value::Number(x), Value::Number(y)) => f(*x, *y),
            (Value::String(x), Value::String(y)) => {
                let ord = x.cmp(y);
                f(ord as i32 as f64, 0.0)
            }
            _ => {
                return Err(RuntimeError::TypeError {
                    expected: "comparable types".into(),
                    got: format!("{} vs {}", a.type_name(), b.type_name()),
                });
            }
        };
        self.push(Value::Bool(result));
        Ok(())
    }

    // ── Index access ────────────────────────────────────────────

    fn index_get(&self, object: &Value, index: &Value) -> Result<Value, RuntimeError> {
        match object {
            Value::List(items) => {
                let i = index.as_number()? as i64;
                let items = items.borrow();
                let len = items.len() as i64;
                let idx = if i < 0 { len + i } else { i };
                if idx < 0 || idx >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: i,
                        len: items.len(),
                    });
                }
                Ok(items[idx as usize].clone())
            }
            Value::String(s) => {
                let i = index.as_number()? as i64;
                let len = s.len() as i64;
                let idx = if i < 0 { len + i } else { i };
                if idx < 0 || idx >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: i,
                        len: s.len(),
                    });
                }
                let ch = s.chars().nth(idx as usize).unwrap_or('\0');
                Ok(Value::String(Rc::new(ch.to_string())))
            }
            Value::Map(map) => {
                let key = index.to_display_string();
                map.borrow()
                    .get(&key)
                    .cloned()
                    .ok_or_else(|| RuntimeError::UndefinedField(key))
            }
            _ => Err(RuntimeError::TypeError {
                expected: "list, string, or map".into(),
                got: object.type_name().into(),
            }),
        }
    }

    fn index_set(&self, object: &Value, index: &Value, value: &Value) -> Result<(), RuntimeError> {
        match object {
            Value::List(items) => {
                let i = index.as_number()? as i64;
                let mut items = items.borrow_mut();
                let len = items.len() as i64;
                let idx = if i < 0 { len + i } else { i };
                if idx < 0 || idx >= len {
                    return Err(RuntimeError::IndexOutOfBounds {
                        index: i,
                        len: items.len(),
                    });
                }
                items[idx as usize] = value.clone();
                Ok(())
            }
            Value::Map(map) => {
                let key = index.to_display_string();
                map.borrow_mut().insert(key, value.clone());
                Ok(())
            }
            _ => Err(RuntimeError::TypeError {
                expected: "list or map".into(),
                got: object.type_name().into(),
            }),
        }
    }

    // ── isa check ───────────────────────────────────────────────

    fn check_isa(&self, child: &Value, parent: &Value) -> bool {
        match (child, parent) {
            // isa against prototype maps: `"hello" isa string`
            (Value::String(_), Value::Map(m)) if Rc::ptr_eq(m, &self.string_proto) => true,
            (Value::List(_), Value::Map(m)) if Rc::ptr_eq(m, &self.list_proto) => true,
            (Value::Map(_), Value::Map(m)) if Rc::ptr_eq(m, &self.map_proto) => true,
            (Value::Map(child_map), Value::Map(_)) => {
                let child_map = child_map.borrow();
                if let Some(Value::Map(isa)) = child_map.get("__isa") {
                    if Rc::ptr_eq(
                        isa,
                        match parent {
                            Value::Map(m) => m,
                            _ => unreachable!(),
                        },
                    ) {
                        return true;
                    }
                    return self.check_isa(&Value::Map(isa.clone()), parent);
                }
                false
            }
            // Type checking with type name strings
            (Value::Number(_), Value::String(s)) if s.as_ref() == "number" => true,
            (Value::String(_), Value::String(s)) if s.as_ref() == "string" => true,
            (Value::List(_), Value::String(s)) if s.as_ref() == "list" => true,
            (Value::Map(_), Value::String(s)) if s.as_ref() == "map" => true,
            _ => false,
        }
    }

    // ── Stack & frame helpers ───────────────────────────────────

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    fn peek(&self, distance: usize) -> Result<&Value, RuntimeError> {
        let idx = self
            .stack
            .len()
            .checked_sub(1 + distance)
            .ok_or(RuntimeError::StackUnderflow)?;
        Ok(&self.stack[idx])
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().expect("no call frame")
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        self.frames.last_mut().expect("no call frame")
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().expect("no call frame");
        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_u16(&mut self) -> usize {
        let hi = self.read_byte() as usize;
        let lo = self.read_byte() as usize;
        (hi << 8) | lo
    }

    fn current_constant(&self, idx: usize) -> &Value {
        let frame = self.frames.last().expect("no call frame");
        &frame.function.chunk.constants[idx]
    }
}
