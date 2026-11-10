use crate::{RuntimeError, Value};

pub fn round(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let n = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0);

    let d = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0).floor() as i32;

    let dec = 10.0f64.powi(d);

    if dec == 0.0 {
        Ok(Value::Number(n.round()))
    } else {
        Ok(Value::Number((n * dec).round() / dec))
    }
}
