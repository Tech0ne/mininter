use crate::{RuntimeError, Value};

pub fn err(args: &[Value]) -> Result<Value, RuntimeError> {
    let msg = args
        .first()
        .map(|n| n.to_display_string())
        .unwrap_or("".to_string());

    Err(RuntimeError::Custom(msg))
}
