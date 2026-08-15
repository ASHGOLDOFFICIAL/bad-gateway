use std::time::Duration;

use physics::{Frequency, ops::CheckedMul};

/// How many projectiles per unit of time does weapon fires.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, derive_more::From, derive_more::Into)]
pub struct FireRate(Frequency);

impl FireRate {
    /// Returns interval in [`Duration`] between shots.
    #[inline]
    pub fn interval(&self) -> Duration {
        self.0
            .period()
            .expect("a game fire rate always produces a representable period")
    }

    /// Scales this fire rate by `rhs`.
    ///
    /// Returns [`None`] if the result is non-finite or non-positive.
    #[inline]
    pub fn checked_mul(&self, rhs: f32) -> Option<Self> {
        self.0.checked_mul(rhs).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fire_rate(hertz: f32) -> FireRate {
        Frequency::from_hertz_f32(hertz).into()
    }

    #[test]
    fn interval_is_reciprocal() {
        let value = 2.0;
        assert_eq!(
            fire_rate(value).interval(),
            Duration::from_secs_f32(1.0 / value)
        );
    }

    #[test]
    fn checked_mul_scales_rate() {
        assert_eq!(fire_rate(2.0).checked_mul(0.5).unwrap(), fire_rate(1.0));
    }

    #[test]
    fn checked_mul_rejects_non_positive_result() {
        assert!(fire_rate(2.0).checked_mul(0.0).is_none());
        assert!(fire_rate(2.0).checked_mul(-1.0).is_none());
    }
}
