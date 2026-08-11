use crate::{
    CalculationError, CalculationResult, HeatCapacity, Mass, ValidationError, ValidationResult, ops,
};

/// Specific heat capacity, dimension L²T⁻²Θ⁻¹ (energy per mass per
/// temperature).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpecificHeatCapacity(f32);

impl SpecificHeatCapacity {
    /// The largest representable specific heat capacity.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `SpecificHeatCapacity` from the specified [`f32`], in
    /// joules per kilogram-kelvin.
    ///
    /// # Panics
    /// This constructor will panic if value is not positive, overflows
    /// `SpecificHeatCapacity` or not finite.
    #[inline(always)]
    pub fn from_joules_per_kilogram_kelvin_f32(value: f32) -> Self {
        Self::try_from_joules_per_kilogram_kelvin_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_joules_per_kilogram_kelvin_f32`](Self::from_joules_per_kilogram_kelvin_f32).
    ///
    /// This constructor will return an `Err` if value is not positive,
    /// overflows `SpecificHeatCapacity` or not finite.
    #[inline]
    pub const fn try_from_joules_per_kilogram_kelvin_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            Err(ValidationError(
                "specific heat capacity must be finite and positive",
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `SpecificHeatCapacity` as [`f32`], in joules per
    /// kilogram-kelvin.
    #[inline(always)]
    pub const fn as_joules_per_kilogram_kelvin_f32(&self) -> f32 {
        self.0
    }

    /// Derives total [`HeatCapacity`] from this `SpecificHeatCapacity` and
    /// [`Mass`].
    ///
    /// It uses `C = c * m`, where
    /// - `C` is total heat capacity,
    /// - `c` is this specific heat capacity,
    /// - `m` is mass.
    ///
    /// Errors if `mass` is zero.
    #[inline]
    pub fn total(self, mass: Mass) -> CalculationResult<HeatCapacity> {
        if mass.is_zero() {
            Err(CalculationError::InvalidArgument("mass must be non-zero"))
        } else {
            HeatCapacity::try_from_joules_per_kelvin_f32(self.0 * mass.as_kilograms_f32())
                .map_err(CalculationError::from)
        }
    }
}

impl Eq for SpecificHeatCapacity {}

impl PartialOrd for SpecificHeatCapacity {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SpecificHeatCapacity {
    /// Compares two specific heat capacities.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("specific heat capacity is always finite, so a total order exists")
    }
}

impl ops::Validated for SpecificHeatCapacity {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_joules_per_kilogram_kelvin_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_joules_per_kilogram_kelvin_f32(value).ok()
    }
}

impl ops::UpperBounded for SpecificHeatCapacity {
    const MAX: Self = Self::MAX;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_joules_per_kilogram_kelvin_rejects_non_positive() {
        assert!(SpecificHeatCapacity::try_from_joules_per_kilogram_kelvin_f32(0.0).is_err());
        assert!(SpecificHeatCapacity::try_from_joules_per_kilogram_kelvin_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_joules_per_kilogram_kelvin_rejects_non_finite() {
        assert!(SpecificHeatCapacity::try_from_joules_per_kilogram_kelvin_f32(f32::NAN).is_err());
        assert!(
            SpecificHeatCapacity::try_from_joules_per_kilogram_kelvin_f32(f32::INFINITY).is_err()
        );
    }

    #[test]
    fn total_rejects_zero_mass() {
        let specific = SpecificHeatCapacity::from_joules_per_kilogram_kelvin_f32(500.0);
        assert!(specific.total(Mass::ZERO).is_err());
    }

    #[test]
    fn total_multiplies_by_mass() {
        let specific = SpecificHeatCapacity::from_joules_per_kilogram_kelvin_f32(500.0);
        let mass = Mass::from_kilograms_f32(2.0);
        assert_eq!(
            specific.total(mass).unwrap().as_joules_per_kelvin_f32(),
            1_000.0
        );
    }
}
