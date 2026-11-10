use std::io::{Write, stdout};

use crate::{RuntimeError, Value};

pub fn print(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let msg = args
        .next()
        .map(|v| v.to_display_string())
        .unwrap_or_default();
    let delim = args
        .next()
        .map(|v| v.to_display_string())
        .unwrap_or("\n".to_string());
    print!("{msg}{delim}");
    stdout()
        .flush()
        .map_err(|e| RuntimeError::System(format!("failed to flush stdout: {e}")))?;
    Ok(Value::Null)
}
