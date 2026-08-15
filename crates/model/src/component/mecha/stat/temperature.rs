use physics::Temperature;

use crate::component::{ComponentResult, DamagePerSecond};

/// Indicates that this part has safe temperature range
/// working outside of which damages it.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafeTemperatureRange {
    min: Temperature,
    max: Temperature,
    damage_per_second: DamagePerSecond,
}

impl SafeTemperatureRange {
    /// Makes new `SafeTemperatureRange` from given values.
    ///
    /// `max` must be greater than `min`.
    #[inline]
    pub fn new(
        min: Temperature,
        max: Temperature,
        damage_per_second: DamagePerSecond,
    ) -> ComponentResult<Self> {
        if min >= max {
            Err("max must be greater than min")
        } else {
            Ok(Self {
                min,
                max,
                damage_per_second,
            })
        }
    }

    /// [`DamagePerSecond`] received by a part
    /// when working outside of safe range.
    pub const fn damage_per_second(&self) -> DamagePerSecond {
        self.damage_per_second
    }

    /// Checks if given [`Temperature`] is below this `SafeTemperatureRange`.
    #[inline(always)]
    pub fn is_below(&self, current: &Temperature) -> bool {
        *current < self.min
    }

    /// Checks if given [`Temperature`] is above this `SafeTemperatureRange`.
    #[inline(always)]
    pub fn is_above(&self, current: &Temperature) -> bool {
        *current > self.max
    }

    /// Checks if given [`Temperature`] is within this `SafeTemperatureRange`.
    #[inline(always)]
    pub fn contains(&self, current: &Temperature) -> bool {
        *current >= self.min && *current <= self.max
    }
}

#[cfg(test)]
mod tests {
    use crate::component::DamageUnit;

    use super::*;

    fn range(min: f32, max: f32) -> SafeTemperatureRange {
        SafeTemperatureRange::new(
            Temperature::from_kelvins_f32(min),
            Temperature::from_kelvins_f32(max),
            DamagePerSecond::from(DamageUnit::try_from(1.0).unwrap()),
        )
        .unwrap()
    }

    #[test]
    fn new_rejects_max_not_greater_than_min() {
        let temperature = Temperature::from_kelvins_f32(100.0);
        let dps = DamagePerSecond::from(DamageUnit::try_from(1.0).unwrap());
        assert!(SafeTemperatureRange::new(temperature, temperature, dps).is_err());
        assert!(
            SafeTemperatureRange::new(Temperature::from_kelvins_f32(200.0), temperature, dps)
                .is_err()
        );
    }

    #[test]
    fn safe_boundaries_are_inclusive() {
        let min = 100.0;
        let max = 200.0;

        let range = range(min, max);
        let min = Temperature::from_kelvins_f32(min);
        let max = Temperature::from_kelvins_f32(max);

        assert!(range.contains(&min));
        assert!(!range.is_below(&min));
        assert!(!range.is_above(&min));

        assert!(range.contains(&max));
        assert!(!range.is_below(&max));
        assert!(!range.is_above(&max));
    }

    #[test]
    fn below_min_is_below_only() {
        let range = range(100.0, 200.0);
        let cold = Temperature::from_kelvins_f32(50.0);
        assert!(range.is_below(&cold));
        assert!(!range.is_above(&cold));
        assert!(!range.contains(&cold));
    }

    #[test]
    fn above_max_is_above_only() {
        let range = range(100.0, 200.0);
        let hot = Temperature::from_kelvins_f32(250.0);
        assert!(range.is_above(&hot));
        assert!(!range.is_below(&hot));
        assert!(!range.contains(&hot));
    }

    #[test]
    fn within_range_is_contained_only() {
        let range = range(100.0, 200.0);
        let mild = Temperature::from_kelvins_f32(150.0);
        assert!(range.contains(&mild));
        assert!(!range.is_below(&mild));
        assert!(!range.is_above(&mild));
    }
}
