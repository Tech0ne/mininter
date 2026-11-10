use std::{cell::RefCell, rc::Rc};

use crate::{RuntimeError, Value};

pub fn range(args: &[Value]) -> Result<Value, RuntimeError> {
    let (start, end, step) = match args.len() {
        0 => (0.0, 0.0, 0.0),
        1 => {
            let start = args[0].as_number()?;
            (start, 0.0, if start > 0.0 { -1.0 } else { 1.0 })
        }
        2 => {
            let start = args[0].as_number()?;
            let end = args[1].as_number()?;
            (start, end, if start > end { -1.0 } else { 1.0 })
        }
        3 => (
            args[0].as_number()?,
            args[1].as_number()?,
            args[2].as_number()?,
        ),
        n => {
            return Err(RuntimeError::WrongArgCount {
                expected: 3,
                got: n,
            });
        }
    };

    let mut items = Vec::new();
    let mut i = start;
    if step > 0.0 {
        while i < end {
            items.push(Value::Number(i));
            i += step;
        }
    } else if step < 0.0 {
        while i > end {
            items.push(Value::Number(i));
            i += step;
        }
    } else {
        items.push(Value::Number(0.0));
    }
    Ok(Value::List(Rc::new(RefCell::new(items))))
}
