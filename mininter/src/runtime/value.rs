use crate::compiler::Chunk;
use crate::error::RuntimeError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// A compiled function.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: usize,         // minimum required args
    pub max_arity: usize,     // total params (including optional)
    pub defaults: Vec<Value>, // defaults for optional params
    pub chunk: Chunk,
}

/// A native (Rust) function callable from scripts.
#[derive(Clone)]
pub struct NativeFunction {
    pub name: String,
    pub func: Rc<dyn Fn(&[Value]) -> Result<Value, RuntimeError>>,
    pub max_args: usize,
}

impl fmt::Debug for NativeFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native {}>", self.name)
    }
}

/// Runtime value type for MiniScript.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Number(f64),
    Bool(bool),
    String(Rc<String>),
    List(Rc<RefCell<Vec<Value>>>),
    Map(Rc<RefCell<HashMap<String, Value>>>),
    Function(Rc<Function>),
    NativeFunction(Rc<NativeFunction>),
    /// A method bound to its receiver — created by dot-access on prototypes.
    /// Calling it invokes `func` with `self_val` prepended to args.
    BoundMethod {
        func: Box<Value>,
        self_val: Box<Value>,
    },
}

impl Value {
    pub fn type_name(&self) -> &str {
        match self {
            Value::Null => "null",
            Value::Number(_) => "number",
            Value::Bool(_) => "bool",
            Value::String(_) => "string",
            Value::List(_) => "list",
            Value::Map(_) => "map",
            Value::Function(_) | Value::NativeFunction(_) => "function",
            Value::BoundMethod { .. } => "function",
        }
    }

    /// MiniScript truthiness: null, false, 0, and "" are falsy.
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            _ => true,
        }
    }

    pub fn is_callable(&self) -> bool {
        matches!(
            self,
            Value::Function(_) | Value::NativeFunction(_) | Value::BoundMethod { .. }
        )
    }

    pub fn as_number(&self) -> Result<f64, RuntimeError> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(RuntimeError::TypeError {
                expected: "number".into(),
                got: self.type_name().into(),
            }),
        }
    }

    pub fn as_string(&self) -> Result<&str, RuntimeError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(RuntimeError::TypeError {
                expected: "string".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Convert any value to a display string.
    pub fn to_display_string(&self) -> String {
        match self {
            Value::Null => "null".into(),
            Value::Number(n) => {
                if *n == (*n as i64) as f64 && n.is_finite() {
                    format!("{}", *n as i64)
                } else {
                    format!("{n}")
                }
            }
            Value::Bool(b) => if *b { "1" } else { "0" }.into(),
            Value::String(s) => s.as_ref().clone(),
            Value::List(items) => {
                let items = items.borrow();
                let parts: Vec<String> = items.iter().map(|v| v.to_display_string()).collect();
                format!("[{}]", parts.join(", "))
            }
            Value::Map(map) => {
                let map = map.borrow();
                let parts: Vec<String> = map
                    .iter()
                    .filter(|(k, _)| !k.starts_with("__"))
                    .map(|(k, v)| format!("{k}: {}", v.to_display_string()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
            Value::Function(f) => format!("<function {}>", f.name),
            Value::NativeFunction(f) => format!("<native {}>", f.name),
            Value::BoundMethod { func, .. } => match func.as_ref() {
                Value::NativeFunction(f) => format!("<bound {}>", f.name),
                Value::Function(f) => format!("<bound {}>", f.name),
                _ => "<bound method>".into(),
            },
        }
    }

    pub fn equals(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::NativeFunction(a), Value::NativeFunction(b)) => Rc::ptr_eq(a, b),
            (Value::Function(a), Value::Function(b)) => Rc::ptr_eq(a, b),
            (Value::List(a), Value::List(b)) => Rc::ptr_eq(a, b),
            (Value::Map(a), Value::Map(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_display_string())
    }
}

// ── Conversion traits for the #[api] macro ──────────────────────

pub trait FromValue: Sized {
    fn from_value(val: &Value) -> Result<Self, RuntimeError>;
}

impl FromValue for f64 {
    fn from_value(val: &Value) -> Result<Self, RuntimeError> {
        val.as_number()
    }
}

impl FromValue for bool {
    fn from_value(val: &Value) -> Result<Self, RuntimeError> {
        match val {
            Value::Bool(b) => Ok(*b),
            _ => Err(RuntimeError::TypeError {
                expected: "bool".into(),
                got: val.type_name().into(),
            }),
        }
    }
}

impl FromValue for String {
    fn from_value(val: &Value) -> Result<Self, RuntimeError> {
        match val {
            Value::String(s) => Ok(s.as_ref().clone()),
            _ => Err(RuntimeError::TypeError {
                expected: "string".into(),
                got: val.type_name().into(),
            }),
        }
    }
}

impl FromValue for Value {
    fn from_value(val: &Value) -> Result<Self, RuntimeError> {
        Ok(val.clone())
    }
}

pub trait IntoValue {
    fn into_value(self) -> Value;
}

impl IntoValue for f64 {
    fn into_value(self) -> Value {
        Value::Number(self)
    }
}

impl IntoValue for bool {
    fn into_value(self) -> Value {
        Value::Bool(self)
    }
}

impl IntoValue for String {
    fn into_value(self) -> Value {
        Value::String(Rc::new(self))
    }
}

impl IntoValue for () {
    fn into_value(self) -> Value {
        Value::Null
    }
}

impl IntoValue for Value {
    fn into_value(self) -> Value {
        self
    }
}
