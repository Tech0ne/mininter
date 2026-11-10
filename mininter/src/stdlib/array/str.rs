use std::rc::Rc;

use crate::{RuntimeError, Value};

pub fn str(args: &[Value]) -> Result<Value, RuntimeError> {
    let val = args.first().cloned().unwrap_or(Value::Null);
    Ok(Value::String(Rc::new(val.to_display_string())))
}
