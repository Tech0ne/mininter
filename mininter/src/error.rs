use std::fmt;

#[derive(Debug)]
pub struct MiniError {
    pub message: String,
}

impl fmt::Display for MiniError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for MiniError {}

impl From<LexError> for MiniError {
    fn from(e: LexError) -> Self {
        MiniError {
            message: format!("Lex error (line {}): {}", e.line, e.message),
        }
    }
}

impl From<ParseError> for MiniError {
    fn from(e: ParseError) -> Self {
        MiniError {
            message: format!("Parse error (line {}): {}", e.line, e.message),
        }
    }
}

impl From<CompileError> for MiniError {
    fn from(e: CompileError) -> Self {
        MiniError {
            message: format!("Compile error: {e}"),
        }
    }
}

impl From<RuntimeError> for MiniError {
    fn from(e: RuntimeError) -> Self {
        MiniError {
            message: format!("Runtime error: {}", e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LexError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum CompileError {
    TooManyConstants,
    TooManyLocals,
    JumpTooLarge,
    BreakOutsideLoop,
    ContinueOutsideLoop,
    InvalidAssignTarget,
    Custom(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyConstants => write!(f, "too many constants in one chunk"),
            Self::TooManyLocals => write!(f, "too many local variables"),
            Self::JumpTooLarge => write!(f, "jump offset too large"),
            Self::BreakOutsideLoop => write!(f, "'break' outside of loop"),
            Self::ContinueOutsideLoop => write!(f, "'continue' outside of loop"),
            Self::InvalidAssignTarget => write!(f, "invalid assignment target"),
            Self::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError { expected: String, got: String },
    WrongArgCount { expected: usize, got: usize },
    UndefinedVariable(String),
    UndefinedField(String),
    NotCallable(String),
    IndexOutOfBounds { index: i64, len: usize },
    DivisionByZero,
    StackUnderflow,
    System(String),
    Custom(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TypeError { expected, got } => {
                write!(f, "type error: expected {expected}, got {got}")
            }
            Self::WrongArgCount { expected, got } => write!(
                f,
                "wrong number of arguments: expected {expected}, got {got}"
            ),
            Self::UndefinedVariable(name) => write!(f, "undefined variable '{name}'"),
            Self::UndefinedField(name) => write!(f, "undefined field '{name}'"),
            Self::NotCallable(t) => write!(f, "cannot call value of type {t}"),
            Self::IndexOutOfBounds { index, len } => {
                write!(f, "index {index} out of bounds (len {len})")
            }
            Self::DivisionByZero => write!(f, "division by zero"),
            Self::StackUnderflow => write!(f, "stack underflow"),
            Self::System(msg) => write!(f, "error with inner systems: {msg}"),
            Self::Custom(msg) => write!(f, "{msg}"),
        }
    }
}
