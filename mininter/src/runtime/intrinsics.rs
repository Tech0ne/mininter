//! Built-in prototype methods for string, list, and map types.
//!
//! Each method is a native function where `args[0]` is `self` (the receiver).
//! These are registered into prototype maps (`string`, `list`, `map`) which are
//! also available as MiniScript globals, enabling runtime extension:
//!
//!     string.myMethod = function(self)
//!         return self.upper + "!"
//!     end function
//!     print "hello".myMethod   // "HELLO!"

use crate::error::RuntimeError;
use crate::runtime::value::{NativeFunction, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

// ── Prototype map construction ─────────────────────────────────

/// Helper to insert a native function into a prototype map.
fn register(
    map: &mut HashMap<String, Value>,
    name: &str,
    func: fn(&[Value]) -> Result<Value, RuntimeError>,
    max_args: usize,
) {
    map.insert(
        name.to_string(),
        Value::NativeFunction(Rc::new(NativeFunction {
            name: name.to_string(),
            func: Rc::new(move |args| func(args)),
            max_args: max_args + 1,
        })),
    );
}

pub fn make_string_proto() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    register(&mut m, "len", string_len, 0);
    register(&mut m, "upper", string_upper, 0);
    register(&mut m, "lower", string_lower, 0);
    register(&mut m, "trim", string_trim, 1);
    register(&mut m, "val", string_val, 0);
    register(&mut m, "code", string_code, 0);
    register(&mut m, "hasIndex", string_has_index, 1);
    register(&mut m, "indexes", string_indexes, 0);
    register(&mut m, "values", string_values, 0);
    register(&mut m, "indexOf", string_index_of, 2);
    register(&mut m, "insert", string_insert, 2);
    register(&mut m, "remove", string_remove, 1);
    register(&mut m, "replace", string_replace, 3);
    register(&mut m, "split", string_split, 2);
    register(&mut m, "__slice", string_slice, 2);
    m
}

pub fn make_list_proto() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    register(&mut m, "len", list_len, 0);
    register(&mut m, "sum", list_sum, 0);
    register(&mut m, "hasIndex", list_has_index, 1);
    register(&mut m, "indexes", list_indexes, 0);
    register(&mut m, "values", list_values, 0);
    register(&mut m, "indexOf", list_index_of, 2);
    register(&mut m, "insert", list_insert, 2);
    register(&mut m, "join", list_join, 1);
    register(&mut m, "pop", list_pop, 0);
    register(&mut m, "pull", list_pull, 0);
    register(&mut m, "push", list_push, 1);
    register(&mut m, "remove", list_remove, 1);
    register(&mut m, "replace", list_replace, 3);
    #[cfg(feature = "rand")]
    register(&mut m, "shuffle", list_shuffle, 0);
    register(&mut m, "sort", list_sort, 2);
    register(&mut m, "__slice", list_slice, 2);
    m
}

pub fn make_map_proto() -> HashMap<String, Value> {
    let mut m = HashMap::new();
    register(&mut m, "len", map_len, 0);
    register(&mut m, "hasIndex", map_has_index, 1);
    register(&mut m, "indexes", map_indexes, 0);
    register(&mut m, "values", map_values, 0);
    register(&mut m, "indexOf", map_index_of, 2);
    register(&mut m, "pop", map_pop, 0);
    register(&mut m, "push", map_push, 1);
    register(&mut m, "remove", map_remove, 1);
    register(&mut m, "replace", map_replace, 3);
    #[cfg(feature = "rand")]
    register(&mut m, "shuffle", map_shuffle, 0);
    register(&mut m, "sum", map_sum, 0);
    m
}

// ── __isa chain lookup (used by VM for maps) ───────────────────

/// Walk the `__isa` prototype chain looking for `field`.
pub fn isa_chain_lookup(map: &HashMap<String, Value>, field: &str) -> Option<Value> {
    if let Some(val) = map.get(field) {
        return Some(val.clone());
    }
    if let Some(Value::Map(parent)) = map.get("__isa") {
        return isa_chain_lookup(&parent.borrow(), field);
    }
    None
}

/// Walk __isa chain but skip the map itself (only parents).
pub fn isa_chain_lookup_skip_self(map: &HashMap<String, Value>, field: &str) -> Option<Value> {
    if let Some(Value::Map(parent)) = map.get("__isa") {
        return isa_chain_lookup(&parent.borrow(), field);
    }
    None
}

