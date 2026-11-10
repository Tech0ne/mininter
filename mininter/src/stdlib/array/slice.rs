use std::{cell::RefCell, rc::Rc};

use crate::{RuntimeError, Value};

fn slice_string(string: &Rc<String>, start: Option<isize>, end: Option<isize>) -> Value {
    let len = string.len() as isize;

    let normalize = |index: Option<isize>, default: isize| -> usize {
        let i = index.unwrap_or(default);

        if i < 0 {
            (len + i).clamp(0, len) as usize
        } else {
            i.clamp(0, len) as usize
        }
    };

    let start = normalize(start, 0);

    let end = normalize(end, len);

    if start >= end {
        Value::String(Rc::new(String::new()))
    } else {
        Value::String(Rc::new(
            string.chars().collect::<Vec<_>>()[start..end]
                .iter()
                .collect(),
        ))
    }
}

fn slice_list(list: &Rc<RefCell<Vec<Value>>>, start: Option<isize>, end: Option<isize>) -> Value {
    let len = list.take().len() as isize;

    let normalize = |index: Option<isize>, default: isize| -> usize {
        let i = index.unwrap_or(default);

        if i < 0 {
            (len + i).clamp(0, len) as usize
        } else {
            i.clamp(0, len) as usize
        }
    };

    let start = normalize(start, 0);

    let end = normalize(end, len);

    if start >= end {
        Value::List(Rc::new(RefCell::new(vec![])))
    } else {
        Value::List(Rc::new(RefCell::new(list.borrow()[start..end].to_vec())))
    }
}

pub fn slice(args: &[Value]) -> Result<Value, RuntimeError> {
    let mut args = args.iter();

    let input = args.next();

    let start = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0).floor() as isize);

    let end = args
        .next()
        .map(|n| n.as_number())
        .map(|e| e.unwrap_or(0.0).floor() as isize);

    Ok(match input {
        Some(Value::String(s)) => slice_string(s, start, end),
        Some(Value::List(l)) => slice_list(l, start, end),
        _ => Value::Null,
    })
}
