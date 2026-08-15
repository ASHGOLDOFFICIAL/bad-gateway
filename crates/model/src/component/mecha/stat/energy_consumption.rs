use physics::Power;

/// Indicates that this part requires [`Energy`](physics::Energy) to work.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnergyConsumption {
    idle: Power,
    active: Power,
}

impl EnergyConsumption {
    /// Creates new `EnergyConsumption` from the given values.
    #[inline(always)]
    pub const fn new(idle: Power, active: Power) -> Self {
        Self { idle, active }
    }

    /// [`Power`] requirements of the part.
    #[inline(always)]
    pub const fn draw(&self, is_active: bool) -> Power {
        if is_active { self.active } else { self.idle }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn draw_depends_on_is_active() {
        let idle = Power::from_watts_f32(1.0);
        let active = Power::from_watts_f32(10.0);
        let drain = EnergyConsumption::new(idle, active);

        assert_eq!(drain.draw(false), idle);
        assert_eq!(drain.draw(true), active);
    }
}
