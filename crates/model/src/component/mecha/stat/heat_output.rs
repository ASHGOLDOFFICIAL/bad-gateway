use physics::Power;

/// Indicates that this part exhaust heat.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeatOutput {
    idle: Power,
    active: Power,
}

impl HeatOutput {
    /// Creates new `HeatOutput` from the given values.
    #[inline(always)]
    pub const fn new(idle: Power, active: Power) -> Self {
        Self { idle, active }
    }

    /// Exhausted heat [`Power`] of the part.
    #[inline(always)]
    pub const fn output(&self, is_active: bool) -> Power {
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
        let drain = HeatOutput::new(idle, active);

        assert_eq!(drain.output(false), idle);
        assert_eq!(drain.output(true), active);
    }
}
