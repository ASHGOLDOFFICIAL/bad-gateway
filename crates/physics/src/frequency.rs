use std::time::Duration;

use crate::{
    CalculationError, CalculationResult, ValidationError, ValidationResult,
    traits::{UpperBounded, Validated},
};

/// Frequency, dimension T⁻¹ (reciprocal time).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frequency(f32);

impl Frequency {
    /// The largest representable frequency.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Frequency` from the specified [`f32`], in hertz.
    ///
    /// # Panics
    /// This constructor will panic if value is not positive, overflows
    /// `Frequency` or not finite.
    #[inline(always)]
    pub fn from_hertz_f32(value: f32) -> Self {
        Self::try_from_hertz_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_hertz_f32`](Self::from_hertz_f32).
    ///
    /// This constructor will return an `Err` if value is not positive,
    /// overflows `Frequency` or not finite.
    #[inline]
    pub const fn try_from_hertz_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            Err(ValidationError("frequency must be finite and positive"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Frequency` as [`f32`], in hertz.
    #[inline(always)]
    pub const fn as_hertz_f32(&self) -> f32 {
        self.0
    }

    /// Derives [`Duration`] from this `Frequency`'s period.
    ///
    /// It uses `t = 1 / f`, where
    /// - `t` is duration,
    /// - `f` is this frequency.
    ///
    /// Errors if the period would exceed [`Duration`]'s representable range.
    #[inline]
    pub fn period(&self) -> CalculationResult<Duration> {
        Duration::try_from_secs_f32(1.0 / self.0).map_err(|_| {
            CalculationError::Overflow("period exceeds the maximum representable duration")
        })
    }
}

impl Eq for Frequency {}

impl PartialOrd for Frequency {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Frequency {
    /// Compares two frequencies.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("frequency is always finite, so a total order exists")
    }
}

impl Validated for Frequency {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_hertz_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_hertz_f32(value).ok()
    }
}

impl UpperBounded for Frequency {
    const MAX: Self = Self::MAX;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_hertz_rejects_non_positive() {
        assert!(Frequency::try_from_hertz_f32(0.0).is_err());
        assert!(Frequency::try_from_hertz_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_hertz_rejects_non_finite() {
        assert!(Frequency::try_from_hertz_f32(f32::NAN).is_err());
        assert!(Frequency::try_from_hertz_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn period_is_inverse_of_frequency() {
        let value = 2.0;
        let frequency = Frequency::from_hertz_f32(value);
        assert_eq!(
            frequency.period().unwrap(),
            Duration::from_secs_f32(1.0 / value)
        );
    }

    #[test]
    fn period_errors_when_out_of_range() {
        let frequency = Frequency::from_hertz_f32(f32::MIN_POSITIVE);
        assert!(frequency.period().is_err());
    }
}
