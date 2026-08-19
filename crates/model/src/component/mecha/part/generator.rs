use physics::Power;

/// Indicates that this part is a generator with the ability
/// to use its output [`Energy`](physics::Energy) using [`Power`].
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Generator {
    power: Power,
}

impl Generator {
    /// Returns this `Generator`'s current [`Power`] value.
    #[inline]
    pub fn power(&self) -> Power {
        self.power
    }
}

impl From<Power> for Generator {
    fn from(value: Power) -> Self {
        Self { power: value }
    }
}
