use crate::{RuntimeError, Value};

pub fn bit_and(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let a = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0) as u64;

    let b = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0) as u64;

    Ok(Value::Number((a & b) as f64))
}
