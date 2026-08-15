use crate::component::ComponentResult;

/// Indicates that this part has the ability
/// to brake by using part of the traction.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Brake {
    scale: f32,
}

impl Brake {
    /// Creates new `Brake` from given `scale`. Must be in [0, 1].
    #[inline]
    pub const fn new(scale: f32) -> ComponentResult<Self> {
        if !scale.is_finite() || scale < 0.0 || scale > 1.0 {
            Err("scale must be in [0, 1]")
        } else {
            Ok(Self { scale })
        }
    }

    /// Returns this `Brake`'s current braking scale.
    #[inline(always)]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// Updates this `Brake`'s current scale. Must be in [0, 1].
    #[inline]
    pub const fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        if !scale.is_finite() || scale < 0.0 || scale > 1.0 {
            Err("scale must be in [0, 1]")
        } else {
            self.scale = scale;
            Ok(())
        }
    }

    /// Resets this `Brake`'s current braking scale to 0.
    #[inline(always)]
    pub const fn reset_scale(&mut self) {
        self.scale = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_scale_out_of_range() {
        assert!(Brake::new(f32::INFINITY).is_err());
        assert!(Brake::new(f32::NAN).is_err());
        assert!(Brake::new(1.1).is_err());
        assert!(Brake::new(-0.1).is_err());
    }

    #[test]
    fn new_accepts_scale_within_range() {
        assert_eq!(Brake::new(0.0).unwrap().scale(), 0.0);
        assert_eq!(Brake::new(1.0).unwrap().scale(), 1.0);
        assert_eq!(Brake::new(0.5).unwrap().scale(), 0.5);
    }

    #[test]
    fn set_scale_rejects_out_of_range() {
        let mut brake = Brake::new(0.0).unwrap();
        assert!(brake.set_scale(1.1).is_err());
        assert!(brake.set_scale(-0.1).is_err());
    }

    #[test]
    fn reset_scale_sets_to_zero() {
        let mut brake = Brake::new(1.0).unwrap();
        brake.reset_scale();
        assert_eq!(brake.scale(), 0.0);
    }
}
