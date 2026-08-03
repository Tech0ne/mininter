use std::fmt::Display;

#[derive(Debug)]
pub struct Token<T> {
    pub kind: T,
    pub line: usize,
    pub col: usize,
}

impl<T> Token<T> {
    pub fn new(kind: T, line: usize, col: usize) -> Self {
        Self { kind, line, col }
    }
}

impl<T> Display for Token<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (at {}:{})", self.kind, self.line, self.col)
    }
}
