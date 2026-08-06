use std::cmp::Ordering;
// use std::error::Error;
use std::fmt::Display;
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use crate::error::Error;

#[derive(Clone, Copy, Debug, Default)]
pub struct Float(f64);

impl Float {
    pub fn new(x: f64) -> Self {
        Self(x)
    }

    pub fn get(self) -> f64 {
        self.0
    }

    fn canonical_bits(self) -> u64 {
        if self.0.is_nan() {
            // Canonical quiet NaN
            0x7ffc_0000_0000_0000
        } else if self.0 == 0.0 {
            // Merge +0 and -0
            0
        } else {
            self.0.to_bits()
        }
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialEq for Float {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => true,
            _ => self.0 == other.0,
        }
    }
}

impl Eq for Float {}

impl Hash for Float {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.canonical_bits().hash(state);
    }
}

impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Float {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.0.is_nan(), other.0.is_nan()) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Greater,
            (false, true) => Ordering::Less,
            (false, false) => self.0.partial_cmp(&other.0).unwrap(),
        }
    }
}

impl FromStr for Float {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<f64>()
            .map(Self::new)
            .map_err(|e| Error::parse(e.to_string(), None))
    }
}
