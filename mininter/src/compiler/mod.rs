pub mod chunk;
pub use chunk::{Chunk, Op};

use crate::error::CompileError;
use crate::parser::ast::*;
use crate::runtime::value::{Function, Value};
use std::path::PathBuf;
use std::rc::Rc;

/// Tracks a local variable during compilation.
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: usize,
}

/// Tracks the start and break-jump patches for a loop.
struct LoopCtx {
    start: usize,          // where the loop begins (for Loop instruction)
    breaks: Vec<usize>,    // forward jumps to patch after the loop
    continues: Vec<usize>, // forward jumps to patch at the increment point (for-loops)
    scope_depth: usize,
    /// true for for-loops where continue should jump to increment, not loop start
    is_for_loop: bool,
}

pub struct Compiler {
    chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    loops: Vec<LoopCtx>,
    /// true for the top-level script, false for function bodies
    is_script: bool,
    base_path: std::path::PathBuf,
}

impl Compiler {
    // ── Public API ───────────────────────────────────────────────

    pub fn compile_script(stmts: &[Stmt]) -> Result<Function, CompileError> {
        Self::compile_script_with_mod_path(stmts, std::env::current_dir().unwrap_or_default())
    }

    pub fn compile_script_with_mod_path(
        stmts: &[Stmt],
        base_path: PathBuf,
    ) -> Result<Function, CompileError> {
        let mut c = Compiler {
            chunk: Chunk::new(),
            locals: vec![Local {
                name: String::new(),
                depth: 0,
            }], // slot 0 sentinel
            scope_depth: 0,
            loops: Vec::new(),
            is_script: true,
            base_path,
        };
        for stmt in stmts {
            c.compile_stmt(stmt)?;
        }
        c.emit_op(Op::Null, 0);
        c.emit_op(Op::Return, 0);
        Ok(Function {
            name: "<script>".into(),
            arity: 0,
            max_arity: 0,
            defaults: Vec::new(),
            chunk: c.chunk,
        })
    }

    fn compile_function(
        name: &str,
        params: &[Param],
        body: &[Stmt],
    ) -> Result<Function, CompileError> {
        Self::compile_function_with_mod_path(
            name,
            params,
            body,
            std::env::current_dir().unwrap_or_default(),
        )
    }

    fn compile_function_with_mod_path(
        name: &str,
        params: &[Param],
        body: &[Stmt],
        base_path: PathBuf,
    ) -> Result<Function, CompileError> {
        let mut c = Compiler {
            chunk: Chunk::new(),
            locals: vec![Local {
                name: String::new(),
                depth: 0,
            }], // slot 0 = function itself
            scope_depth: 0,
            loops: Vec::new(),
            is_script: false,
            base_path,
        };
        // Params become locals
        c.begin_scope();
        let mut arity = 0;
        let mut defaults = Vec::new();
        for p in params {
            c.add_local(&p.name)?;
            if p.default.is_some() {
                defaults.push(Self::eval_default(p.default.as_ref().unwrap())?);
            } else {
                arity += 1;
            }
        }
        for stmt in body {
            c.compile_stmt(stmt)?;
        }
        // Implicit return null
        c.emit_op(Op::Null, 0);
        c.emit_op(Op::Return, 0);

        Ok(Function {
            name: name.to_string(),
            arity,
            max_arity: params.len(),
            defaults,
            chunk: c.chunk,
        })
    }

