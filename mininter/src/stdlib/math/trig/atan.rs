use crate::{RuntimeError, Value};

pub fn atan(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let y = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0);

    let x = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(1.0);

    Ok(Value::Number(y.atan2(x)))
}
