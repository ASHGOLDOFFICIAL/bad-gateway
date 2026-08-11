use crate::{ValidationError, ValidationResult, ops};

/// Temperature, dimension Θ.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Temperature(f32);

impl Temperature {
    /// `Temperature` of zero kelvins, absolute zero.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable temperature.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Temperature` from the specified [`f32`], in kelvins.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `Temperature` or not finite.
    #[inline(always)]
    pub fn from_kelvins_f32(value: f32) -> Self {
        Self::try_from_kelvins_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_kelvins_f32`](Self::from_kelvins_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Temperature` or not finite.
    #[inline]
    pub const fn try_from_kelvins_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError(
                "temperature must be finite and non-negative kelvins",
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a new `Temperature` from the specified [`f32`], in degrees
    /// Celsius.
    ///
    /// # Panics
    /// This constructor will panic if value is colder than absolute zero,
    /// overflows `Temperature` or not finite.
    #[inline(always)]
    pub fn from_celsius_f32(value: f32) -> Self {
        Self::try_from_celsius_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_celsius_f32`](Self::from_celsius_f32).
    ///
    /// This constructor will return an `Err` if value is colder than
    /// absolute zero, overflows `Temperature` or not finite.
    #[inline(always)]
    pub const fn try_from_celsius_f32(value: f32) -> ValidationResult<Self> {
        Self::try_from_kelvins_f32(value + 273.15)
    }

    /// Returns this `Temperature` as [`f32`], in kelvins.
    #[inline(always)]
    pub const fn as_kelvins_f32(&self) -> f32 {
        self.0
    }

    /// Returns this `Temperature` as [`f32`], in degrees Celsius.
    #[inline(always)]
    pub const fn as_celsius_f32(&self) -> f32 {
        self.0 - 273.15
    }

    /// Returns `true` if this `Temperature` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }
}

impl Eq for Temperature {}

impl PartialOrd for Temperature {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Temperature {
    /// Compares two temperatures.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("temperature is always finite, so a total order exists")
    }
}

impl ops::Validated for Temperature {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_kelvins_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_kelvins_f32(value).ok()
    }
}

impl ops::UpperBounded for Temperature {
    const MAX: Self = Self::MAX;
}

impl ops::LowerBounded for Temperature {
    const MIN: Self = Self::ZERO;
}

impl ops::NonNegative for Temperature {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_kelvins_rejects_negative() {
        assert!(Temperature::try_from_kelvins_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_kelvins_rejects_non_finite() {
        assert!(Temperature::try_from_kelvins_f32(f32::NAN).is_err());
        assert!(Temperature::try_from_kelvins_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_kelvins_accepts_non_negative() {
        assert_eq!(Temperature::from_kelvins_f32(0.0), Temperature::ZERO);
        assert_eq!(Temperature::from_kelvins_f32(300.0).as_kelvins_f32(), 300.0);
    }

    #[test]
    fn from_celsius_converts_to_kelvins() {
        let temperature = Temperature::from_celsius_f32(0.0);
        assert_eq!(temperature.as_kelvins_f32(), 273.15);
    }
}
