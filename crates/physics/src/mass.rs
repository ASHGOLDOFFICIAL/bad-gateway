use crate::{ValidationError, ValidationResult, ops};

/// Mass, dimension M.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mass(f32);

impl Mass {
    /// `Mass` of zero kilograms.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable mass.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Mass` from the specified [`f32`], in kilograms.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows `Mass`
    /// or not finite.
    #[inline(always)]
    pub fn from_kilograms_f32(value: f32) -> Self {
        Self::try_from_kilograms_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_kilograms_f32`](Self::from_kilograms_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Mass` or not finite.
    #[inline]
    pub const fn try_from_kilograms_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError("mass must be finite and non-negative"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Mass` as [`f32`], in kilograms.
    #[inline(always)]
    pub const fn as_kilograms_f32(&self) -> f32 {
        self.0
    }

    /// Returns `true` if this `Mass` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }
}

impl Eq for Mass {}

impl PartialOrd for Mass {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Mass {
    /// Compares two masses.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("mass is always finite, so a total order exists")
    }
}

impl ops::Validated for Mass {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_kilograms_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_kilograms_f32(value).ok()
    }
}

impl ops::UpperBounded for Mass {
    const MAX: Self = Self::MAX;
}

impl ops::LowerBounded for Mass {
    const MIN: Self = Self::ZERO;
}

impl ops::NonNegative for Mass {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_kilogram_rejects_negative() {
        assert!(Mass::try_from_kilograms_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_kilogram_rejects_non_finite() {
        assert!(Mass::try_from_kilograms_f32(f32::NAN).is_err());
        assert!(Mass::try_from_kilograms_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn try_from_kilogram_accepts_zero() {
        let mass = Mass::try_from_kilograms_f32(0.0).unwrap();
        assert!(mass.is_zero());
    }

    #[test]
    fn try_from_kilogram_accepts_positive() {
        let mass = Mass::try_from_kilograms_f32(10.0).unwrap();
        assert_eq!(mass.as_kilograms_f32(), 10.0);
    }
}