// ═══════════════════════════════════════════════════════════════
//  STRING METHODS — args[0] = self (the string)
// ═══════════════════════════════════════════════════════════════

fn string_len(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::Number(s.chars().count() as f64))
}

fn string_upper(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::String(Rc::new(s.to_uppercase())))
}

fn string_lower(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::String(Rc::new(s.to_lowercase())))
}

fn string_trim(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::String(Rc::new(s.trim().to_string())))
}

fn string_val(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::Number(s.trim().parse::<f64>().unwrap_or(0.0)))
}

fn string_code(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    Ok(Value::Number(
        s.chars().next().map(|c| c as u32 as f64).unwrap_or(0.0),
    ))
}

fn string_has_index(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let i = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .as_number()? as i64;
    let len = s.chars().count() as i64;
    Ok(Value::Bool(i >= 0 && i < len))
}

fn string_indexes(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let len = s.chars().count();
    let idxs: Vec<Value> = (0..len).map(|i| Value::Number(i as f64)).collect();
    Ok(Value::List(Rc::new(RefCell::new(idxs))))
}

fn string_values(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let chars: Vec<Value> = s
        .chars()
        .map(|c| Value::String(Rc::new(c.to_string())))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(chars))))
}

fn string_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let needle = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .as_string()?;
    let after = args
        .get(2)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize);
    let search_from = after.unwrap_or(0);
    match s[search_from..].find(needle) {
        Some(i) => Ok(Value::Number((search_from + i) as f64)),
        None => Ok(Value::Null),
    }
}

fn string_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let idx = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 1,
        })?
        .as_number()? as usize;
    let sub = args
        .get(2)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 2,
        })?
        .as_string()?;
    let mut result = s.to_string();
    result.insert_str(idx.min(result.len()), sub);
    Ok(Value::String(Rc::new(result)))
}

fn string_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let sub = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .as_string()?;
    Ok(Value::String(Rc::new(s.replacen(sub, "", 1))))
}

fn string_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let old = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 1,
        })?
        .as_string()?;
    let new = args
        .get(2)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 2,
        })?
        .as_string()?;
    let max_count = args
        .get(3)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize);
    let result = match max_count {
        Some(n) => {
            let mut r = s.to_string();
            for _ in 0..n {
                if let Some(pos) = r.find(old) {
                    r = format!("{}{}{}", &r[..pos], new, &r[pos + old.len()..]);
                } else {
                    break;
                }
            }
            r
        }
        None => s.replace(old, new),
    };
    Ok(Value::String(Rc::new(result)))
}

fn string_split(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let delim = args
        .get(1)
        .map(|v| v.as_string())
        .transpose()?
        .unwrap_or(" ");
    let max_count = args
        .get(2)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize);
    let parts: Vec<Value> = match max_count {
        Some(n) => s
            .splitn(n + 1, delim)
            .map(|p| Value::String(Rc::new(p.to_string())))
            .collect(),
        None => s
            .split(delim)
            .map(|p| Value::String(Rc::new(p.to_string())))
            .collect(),
    };
    Ok(Value::List(Rc::new(RefCell::new(parts))))
}

fn string_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    let s = args[0].as_string()?;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len() as i64;
    let from = match args.get(1) {
        Some(Value::Number(n)) => {
            let i = *n as i64;
            if i < 0 {
                (len + i).max(0) as usize
            } else {
                i.min(len) as usize
            }
        }
        _ => 0,
    };
    let to = match args.get(2) {
        Some(Value::Number(n)) => {
            let i = *n as i64;
            if i < 0 {
                (len + i).max(0) as usize
            } else {
                i.min(len) as usize
            }
        }
        _ => len as usize,
    };
    if from >= to {
        Ok(Value::String(Rc::new(String::new())))
    } else {
        Ok(Value::String(Rc::new(chars[from..to].iter().collect())))
    }
}

// ═══════════════════════════════════════════════════════════════
//  LIST METHODS — args[0] = self (the list)
// ═══════════════════════════════════════════════════════════════

fn as_list(args: &[Value]) -> Result<Rc<RefCell<Vec<Value>>>, RuntimeError> {
    match &args[0] {
        Value::List(items) => Ok(items.clone()),
        other => Err(RuntimeError::TypeError {
            expected: "list".into(),
            got: other.type_name().into(),
        }),
    }
}

