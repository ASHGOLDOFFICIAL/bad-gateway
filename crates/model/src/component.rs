mod damage;
mod entity;
pub mod mecha;
mod physics;
mod shooting;
mod utils;

pub use damage::*;
pub use entity::*;
pub use physics::*;
pub use shooting::*;

pub(crate) type ComponentResult<T> = Result<T, &'static str>;
