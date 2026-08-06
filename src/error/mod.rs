mod builder;
use builder::errors;

errors!(Parse);

pub type Result<T> = core::result::Result<T, Error>;
