use physics::{Energy, ops::CheckedSub};

use crate::component::ComponentResult;

/// Indicates that this part stores energy.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Battery {
    charge: Energy,
    capacity: Energy,
}

impl Battery {
    /// Makes new `Battery` from the given values.
    ///
    /// `capacity` must be positive. `charge` must be in range `[0, capacity]`.
    #[inline]
    pub fn new(capacity: Energy, charge: Energy) -> ComponentResult<Self> {
        if capacity.is_zero() {
            Err("capacity must be positive")
        } else if charge > capacity {
            Err("charge must be within [0, capacity]")
        } else {
            Ok(Self { capacity, charge })
        }
    }

    /// Makes new `Battery` with `charge` = `capacity`.
    #[inline(always)]
    pub fn full(capacity: Energy) -> ComponentResult<Self> {
        Self::new(capacity, capacity)
    }

    /// Returns this `Battery`'s `charge` to `capacity` ratio.
    ///
    /// Returned value is guaranteed to be in [0, 1] range.
    #[inline]
    pub const fn charge_ratio(&self) -> f32 {
        let capacity_value = self.capacity.as_joules_f32();
        let charge_value = self.charge.as_joules_f32();
        (charge_value / capacity_value).clamp(0.0, 1.0)
    }

    /// Drains up to `amount`, returning how much [`Energy`] was actually taken.
    #[inline]
    pub fn discharge(&mut self, amount: Energy) -> Energy {
        let taken = amount.min(self.charge);
        self.charge = self
            .charge
            .checked_sub(taken)
            .expect("taken is at most current charge");
        taken
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn battery(capacity: f32, charge: f32) -> Battery {
        let capacity = Energy::from_joules_f32(capacity);
        let charge = Energy::from_joules_f32(charge);
        Battery::new(capacity, charge).unwrap()
    }

    #[test]
    fn new_rejects_zero_capacity() {
        assert!(Battery::new(Energy::ZERO, Energy::ZERO).is_err());
    }

    #[test]
    fn new_rejects_charge_greater_than_capacity() {
        let capacity = Energy::from_joules_f32(10.0);
        assert!(Battery::new(capacity, Energy::from_joules_f32(20.0)).is_err());
    }

    #[test]
    fn charge_ratio() {
        assert_eq!(battery(10.0, 5.0).charge_ratio(), 0.5);
    }

    #[test]
    fn discharge_drains_up_to_available_charge() {
        let mut battery = battery(10.0, 10.0);

        let taken = battery.discharge(Energy::from_joules_f32(3.0));
        assert_eq!(taken.as_joules_f32(), 3.0);
        assert_eq!(battery.charge_ratio(), 0.7);

        let taken = battery.discharge(Energy::from_joules_f32(100.0));
        assert_eq!(taken.as_joules_f32(), 7.0);
        assert_eq!(battery.charge_ratio(), 0.0);
    }
}
