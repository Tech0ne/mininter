use std::cell::RefMut;

use rand::{RngExt, SeedableRng, rngs::StdRng};

use crate::{RuntimeError, Value};

pub fn rnd(args: &[Value], mut rng: RefMut<StdRng>) -> Result<Value, RuntimeError> {
    Ok(Value::Number(match args.first() {
        Some(n) => {
            *rng = StdRng::seed_from_u64(n.as_number().unwrap_or(0.0).to_bits());
            rng.random_range(0.0..1.0)
        }
        None => rng.random_range(0.0..1.0),
    }))
}
