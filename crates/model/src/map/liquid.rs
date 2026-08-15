use physics::{Density, Length};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LiquidType {
    Water,
}

/// A body of liquid covering a cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Liquid {
    pub liquid_type: LiquidType,
    pub level: Length,
}

impl Liquid {
    /// Makes new `Liquid` from the given values.
    pub fn new(liquid_type: LiquidType, level: Length) -> Self {
        Self { liquid_type, level }
    }

    /// This `Liquid`'s [`Density`].
    pub fn density(&self) -> Density {
        match self.liquid_type {
            LiquidType::Water => Density::from_kilograms_per_cubic_meter_f32(1000.0),
        }
    }
}
