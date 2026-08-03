use std::collections::HashMap;

use super::object::Object;

pub struct Context {
    values: HashMap<String, Object>
}
