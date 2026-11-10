use crate::{RuntimeError, Value};

pub fn val(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args
        .first()
        .map(|v| v.to_display_string())
        .unwrap_or_default();
    let n = s.trim().parse::<f64>().unwrap_or(0.0);
    Ok(Value::Number(n))
}
