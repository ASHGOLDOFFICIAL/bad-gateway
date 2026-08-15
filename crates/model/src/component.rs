mod damage;
mod shooting;
mod utils;

pub use damage::*;
pub use shooting::*;

pub(crate) type ComponentResult<T> = Result<T, &'static str>;
