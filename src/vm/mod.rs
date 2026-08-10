use crate::{
    compiler::{chunk::Chunk, opcode::OpCode, value::Value},
    error::{Error, Result},
};

pub enum VmResult {
    Ok,
    Error,
}

pub struct Vm {
    stack: Vec<Value>,
}

impl Vm {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    // #region Stack Manipulation

    pub fn clean_stack(&mut self) {
        self.stack.clear();
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn peek(&mut self) -> Result<&Value> {
        self.stack.last().ok_or(Error::stack_underflow(
            "failed to peek at last stack value. reached the end of the stack ?",
            None,
        ))
    }

    fn pop(&mut self) -> Result<Value> {
        self.stack.pop().ok_or(Error::stack_underflow(
            "failed to pop the last stack value. reached the end of the stack ?",
            None,
        ))
    }

    // #endregion

    pub fn run<'op>(&mut self, input: &'op Chunk) -> Result<VmResult> {
        let mut ip = 0;

        loop {
            let op: &OpCode = input
                .code()
                .get(ip)
                .ok_or(Error::eoi("no more input", None))?;

            match op {
                OpCode::Return => {
                    println!("{}", self.pop()?);
                    return Ok(VmResult::Ok);
                }

                OpCode::Const(i) => {
                    self.push(input.get_const(*i)?.clone());
                }

                OpCode::LongConst(i) => {
                    self.push(input.get_const(*i)?.clone());
                }

                OpCode::Negate => {
                    let mut val = self.pop()?;

                    if let Value::Number(ref mut n) = val {
                        let inner = n.get_mut();
                        *inner = -*inner;
                    } else {
                        return Err(Error::invalid_operation(
                            "tried negating a value which is not a number",
                            None,
                        ));
                    }

                    self.push(val);
                }

                OpCode::Add => {
                    let mut alpha = self.pop()?;
                    let bravo = self.pop()?;
                    if let Value::Number(ref mut n1) = alpha
                        && let Value::Number(ref n2) = bravo
                    {
                        let inner = n1.get_mut();
                        *inner += n2.get();
                    } else {
                        return Err(Error::invalid_operation(
                            "tried negating a value which is not a number",
                            None,
                        ));
                    }

                    self.push(alpha);
                }

                OpCode::Substract => {
                    let mut alpha = self.pop()?;
                    let bravo = self.pop()?;
                    if let Value::Number(ref mut n1) = alpha
                        && let Value::Number(ref n2) = bravo
                    {
                        let inner = n1.get_mut();
                        *inner -= n2.get();
                    } else {
                        return Err(Error::invalid_operation(
                            "tried negating a value which is not a number",
                            None,
                        ));
                    }

                    self.push(alpha);
                }

                OpCode::Multiply => {
                    let mut alpha = self.pop()?;
                    let bravo = self.pop()?;
                    if let Value::Number(ref mut n1) = alpha
                        && let Value::Number(ref n2) = bravo
                    {
                        let inner = n1.get_mut();
                        *inner *= n2.get();
                    } else {
                        return Err(Error::invalid_operation(
                            "tried negating a value which is not a number",
                            None,
                        ));
                    }

                    self.push(alpha);
                }

                OpCode::Divide => {
                    let mut alpha = self.pop()?;
                    let bravo = self.pop()?;
                    if let Value::Number(ref mut n1) = alpha
                        && let Value::Number(ref n2) = bravo
                    {
                        let inner = n1.get_mut();
                        *inner /= n2.get();
                    } else {
                        return Err(Error::invalid_operation(
                            "tried negating a value which is not a number",
                            None,
                        ));
                    }

                    self.push(alpha);
                }

                _ => {
                    eprintln!("unhandled instruction {}", op);
                }
            }
            ip += 1;
        }
    }
}
