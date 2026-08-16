use std::time::Duration;

use hecs::{Entity, World};
use log::warn;
use physics::{
    HeatCapacity, formulae,
    ops::{CheckedAdd, Delta, SaturatingAdd},
};

use crate::{
    component::{HeatDelta, SurfaceArea, Temperature},
    map::Map,
};

/// Heat transfer coefficient between a part's surface and the surrounding
/// air, in W/(m^2*K).
const AMBIENT_HEAT_TRANSFER_COEFFICIENT: f32 = 1.5;

/// Drifts every entity's [`Temperature`] toward ambient and applies any pending
/// [`HeatDelta`].
///
/// Reads:
/// - [`Temperature`] - current temperature needed.
/// - [`HeatCapacity`] - used to turn energy into a temperature delta.
/// - [`SurfaceArea`] - exposed to ambient air, used for cooling.
/// - [`Option<HeatDelta>`](HeatDelta) - pending heat that needs to be applied
///   if present.
///
/// Writes:
/// - [`Temperature`] - updated temperature.
///
/// Removes:
/// - [`HeatDelta`] after consumption.
pub fn temperature_system(world: &mut World, map: &dyn Map, delta_time: Duration) {
    let ambient = map.temperature();

    let consumed: Vec<Entity> = world
        .query::<(
            Entity,
            &mut Temperature,
            &HeatCapacity,
            &SurfaceArea,
            Option<&HeatDelta>,
        )>()
        .iter()
        .filter_map(|(entity, temperature, capacity, area, pending)| {
            let exhausted = exhausted_delta(*capacity, pending);
            let drift = drift_delta(*capacity, *temperature, ambient, *area, delta_time);
            *temperature = applied(entity, *temperature, combined(entity, exhausted, drift));
            pending.map(|_| entity)
        })
        .collect();

    for entity in consumed {
        let _ = world.remove_one::<HeatDelta>(entity);
    }
}

fn exhausted_delta(capacity: HeatCapacity, pending: Option<&HeatDelta>) -> Delta<Temperature> {
    let energy = Delta::from(pending.copied().unwrap_or_default());
    capacity.temperature_delta(energy).unwrap_or_default()
}

fn drift_delta(
    capacity: HeatCapacity,
    temperature: Temperature,
    ambient: Temperature,
    area: SurfaceArea,
    delta_time: Duration,
) -> Delta<Temperature> {
    let energy = formulae::newtonian_cooling(
        temperature,
        ambient,
        AMBIENT_HEAT_TRANSFER_COEFFICIENT,
        area.into(),
    )
    .unwrap_or_default()
    .map(|power| power.energy(delta_time))
    .transpose()
    .unwrap_or_default();

    capacity.temperature_delta(energy).unwrap_or_default()
}

fn combined(
    entity: Entity,
    exhausted: Delta<Temperature>,
    drift: Delta<Temperature>,
) -> Delta<Temperature> {
    exhausted.checked_add(drift).unwrap_or_else(|| {
        warn!(
            entity:? = entity,
            exhausted:? = exhausted,
            drift:? = drift;
            "combining temperature deltas overflowed, saturating"
        );
        exhausted.saturating_add(drift)
    })
}

fn applied(entity: Entity, current: Temperature, delta: Delta<Temperature>) -> Temperature {
    delta.checked_apply_to(current).unwrap_or_else(|| {
        warn!(
            entity:? = entity,
            temperature:? = current,
            delta:? = delta;
            "applying temperature delta overflowed, keeping previous temperature"
        );
        delta.saturating_apply_to(current)
    })
}

#[cfg(test)]
mod tests {
    use physics::{Area, Density, Energy, Length};

    use super::*;
    use crate::map::{Cell, Surface};

    const AMBIENT_CELSIUS: f32 = 20.0;

    struct MapStub;

    impl Map for MapStub {
        fn cell_at(&self, _x: f32, _y: f32) -> Cell {
            Cell {
                surface: Surface::Grass,
                elevation: Length::from_meters_f32(1.0),
                structure: None,
                liquid: None,
            }
        }

        fn temperature(&self) -> Temperature {
            Temperature::from_celsius_f32(AMBIENT_CELSIUS)
        }

        fn air_density(&self) -> Density {
            Density::from_kilograms_per_cubic_meter_f32(1.2255)
        }

        fn gravity(&self) -> f32 {
            9.8
        }
    }

    fn spawn_body(world: &mut World, celsius: f32, square_meters: f32) -> Entity {
        world.spawn((
            Temperature::from_celsius_f32(celsius),
            HeatCapacity::from_joules_per_kelvin_f32(100.0),
            SurfaceArea::from(Area::from_square_meters_f32(square_meters)),
        ))
    }

    fn add_pending_heat(world: &mut World, entity: Entity, joules: f32) {
        let delta = Delta::Positive(Energy::from_joules_f32(joules));
        world.insert_one(entity, HeatDelta::from(delta)).unwrap();
    }

    fn celsius(world: &World, entity: Entity) -> f32 {
        world.get::<&Temperature>(entity).unwrap().as_celsius_f32()
    }

    fn tick(world: &mut World, delta_time: Duration) {
        temperature_system(world, &MapStub, delta_time);
    }

    #[test]
    fn body_at_ambient_temperature_does_not_drift() {
        let mut world = World::new();
        let entity = spawn_body(&mut world, AMBIENT_CELSIUS, 1.0);

        tick(&mut world, Duration::from_secs(1));

        assert_eq!(celsius(&world, entity), AMBIENT_CELSIUS);
    }

    #[test]
    fn hot_body_cools_toward_ambient() {
        let mut world = World::new();
        let original = 100.0;
        let entity = spawn_body(&mut world, original, 1.0);

        tick(&mut world, Duration::from_secs(1));

        assert!(celsius(&world, entity) < original);
    }

    #[test]
    fn cold_body_warms_toward_ambient() {
        let mut world = World::new();
        let original = 0.0;
        let entity = spawn_body(&mut world, original, 1.0);

        tick(&mut world, Duration::from_secs(1));

        assert!(celsius(&world, entity) > original);
    }

    #[test]
    fn more_surface_area_means_faster_cooling() {
        let mut world = World::new();
        let a = spawn_body(&mut world, 100.0, 2.0);
        let b = spawn_body(&mut world, 100.0, 4.0);

        tick(&mut world, Duration::from_secs(1));

        assert!(celsius(&world, a) > celsius(&world, b));
    }

    #[test]
    fn pending_heat_raises_temperature() {
        let mut world = World::new();
        let entity = spawn_body(&mut world, AMBIENT_CELSIUS, 1.0);
        add_pending_heat(&mut world, entity, 1000.0);

        tick(&mut world, Duration::from_secs(1));

        assert!(celsius(&world, entity) > AMBIENT_CELSIUS);
    }

    #[test]
    fn pending_heat_is_consumed() {
        let mut world = World::new();
        let entity = spawn_body(&mut world, AMBIENT_CELSIUS, 1.0);
        add_pending_heat(&mut world, entity, 1000.0);

        tick(&mut world, Duration::from_secs(1));

        assert!(world.get::<&HeatDelta>(entity).is_err());
    }

    #[test]
    fn ignores_entities_without_surface_area() {
        let mut world = World::new();
        let entity = world.spawn((
            Temperature::from_celsius_f32(100.0),
            HeatCapacity::from_joules_per_kelvin_f32(100.0),
        ));

        tick(&mut world, Duration::from_secs(1));

        assert_eq!(celsius(&world, entity), 100.0);
    }
}
