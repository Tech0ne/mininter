use crate::runtime::value::Value;

/// Bytecode opcodes for the MiniScript VM.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    // Constants & literals
    Constant, // [u8: index into constants table]
    Null,
    True,
    False,
    Pop,

    // Variables
    GetGlobal, // [u8: name constant index]
    SetGlobal, // [u8: name constant index]
    GetLocal,  // [u8: stack slot]
    SetLocal,  // [u8: stack slot]

    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Power,
    Negate,

    // Logic & comparison
    Not,
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Isa,

    // Control flow
    Jump,        // [u16: forward offset]
    JumpIfFalse, // [u16: forward offset]
    Loop,        // [u16: backward offset]

    // Functions
    Call, // [u8: arg count]
    Return,

    // Collections
    NewList, // [u8: element count]
    NewMap,  // [u8: pair count]
    GetIndex,
    SetIndex,

    // Objects & members
    GetField,    // [u8: field name constant index]
    SetField,    // [u8: field name constant index]
    CallMethod,  // [u8: field name constant index, u8: arg count]
    NewObject,   // pop parent map, push new child map with __isa
    GetSelf,     // push current self value
    ResolveCall, // if top of stack is callable, call it with 0 args
    Module, // [u8: constant index of compiled module Function, u8: constant index of module name]
}

impl Op {
    pub fn from_byte(b: u8) -> Option<Op> {
        if b <= Op::Module as u8 {
            Some(unsafe { std::mem::transmute(b) })
        } else {
            None
        }
    }
}

/// A chunk of bytecode with its constant pool and line info.
#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
}
