use crate::{RuntimeError, Value};

pub fn sin(args: &[Value]) -> Result<Value, RuntimeError> {
    let n = args
        .first()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0);

    Ok(Value::Number(n.sin()))
}
