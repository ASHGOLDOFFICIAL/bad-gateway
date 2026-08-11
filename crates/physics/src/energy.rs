use crate::{ValidationError, ValidationResult, ops};

/// Energy, dimension ML²T⁻² (mass times length squared per time squared).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Energy(f32);

impl Energy {
    /// `Energy` of zero joules.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable energy.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Energy` from the specified [`f32`], in joules.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `Energy` or not finite.
    #[inline(always)]
    pub fn from_joules_f32(value: f32) -> Self {
        Self::try_from_joules_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_joules_f32`](Self::from_joules_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Energy` or not finite.
    #[inline]
    pub const fn try_from_joules_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError("energy must be finite and non-negative"))
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a new `Energy` from the specified [`f32`], in watt-hours.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `Energy` or not finite.
    #[inline(always)]
    pub fn from_watt_hours_f32(value: f32) -> Self {
        Self::try_from_watt_hours_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_watt_hours_f32`](Self::from_watt_hours_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Energy` or not finite.
    #[inline(always)]
    pub const fn try_from_watt_hours_f32(value: f32) -> ValidationResult<Self> {
        Self::try_from_joules_f32(value * 3_600.0)
    }

    /// Creates a new `Energy` from the specified [`f32`], in kilowatt-hours.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `Energy` or not finite.
    #[inline(always)]
    pub fn from_kilowatt_hours_f32(value: f32) -> Self {
        Self::try_from_kilowatt_hours_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_kilowatt_hours_f32`](Self::from_kilowatt_hours_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Energy` or not finite.
    #[inline(always)]
    pub const fn try_from_kilowatt_hours_f32(value: f32) -> ValidationResult<Self> {
        Self::try_from_joules_f32(value * 3_600_000.0)
    }

    /// Returns this `Energy` as [`f32`], in joules.
    #[inline(always)]
    pub const fn as_joules_f32(&self) -> f32 {
        self.0
    }

    /// Returns this `Energy` as [`f32`], in watt-hours.
    #[inline(always)]
    pub const fn as_watt_hours_f32(&self) -> f32 {
        self.0 / 3_600.0
    }

    /// Returns this `Energy` as [`f32`], in kilowatt-hours.
    #[inline(always)]
    pub const fn as_kilowatt_hours_f32(&self) -> f32 {
        self.0 / 3_600_000.0
    }

    /// Returns `true` if this `Energy` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }
}

impl Eq for Energy {}

impl PartialOrd for Energy {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Energy {
    /// Compares two energies.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("energy is always finite, so a total order exists")
    }
}

impl ops::Validated for Energy {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_joules_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_joules_f32(value).ok()
    }
}

impl ops::UpperBounded for Energy {
    const MAX: Self = Self::MAX;
}

impl ops::LowerBounded for Energy {
    const MIN: Self = Self::ZERO;
}

impl ops::NonNegative for Energy {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_joules_rejects_negative() {
        assert!(Energy::try_from_joules_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_joules_rejects_non_finite() {
        assert!(Energy::try_from_joules_f32(f32::NAN).is_err());
        assert!(Energy::try_from_joules_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn conversions_round_trip_through_joules() {
        let watt_hours = 1000.0;
        let from_watt_hours = Energy::from_watt_hours_f32(watt_hours);

        let joules = from_watt_hours.as_joules_f32();
        let from_joules = Energy::from_joules_f32(joules);

        let kilowatt_hours = from_joules.as_kilowatt_hours_f32();
        let from_kilowatt_hours = Energy::from_kilowatt_hours_f32(kilowatt_hours);

        let watt_hours_again = from_kilowatt_hours.as_watt_hours_f32();

        assert_eq!(watt_hours, watt_hours_again);
    }
}
