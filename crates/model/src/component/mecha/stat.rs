//! Contains stats used by systems operating on [`Mecha`](super::Mecha) and its
//! [`Part`](super::part::Part)s

mod energy_consumption;
mod heat_output;
mod temperature;

pub use energy_consumption::*;
pub use heat_output::*;
pub use temperature::*;
