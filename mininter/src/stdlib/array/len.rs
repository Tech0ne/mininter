use crate::{RuntimeError, Value};

pub fn len(args: &[Value]) -> Result<Value, RuntimeError> {
    let val = match args.first() {
        Some(val) => val,
        None => {
            return Ok(Value::Null);
        }
    };

    let n = match val {
        Value::String(s) => s.len(),
        Value::List(items) => items.borrow().len(),
        Value::Map(map) => map.borrow().len(),
        _ => {
            return Ok(Value::Null);
        }
    };
    Ok(Value::Number(n as f64))
}
