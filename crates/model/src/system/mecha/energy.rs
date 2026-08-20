use std::time::Duration;

use hecs::{Entity, Satisfies, Without, World};
use physics::{
    Energy,
    ops::{CheckedMul, SaturatingAdd, SaturatingSub},
};

use crate::component::{
    Active, Disabled, Power, Unpowered,
    mecha::{
        Mecha,
        part::{Battery, Generator},
        stat::EnergyConsumption,
    },
};

#[derive(Default)]
struct PartTotals {
    draw: Power,
    generators: Vec<(Entity, Generator)>,
    batteries: Vec<(Entity, Battery)>,
    consumers: Vec<Entity>,
}

impl PartTotals {
    fn combine(mut self, other: Self) -> Self {
        self.draw = self.draw.saturating_add(other.draw);
        self.generators.extend(other.generators);
        self.batteries.extend(other.batteries);
        self.consumers.extend(other.consumers);
        self
    }
}

/// Charges each non-[`Disabled`] [`Battery`] from the tree's non-[`Disabled`]
/// [`Generator`]s, then drains it to cover the tree's [`EnergyConsumption`].
///
/// Reports a consumption shortfall as [`Unpowered`].
///
/// Reads:
/// - [`Mecha`] to check which parts belong to it.
/// - [`Disabled`] - excludes a part from drawing, generating or storing power
///   entirely.
/// - [`Active`] to determine a consumer's power profile.
/// - [`Option<EnergyConsumption>`](EnergyConsumption) to calculate a part's
///   needed energy.
/// - [`Option<Generator>`](Generator) for its rated [`Power`].
/// - [`Option<Battery>`](Battery) to use as energy source and sink.
///
/// Writes:
/// - [`Unpowered`] for every power-consuming part whose demand wasn't fully
///   met.
/// - [`Active`] for every non-[`Disabled`] [`Battery`] if the tree drew any
///   [`Energy`], and non-[`Disabled`] [`Generator`] if it charged a battery,
///   and unmarked otherwise.
/// - [`Battery`] to update its charge.
///
/// Removes:
/// - [`Unpowered`] for every power-consuming part whose demand was met.
/// - [`Active`] for every non-[`Disabled`] [`Battery`] or [`Generator`] that
///   was used this call.
pub fn mecha_energy_system(world: &mut World, delta_time: Duration) {
    let mechas: Vec<Mecha> = world.query::<&Mecha>().iter().cloned().collect();

    for mecha in mechas {
        let mut totals = mecha
            .parts()
            .filter_map(|entity| part_totals(world, entity))
            .fold(PartTotals::default(), PartTotals::combine);

        let generated = total_generation(&totals.generators, delta_time);
        let charged = charge_batteries(world, generated, &mut totals.batteries);

        let demand = totals.draw.energy(delta_time).unwrap_or_default();
        let shortfall = drain_batteries(world, demand, &mut totals.batteries);

        set_consumers_unpowered(world, &totals.consumers, !shortfall.is_zero());
        activate_or_deactivate(world, &totals.batteries, !totals.draw.is_zero());
        activate_or_deactivate(world, &totals.generators, !charged.is_zero());
    }
}

/// A single part's contribution to its mecha's power [`PartTotals`].
/// Returns [`None`] if part is [`Disabled`] or has been despawned
/// since the tree was built.
fn part_totals(world: &World, entity: Entity) -> Option<PartTotals> {
    let mut query = world.query_one::<Without<
        (
            Satisfies<&Active>,
            Option<&Generator>,
            Option<&Battery>,
            Option<&EnergyConsumption>,
        ),
        &Disabled,
    >>(entity);
    let (is_active, generator, battery, power_drain) = query.get().ok()?;

    Some(PartTotals {
        draw: power_drain.map_or(Power::ZERO, |drain| drain.draw(is_active)),
        consumers: power_drain.map(|_| entity).into_iter().collect(),
        batteries: battery
            .map(|&battery| (entity, battery))
            .into_iter()
            .collect(),
        generators: generator
            .map(|&generator| (entity, generator))
            .into_iter()
            .collect(),
    })
}

/// Total [`Energy`] the tree's generators could
/// contribute, from their rated [`Power`] alone.
fn total_generation(generators: &[(Entity, Generator)], delta_time: Duration) -> Energy {
    generators
        .iter()
        .map(|(_, generator)| generator.power())
        .fold(Power::ZERO, Power::saturating_add)
        .energy(delta_time)
        .unwrap_or_default()
}

