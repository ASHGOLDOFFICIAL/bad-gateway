use std::time::Duration;

use crate::component::DamageUnit;

/// [`DamageUnit`] per second.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::From, derive_more::Into)]
pub struct DamagePerSecond(DamageUnit);

impl std::ops::Mul<Duration> for DamagePerSecond {
    type Output = DamageUnit;

    /// Derives [`DamageUnit`] from this `DamagePerSecond` and given
    /// [`Duration`]. Clamps at [`DamageUnit::MAX`] on overflow.
    #[inline]
    fn mul(self, rhs: Duration) -> Self::Output {
        self.0
            .checked_mul(rhs.as_secs_f32())
            .unwrap_or(DamageUnit::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_duration_yields_damage() {
        let dps = DamagePerSecond::from(DamageUnit::try_from(2.0).unwrap());
        let damage = dps * Duration::from_secs(3);
        assert_eq!(damage, DamageUnit::try_from(6.0).unwrap());
    }

    #[test]
    fn mul_duration_clamps_at_overflow() {
        let dps = DamagePerSecond::from(DamageUnit::MAX);
        let damage = dps * Duration::from_secs(u64::MAX);
        assert_eq!(damage, DamageUnit::MAX);
    }
}
