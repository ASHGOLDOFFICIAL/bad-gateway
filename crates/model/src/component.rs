mod damage;
mod utils;

pub use damage::*;

pub(crate) type ComponentResult<T> = Result<T, &'static str>;
