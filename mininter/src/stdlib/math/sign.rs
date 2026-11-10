use crate::{RuntimeError, Value};

pub fn sign(args: &[Value]) -> Result<Value, RuntimeError> {
    let n = args
        .first()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0);

    Ok(Value::Number(if n > 0.0 {
        1.0
    } else if n < 0.0 {
        -1.0
    } else {
        0.0
    }))
}
