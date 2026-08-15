use crate::component::ComponentResult;

/// Indicates that the user or the system request to activate
/// all non-disabled thrusters at given force scale.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ThrustersActive(f32);

impl ThrustersActive {
    /// Scale must be in (0, 1].
    #[inline]
    pub const fn new(scale: f32) -> ComponentResult<Self> {
        if !scale.is_finite() || scale <= 0.0 || 1.0 < scale {
            Err("thrusters scale should be in (0, 1]")
        } else {
            Ok(Self(scale))
        }
    }

    /// This `ThrustersActive`'s thruster output scale.
    #[inline(always)]
    pub const fn scale(&self) -> f32 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_scale_out_of_range() {
        assert!(ThrustersActive::new(f32::INFINITY).is_err());
        assert!(ThrustersActive::new(f32::NAN).is_err());
        assert!(ThrustersActive::new(0.0).is_err());
        assert!(ThrustersActive::new(1.1).is_err());
        assert!(ThrustersActive::new(-0.1).is_err());
    }

    #[test]
    fn new_accepts_scale_within_range() {
        assert_eq!(ThrustersActive::new(1.0).unwrap().scale(), 1.0);
        assert_eq!(ThrustersActive::new(0.5).unwrap().scale(), 0.5);
    }
}
