use physics::Temperature;

use crate::component::{ComponentResult, DamagePerSecond};

/// Indicates that this part has safe temperature range
/// working outside of which damages it.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SafeTemperatureRange {
    min: Temperature,
    max: Temperature,
    idle_damage_per_second: DamagePerSecond,
    active_damage_per_second: DamagePerSecond,
}

impl SafeTemperatureRange {
    /// Makes new `SafeTemperatureRange` from given values.
    ///
    /// `max` must be greater than `min`.
    #[inline]
    pub fn new(
        min: Temperature,
        max: Temperature,
        idle_damage_per_second: DamagePerSecond,
        active_damage_per_second: DamagePerSecond,
    ) -> ComponentResult<Self> {
        if min >= max {
            Err("max must be greater than min")
        } else {
            Ok(Self {
                min,
                max,
                idle_damage_per_second,
                active_damage_per_second,
            })
        }
    }

    /// [`DamagePerSecond`] received by a part outside of safe
    /// range, depending on whether it's currently active.
    #[inline(always)]
    pub const fn damage_per_second(&self, is_active: bool) -> DamagePerSecond {
        if is_active {
            self.active_damage_per_second
        } else {
            self.idle_damage_per_second
        }
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

    fn dps(value: f32) -> DamagePerSecond {
        DamagePerSecond::from(DamageUnit::try_from(value).unwrap())
    }

    fn range(min: f32, max: f32) -> SafeTemperatureRange {
        SafeTemperatureRange::new(
            Temperature::from_kelvins_f32(min),
            Temperature::from_kelvins_f32(max),
            dps(1.0),
            dps(1.0),
        )
        .unwrap()
    }

    #[test]
    fn new_rejects_max_not_greater_than_min() {
        let temperature = Temperature::from_kelvins_f32(100.0);
        assert!(SafeTemperatureRange::new(temperature, temperature, dps(1.0), dps(1.0)).is_err());
        assert!(
            SafeTemperatureRange::new(
                Temperature::from_kelvins_f32(200.0),
                temperature,
                dps(1.0),
                dps(1.0)
            )
            .is_err()
        );
    }

    #[test]
    fn damage_per_second_depends_on_is_active() {
        let idle = dps(1.0);
        let active = dps(10.0);
        let range = SafeTemperatureRange::new(
            Temperature::from_kelvins_f32(100.0),
            Temperature::from_kelvins_f32(200.0),
            idle,
            active,
        )
        .unwrap();

        assert_eq!(range.damage_per_second(false), idle);
        assert_eq!(range.damage_per_second(true), active);
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
