use std::{fmt::Display, rc::Rc};

use crate::core::number::Float;

#[derive(Debug, Clone)]
pub enum Value {
    Number(Float),
    Boolean(bool),
    String(Rc<str>),
    List(Rc<Vec<Value>>),
}

// #region From

impl Value {
    pub fn number(f: Float) -> Self {
        Self::Number(f)
    }

    pub fn boolean(b: bool) -> Self {
        Self::Boolean(b)
    }

    pub fn string(s: impl AsRef<str>) -> Self {
        Self::String(Rc::from(s.as_ref()))
    }

    pub fn list<T>(v: Vec<T>) -> Self
    where
        T: Into<Value>,
    {
        Self::List(Rc::new(v.into_iter().map(Into::into).collect()))
    }
}

impl From<Float> for Value {
    fn from(value: Float) -> Self {
        Self::number(value)
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::boolean(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::string(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::string(value)
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(value: Vec<T>) -> Self {
        Self::list(value)
    }
}

// #endregion

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Boolean(b) => write!(f, "{}", b),
            Self::String(s) => write!(f, "{}", s),
            Self::List(l) => {
                write!(f, "[")?;

                for (i, value) in l.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{value}")?;
                }

                write!(f, "]")
            }
        }
    }
}
