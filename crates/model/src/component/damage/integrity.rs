use physics::ops::Delta;

use crate::component::{ComponentResult, DamageUnit};

/// Hit points.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Integrity {
    current: DamageUnit,
    max: DamageUnit,
}

impl Integrity {
    /// Makes new `Integrity` from given values.
    ///
    /// `max` must be positive. `current` must be in range `[0, max]`.
    #[inline]
    pub fn new(current: DamageUnit, max: DamageUnit) -> ComponentResult<Self> {
        if max.is_zero() {
            Err("max must be positive")
        } else if current > max {
            Err("current must be within [0, max]")
        } else {
            Ok(Self { current, max })
        }
    }

    /// Makes new `Integrity` with `current` = `max`.
    #[inline(always)]
    pub fn full(max: DamageUnit) -> ComponentResult<Self> {
        Self::new(max, max)
    }

    /// Returns this `Integrity`'s `current` to `max` ratio.
    ///
    /// Returned value is guaranteed to be in [0, 1] range.
    #[inline]
    pub fn ratio(&self) -> f32 {
        let current = f32::from(self.current);
        let max = f32::from(self.max);
        (current / max).clamp(0.0, 1.0)
    }

    /// Returns `true` if there's no integrity units left.
    #[inline(always)]
    pub const fn is_destroyed(&self) -> bool {
        self.current.is_zero()
    }

    /// Applies given [`Delta<DamageUnit>`] to this `Integrity`'s
    /// current value, saturating within `[0, max]`.
    #[inline]
    pub fn apply(&mut self, delta: Delta<DamageUnit>) {
        let updated = delta.saturating_apply_to(self.current);
        self.current = updated.min(self.max);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_non_positive_max() {
        assert!(Integrity::new(DamageUnit::ZERO, DamageUnit::ZERO).is_err());
    }

    #[test]
    fn new_rejects_current_out_of_range() {
        let current = DamageUnit::try_from(11.0).unwrap();
        let max = DamageUnit::try_from(10.0).unwrap();
        assert!(Integrity::new(current, max).is_err());
    }

    #[test]
    fn full_sets_current_to_max() {
        let max = DamageUnit::try_from(10.0).unwrap();
        let integrity = Integrity::full(max).unwrap();
        assert_eq!(integrity.ratio(), 1.0);
        assert!(!integrity.is_destroyed());
    }

    #[test]
    fn ratio_reflects_current_over_max() {
        let current = DamageUnit::try_from(5.0).unwrap();
        let max = DamageUnit::try_from(10.0).unwrap();
        let integrity = Integrity::new(current, max).unwrap();
        assert_eq!(integrity.ratio(), 0.5);
    }

    #[test]
    fn is_destroyed_when_current_is_zero() {
        let current = DamageUnit::try_from(0.0).unwrap();
        let max = DamageUnit::try_from(10.0).unwrap();
        let integrity = Integrity::new(current, max).unwrap();
        assert!(integrity.is_destroyed());
    }

    #[test]
    fn apply_reduces_current_on_negative() {
        let max = DamageUnit::try_from(10.0).unwrap();
        let mut integrity = Integrity::full(max).unwrap();

        let damage = Delta::Negative(DamageUnit::try_from(4.0).unwrap());
        integrity.apply(damage);

        assert_eq!(integrity.ratio(), 0.6);
    }

    #[test]
    fn apply_clamps_current_on_negative_overflow() {
        let max = DamageUnit::try_from(10.0).unwrap();
        let mut integrity = Integrity::full(max).unwrap();

        let damage = Delta::Negative(DamageUnit::try_from(20.0).unwrap());
        integrity.apply(damage);

        assert_eq!(integrity.ratio(), 0.0);
        assert!(integrity.is_destroyed());
    }

    #[test]
    fn apply_increases_current_on_positive() {
        let max = DamageUnit::try_from(10.0).unwrap();
        let current = DamageUnit::try_from(1.0).unwrap();
        let mut integrity = Integrity::new(current, max).unwrap();

        let damage = Delta::Positive(DamageUnit::try_from(4.0).unwrap());
        integrity.apply(damage);

        assert_eq!(integrity.ratio(), 0.5);
    }

    #[test]
    fn apply_clamps_current_at_max_on_positive_overflow() {
        let max = DamageUnit::try_from(10.0).unwrap();
        let current = DamageUnit::try_from(1.0).unwrap();
        let mut integrity = Integrity::new(current, max).unwrap();

        let damage = Delta::Positive(DamageUnit::try_from(20.0).unwrap());
        integrity.apply(damage);

        assert_eq!(integrity.ratio(), 1.0);
    }
}
