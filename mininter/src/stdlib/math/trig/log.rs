use crate::{RuntimeError, Value};

pub fn log(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let x = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0);

    let base = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(10.0);

    Ok(Value::Number(x.log(base)))
}