fn list_len(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Number(as_list(args)?.borrow().len() as f64))
}

fn list_sum(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let total: f64 = items
        .borrow()
        .iter()
        .map(|v| v.as_number().unwrap_or(0.0))
        .sum();
    Ok(Value::Number(total))
}

fn list_has_index(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let idx = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .as_number()? as i64;
    Ok(Value::Bool(
        idx >= 0 && (idx as usize) < items.borrow().len(),
    ))
}

fn list_indexes(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let len = items.borrow().len();
    let idxs: Vec<Value> = (0..len).map(|i| Value::Number(i as f64)).collect();
    Ok(Value::List(Rc::new(RefCell::new(idxs))))
}

fn list_values(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let cloned = items.borrow().clone();
    Ok(Value::List(Rc::new(RefCell::new(cloned))))
}

fn list_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let needle = args.get(1).ok_or(RuntimeError::WrongArgCount {
        expected: 2,
        got: 1,
    })?;
    let after = args
        .get(2)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize)
        .unwrap_or(0);
    let v = items.borrow();
    for (i, item) in v.iter().enumerate().skip(after) {
        if item.equals(needle) {
            return Ok(Value::Number(i as f64));
        }
    }
    Ok(Value::Null)
}

fn list_insert(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let idx = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 1,
        })?
        .as_number()? as usize;
    let val = args
        .get(2)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 3,
            got: 2,
        })?
        .clone();
    let mut v = items.borrow_mut();
    let idx = idx.min(v.len());
    v.insert(idx, val);
    Ok(Value::Null)
}

fn list_join(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let sep = args
        .get(1)
        .map(|v| v.as_string())
        .transpose()?
        .unwrap_or(" ");
    let v = items.borrow();
    let result: Vec<String> = v.iter().map(|item| item.to_display_string()).collect();
    Ok(Value::String(Rc::new(result.join(sep))))
}

fn list_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let val = items.borrow_mut().pop().unwrap_or(Value::Null);
    Ok(val)
}

fn list_pull(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let mut v = items.borrow_mut();
    if v.is_empty() {
        Ok(Value::Null)
    } else {
        Ok(v.remove(0))
    }
}

fn list_push(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let val = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .clone();
    items.borrow_mut().push(val);
    Ok(Value::Null)
}

fn list_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let idx = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .as_number()? as usize;
    let mut v = items.borrow_mut();
    if idx < v.len() {
        v.remove(idx);
    }
    Ok(Value::Null)
}

fn list_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let old = args.get(1).ok_or(RuntimeError::WrongArgCount {
        expected: 3,
        got: 1,
    })?;
    let new = args.get(2).ok_or(RuntimeError::WrongArgCount {
        expected: 3,
        got: 2,
    })?;
    let max_count = args
        .get(3)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize);
    let mut v = items.borrow_mut();
    let mut count = 0;
    for item in v.iter_mut() {
        if item.equals(old) {
            *item = new.clone();
            count += 1;
            if let Some(max) = max_count {
                if count >= max {
                    break;
                }
            }
        }
    }
    Ok(Value::Null)
}

#[cfg(feature = "rand")]
fn list_shuffle(args: &[Value]) -> Result<Value, RuntimeError> {
    use rand::seq::SliceRandom;

    let items = as_list(args)?;
    let mut rng = rand::rng();
    items.borrow_mut().shuffle(&mut rng);

    Ok(Value::Null)
}

fn list_sort(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    // args[1] = key (unused for now), args[2] = ascending (default true)
    let ascending = args.get(2).map(|v| v.is_truthy()).unwrap_or(true);
    items.borrow_mut().sort_by(|a, b| {
        let na = a.as_number().unwrap_or(f64::NAN);
        let nb = b.as_number().unwrap_or(f64::NAN);
        let ord = na.partial_cmp(&nb).unwrap_or(std::cmp::Ordering::Equal);
        if ascending { ord } else { ord.reverse() }
    });
    Ok(Value::Null)
}

