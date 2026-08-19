use physics::{Mass, Speed};

use crate::component::{Agility, ComponentResult};

/// Indicates that this part carries the mecha: it sets how fast the mecha can
/// travel and how sharply it changes velocity.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Legs {
    travel_speed: Speed,
    agility: Agility,
    load_capacity: Mass,
    scale: f32,
}

impl Legs {
    /// Makes new `Legs` from the given values.
    ///
    /// `travel_speed` and `load_capacity` must be positive.
    #[inline]
    pub fn new(
        travel_speed: Speed,
        load_capacity: Mass,
        agility: Agility,
    ) -> ComponentResult<Self> {
        if travel_speed.is_zero() {
            Err("travel speed must be positive")
        } else if load_capacity.is_zero() {
            Err("load capacity must be positive")
        } else {
            Ok(Self {
                travel_speed,
                agility,
                load_capacity,
                scale: 0.0,
            })
        }
    }

    /// Returns the top [`Speed`] these `Legs` reach at or under their rated
    /// load.
    #[inline(always)]
    pub const fn travel_speed(&self) -> Speed {
        self.travel_speed
    }

    /// Returns the [`Mass`] these `Legs` carry without a speed penalty.
    #[inline(always)]
    pub const fn load_capacity(&self) -> Mass {
        self.load_capacity
    }

    /// Returns these `Legs`' [`Agility`] at their rated load.
    #[inline(always)]
    pub const fn agility(&self) -> Agility {
        self.agility
    }

    /// Returns this `Legs`'s current speed scale.
    #[inline(always)]
    pub const fn scale(&self) -> f32 {
        self.scale
    }

    /// Updates this `Legs`'s current scale. Must be in [0, 1].
    #[inline]
    pub const fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        if !scale.is_finite() || scale < 0.0 || scale > 1.0 {
            Err("scale must be in [0, 1]")
        } else {
            self.scale = scale;
            Ok(())
        }
    }

    /// Resets this `Legs`'s current speed scale to 0.
    #[inline(always)]
    pub const fn reset_scale(&mut self) {
        self.scale = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn agility() -> Agility {
        Agility::from(Duration::from_millis(280))
    }

    #[test]
    fn new_rejects_zero_base_speed() {
        let legs = Legs::new(Speed::ZERO, Mass::from_kilograms_f32(9000.0), agility());
        assert!(legs.is_err());
    }

    #[test]
    fn new_rejects_zero_load_capacity() {
        let legs = Legs::new(
            Speed::from_meters_per_second_f32(14.0),
            Mass::ZERO,
            agility(),
        );
        assert!(legs.is_err());
    }

    #[test]
    fn new_accepts_positive_values() {
        let speed = Speed::from_meters_per_second_f32(14.0);
        let load_capacity = Mass::from_kilograms_f32(9000.0);
        let agility = agility();

        let legs = Legs::new(speed, load_capacity, agility).unwrap();

        assert_eq!(legs.travel_speed(), speed);
        assert_eq!(legs.load_capacity(), load_capacity);
        assert_eq!(legs.agility(), agility);
    }
}
