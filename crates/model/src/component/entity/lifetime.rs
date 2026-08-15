use std::time::Duration;

/// How much longer this entity exists before it should be despawned.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lifetime(Duration);

impl Lifetime {
    /// Reduces this `Lifetime` by given [`Duration`].
    #[inline]
    pub const fn reduce(&mut self, dt: Duration) {
        self.0 = self.0.saturating_sub(dt);
    }

    /// Checks if this `Lifetime` is done and entity should be despawned.
    #[inline(always)]
    pub const fn is_expired(&self) -> bool {
        self.0.is_zero()
    }
}

impl From<Duration> for Lifetime {
    #[inline]
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_is_not_expired_when_positive() {
        let lifetime = Lifetime::from(Duration::from_secs(1));
        assert!(!lifetime.is_expired());
    }

    #[test]
    fn reduce_reduces_remaining_time() {
        let mut lifetime = Lifetime::from(Duration::from_secs(2));
        lifetime.reduce(Duration::from_secs(1));
        assert_eq!(lifetime, Lifetime::from(Duration::from_secs(1)));
    }

    #[test]
    fn reduce_saturates_at_zero_and_expires() {
        let mut lifetime = Lifetime::from(Duration::from_secs(1));
        lifetime.reduce(Duration::from_secs(2));
        assert!(lifetime.is_expired());
    }
}
