use physics::{ForceMagnitude, ops::CheckedMul};

use crate::component::{ComponentResult, Thrust};

/// Indicates that this part is a thruster with the ability
/// to use its rated [`Thrust`] to propel mecha forward.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thruster {
    thrust: Thrust,
    scale: f32,
}

impl Thruster {
    /// Returns this `Thruster`'s current [`Thrust`] value.
    #[inline]
    pub fn thrust(&self) -> Thrust {
        let rated: ForceMagnitude = self.thrust.into();
        rated
            .checked_mul(self.scale)
            .map(Thrust::from)
            .expect("scale is in [0, 1]")
    }

    /// Updates this `Thruster`'s current [`Thrust`] scale.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        if !scale.is_finite() || !(0.0..=1.0).contains(&scale) {
            Err("scale must be in [0, 1]")
        } else {
            self.scale = scale;
            Ok(())
        }
    }

    /// Resets this `Thruster`'s current [`Thrust`] scale to 0.
    #[inline(always)]
    pub fn reset_scale(&mut self) {
        self.scale = 0.0;
    }
}

impl From<Thrust> for Thruster {
    fn from(value: Thrust) -> Self {
        Self {
            thrust: value,
            scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn thruster(newtons: f32) -> Thruster {
        Thruster::from(Thrust::from(ForceMagnitude::from_newtons_f32(newtons)))
    }

    fn newtons(thrust: Thrust) -> f32 {
        ForceMagnitude::from(thrust).as_newtons_f32()
    }

    #[test]
    fn set_scale_rejects_out_of_range() {
        let mut thruster = thruster(1.0);
        assert!(thruster.set_scale(f32::INFINITY).is_err());
        assert!(thruster.set_scale(f32::NAN).is_err());
        assert!(thruster.set_scale(1.1).is_err());
        assert!(thruster.set_scale(-0.1).is_err());
    }

    #[test]
    fn set_scale_accepts_within_range() {
        let mut thruster = thruster(1.0);

        thruster.set_scale(0.0).unwrap();
        assert_eq!(newtons(thruster.thrust()), 0.0);

        thruster.set_scale(0.5).unwrap();
        assert_eq!(newtons(thruster.thrust()), 0.5);

        thruster.set_scale(1.0).unwrap();
        assert_eq!(newtons(thruster.thrust()), 1.0);
    }

    #[test]
    fn thrust_reflects_current_scale() {
        let mut thruster = thruster(1.0);
        thruster.set_scale(0.5).unwrap();
        assert_eq!(newtons(thruster.thrust()), 0.5);

        thruster.reset_scale();
        assert_eq!(newtons(thruster.thrust()), 0.0);
    }
}