fn list_slice(args: &[Value]) -> Result<Value, RuntimeError> {
    let items = as_list(args)?;
    let v = items.borrow();
    let len = v.len() as i64;
    let from = match args.get(1) {
        Some(Value::Number(n)) => {
            let i = *n as i64;
            if i < 0 {
                (len + i).max(0) as usize
            } else {
                i.min(len) as usize
            }
        }
        _ => 0,
    };
    let to = match args.get(2) {
        Some(Value::Number(n)) => {
            let i = *n as i64;
            if i < 0 {
                (len + i).max(0) as usize
            } else {
                i.min(len) as usize
            }
        }
        _ => len as usize,
    };
    if from >= to {
        Ok(Value::List(Rc::new(RefCell::new(Vec::new()))))
    } else {
        Ok(Value::List(Rc::new(RefCell::new(v[from..to].to_vec()))))
    }
}

// ═══════════════════════════════════════════════════════════════
//  MAP METHODS — args[0] = self (the map)
// ═══════════════════════════════════════════════════════════════

fn as_map(args: &[Value]) -> Result<Rc<RefCell<HashMap<String, Value>>>, RuntimeError> {
    match &args[0] {
        Value::Map(m) => Ok(m.clone()),
        other => Err(RuntimeError::TypeError {
            expected: "map".into(),
            got: other.type_name().into(),
        }),
    }
}

fn map_len(args: &[Value]) -> Result<Value, RuntimeError> {
    Ok(Value::Number(as_map(args)?.borrow().len() as f64))
}

fn map_has_index(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let key = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .to_display_string();
    let has = m.borrow().contains_key(&key);
    Ok(Value::Bool(has))
}

fn map_indexes(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let keys: Vec<Value> = m
        .borrow()
        .keys()
        .map(|k| Value::String(Rc::new(k.clone())))
        .collect();
    Ok(Value::List(Rc::new(RefCell::new(keys))))
}

fn map_values(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let vals: Vec<Value> = m.borrow().values().cloned().collect();
    Ok(Value::List(Rc::new(RefCell::new(vals))))
}

fn map_index_of(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let needle = args.get(1).ok_or(RuntimeError::WrongArgCount {
        expected: 2,
        got: 1,
    })?;
    let borrowed = m.borrow();
    for (k, v) in borrowed.iter() {
        if v.equals(needle) {
            return Ok(Value::String(Rc::new(k.clone())));
        }
    }
    Ok(Value::Null)
}

fn map_pop(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let mut borrowed = m.borrow_mut();
    let key = borrowed.keys().next().cloned();
    match key {
        Some(k) => {
            borrowed.remove(&k);
            Ok(Value::String(Rc::new(k)))
        }
        None => Ok(Value::Null),
    }
}

fn map_push(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let key = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .to_display_string();
    m.borrow_mut().insert(key, Value::Number(1.0));
    Ok(Value::Null)
}

fn map_remove(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let key = args
        .get(1)
        .ok_or(RuntimeError::WrongArgCount {
            expected: 2,
            got: 1,
        })?
        .to_display_string();
    let removed = m.borrow_mut().remove(&key);
    Ok(removed.unwrap_or(Value::Null))
}

fn map_replace(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let old = args.get(1).ok_or(RuntimeError::WrongArgCount {
        expected: 3,
        got: 1,
    })?;
    let new = args.get(2).ok_or(RuntimeError::WrongArgCount {
        expected: 3,
        got: 2,
    })?;
    let max_count = args
        .get(3)
        .and_then(|v| v.as_number().ok())
        .map(|n| n as usize);
    let mut borrowed = m.borrow_mut();
    let mut count = 0;
    for v in borrowed.values_mut() {
        if v.equals(old) {
            *v = new.clone();
            count += 1;
            if let Some(max) = max_count {
                if count >= max {
                    break;
                }
            }
        }
    }
    Ok(Value::Null)
}

#[cfg(feature = "rand")]
fn map_shuffle(args: &[Value]) -> Result<Value, RuntimeError> {
    use rand::seq::SliceRandom;

    let m = as_map(args)?;
    let mut map = m.borrow_mut();

    let keys: Vec<_> = map.keys().cloned().collect();

    let mut values: Vec<_> = map
        .values_mut()
        .map(|v| std::mem::replace(v, unsafe { std::mem::zeroed() }))
        .collect();

    values.shuffle(&mut rand::rng());

    for (key, value) in keys.into_iter().zip(values) {
        map.insert(key, value);
    }

    Ok(Value::Null)
}

fn map_sum(args: &[Value]) -> Result<Value, RuntimeError> {
    let m = as_map(args)?;
    let total: f64 = m
        .borrow()
        .values()
        .map(|v| v.as_number().unwrap_or(0.0))
        .sum();
    Ok(Value::Number(total))
}
