use crate::{RuntimeError, Value};

pub fn code(args: &[Value]) -> Result<Value, RuntimeError> {
    let code = args
        .first()
        .map(|v| v.to_display_string())
        .and_then(|s| s.chars().next())
        .unwrap_or('\0');

    Ok(Value::Number(code as u8 as f64))
}
