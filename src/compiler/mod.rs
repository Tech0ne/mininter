use crate::scanner::Scanner;

pub mod opcode;

pub struct Compiler<'scan> {
    input: &'scan Scanner<'scan>,
}

impl<'scan> Compiler<'scan> {
    pub fn new(input: &'scan Scanner<'scan>) -> Self {
        Self { input }
    }
}
