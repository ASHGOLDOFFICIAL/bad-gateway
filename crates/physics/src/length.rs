use std::time::Duration;

use crate::{
    Area, CalculationError, CalculationResult, Speed, ValidationError, ValidationResult,
    traits::{Bounded, NonNegative, Validated},
};

/// Length, dimension L.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Length(f32);

impl Length {
    /// `Length` of zero meters.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable length.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Length` from the specified [`f32`], in meters.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `Length` or not finite.
    #[inline(always)]
    pub fn from_meters_f32(value: f32) -> Self {
        Self::try_from_meters_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_meters_f32`](Self::from_meters_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Length` or not finite.
    #[inline]
    pub const fn try_from_meters_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError("length must be finite and non-negative"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Length` as [`f32`], in meters.
    #[inline(always)]
    pub const fn as_meters_f32(&self) -> f32 {
        self.0
    }

    /// Returns `true` if this `Length` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives [`Area`] from this `Length` and `other`.
    ///
    /// It uses `A = l1 * l2`, where
    /// - `A` is area,
    /// - `l1` is this length,
    /// - `l2` is `other`.
    #[inline]
    pub fn area(self, other: Self) -> CalculationResult<Area> {
        Area::try_from_square_meters_f32(self.0 * other.0).map_err(CalculationError::from)
    }

    /// Derives [`Duration`] from this `Length` and given [`Speed`].
    ///
    /// It uses `t = l / v`, where
    /// - `t` is duration,
    /// - `l` is this length,
    /// - `v` is speed.
    ///
    /// Errors if `speed` is zero.
    #[inline]
    pub fn duration(self, speed: Speed) -> CalculationResult<Duration> {
        if speed.is_zero() {
            Err(CalculationError::InvalidArgument(
                "can divide by zero speed",
            ))
        } else {
            let secs = self.0 / speed.as_meters_per_second_f32();
            Duration::try_from_secs_f32(secs)
                .map_err(|_| CalculationError::Overflow("result overflowed"))
        }
    }
}

impl Eq for Length {}

impl PartialOrd for Length {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Length {
    /// Compares two lengths.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("length is always finite, so a total order exists")
    }
}

impl Validated for Length {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_meters_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_meters_f32(value).ok()
    }
}

impl Bounded for Length {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::ZERO;
}

impl NonNegative for Length {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_meters_rejects_negative() {
        assert!(Length::try_from_meters_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_meters_rejects_non_finite() {
        assert!(Length::try_from_meters_f32(f32::NAN).is_err());
        assert!(Length::try_from_meters_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_meters_accepts_non_negative() {
        assert!(Length::from_meters_f32(0.0).is_zero());
        assert_eq!(Length::from_meters_f32(5.0).as_meters_f32(), 5.0);
    }

    #[test]
    fn area_multiplies_two_lengths() {
        let a = Length::from_meters_f32(3.0);
        let b = Length::from_meters_f32(4.0);
        assert_eq!(a.area(b).unwrap().as_square_meters_f32(), 12.0);
    }

    #[test]
    fn duration_yields_travel_time() {
        let length = Length::from_meters_f32(100.0);
        let speed = Speed::from_meters_per_second_f32(20.0);
        assert_eq!(
            length.duration(speed).unwrap(),
            Duration::from_secs_f32(5.0)
        );
    }
}