/// Distributes `amount` across `batteries` as evenly as possible:
/// if a battery can't take its equal share, what remains is given
/// to the others. Returns how much was actually moved.
fn distribute_evenly(
    world: &mut World,
    batteries: &mut [(Entity, Battery)],
    amount: Energy,
    capacity_of: impl Fn(&Battery) -> Energy,
    apply: impl Fn(&mut Battery, Energy) -> Energy,
) -> Energy {
    batteries.sort_by_key(|(_, battery)| capacity_of(battery));

    let total = batteries.len();
    let mut remaining = amount;
    let mut moved = Energy::ZERO;

    for (i, (entity, battery)) in batteries.iter_mut().enumerate() {
        if remaining.is_zero() {
            break;
        }

        let count = total - i;
        let share = remaining
            .checked_mul(1.0 / count as f32)
            .expect("divisor is in [0.0, 1.0] which yields a finite share after mul");
        let offer = capacity_of(battery).min(share);

        let taken = apply(battery, offer);
        let _ = world.insert_one(*entity, *battery);

        remaining = remaining.saturating_sub(taken);
        moved = moved.saturating_add(taken);
    }

    moved
}

/// Charges `batteries` from `generation`, sharing
/// it evenly, until it's fully used or every battery
/// is full. Returns how much was accepted.
fn charge_batteries(
    world: &mut World,
    generation: Energy,
    batteries: &mut [(Entity, Battery)],
) -> Energy {
    distribute_evenly(
        world,
        batteries,
        generation,
        Battery::headroom,
        Battery::charge,
    )
}

/// Discharges `batteries` to cover `demand`, sharing
/// the load evenly, until it's fully met or every battery
/// runs dry. Returns whatever demand couldn't be covered.
fn drain_batteries(
    world: &mut World,
    demand: Energy,
    batteries: &mut [(Entity, Battery)],
) -> Energy {
    let covered = distribute_evenly(
        world,
        batteries,
        demand,
        Battery::available,
        Battery::discharge,
    );
    demand.saturating_sub(covered)
}

/// Marks every consumer [`Unpowered`] if the tree
/// couldn't cover its demand, and unmarks it otherwise.
fn set_consumers_unpowered(world: &mut World, consumers: &[Entity], starved: bool) {
    if starved {
        consumers.iter().for_each(|&entity| {
            let _ = world.insert_one(entity, Unpowered);
        });
    } else {
        consumers.iter().for_each(|&entity| {
            let _ = world.remove_one::<Unpowered>(entity);
        });
    }
}

