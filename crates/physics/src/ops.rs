//! Generic arithmetic traits shared across physics quantities.
//!
//! Checked traits return `None` instead of an invalid value. Saturating traits
//! clamp into range instead. Delta allows signed representation of non-negative
//! quantities.

mod checked;
mod delta;
mod saturating;

pub use checked::*;
pub use delta::*;
pub use saturating::*;

#[cfg(test)]
mod tests {
    use crate::traits::{Bounded, NonNegative, Validated};

    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Quantity(f32);

    impl Validated for Quantity {
        type Repr = f32;

        fn as_repr(&self) -> f32 {
            self.0
        }

        fn validate(value: f32) -> Option<Self> {
            (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(Self(value))
        }
    }

    impl Bounded for Quantity {
        const MAX: Self = Self(100.0);
        const MIN: Self = Self(0.0);
    }

    impl NonNegative for Quantity {}

    #[test]
    fn checked_add_rejects_invalid_result() {
        assert!(Quantity(f32::MAX).checked_add(Quantity(f32::MAX)).is_none());
    }

    #[test]
    fn checked_add_sums_valid_result() {
        assert_eq!(
            Quantity(1.0).checked_add(Quantity(2.0)),
            Some(Quantity(3.0))
        );
    }

    #[test]
    fn checked_sub_rejects_invalid_result() {
        assert!(Quantity(1.0).checked_sub(Quantity(2.0)).is_none());
    }

    #[test]
    fn checked_sub_accepts_valid_result() {
        assert_eq!(
            Quantity(5.0).checked_sub(Quantity(2.0)),
            Some(Quantity(3.0))
        );
    }

    #[test]
    fn checked_mul_f32_rejects_invalid_result() {
        assert!(Quantity(10.0).checked_mul(-1.0).is_none());
    }

    #[test]
    fn checked_mul_f32_scales_by_coefficient() {
        assert_eq!(Quantity(10.0).checked_mul(2.0), Some(Quantity(20.0)));
    }

    #[test]
    fn saturating_add_sums_valid_result() {
        assert_eq!(Quantity(1.0).saturating_add(Quantity(2.0)), Quantity(3.0));
    }

    #[test]
    fn saturating_add_clamps_at_max() {
        assert_eq!(Quantity::MAX.saturating_add(Quantity(1.0)), Quantity::MAX);
    }

    #[test]
    fn saturating_sub_clamps_at_min() {
        assert_eq!(Quantity(1.0).saturating_sub(Quantity(2.0)), Quantity::MIN);
    }
}
