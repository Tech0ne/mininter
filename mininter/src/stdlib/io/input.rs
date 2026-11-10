use std::{
    io::{self, Write},
    rc::Rc,
};

use crate::{RuntimeError, Value};

pub fn input(args: &[Value]) -> Result<Value, RuntimeError> {
    if let Some(prompt) = args.first() {
        print!("{}", prompt.to_display_string());
        io::stdout()
            .flush()
            .map_err(|e| RuntimeError::System(format!("failed to flush stdout: {e}")))?;
    }

    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .map_err(|e| RuntimeError::System(format!("failed to read line from stdin: {e}")))?;

    Ok(Value::String(Rc::new(
        line.trim_end_matches('\n').to_string(),
    )))
}
