use physics::{Power, ops::CheckedMul};

use crate::component::ComponentResult;

/// Indicates that this part is an engine with the ability
/// to use its rated [`Power`] to move.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Engine {
    power: Power,
    scale: f32,
}

impl Engine {
    /// Returns this `Engine`'s current [`Power`] value.
    #[inline]
    pub fn power(&self) -> Power {
        self.power
            .checked_mul(self.scale)
            .expect("scale is in (0, 1]")
    }

    /// Updates this `Engine`'s current [`Power`] scale.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        if !scale.is_finite() || scale <= 0.0 || scale > 1.0 {
            Err("scale must be in (0, 1]")
        } else {
            self.scale = scale;
            Ok(())
        }
    }

    /// Resets this `Engine`'s current [`Power`] scale to 1.
    #[inline(always)]
    pub fn reset_scale(&mut self) {
        self.scale = 1.0;
    }
}

impl From<Power> for Engine {
    fn from(value: Power) -> Self {
        Self {
            power: value,
            scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine(watts: f32) -> Engine {
        Engine::from(Power::from_watts_f32(watts))
    }

    #[test]
    fn power_reflects_current_scale() {
        let mut engine: Engine = engine(1.0);
        engine.set_scale(0.5).unwrap();
        assert_eq!(engine.power(), Power::from_watts_f32(0.5));

        engine.reset_scale();
        assert_eq!(engine.power(), Power::from_watts_f32(1.0));
    }

    #[test]
    fn set_scale_rejects_out_of_range() {
        let mut engine = engine(1.0);
        assert!(engine.set_scale(0.0).is_err());
        assert!(engine.set_scale(1.1).is_err());
        assert!(engine.set_scale(-0.1).is_err());
    }

    #[test]
    fn set_scale_rejects_non_finite() {
        let mut engine = engine(1.0);
        assert!(engine.set_scale(f32::NAN).is_err());
        assert!(engine.set_scale(f32::INFINITY).is_err());
    }
}
