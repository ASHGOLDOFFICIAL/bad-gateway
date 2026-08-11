use crate::{
    CalculationError, CalculationResult, Energy, Temperature, ValidationError, ValidationResult,
    ops,
};

/// Total heat capacity, dimension ML²T⁻²Θ⁻¹ (energy per temperature).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatCapacity(f32);

impl HeatCapacity {
    /// The largest representable heat capacity.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `HeatCapacity` from the specified [`f32`], in joules
    /// per kelvin.
    ///
    /// # Panics
    /// This constructor will panic if value is not positive, overflows
    /// `HeatCapacity` or not finite.
    #[inline(always)]
    pub fn from_joules_per_kelvin_f32(value: f32) -> Self {
        Self::try_from_joules_per_kelvin_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_joules_per_kelvin_f32`](Self::from_joules_per_kelvin_f32).
    ///
    /// This constructor will return an `Err` if value is not positive,
    /// overflows `HeatCapacity` or not finite.
    #[inline]
    pub const fn try_from_joules_per_kelvin_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            Err(ValidationError("heat capacity must be finite and positive"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `HeatCapacity` as [`f32`], in joules per kelvin.
    #[inline(always)]
    pub const fn as_joules_per_kelvin_f32(&self) -> f32 {
        self.0
    }

    /// Derives the resulting [`Temperature`] [`Delta`](ops::Delta) from an
    /// [`Energy`] [`Delta`](ops::Delta) at this `HeatCapacity`.
    ///
    /// It uses `dT = dQ / C`, where
    /// - `dT` is the temperature delta,
    /// - `dQ` is the energy delta,
    /// - `C` is this heat capacity.
    ///
    /// Errors if the result overflows `Temperature`'s valid range.
    #[inline]
    pub fn temperature_delta(
        self,
        energy: ops::Delta<Energy>,
    ) -> CalculationResult<ops::Delta<Temperature>> {
        energy.try_map(|energy| {
            Temperature::try_from_kelvins_f32(energy.as_joules_f32() / self.0)
                .map_err(CalculationError::from)
        })
    }
}

impl Eq for HeatCapacity {}

impl PartialOrd for HeatCapacity {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HeatCapacity {
    /// Compares two heat capacities.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("heat capacity is always finite, so a total order exists")
    }
}

impl ops::Validated for HeatCapacity {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_joules_per_kelvin_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_joules_per_kelvin_f32(value).ok()
    }
}

impl ops::UpperBounded for HeatCapacity {
    const MAX: Self = Self::MAX;
}

#[cfg(test)]
mod tests {
    use crate::ops;

    use super::*;

    #[test]
    fn try_from_joules_per_kelvin_rejects_non_positive() {
        assert!(HeatCapacity::try_from_joules_per_kelvin_f32(0.0).is_err());
        assert!(HeatCapacity::try_from_joules_per_kelvin_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_joules_per_kelvin_rejects_non_finite() {
        assert!(HeatCapacity::try_from_joules_per_kelvin_f32(f32::NAN).is_err());
        assert!(HeatCapacity::try_from_joules_per_kelvin_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_joules_per_kelvin_accepts_positive() {
        let capacity = HeatCapacity::from_joules_per_kelvin_f32(5.0);
        assert_eq!(capacity.as_joules_per_kelvin_f32(), 5.0);
    }

    #[test]
    fn temperature_delta_preserves_direction() {
        let capacity = HeatCapacity::from_joules_per_kelvin_f32(2.0);
        let energy = Energy::from_joules_f32(10.0);
        let step = Temperature::from_kelvins_f32(5.0);

        assert_eq!(
            capacity
                .temperature_delta(ops::Delta::Positive(energy))
                .unwrap(),
            ops::Delta::Positive(step)
        );
        assert_eq!(
            capacity
                .temperature_delta(ops::Delta::Negative(energy))
                .unwrap(),
            ops::Delta::Negative(step)
        );
        assert_eq!(
            capacity.temperature_delta(ops::Delta::None).unwrap(),
            ops::Delta::<Temperature>::None
        );
    }
}