/// Marks every `T` as [`Active`] if `activate`
/// is `true`, and unmarks it otherwise.
fn activate_or_deactivate<T>(world: &mut World, ts: &[(Entity, T)], activate: bool) {
    if activate {
        ts.iter().for_each(|(entity, _)| {
            let _ = world.insert_one(*entity, Active);
        });
    } else {
        ts.iter().for_each(|(entity, _)| {
            let _ = world.remove_one::<Active>(*entity);
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::component::mecha::{build_mecha, part::Part};

    use super::*;

    struct TestMecha {
        root: Entity,
        frame: Entity,
    }

    impl TestMecha {
        fn rebuild(&self, world: &mut World) {
            let tree = build_mecha(world, self.frame);
            world.insert_one(self.root, tree).unwrap();
        }
    }

    fn mecha(world: &mut World) -> TestMecha {
        let root = world.spawn(());
        let frame = world.spawn((Part::new("frame".to_string(), Some(root)).unwrap(),));
        TestMecha { root, frame }
    }

    fn joules(value: f32) -> Energy {
        Energy::from_joules_f32(value)
    }

    fn watts(value: f32) -> Power {
        Power::from_watts_f32(value)
    }

    fn spawn_consumer(
        world: &mut World,
        mounted_on: Entity,
        consumption: EnergyConsumption,
        active: bool,
    ) -> Entity {
        let entity = world.spawn((
            Part::new("consumer".to_string(), Some(mounted_on)).unwrap(),
            consumption,
        ));
        if active {
            world.insert_one(entity, Active).unwrap();
        }
        entity
    }

    fn spawn_battery(world: &mut World, mounted_on: Entity, battery: Battery) -> Entity {
        world.spawn((
            Part::new("battery".to_string(), Some(mounted_on)).unwrap(),
            battery,
        ))
    }

    fn spawn_generator(world: &mut World, mounted_on: Entity, power: Power) -> Entity {
        world.spawn((
            Part::new("generator".to_string(), Some(mounted_on)).unwrap(),
            Generator::from(power),
        ))
    }

    #[test]
    fn battery_with_enough_charge_covers_demand_and_stays_online() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(10.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert!(world.get::<&Unpowered>(consumer).is_err());
        assert_eq!(battery.charge_ratio(), 0.9);
    }

    #[test]
    fn consumer_is_marked_unpowered_when_demand_cannot_be_fully_met() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(10.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(1000.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert!(world.get::<&Unpowered>(consumer).is_ok());
        assert_eq!(battery.charge_ratio(), 0.0);
    }

    #[test]
    fn unpowered_is_cleared_once_demand_can_be_met_again() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let _battery = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(10.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(1000.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));
        assert!(world.get::<&Unpowered>(consumer).is_ok());

        world.remove_one::<Active>(consumer).unwrap();
        mecha_energy_system(&mut world, Duration::from_secs(1));
        assert!(world.get::<&Unpowered>(consumer).is_err());
    }

    #[test]
    fn load_is_shared_evenly_across_batteries() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let first = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let second = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let _consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(20.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let first = *world.get::<&Battery>(first).unwrap();
        let second = *world.get::<&Battery>(second).unwrap();
        assert_eq!(first.charge_ratio(), 0.9);
        assert_eq!(second.charge_ratio(), 0.9);
    }

    #[test]
    fn demand_exceeding_one_battery_falls_to_the_rest() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let small = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(10.0)).unwrap(),
        );
        let large = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(1000.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(90.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let small = *world.get::<&Battery>(small).unwrap();
        let large = *world.get::<&Battery>(large).unwrap();
        assert!(world.get::<&Unpowered>(consumer).is_err());
        assert_eq!(small.charge_ratio(), 0.0);
        assert_eq!(large.charge_ratio(), 0.92);
    }

    #[test]
    fn disabled_battery_is_never_discharged() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let online_battery = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let disabled_battery = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let _consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(10.0)),
            true,
        );
        world.insert_one(disabled_battery, Disabled).unwrap();
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let disabled_battery_state = *world.get::<&Battery>(disabled_battery).unwrap();
        let online_battery = *world.get::<&Battery>(online_battery).unwrap();
        assert_eq!(disabled_battery_state.charge_ratio(), 1.0);
        assert!(world.get::<&Active>(disabled_battery).is_err());
        assert_eq!(online_battery.charge_ratio(), 0.9);
    }

    #[test]
    fn disabled_consumer_draws_no_power() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(10.0)),
            true,
        );
        world.insert_one(consumer, Disabled).unwrap();
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge_ratio(), 1.0);
    }

    #[test]
    fn battery_is_marked_active_only_while_the_tree_draws_power() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let consumer = spawn_consumer(
            &mut world,
            mecha.frame,
            EnergyConsumption::new(watts(0.0), watts(10.0)),
            true,
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));
        assert!(world.get::<&Active>(battery_entity).is_ok());

        world.remove_one::<Active>(consumer).unwrap();
        mecha_energy_system(&mut world, Duration::from_secs(1));
        assert!(world.get::<&Active>(battery_entity).is_err());
    }

    #[test]
    fn no_consumers_means_no_draw_and_batteries_stay_idle() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge_ratio(), 1.0);
        assert!(world.get::<&Active>(battery_entity).is_err());
    }

    #[test]
    fn generator_charges_an_empty_battery_and_goes_active() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::new(joules(100.0), joules(0.0)).unwrap(),
        );
        let generator = spawn_generator(&mut world, mecha.frame, watts(10.0));
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge_ratio(), 0.1);
        assert!(world.get::<&Active>(generator).is_ok());
    }

    #[test]
    fn generator_goes_idle_once_every_battery_is_full() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::full(joules(100.0)).unwrap(),
        );
        let generator = spawn_generator(&mut world, mecha.frame, watts(10.0));
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge_ratio(), 1.0);
        assert!(world.get::<&Active>(generator).is_err());
    }

    #[test]
    fn generation_is_shared_evenly_across_batteries() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let first = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::new(joules(100.0), joules(0.0)).unwrap(),
        );
        let second = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::new(joules(100.0), joules(0.0)).unwrap(),
        );
        spawn_generator(&mut world, mecha.frame, watts(20.0));
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let first = *world.get::<&Battery>(first).unwrap();
        let second = *world.get::<&Battery>(second).unwrap();
        assert_eq!(first.charge_ratio(), 0.1);
        assert_eq!(second.charge_ratio(), 0.1);
    }

    #[test]
    fn disabled_generator_contributes_nothing() {
        let mut world = World::new();
        let mecha = mecha(&mut world);

        let battery_entity = spawn_battery(
            &mut world,
            mecha.frame,
            Battery::new(joules(100.0), joules(0.0)).unwrap(),
        );
        let generator = spawn_generator(&mut world, mecha.frame, watts(10.0));
        world.insert_one(generator, Disabled).unwrap();
        mecha.rebuild(&mut world);

        mecha_energy_system(&mut world, Duration::from_secs(1));

        let battery = *world.get::<&Battery>(battery_entity).unwrap();
        assert_eq!(battery.charge_ratio(), 0.0);
        assert!(world.get::<&Active>(generator).is_err());
    }
}
