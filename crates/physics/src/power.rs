use std::time::Duration;

use crate::{
    CalculationError, CalculationResult, Energy, ValidationError, ValidationResult,
    traits::{Bounded, NonNegative, Validated},
};

/// Power, dimension ML²T⁻³ (mass times length squared per time cubed).
#[must_use]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Power(f32);

impl Power {
    /// `Power` of zero watts.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable power.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Power` from the specified [`f32`], in watts.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows `Power`
    /// or not finite.
    #[inline(always)]
    pub fn from_watts_f32(value: f32) -> Self {
        Self::try_from_watts_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_watts_f32`](Self::from_watts_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Power` or not finite.
    #[inline]
    pub const fn try_from_watts_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError("power must be finite and non-negative"))
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a new `Power` from the specified [`f32`], in kilowatts.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows `Power`
    /// or not finite.
    #[inline(always)]
    pub fn from_kilowatts_f32(value: f32) -> Self {
        Self::try_from_kilowatts_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_kilowatts_f32`](Self::from_kilowatts_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Power` or not finite.
    #[inline(always)]
    pub const fn try_from_kilowatts_f32(value: f32) -> ValidationResult<Self> {
        Self::try_from_watts_f32(value * 1_000.0)
    }

    /// Returns this `Power` as [`f32`], in watts.
    #[inline(always)]
    pub const fn as_watts_f32(&self) -> f32 {
        self.0
    }

    /// Returns this `Power` as [`f32`], in kilowatts.
    #[inline(always)]
    pub const fn as_kilowatts_f32(&self) -> f32 {
        self.0 / 1_000.0
    }

    /// Returns `true` if this `Power` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives [`Energy`] from this `Power` sustained over given [`Duration`].
    ///
    /// It uses `E = P * dt`, where
    /// - `E` is energy,
    /// - `P` is this power,
    /// - `dt` is duration.
    #[inline]
    pub fn energy(self, duration: Duration) -> CalculationResult<Energy> {
        Energy::try_from_joules_f32(self.as_watts_f32() * duration.as_secs_f32())
            .map_err(CalculationError::from)
    }
}

impl Eq for Power {}

impl PartialOrd for Power {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Power {
    /// Compares two powers.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("power is always finite, so a total order exists")
    }
}

impl Validated for Power {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_watts_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_watts_f32(value).ok()
    }
}

impl Bounded for Power {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::ZERO;
}

impl NonNegative for Power {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_watts_rejects_negative() {
        assert!(Power::try_from_watts_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_watts_rejects_non_finite() {
        assert!(Power::try_from_watts_f32(f32::NAN).is_err());
        assert!(Power::try_from_watts_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_watts_accepts_non_negative() {
        assert!(Power::from_watts_f32(0.0).is_zero());
        assert_eq!(Power::from_watts_f32(5.0).as_watts_f32(), 5.0);
    }

    #[test]
    fn from_kilowatts_converts_to_watts() {
        assert_eq!(Power::from_kilowatts_f32(1.0).as_watts_f32(), 1_000.0);
        assert_eq!(Power::from_kilowatts_f32(1.0).as_kilowatts_f32(), 1.0);
    }

    #[test]
    fn energy_multiplies_by_duration() {
        let power = Power::from_watts_f32(10.0);
        let energy = power.energy(Duration::from_secs(2)).unwrap();
        assert_eq!(energy.as_joules_f32(), 20.0);
    }
}
