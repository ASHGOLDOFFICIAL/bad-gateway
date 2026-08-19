use crate::component::ComponentResult;

/// Indicates that the user or the system request
/// to activate mecha's legs at given speed scale.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegsActive(f32);

impl LegsActive {
    /// Scale must be in (0, 1].
    #[inline]
    pub const fn new(scale: f32) -> ComponentResult<Self> {
        if !scale.is_finite() || scale <= 0.0 || 1.0 < scale {
            Err("legs speed scale should be in (0, 1]")
        } else {
            Ok(Self(scale))
        }
    }

    /// This `LegsActive`'s speed scale.
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
        assert!(LegsActive::new(f32::INFINITY).is_err());
        assert!(LegsActive::new(f32::NAN).is_err());
        assert!(LegsActive::new(0.0).is_err());
        assert!(LegsActive::new(1.1).is_err());
        assert!(LegsActive::new(-0.1).is_err());
    }

    #[test]
    fn new_accepts_scale_within_range() {
        assert_eq!(LegsActive::new(1.0).unwrap().scale(), 1.0);
        assert_eq!(LegsActive::new(0.5).unwrap().scale(), 0.5);
    }
}
