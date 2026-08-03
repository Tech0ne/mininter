use std::fmt::Display;

#[derive(Debug, Default)]
pub enum Object {
    #[default]
    Null,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
