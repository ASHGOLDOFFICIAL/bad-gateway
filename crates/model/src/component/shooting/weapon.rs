use physics::{Angle, Speed};

use crate::component::{AmmoType, ComponentResult, FireRate};

/// Ranged weapon that can shoot.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangedWeapon {
    fire_rate: FireRate,
    ammo_type: AmmoType,
    muzzle_speed: Speed,
    half_spread: Angle,
    scale: f32,
}

impl RangedWeapon {
    /// Makes new `RangedWeapon` from the given values.
    ///
    /// `muzzle_speed` must be non-empty.
    #[inline]
    pub fn new(
        fire_rate: FireRate,
        ammo_type: AmmoType,
        muzzle_speed: Speed,
        spread: Angle,
    ) -> ComponentResult<Self> {
        if muzzle_speed.is_zero() {
            Err("projectile speed must be positive")
        } else {
            Ok(Self {
                fire_rate,
                ammo_type,
                muzzle_speed,
                half_spread: Angle::from_radians_f32(spread.as_radians_f32() / 2.0),
                scale: 1.0,
            })
        }
    }

    /// This `RangedWeapon`'s [`FireRate`].
    #[inline]
    pub fn fire_rate(&self) -> FireRate {
        self.fire_rate
            .checked_mul(self.scale)
            .expect("scale is in (0, 1]")
    }

    /// This `RangedWeapon`'s muzzle [`Speed`].
    #[inline(always)]
    pub fn muzzle_speed(&self) -> Speed {
        self.muzzle_speed
    }

    /// This `RangedWeapon`'s half-spread of projectiles.
    #[inline(always)]
    pub fn half_spread(&self) -> Angle {
        self.half_spread
    }

    /// Sets the requested [`FireRate`] scale, which must be in `(0, 1]`.
    #[inline]
    pub fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        if !scale.is_finite() || scale <= 0.0 || scale > 1.0 {
            Err("scale must be in (0, 1]")
        } else {
            self.scale = scale;
            Ok(())
        }
    }

    /// Resets this `RangedWeapon`'s scale to 1.
    #[inline(always)]
    pub fn reset_scale(&mut self) {
        self.scale = 1.0;
    }
}

#[cfg(test)]
mod tests {
    use physics::Frequency;

    use super::*;

    fn weapon() -> RangedWeapon {
        RangedWeapon::new(
            FireRate::from(Frequency::from_hertz_f32(2.0)),
            AmmoType::Bullets,
            Speed::from_meters_per_second_f32(10.0),
            Angle::default(),
        )
        .unwrap()
    }

    #[test]
    fn new_rejects_non_positive_projectile_speed() {
        assert!(
            RangedWeapon::new(
                FireRate::from(Frequency::from_hertz_f32(2.0)),
                AmmoType::Bullets,
                Speed::ZERO,
                Angle::default()
            )
            .is_err()
        );
    }

    #[test]
    fn fire_rate_reflects_current_scale() {
        let mut weapon = RangedWeapon::new(
            FireRate::from(Frequency::from_hertz_f32(2.0)),
            AmmoType::Bullets,
            Speed::from_meters_per_second_f32(10.0),
            Angle::default(),
        )
        .unwrap();

        weapon.set_scale(0.5).unwrap();
        assert_eq!(Frequency::from(weapon.fire_rate()).as_hertz_f32(), 1.0);

        weapon.reset_scale();
        assert_eq!(Frequency::from(weapon.fire_rate()).as_hertz_f32(), 2.0);
    }

    #[test]
    fn half_spread_is_half_of_full_angle_given_to_new() {
        let full_spread = Angle::from_arcminute_f32(60.0);
        let weapon = RangedWeapon::new(
            FireRate::from(Frequency::from_hertz_f32(2.0)),
            AmmoType::Bullets,
            Speed::from_meters_per_second_f32(10.0),
            full_spread,
        )
        .unwrap();

        assert_eq!(weapon.half_spread().as_arcminute_f32(), 30.0);
    }

    #[test]
    fn set_scale_rejects_out_of_range() {
        let mut weapon = weapon();
        assert!(weapon.set_scale(0.0).is_err());
        assert!(weapon.set_scale(1.1).is_err());
        assert!(weapon.set_scale(-0.1).is_err());
    }

    #[test]
    fn set_scale_rejects_non_finite() {
        let mut weapon = weapon();
        assert!(weapon.set_scale(f32::NAN).is_err());
        assert!(weapon.set_scale(f32::INFINITY).is_err());
    }
}