    fn eval_default(expr: &Expr) -> Result<Value, CompileError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::StringLit(s) => Ok(Value::String(Rc::new(s.clone()))),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            _ => Err(CompileError::Custom(
                "default parameter value must be a literal".into(),
            )),
        }
    }

    // ── Statements ───────────────────────────────────────────────

    fn compile_stmt(&mut self, stmt: &Stmt) -> Result<(), CompileError> {
        match stmt {
            Stmt::Expression(expr) => {
                self.compile_expr(expr)?;
                self.emit_op(Op::Pop, 0);
            }
            Stmt::Assignment { target, value } => {
                self.compile_assignment(target, value)?;
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                self.compile_if(branches, else_body)?;
            }
            Stmt::While { condition, body } => {
                self.compile_while(condition, body)?;
            }
            Stmt::For {
                var,
                iterable,
                body,
            } => {
                self.compile_for(var, iterable, body)?;
            }
            Stmt::Return(expr) => {
                match expr {
                    Some(e) => self.compile_expr(e)?,
                    None => self.emit_op(Op::Null, 0),
                }
                self.emit_op(Op::Return, 0);
            }
            Stmt::Break => {
                let loop_ctx = self
                    .loops
                    .last_mut()
                    .ok_or(CompileError::BreakOutsideLoop)?;
                // Pop locals down to loop scope
                let pops = self
                    .locals
                    .iter()
                    .rev()
                    .take_while(|l| l.depth > loop_ctx.scope_depth)
                    .count();
                for _ in 0..pops {
                    self.emit_op(Op::Pop, 0);
                }
                let jump = self.emit_jump(Op::Jump, 0);
                // Can't borrow loop_ctx across emit_jump, re-borrow:
                self.loops.last_mut().unwrap().breaks.push(jump);
            }
            Stmt::Continue => {
                let loop_ctx = self.loops.last().ok_or(CompileError::ContinueOutsideLoop)?;
                let start = loop_ctx.start;
                let depth = loop_ctx.scope_depth;
                let is_for = loop_ctx.is_for_loop;
                // Pop locals down to loop scope
                let pops = self
                    .locals
                    .iter()
                    .rev()
                    .take_while(|l| l.depth > depth)
                    .count();
                for _ in 0..pops {
                    self.emit_op(Op::Pop, 0);
                }
                if is_for {
                    // For-loops: forward jump to the increment point (patched later)
                    let jump = self.emit_jump(Op::Jump, 0);
                    self.loops.last_mut().unwrap().continues.push(jump);
                } else {
                    // While-loops: jump back to condition
                    self.emit_loop(start, 0)?;
                }
            }
            Stmt::Module(name) => {
                let file = self.base_path.join(format!("{name}.ms"));
                let (source, mod_path) = if file.exists() {
                    (
                        std::fs::read_to_string(&file).map_err(|e| {
                            CompileError::Custom(format!("cannot read {}: {e}", file.display()))
                        })?,
                        file,
                    )
                } else {
                    let dir_file = self.base_path.join(name).join("mod.ms");
                    let src = std::fs::read_to_string(&dir_file).map_err(|_| {
                        CompileError::Custom(format!(
                            "module '{}' not found (tried {}.ms and {}/mod.ms)",
                            name, name, name
                        ))
                    })?;
                    (src, dir_file)
                };

                // compile mod
                let tokens = crate::lexer::Lexer::new(&source)
                    .tokenize()
                    .map_err(|e| CompileError::Custom(format!("in module '{name}': {e:?}")))?;
                let stmts = crate::parser::Parser::new(tokens)
                    .parse()
                    .map_err(|e| CompileError::Custom(format!("in module '{name}': {e:?}")))?;
                let mod_base = mod_path.parent().unwrap_or(&self.base_path).to_path_buf();
                let module_fn = Compiler::compile_script_with_mod_path(&stmts, mod_base)?;

                // push compiled function
                let fn_idx = self.chunk.add_constant(Value::Function(Rc::new(module_fn)));
                let name_idx = self.string_constant(name)?;
                self.emit_op(Op::Module, 0);
                self.emit_byte(fn_idx as u8, 0);
                self.emit_byte(name_idx as u8, 0);
            }
        }
        Ok(())
    }

    fn compile_assignment(
        &mut self,
        target: &AssignTarget,
        value: &Expr,
    ) -> Result<(), CompileError> {
        match target {
            AssignTarget::Variable(name) => {
                self.compile_expr(value)?;
                if let Some(slot) = self.resolve_local(name) {
                    self.emit_op(Op::SetLocal, 0);
                    self.emit_byte(slot as u8, 0);
                    self.emit_op(Op::Pop, 0);
                } else if self.is_script {
                    // Top-level: set global
                    let idx = self.string_constant(name)?;
                    self.emit_op(Op::SetGlobal, 0);
                    self.emit_byte(idx, 0);
                    self.emit_op(Op::Pop, 0);
                } else {
                    // Inside function: create a new local (MiniScript default)
                    // The value stays on the stack as the local's slot.
                    self.add_local(name)?;
                }
            }
            AssignTarget::Index { object, index } => {
                self.compile_expr(value)?;
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.emit_op(Op::SetIndex, 0);
            }
            AssignTarget::Member { object, field } => {
                self.compile_expr(value)?;
                self.compile_expr(object)?;
                let idx = self.string_constant(field)?;
                self.emit_op(Op::SetField, 0);
                self.emit_byte(idx, 0);
            }
            AssignTarget::Global(name) => {
                self.compile_expr(value)?;
                let idx = self.string_constant(name)?;
                self.emit_op(Op::SetGlobal, 0);
                self.emit_byte(idx, 0);
                self.emit_op(Op::Pop, 0);
            }
        }
        Ok(())
    }

    fn compile_if(
        &mut self,
        branches: &[(Expr, Vec<Stmt>)],
        else_body: &Option<Vec<Stmt>>,
    ) -> Result<(), CompileError> {
        let mut end_jumps = Vec::new();

        for (i, (cond, body)) in branches.iter().enumerate() {
            self.compile_expr(cond)?;
            let false_jump = self.emit_jump(Op::JumpIfFalse, 0);
            self.emit_op(Op::Pop, 0); // pop condition

            for stmt in body {
                self.compile_stmt(stmt)?;
            }

            // Jump to end (skip remaining branches)
            if i < branches.len() - 1 || else_body.is_some() {
                end_jumps.push(self.emit_jump(Op::Jump, 0));
            }

            self.patch_jump(false_jump)?;
            self.emit_op(Op::Pop, 0); // pop condition (false path)
        }

        if let Some(body) = else_body {
            for stmt in body {
                self.compile_stmt(stmt)?;
            }
        }

        for j in end_jumps {
            self.patch_jump(j)?;
        }
        Ok(())
    }

    fn compile_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), CompileError> {
        let loop_start = self.chunk.code.len();
        self.loops.push(LoopCtx {
            start: loop_start,
            breaks: Vec::new(),
            continues: Vec::new(),
            scope_depth: self.scope_depth,
            is_for_loop: false,
        });

        self.compile_expr(condition)?;
        let exit_jump = self.emit_jump(Op::JumpIfFalse, 0);
        self.emit_op(Op::Pop, 0);

        for stmt in body {
            self.compile_stmt(stmt)?;
        }
        self.emit_loop(loop_start, 0)?;

        self.patch_jump(exit_jump)?;
        self.emit_op(Op::Pop, 0);

        let ctx = self.loops.pop().unwrap();
        for b in ctx.breaks {
            self.patch_jump(b)?;
        }
        Ok(())
    }

    fn compile_for(
        &mut self,
        var: &str,
        iterable: &Expr,
        body: &[Stmt],
    ) -> Result<(), CompileError> {
        // Desugar: for x in iterable  →
        //   __iter = iterable; __i = 0
        //   while __i < len(__iter):  x = __iter[__i]; <body>; __i += 1
        self.begin_scope();

        // __iter = iterable
        self.compile_expr(iterable)?;
        self.add_local("__iter")?;
        let iter_slot = self.locals.len() - 1;

        // __i = 0
        self.emit_constant(Value::Number(0.0), 0)?;
        self.add_local("__i")?;
        let i_slot = self.locals.len() - 1;

        // continue target: jumps here to increment and re-check
        let loop_start = self.chunk.code.len();

        // The loop variable will be in a deeper scope so continue/break can pop it.
        let outer_depth = self.scope_depth;
        self.loops.push(LoopCtx {
            start: loop_start,
            breaks: Vec::new(),
            continues: Vec::new(),
            scope_depth: outer_depth,
            is_for_loop: true,
        });

        // Condition: __i < len(__iter)
        self.emit_op(Op::GetLocal, 0);
        self.emit_byte(i_slot as u8, 0);
        let len_idx = self.string_constant("len")?;
        self.emit_op(Op::GetGlobal, 0);
        self.emit_byte(len_idx, 0);
        self.emit_op(Op::GetLocal, 0);
        self.emit_byte(iter_slot as u8, 0);
        self.emit_op(Op::Call, 0);
        self.emit_byte(1, 0);
        self.emit_op(Op::Less, 0);

        let exit_jump = self.emit_jump(Op::JumpIfFalse, 0);
        self.emit_op(Op::Pop, 0); // pop condition

        // Begin inner scope for loop variable
        self.begin_scope();

        // x = __iter[__i]
        self.emit_op(Op::GetLocal, 0);
        self.emit_byte(iter_slot as u8, 0);
        self.emit_op(Op::GetLocal, 0);
        self.emit_byte(i_slot as u8, 0);
        self.emit_op(Op::GetIndex, 0);
        self.add_local(var)?;

        for stmt in body {
            self.compile_stmt(stmt)?;
        }

        // End inner scope (pops loop variable)
        self.end_scope();

        // Patch continue jumps to here (the increment point)
        let continues: Vec<usize> = self.loops.last().unwrap().continues.clone();
        for c in continues {
            self.patch_jump(c)?;
        }

        // __i += 1
        self.emit_op(Op::GetLocal, 0);
        self.emit_byte(i_slot as u8, 0);
        self.emit_constant(Value::Number(1.0), 0)?;
        self.emit_op(Op::Add, 0);
        self.emit_op(Op::SetLocal, 0);
        self.emit_byte(i_slot as u8, 0);
        self.emit_op(Op::Pop, 0);

        self.emit_loop(loop_start, 0)?;
        self.patch_jump(exit_jump)?;
        self.emit_op(Op::Pop, 0); // pop condition (false path)

        let ctx = self.loops.pop().unwrap();
        for b in ctx.breaks {
            self.patch_jump(b)?;
        }

        self.end_scope(); // pops __iter, __i
        Ok(())
    }

    // ── Expressions ──────────────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Number(n) => self.emit_constant(Value::Number(*n), 0)?,
            Expr::StringLit(s) => self.emit_constant(Value::String(Rc::new(s.clone())), 0)?,
            Expr::Bool(true) => self.emit_op(Op::True, 0),
            Expr::Bool(false) => self.emit_op(Op::False, 0),
            Expr::Null => self.emit_op(Op::Null, 0),

            Expr::Identifier(name) => {
                self.compile_load_var(name)?;
                self.emit_op(Op::ResolveCall, 0);
            }

            Expr::Self_ => {
                self.emit_op(Op::GetSelf, 0);
            }

            Expr::GlobalAccess(name) => {
                let idx = self.string_constant(name)?;
                self.emit_op(Op::GetGlobal, 0);
                self.emit_byte(idx, 0);
            }

            Expr::BinaryOp { left, op, right } => {
                // Short-circuit for and/or
                match op {
                    BinOp::And => {
                        self.compile_expr(left)?;
                        let jump = self.emit_jump(Op::JumpIfFalse, 0);
                        self.emit_op(Op::Pop, 0);
                        self.compile_expr(right)?;
                        self.patch_jump(jump)?;
                    }
                    BinOp::Or => {
                        self.compile_expr(left)?;
                        let else_jump = self.emit_jump(Op::JumpIfFalse, 0);
                        let end_jump = self.emit_jump(Op::Jump, 0);
                        self.patch_jump(else_jump)?;
                        self.emit_op(Op::Pop, 0);
                        self.compile_expr(right)?;
                        self.patch_jump(end_jump)?;
                    }
                    _ => {
                        self.compile_expr(left)?;
                        self.compile_expr(right)?;
                        match op {
                            BinOp::Add => self.emit_op(Op::Add, 0),
                            BinOp::Sub => self.emit_op(Op::Sub, 0),
                            BinOp::Mul => self.emit_op(Op::Mul, 0),
                            BinOp::Div => self.emit_op(Op::Div, 0),
                            BinOp::Mod => self.emit_op(Op::Mod, 0),
                            BinOp::Power => self.emit_op(Op::Power, 0),
                            BinOp::Eq => self.emit_op(Op::Equal, 0),
                            BinOp::NotEq => self.emit_op(Op::NotEqual, 0),
                            BinOp::Lt => self.emit_op(Op::Less, 0),
                            BinOp::Gt => self.emit_op(Op::Greater, 0),
                            BinOp::LtEq => self.emit_op(Op::LessEq, 0),
                            BinOp::GtEq => self.emit_op(Op::GreaterEq, 0),
                            BinOp::Isa => self.emit_op(Op::Isa, 0),
                            BinOp::And | BinOp::Or => unreachable!(),
                        }
                    }
                }
            }

            Expr::UnaryOp { op, expr } => {
                match op {
                    UnaryOp::AddressOf => {
                        // @ suppresses auto-invocation: load without ResolveCall
                        self.compile_expr_raw(expr)?;
                    }
                    _ => {
                        self.compile_expr(expr)?;
                        match op {
                            UnaryOp::Negate => self.emit_op(Op::Negate, 0),
                            UnaryOp::Not => self.emit_op(Op::Not, 0),
                            UnaryOp::New => self.emit_op(Op::NewObject, 0),
                            UnaryOp::AddressOf => unreachable!(),
                        }
                    }
                }
            }

            Expr::Call { callee, args } => {
                // Check if this is a method call: callee is Member { object, field }
                if let Expr::Member { object, field } = callee.as_ref() {
                    // Method call: obj.method(args)
                    self.compile_expr(object)?;
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    let idx = self.string_constant(field)?;
                    self.emit_op(Op::CallMethod, 0);
                    self.emit_byte(idx, 0);
                    self.emit_byte(args.len() as u8, 0);
                } else {
                    // Regular call — don't auto-invoke callee, we're about to Call it
                    self.compile_expr_raw(callee)?;
                    for arg in args {
                        self.compile_expr(arg)?;
                    }
                    self.emit_op(Op::Call, 0);
                    self.emit_byte(args.len() as u8, 0);
                }
            }

            Expr::Index { object, index } => {
                self.compile_expr(object)?;
                self.compile_expr(index)?;
                self.emit_op(Op::GetIndex, 0);
            }

            Expr::Slice { object, from, to } => {
                // TODO: implement slice as a runtime intrinsic call
                // For now, compile as a method call: object.slice(from, to)
                // We'll handle this in the VM with a special "slice" operation
                self.compile_expr(object)?;
                match from {
                    Some(f) => self.compile_expr(f)?,
                    None => self.emit_op(Op::Null, 0),
                }
                match to {
                    Some(t) => self.compile_expr(t)?,
                    None => self.emit_op(Op::Null, 0),
                }
                let idx = self.string_constant("__slice")?;
                self.emit_op(Op::CallMethod, 0);
                self.emit_byte(idx, 0);
                self.emit_byte(2, 0); // from, to
            }

            Expr::Member { object, field } => {
                self.compile_expr(object)?;
                let idx = self.string_constant(field)?;
                self.emit_op(Op::GetField, 0);
                self.emit_byte(idx, 0);
                // Auto-invoke: if result is callable, call with 0 args
                self.emit_op(Op::ResolveCall, 0);
            }

            Expr::ListLiteral(items) => {
                for item in items {
                    self.compile_expr(item)?;
                }
                self.emit_op(Op::NewList, 0);
                self.emit_byte(items.len() as u8, 0);
            }

            Expr::MapLiteral(pairs) => {
                for (key, val) in pairs {
                    self.compile_expr(key)?;
                    self.compile_expr(val)?;
                }
                self.emit_op(Op::NewMap, 0);
                self.emit_byte(pairs.len() as u8, 0);
            }

            Expr::FuncExpr { params, body } => {
                let func = Self::compile_function("<anon>", params, body)?;
                self.emit_constant(Value::Function(Rc::new(func)), 0)?;
            }
        }
        Ok(())
    }

    // ── Scope management ─────────────────────────────────────────

    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn end_scope(&mut self) {
        while let Some(local) = self.locals.last() {
            if local.depth < self.scope_depth {
                break;
            }
            self.emit_op(Op::Pop, 0);
            self.locals.pop();
        }
        self.scope_depth -= 1;
    }

    fn add_local(&mut self, name: &str) -> Result<(), CompileError> {
        if self.locals.len() >= 256 {
            return Err(CompileError::TooManyLocals);
        }
        self.locals.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
        });
        Ok(())
    }

    /// Load a variable onto the stack without ResolveCall (used by @).
    fn compile_load_var(&mut self, name: &str) -> Result<(), CompileError> {
        if let Some(slot) = self.resolve_local(name) {
            self.emit_op(Op::GetLocal, 0);
            self.emit_byte(slot as u8, 0);
        } else {
            let idx = self.string_constant(name)?;
            self.emit_op(Op::GetGlobal, 0);
            self.emit_byte(idx, 0);
        }
        Ok(())
    }

    /// Compile an expression without auto-invocation (for @ operator).
    fn compile_expr_raw(&mut self, expr: &Expr) -> Result<(), CompileError> {
        match expr {
            Expr::Identifier(name) => self.compile_load_var(name),
            Expr::Member { object, field } => {
                // GetField without ResolveCall (for call targets / @ operator)
                self.compile_expr(object)?;
                let idx = self.string_constant(field)?;
                self.emit_op(Op::GetField, 0);
                self.emit_byte(idx, 0);
                Ok(())
            }
            _ => self.compile_expr(expr),
        }
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        self.locals.iter().rposition(|l| l.name == name)
    }

    // ── Bytecode emission helpers ────────────────────────────────

    fn emit_op(&mut self, op: Op, line: usize) {
        self.chunk.write(op as u8, line);
    }

    fn emit_byte(&mut self, byte: u8, line: usize) {
        self.chunk.write(byte, line);
    }

    fn emit_constant(&mut self, value: Value, line: usize) -> Result<(), CompileError> {
        let idx = self.chunk.add_constant(value);
        if idx > 255 {
            return Err(CompileError::TooManyConstants);
        }
        self.emit_op(Op::Constant, line);
        self.emit_byte(idx as u8, line);
        Ok(())
    }

    fn string_constant(&mut self, s: &str) -> Result<u8, CompileError> {
        // Reuse existing constant if possible
        for (i, c) in self.chunk.constants.iter().enumerate() {
            if let Value::String(existing) = c {
                if existing.as_ref() == s {
                    return Ok(i as u8);
                }
            }
        }
        let idx = self
            .chunk
            .add_constant(Value::String(Rc::new(s.to_string())));
        if idx > 255 {
            return Err(CompileError::TooManyConstants);
        }
        Ok(idx as u8)
    }

    fn emit_jump(&mut self, op: Op, line: usize) -> usize {
        self.emit_op(op, line);
        self.emit_byte(0xff, line);
        self.emit_byte(0xff, line);
        self.chunk.code.len() - 2
    }

    fn patch_jump(&mut self, offset: usize) -> Result<(), CompileError> {
        let jump = self.chunk.code.len() - offset - 2;
        if jump > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }
        self.chunk.code[offset] = ((jump >> 8) & 0xff) as u8;
        self.chunk.code[offset + 1] = (jump & 0xff) as u8;
        Ok(())
    }

    fn emit_loop(&mut self, loop_start: usize, line: usize) -> Result<(), CompileError> {
        self.emit_op(Op::Loop, line);
        let offset = self.chunk.code.len() - loop_start + 2;
        if offset > u16::MAX as usize {
            return Err(CompileError::JumpTooLarge);
        }
        self.emit_byte(((offset >> 8) & 0xff) as u8, line);
        self.emit_byte((offset & 0xff) as u8, line);
        Ok(())
    }
}
