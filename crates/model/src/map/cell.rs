use physics::{Length, ops::Delta};

use crate::map::{Liquid, Structure, Surface};

/// Slice of terrain. Contains three layers: surface, structure on top of it,
/// and liquid. Also contains surface elevation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Cell {
    pub elevation: Length,
    pub surface: Surface,
    pub structure: Option<Structure>,
    pub liquid: Option<Liquid>,
}

impl Cell {
    /// How deep this cell's liquid sits above its ground elevation.
    /// Returns [`None`] when there's no liquid above surface.
    pub fn liquid_depth(&self) -> Option<Length> {
        let liquid = self.liquid?;
        match Delta::between(liquid.level, self.elevation) {
            Delta::Positive(depth) => Some(depth),
            Delta::Zero | Delta::Negative(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::LiquidType;

    fn cell(elevation_m: f32, liquid_level: Option<f32>) -> Cell {
        Cell {
            surface: Surface::Beach,
            elevation: Length::from_meters_f32(elevation_m),
            structure: None,
            liquid: liquid_level.map(|level| Liquid {
                liquid_type: LiquidType::Water,
                level: Length::from_meters_f32(level),
            }),
        }
    }

    #[test]
    fn liquid_depth_is_some_when_liquid_sits_above_ground() {
        let submerged = cell(1.0, Some(3.0));
        assert_eq!(submerged.liquid_depth(), Some(Length::from_meters_f32(2.0)));
    }

    #[test]
    fn liquid_depth_is_none_when_no_liquid_present() {
        let dry = cell(1.0, None);
        assert_eq!(dry.liquid_depth(), None);
    }

    #[test]
    fn liquid_depth_is_none_when_liquid_level_exactly_matches_ground() {
        let at_ground = cell(5.0, Some(5.0));
        assert_eq!(at_ground.liquid_depth(), None);
    }

    #[test]
    fn liquid_depth_is_none_when_liquid_level_is_below_ground() {
        let below_ground = cell(5.0, Some(2.0));
        assert_eq!(below_ground.liquid_depth(), None);
    }
}
