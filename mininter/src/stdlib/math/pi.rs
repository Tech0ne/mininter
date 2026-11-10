use crate::{RuntimeError, Value};

pub fn pi(_args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Number(3.14159265358979))
}
