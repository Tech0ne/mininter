use std::rc::Rc;

use crate::{RuntimeError, Value};

pub fn char_fn(args: &[Value]) -> Result<Value, RuntimeError> {
    let code = args
        .first()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0))
        .unwrap_or(0.0) as u32;
    let ch = char::from_u32(code).unwrap_or('\0');
    Ok(Value::String(Rc::new(ch.to_string())))
}
