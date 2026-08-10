mod builder;
use builder::errors;

errors!(
    Eoi,
    InvalidConst,
    InvalidOperation,
    NotFound,
    Parse,
    StackUnderflow,
);

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::StdIoError,
            msg: format!("Input/Output Error: {}", value),
            stack_trace: vec![],
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
