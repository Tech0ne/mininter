pub mod chunk;
pub mod opcode;
pub mod value;

use chunk::Chunk;

use crate::{
    compiler::{opcode::OpCode, value::Value},
    core::number::Float,
    error::{Error, Result},
    scanner::{
        Scanner,
        token::{Token, TokenKind},
    },
};

pub struct Compiler<'scan> {
    input: &'scan mut Scanner<'scan>,
    chunk: Chunk,
    had_errors: bool,
    previous: Option<Token<'scan>>,
    current: Option<Token<'scan>>,
}

impl<'scan> Compiler<'scan> {
    pub fn new(input: &'scan mut Scanner<'scan>) -> Result<Self> {
        let mut slf = Self {
            input,
            chunk: Chunk::new(),
            had_errors: false,
            previous: None,
            current: None,
        };

        slf.advance()?;

        Ok(slf)
    }

    pub fn had_errors(&self) -> bool {
        self.had_errors
    }

    fn advance(&mut self) -> Result<()> {
        self.previous = self.current.take();

        self.current = match self.input.next() {
            Some(Ok(t)) => Some(t),
            Some(Err(e)) => {
                self.had_errors = true;
                return Err(e);
            }
            None => None,
        };

        Ok(())
    }

    fn consume(&mut self, expect: TokenKind) -> Result<()> {
        if self.current.as_ref().is_some_and(|t| t.kind() == expect) {
            self.advance()?;
        }
        Ok(())
    }

    fn number(&mut self, number: Float) {
        let index = self.chunk.add_const(Value::number(number));
        if index >= 256 {
            self.chunk.add_op(OpCode::LongConst(index));
        } else {
            self.chunk.add_op(OpCode::Const(index as u8));
        }
    }

    fn grouping(&mut self) -> Result<()> {
        let line_infos = self.input.line_infos();

        self.expression()?;

        if !self
            .current
            .as_ref()
            .is_some_and(|t| t.kind() == TokenKind::RightParen)
        {
            Err(Error::parse("missing closing ')'", Some(line_infos)))
        } else {
            Ok(())
        }
    }

    fn unary(&mut self) -> Result<()> {
        let line_infos = self.input.line_infos();

        let kind = match self.previous.as_ref() {
            Some(t) => t.kind(),
            None => {
                return Err(Error::parse(
                    "called unary on a token with no previous ?",
                    Some(self.input.line_infos()),
                ));
            }
        };

        self.expression()?;

        match kind {
            TokenKind::Minus => {
                self.chunk.add_op(OpCode::Negate);
            }
            _ => {
                return Err(Error::invalid_operation("", Some(line_infos)));
            }
        }
        Ok(())
    }

    fn expression(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn compile(mut self) -> Result<Chunk> {
        let mut token;

        loop {
            token = match self.input.next() {
                Some(Ok(token)) => token,
                Some(Err(e)) => {
                    return Err(e);
                }
                None => {
                    break;
                }
            }
        }

        // self.chunk.push(OpCode::End);
        Ok(self.chunk)
    }

    // fn next_op_code(&mut self) -> Result<OpCode> {
    //     Err(Error::parse("TODO", None))
    // }
}

// #region Iterator

// impl<'scan> Iterator for Compiler<'scan> {
//     type Item = Result<OpCode>;
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.next_op_code() {
//             Ok(o) => {
//                 if let OpCode::End = o {
//                     None
//                 } else {
//                     Some(Ok(o))
//                 }
//             }
//             Err(e) => Some(Err(e)),
//         }
//     }
// }

// #endregion
