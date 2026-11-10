pub mod compiler;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod stdlib;

// Re-exports for convenience
pub use error::{MiniError, RuntimeError};
pub use mininter_macros::api;
pub use runtime::{ApiFunction, FromValue, IntoValue, Value, Vm};

// Path aliases so the proc macro can use miniscript::value::X and miniscript::vm::X
pub mod value {
    pub use crate::runtime::value::*;
}
pub mod vm {
    pub use crate::runtime::vm::*;
}
