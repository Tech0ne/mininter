use std::{fmt::Display, io::Write};

use crate::error::Result;

#[derive(Debug)]
#[repr(u8)]
pub enum OpCode {
    Return,
    Nop,
    Const(u8),
    LongConst(u16),
    Negate,
    Add,
    Substract,
    Multiply,
    Divide,
    End,
}

impl OpCode {
    fn discriminant(&self) -> u8 {
        match self {
            Self::Return => 0x01,
            Self::Nop => 0x90,
            Self::Const(_) => 0x02,
            Self::LongConst(_) => 0x03,
            Self::Negate => 0x04,
            Self::Add => 0x05,
            Self::Substract => 0x06,
            Self::Multiply => 0x07,
            Self::Divide => 0x08,
            Self::End => 0xff,
            // _ => {
            //     eprintln!("OpCode {:?} doesn't yet have a u8 code", self);
            //     todo!();
            // }
        }
    }

    pub fn write_to<W: Write>(&self, w: &mut W) -> Result<()> {
        w.write(&[self.discriminant()])?;

        Ok(())
    }

    pub fn size(&self) -> usize {
        match self {
            Self::Const(_) => 2,
            Self::LongConst(_) => 3,
            _ => 1,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Return => write!(f, "{:02x} | OP_RETURN", self.discriminant()),
            Self::Nop => write!(f, "{:02x} | OP_NOP", self.discriminant()),
            Self::Const(i) => write!(f, "{:02x} {:02x} | OP_CONST", self.discriminant(), i),
            Self::LongConst(i) => {
                let [hi, lo] = i.to_be_bytes();
                write!(
                    f,
                    "{:02x} {:02x} {:02x} | OP_LONG_CONST",
                    self.discriminant(),
                    hi,
                    lo
                )
            }
            Self::Negate => write!(f, "{:02x} | OP_NEGATE", self.discriminant()),
            Self::Add => write!(f, "{:02x} | OP_ADD", self.discriminant()),
            Self::Substract => write!(f, "{:02x} | OP_SUBSTRACT", self.discriminant()),
            Self::Multiply => write!(f, "{:02x} | OP_MULTIPLY", self.discriminant()),
            Self::Divide => write!(f, "{:02x} | OP_DIVIDE", self.discriminant()),
            Self::End => write!(f, "{:02x} | OP_END", self.discriminant()),
            // _ => {
            //     eprintln!("Forgot to handle inst {:?}", self);
            //     todo!();
            // }
        }
    }
}
