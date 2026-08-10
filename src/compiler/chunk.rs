use std::fmt::Display;

use crate::{
    compiler::{opcode::OpCode, value::Value},
    error::{Error, Result},
};

#[derive(Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    consts: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            consts: Vec::new(),
        }
    }

    pub fn code(&self) -> &[OpCode] {
        &self.code
    }

    pub fn get_const(&self, index: impl Into<usize>) -> Result<&Value> {
        let index = index.into();
        self.consts.get(index).ok_or(Error::not_found(
            format!("failed to retreive const value at index {}", index),
            None,
        ))
    }

    pub fn add_const(&mut self, val: Value) -> u16 {
        self.consts.push(val);
        self.consts.len() as u16
    }

    pub fn add_op(&mut self, op: OpCode) {
        self.code.push(op);
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Consts ===")?;
        for (i, value) in self.consts.iter().enumerate() {
            writeln!(f, " {:3} | {}", i, value)?;
        }
        writeln!(f, "=== Code ===")?;
        let mut index = 0;
        for value in &self.code {
            writeln!(f, " {:06x} | {}", index, value)?;
            index += value.size();
        }
        Ok(())
    }
}
