use std::time::Duration;

use hecs::{Entity, Satisfies, Without, World};
use log::warn;
use physics::{Energy, ops::Delta};

use crate::component::{Active, Disabled, HeatDelta, mecha::stat::HeatOutput};

/// Converts each non-[`Disabled`] entity's [`HeatOutput`] to [`HeatDelta`].
///
/// Reads:
/// - [`Active`] to determine heat output profile.
/// - [`HeatOutput`] to get exhausted heat.
/// - [`Option<HeatDelta>`](HeatDelta) to add to it if present.
///
/// Writes:
/// - [`HeatDelta`] - entity's waste heat.
pub fn mecha_heat_generation_system(world: &mut World, delta_time: Duration) {
    let to_add_heat: Vec<(Entity, HeatDelta)> = world
        .query::<Without<
            (
                Entity,
                Satisfies<&Active>,
                &HeatOutput,
                Option<&mut HeatDelta>,
            ),
            &Disabled,
        >>()
        .iter()
        .filter_map(|(entity, is_active, output, existing)| {
            let energy_delta = compute_energy_delta(entity, is_active, output, delta_time);
            match existing {
                Some(existing) => {
                    *existing += energy_delta;
                    None
                }
                None => (!energy_delta.is_zero()).then(|| (entity, HeatDelta::from(energy_delta))),
            }
        })
        .collect();

    for (entity, delta) in to_add_heat {
        let _ = world.insert_one(entity, delta);
    }
}

fn compute_energy_delta(
    entity: Entity,
    is_active: bool,
    output: &HeatOutput,
    delta_time: Duration,
) -> Delta<Energy> {
    let heat_output = output.output(is_active);

    match heat_output.energy(delta_time) {
        Ok(v) if !v.is_zero() => Delta::Positive(v),
        Ok(_) => Delta::Zero,
        Err(err) => {
            warn!(
                entity:? = entity,
                power:? = heat_output,
                delta_time:? = delta_time,
                error:% = err;
                "heat energy calculation failed"
            );
            Delta::Zero
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{Power, mecha::stat::HeatOutput};

    use super::*;

    fn spawn_heater(world: &mut World, output: HeatOutput, is_active: bool) -> Entity {
        let entity = world.spawn((output,));
        if is_active {
            world.insert_one(entity, Active).unwrap();
        }
        entity
    }

    fn watts(value: f32) -> Power {
        Power::from_watts_f32(value)
    }

    fn joules(value: f32) -> Energy {
        Energy::from_joules_f32(value)
    }

    fn output(idle: f32, active: f32) -> HeatOutput {
        HeatOutput::new(watts(idle), watts(active))
    }

    fn heat_delta(world: &World, entity: Entity) -> Delta<Energy> {
        (*world.get::<&HeatDelta>(entity).unwrap()).into()
    }

    #[test]
    fn an_active_entity_exhausts_its_active_output() {
        let mut world = World::new();
        let entity = spawn_heater(&mut world, output(1.0, 10.0), true);

        mecha_heat_generation_system(&mut world, Duration::from_secs(1));

        assert_eq!(heat_delta(&world, entity), Delta::Positive(joules(10.0)));
    }

    #[test]
    fn an_idle_entity_exhausts_its_idle_output() {
        let mut world = World::new();
        let entity = spawn_heater(&mut world, output(1.0, 10.0), false);

        mecha_heat_generation_system(&mut world, Duration::from_secs(1));

        assert_eq!(heat_delta(&world, entity), Delta::Positive(joules(1.0)));
    }

    #[test]
    fn heat_scales_with_delta_time() {
        let mut world = World::new();
        let entity = spawn_heater(&mut world, output(0.0, 10.0), true);

        mecha_heat_generation_system(&mut world, Duration::from_millis(500));

        assert_eq!(heat_delta(&world, entity), Delta::Positive(joules(5.0)));
    }

    #[test]
    fn heat_accumulates_across_ticks() {
        let mut world = World::new();
        let entity = spawn_heater(&mut world, output(0.0, 10.0), true);

        mecha_heat_generation_system(&mut world, Duration::from_secs(1));
        mecha_heat_generation_system(&mut world, Duration::from_secs(1));

        assert_eq!(heat_delta(&world, entity), Delta::Positive(joules(20.0)));
    }

    #[test]
    fn disabled_entity_exhausts_no_heat() {
        let mut world = World::new();
        let entity = spawn_heater(&mut world, output(1.0, 10.0), true);
        world.insert_one(entity, Disabled).unwrap();

        mecha_heat_generation_system(&mut world, Duration::from_secs(1));

        assert!(world.get::<&HeatDelta>(entity).is_err());
    }

    #[test]
    fn ignores_entities_without_heat_output() {
        let mut world = World::new();
        let entity = world.spawn((Active,));

        mecha_heat_generation_system(&mut world, Duration::from_secs(1));

        assert!(world.get::<&HeatDelta>(entity).is_err());
    }
}
