use std::time::Duration;

/// Time remaining before the next shot can be fired.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShootCooldown(Duration);

impl ShootCooldown {
    pub const READY: Self = Self(Duration::ZERO);

    /// Reduces this `ShootCooldown` by given [`Duration`].
    #[inline]
    pub const fn reduce(&mut self, dt: Duration) {
        self.0 = self.0.saturating_sub(dt);
    }

    /// Checks if this `ShootCooldown` is over and shot can be fired.
    #[inline(always)]
    pub const fn is_ready(&self) -> bool {
        self.0.is_zero()
    }

    /// Resets this `ShootCooldown`, sets in to the given [`Duration`].
    #[inline(always)]
    pub const fn reset(&mut self, interval: Duration) {
        self.0 = interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_reduces_remaining_time_until_ready() {
        let mut cooldown = ShootCooldown::READY;
        cooldown.reset(Duration::from_secs(2));

        cooldown.reduce(Duration::from_secs(1));
        assert!(!cooldown.is_ready());

        cooldown.reduce(Duration::from_secs(1));
        assert!(cooldown.is_ready());
    }

    #[test]
    fn ready_is_ready() {
        assert!(ShootCooldown::READY.is_ready());
    }

    #[test]
    fn reset_is_not_ready() {
        let mut cooldown = ShootCooldown::READY;
        cooldown.reset(Duration::from_secs(1));
        assert!(!cooldown.is_ready());
    }

    #[test]
    fn reduce_saturates_at_zero() {
        let mut cooldown = ShootCooldown::READY;
        cooldown.reset(Duration::from_secs(1));
        cooldown.reduce(Duration::from_secs(2));
        assert!(cooldown.is_ready());
    }
}
