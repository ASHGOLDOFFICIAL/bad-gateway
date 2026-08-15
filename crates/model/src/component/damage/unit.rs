use physics::{
    ops::{SaturatingAdd, SaturatingSub},
    traits::NonNegative,
};

use crate::component::ComponentResult;

/// Damage units that reduce integrity.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::Into)]
pub struct DamageUnit(f32);

impl DamageUnit {
    pub const ZERO: Self = Self(0.0);

    pub const MAX: Self = Self(f32::MAX);

    /// Returns `true` if this `DamageUnit` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Multiplies this `DamageUnit` by the given value.
    ///
    /// Errors if `rhs` is not finite or negative,
    /// or if resulting product is not finite.
    #[inline]
    pub const fn checked_mul(self, rhs: f32) -> ComponentResult<Self> {
        if rhs < 0.0 || !rhs.is_finite() {
            return Err("coefficient must be finite and non-negative");
        }
        let product = self.0 * rhs;
        if !product.is_finite() {
            Err("result overflowed")
        } else {
            Ok(Self(product))
        }
    }
}

impl Eq for DamageUnit {}

impl PartialOrd for DamageUnit {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DamageUnit {
    /// Compares two damage units.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("damage unit is always finite, so a total order exists")
    }
}

impl TryFrom<f32> for DamageUnit {
    type Error = &'static str;

    /// Given value must be finite and non-negative.
    #[inline]
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 0.0 || !value.is_finite() {
            Err("damage units must be finite and non-negative")
        } else {
            Ok(Self(value))
        }
    }
}

impl NonNegative for DamageUnit {}

impl SaturatingAdd for DamageUnit {
    type Output = Self;

    #[inline]
    fn saturating_add(self, rhs: Self) -> Self::Output {
        Self((self.0 + rhs.0).min(Self::MAX.0))
    }
}

impl SaturatingSub for DamageUnit {
    type Output = Self;

    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self::Output {
        Self((self.0 - rhs.0).max(0.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_f32_rejects_negative() {
        assert!(DamageUnit::try_from(-1.0).is_err());
    }

    #[test]
    fn try_from_f32_rejects_non_finite() {
        assert!(DamageUnit::try_from(f32::NAN).is_err());
        assert!(DamageUnit::try_from(f32::INFINITY).is_err());
    }

    #[test]
    fn try_from_f32_accepts_non_negative() {
        assert_eq!(f32::from(DamageUnit::try_from(0.0).unwrap()), 0.0);
        assert_eq!(f32::from(DamageUnit::try_from(5.0).unwrap()), 5.0);
    }

    #[test]
    fn checked_mul_rejects_negative() {
        let a = DamageUnit::try_from(1.0).unwrap();
        assert!(a.checked_mul(-1.0).is_err());
    }

    #[test]
    fn checked_mul_rejects_non_finite() {
        let a = DamageUnit::try_from(1.0).unwrap();
        assert!(a.checked_mul(f32::INFINITY).is_err());
        assert!(a.checked_mul(f32::NAN).is_err());
    }

    #[test]
    fn checked_mul_scales_values() {
        let a = DamageUnit::try_from(1.0).unwrap();
        let b = DamageUnit::try_from(3.0).unwrap();
        assert_eq!(a.checked_mul(3.0).unwrap(), b);
    }

    #[test]
    fn checked_mul_errors_on_overflow() {
        let a = DamageUnit::try_from(f32::MAX).unwrap();
        assert!(a.checked_mul(f32::MAX).is_err());
    }

    #[test]
    fn saturating_add_sums_values() {
        let a = DamageUnit::try_from(1.0).unwrap();
        let b = DamageUnit::try_from(2.0).unwrap();
        let c = DamageUnit::try_from(3.0).unwrap();
        assert_eq!(a.saturating_add(b), c);
    }

    #[test]
    fn saturating_add_clamps_at_max() {
        let a = DamageUnit::try_from(f32::MAX).unwrap();
        let b = DamageUnit::try_from(f32::MAX).unwrap();
        assert_eq!(a.saturating_add(b), DamageUnit::MAX);
    }
}
