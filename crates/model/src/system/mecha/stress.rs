use std::time::Duration;

use hecs::{Entity, Satisfies, World};
use physics::ops::Delta;

use crate::component::{Active, DamageDelta, Temperature, mecha::stat::SafeTemperatureRange};

/// Damages entities outside their [`SafeTemperatureRange`] by adding
/// negative [`DamageDelta`].
///
/// Reads:
/// - [`Active`] to pick the idle or active damage rate.
/// - [`Temperature`] to query current temperature.
/// - [`SafeTemperatureRange`] to check against current temperature.
/// - [`Option<DamageDelta>`](DamageDelta) to add new damage to it if present.
///
/// Writes:
/// - [`DamageDelta`] - pending damage for entities outside their safe range.
pub fn mecha_thermal_stress_system(world: &mut World, delta_time: Duration) {
    let to_add_damage: Vec<(Entity, DamageDelta)> = world
        .query::<(
            Entity,
            Satisfies<&Active>,
            &Temperature,
            &SafeTemperatureRange,
            Option<&mut DamageDelta>,
        )>()
        .iter()
        .filter_map(|(entity, is_active, temperature, range, existing)| {
            if range.contains(temperature) {
                return None;
            }
            let damage = range.damage_per_second(is_active) * delta_time;
            if damage.is_zero() {
                return None;
            }
            let damage_delta = Delta::Negative(damage);
            match existing {
                Some(existing) => {
                    *existing += damage_delta;
                    None
                }
                None => Some((entity, DamageDelta::from(damage_delta))),
            }
        })
        .collect();

    for (entity, delta) in to_add_damage {
        let _ = world.insert_one(entity, delta);
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{DamagePerSecond, DamageUnit};

    use super::*;

    fn dps(value: f32) -> DamagePerSecond {
        DamagePerSecond::from(DamageUnit::try_from(value).unwrap())
    }

    fn range(min: f32, max: f32, idle_dps: f32, active_dps: f32) -> SafeTemperatureRange {
        SafeTemperatureRange::new(
            Temperature::from_kelvins_f32(min),
            Temperature::from_kelvins_f32(max),
            dps(idle_dps),
            dps(active_dps),
        )
        .unwrap()
    }

    fn spawn_part(
        world: &mut World,
        range: SafeTemperatureRange,
        temperature: f32,
        active: bool,
    ) -> Entity {
        let entity = world.spawn((Temperature::from_kelvins_f32(temperature), range));
        if active {
            world.insert_one(entity, Active).unwrap();
        }
        entity
    }

    #[test]
    fn idle_part_within_range_gets_no_damage_delta() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 10.0, 0.0), 150.0, false);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        assert!(world.get::<&DamageDelta>(part).is_err());
    }

    #[test]
    fn idle_part_outside_range_accumulates_damage_at_the_idle_rate() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 1.0, 10.0), 300.0, false);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        let delta: Delta<DamageUnit> = (*world.get::<&DamageDelta>(part).unwrap()).into();
        assert_eq!(delta, Delta::Negative(DamageUnit::try_from(1.0).unwrap()));
    }

    #[test]
    fn active_part_within_range_gets_no_damage_delta() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 10.0, 0.0), 150.0, true);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        assert!(world.get::<&DamageDelta>(part).is_err());
    }

    #[test]
    fn active_part_outside_range_accumulates_damage_at_the_active_rate() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 1.0, 10.0), 300.0, true);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        let delta: Delta<DamageUnit> = (*world.get::<&DamageDelta>(part).unwrap()).into();
        assert_eq!(delta, Delta::Negative(DamageUnit::try_from(10.0).unwrap()));
    }

    #[test]
    fn zero_rate_outside_range_adds_no_damage_delta() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 0.0, 0.0), 300.0, true);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        assert!(world.get::<&DamageDelta>(part).is_err());
    }

    #[test]
    fn damage_delta_scales_with_delta_time() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 10.0, 10.0), 300.0, true);

        mecha_thermal_stress_system(&mut world, Duration::from_millis(500));

        let delta: Delta<DamageUnit> = (*world.get::<&DamageDelta>(part).unwrap()).into();
        assert_eq!(delta, Delta::Negative(DamageUnit::try_from(5.0).unwrap()));
    }

    #[test]
    fn damage_delta_accumulates_across_ticks() {
        let mut world = World::new();
        let part = spawn_part(&mut world, range(100.0, 200.0, 10.0, 10.0), 300.0, true);

        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));
        mecha_thermal_stress_system(&mut world, Duration::from_secs(1));

        let delta: Delta<DamageUnit> = (*world.get::<&DamageDelta>(part).unwrap()).into();
        assert_eq!(delta, Delta::Negative(DamageUnit::try_from(20.0).unwrap()));
    }
}
