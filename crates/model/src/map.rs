mod cell;
mod liquid;
mod structure;
mod surface;

pub use cell::*;
pub use liquid::*;
pub use structure::*;
pub use surface::*;

use physics::{Density, Temperature};

/// Game's map.
pub trait Map {
    fn cell_at(&self, x: f32, y: f32) -> Cell;

    /// Ambient temperature across the map.
    fn temperature(&self) -> Temperature;

    /// Ambient air density across the map.
    fn air_density(&self) -> Density;

    /// Ground acceleration in m/s^2.
    fn gravity(&self) -> f32;
}
