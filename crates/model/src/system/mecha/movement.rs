use std::time::Duration;

use hecs::{Entity, Satisfies, With, World};
use log::warn;
use physics::{
    ForceMagnitude, Mass, Position, Speed, Velocity,
    ops::{CheckedMul, SaturatingAdd},
};

use crate::{
    component::{
        Active, Agility, Direction, Disabled, Moveable, Thrust,
        mecha::{
            Mecha,
            part::{Legs, Thruster},
        },
    },
    map::{Map, Surface},
};

const STOP_SPEED_MPS: f32 = 0.01;

#[derive(Default)]
struct PartTotals {
    mass: Mass,
    legs: Option<Legs>,
    thrust: Thrust,
}

impl PartTotals {
    fn combine(self, other: Self) -> Self {
        Self {
            mass: self.mass.saturating_add(other.mass),
            legs: self.legs.or(other.legs),
            thrust: Self::combine_thrust(self.thrust, other.thrust),
        }
    }

    fn combine_thrust(lhs: Thrust, rhs: Thrust) -> Thrust {
        let thrust_force = ForceMagnitude::from(lhs).saturating_add(rhs.into());
        Thrust::from(thrust_force)
    }
}

struct LegsState {
    top_speed: Speed,
    base_tau: Agility,
    throttle: f32,
}

impl LegsState {
    fn new(legs: Legs, mass: Mass) -> Self {
        let load_ratio = mass.as_kilograms_f32() / legs.load_capacity().as_kilograms_f32();
        let derate = (1.0 / load_ratio).min(1.0);

        Self {
            top_speed: legs
                .travel_speed()
                .checked_mul(derate)
                .unwrap_or(Speed::ZERO),
            base_tau: legs.agility().saturating_mul(load_ratio),
            throttle: legs.scale(),
        }
    }
}

/// Drives every mecha's [`Velocity`] towards what its legs are trying to do.
///
/// Reads:
/// - [`Moveable`] as only movable entities should have velocity.
/// - [`Mecha`] to walk part tree.
/// - [`Direction`] as the heading the legs are driving towards.
/// - [`Position`] to look up the terrain underfoot.
/// - [`Option<Velocity>`](Velocity) as the value being approached from.
/// - [`Active`] to only count thrust from active thrusters.
/// - [`Disabled`] - a disabled part still contributes its mass like any other,
///   but not its legs or thrust.
/// - [`Option<Mass>`](Mass) - summed across every present part regardless of
///   status.
/// - [`Option<Legs>`](Legs) - source of the speed and agility needed to move.
///   Its absence means no working legs, so the mecha freewheels.
/// - [`Option<Thruster>`](Thruster) - source of the thrust needed to boost.
///
/// Writes:
/// - [`Velocity`], derived from mecha's parts.
pub fn mecha_movement_system(world: &mut World, map: &dyn Map, delta_time: Duration) {
    let mechas: Vec<(Entity, Mecha, Direction, Position, Velocity)> = world
        .query::<With<(Entity, &Mecha, &Direction, &Position, Option<&Velocity>), &Moveable>>()
        .iter()
        .map(|(entity, mecha, direction, position, velocity)| {
            (
                entity,
                mecha.clone(),
                *direction,
                *position,
                velocity.copied().unwrap_or_default(),
            )
        })
        .collect();

    let updates: Vec<(Entity, Velocity)> = mechas
        .into_iter()
        .map(|(entity, mecha, direction, position, velocity)| {
            let totals = mecha
                .parts()
                .filter_map(|part| part_totals(world, part))
                .fold(PartTotals::default(), PartTotals::combine);

            let legs = if let Some(legs) = totals.legs {
                LegsState::new(legs, totals.mass)
            } else {
                return (entity, Velocity::ZERO);
            };

            let surface = {
                let point = position.as_meters_vec2();
                map.cell_at(point.x, point.y).surface
            };

            let boost = boost_multiplier(totals.thrust, totals.mass, map.gravity());

            let target = {
                let target_speed = target_speed(&legs, surface, boost);
                target_speed.velocity(direction.into())
            };

            let updated_result = {
                let tau = response_time(&legs, surface, boost);
                approach(velocity, target, tau, delta_time)
            };

            let updated = match updated_result {
                Ok(velocity) => velocity,
                Err(error) => {
                    warn!(
                        entity:? = entity,
                        error:? = error;
                        "velocity update failed, keeping previous velocity"
                    );
                    velocity
                }
            };

            (entity, settled(updated, target))
        })
        .collect();

    for (entity, velocity) in updates {
        let _ = world.insert_one(entity, velocity);
    }
}

/// A single part's contribution to [`PartTotals`],
/// or [`None`] if it's since been despawned.
fn part_totals(world: &World, entity: Entity) -> Option<PartTotals> {
    let mut query = world.query_one::<(
        Satisfies<&Active>,
        Satisfies<&Disabled>,
        Option<&Mass>,
        Option<&Legs>,
        Option<&Thruster>,
    )>(entity);
    let Ok((is_active, is_disabled, mass, legs, thruster)) = query.get() else {
        warn!(
            entity:? = entity;
            "failed to get stats from entity"
        );
        None?
    };

    Some(PartTotals {
        mass: mass.copied().unwrap_or_default(),
        legs: (!is_disabled).then(|| legs.copied()).flatten(),
        thrust: is_active
            .then(|| thruster.map(Thruster::thrust))
            .flatten()
            .unwrap_or_default(),
    })
}

/// Speed approaches its target using an exponential model:
/// `alpha = 1 - exp(-dt / tau)`.
fn approach_factor(tau: Duration, delta_time: Duration) -> f32 {
    if tau.is_zero() {
        return 1.0;
    }
    let tau_fraction = delta_time.as_secs_f32() / tau.as_secs_f32();
    1.0 - (-tau_fraction).exp()
}

/// Moves `current` a [`approach_factor`] share of the way towards `target`.
fn approach(
    current: Velocity,
    target: Velocity,
    tau: Duration,
    delta_time: Duration,
) -> Result<Velocity, &'static str> {
    let alpha = approach_factor(tau, delta_time);
    let updated = current
        .as_meters_per_second_vec2()
        .lerp(target.as_meters_per_second_vec2(), alpha);

    Velocity::try_from_meters_per_second_vec2(updated).map_err(|err| err.message())
}

/// Top speed multiplier the thrusters give.
/// Derived from thrust-to-weight ratio.
fn boost_multiplier(thrust: Thrust, mass: Mass, gravity: f32) -> f32 {
    if mass.is_zero() {
        return 1.0;
    }
    let newtons = ForceMagnitude::from(thrust).as_newtons_f32();
    let weight = mass.as_kilograms_f32() * gravity;
    let ratio = newtons / weight;

    if ratio.is_finite() { 1.0 + ratio } else { 1.0 }
}

/// How long this mecha takes to close ~63% of the gap to its target velocity.
fn response_time(legs: &LegsState, surface: Surface, boost: f32) -> Duration {
    let ground = 1.0 + surface.rolling_resistance();
    let scale = ground / boost;
    legs.base_tau.saturating_mul(scale).into()
}

/// The speed this mecha is currently trying to reach.
fn target_speed(legs: &LegsState, surface: Surface, boost: f32) -> Speed {
    let grip = surface.traction().min(1.0);
    let scale = legs.throttle * grip * boost;
    legs.top_speed.checked_mul(scale).unwrap_or(Speed::ZERO)
}

/// Snaps velocity to zero for entity with low speed that's slowing down.
fn settled(velocity: Velocity, target: Velocity) -> Velocity {
    let is_low = velocity
        .speed()
        .is_ok_and(|speed| speed.as_meters_per_second_f32() < STOP_SPEED_MPS);

    if target.is_zero() && is_low {
        Velocity::ZERO
    } else {
        velocity
    }
}

#[cfg(test)]
mod tests {
    use hecs::World;
    use physics::{Density, Length};

    use super::*;
    use crate::{
        component::{
            Temperature,
            mecha::{build_mecha, part::Part},
        },
        map::Cell,
    };

    struct MapStub(Surface);

    impl Default for MapStub {
        fn default() -> Self {
            Self(Surface::Grass)
        }
    }

    impl Map for MapStub {
        fn cell_at(&self, _x: f32, _y: f32) -> Cell {
            Cell {
                surface: self.0,
                elevation: Length::from_meters_f32(1.0),
                structure: None,
                liquid: None,
            }
        }

        fn temperature(&self) -> Temperature {
            Temperature::from_celsius_f32(20.0)
        }

        fn air_density(&self) -> Density {
            Density::from_kilograms_per_cubic_meter_f32(1.2255)
        }

        fn gravity(&self) -> f32 {
            9.8
        }
    }

    fn legs(mps: f32, capacity_kg: f32, tau_seconds: f32) -> Legs {
        Legs::new(
            Speed::from_meters_per_second_f32(mps),
            Mass::from_kilograms_f32(capacity_kg),
            Agility::from(Duration::from_secs_f32(tau_seconds)),
        )
        .unwrap()
    }

    fn spawn_root(world: &mut World) -> Entity {
        world.spawn((
            Moveable,
            Position::new(20.0, 20.0).unwrap(),
            Direction::default(),
        ))
    }

    fn spawn_frame(world: &mut World, root: Entity, mass_kg: f32) -> Entity {
        world.spawn((
            Part::new("frame".to_string(), Some(root)).unwrap(),
            Mass::from_kilograms_f32(mass_kg),
        ))
    }

    fn spawn_legs(world: &mut World, mounted_on: Entity, mut legs: Legs, throttle: f32) -> Entity {
        legs.set_scale(throttle).unwrap();
        world.spawn((
            Part::new("legs".to_string(), Some(mounted_on)).unwrap(),
            legs,
        ))
    }

    fn rebuild_mecha(world: &mut World, root: Entity, frame: Entity) {
        let mecha = build_mecha(world, frame);
        world.insert_one(root, mecha).unwrap();
    }

    fn spawn_driving_mecha(
        world: &mut World,
        legs_stats: Option<Legs>,
        frame_mass_kg: f32,
    ) -> Entity {
        let root = spawn_root(world);
        let frame = spawn_frame(world, root, frame_mass_kg);
        if let Some(legs_stats) = legs_stats {
            spawn_legs(world, frame, legs_stats, 1.0);
        }
        rebuild_mecha(world, root, frame);
        root
    }

    fn speed_of(world: &World, entity: Entity) -> f32 {
        world
            .get::<&Velocity>(entity)
            .unwrap()
            .speed()
            .unwrap_or_default()
            .as_meters_per_second_f32()
    }

    fn run(world: &mut World, map: &dyn Map, ticks: u32, delta_time: Duration) {
        for _ in 0..ticks {
            mecha_movement_system(world, map, delta_time);
        }
    }

    #[test]
    fn converges_to_top_speed_without_exceeding_it() {
        let mut world = World::new();
        let map = MapStub::default();
        let root = spawn_driving_mecha(&mut world, Some(legs(14.0, 9000.0, 0.3)), 4000.0);

        run(&mut world, &map, 200, Duration::from_millis(50));

        let speed = speed_of(&world, root);
        assert!((8.3..=8.4).contains(&speed), "speed = {speed}");
    }

    #[test]
    fn never_overshoots_the_target_with_a_huge_tick() {
        let mut world = World::new();
        let map = MapStub::default();
        let root = spawn_driving_mecha(&mut world, Some(legs(14.0, 9000.0, 0.3)), 4000.0);

        run(&mut world, &map, 1, Duration::from_secs(10));

        assert!(speed_of(&world, root) <= 14.0);
    }

    #[test]
    fn releasing_the_throttle_decays_to_an_exact_stop() {
        let mut world = World::new();
        let map = MapStub::default();
        let root = spawn_root(&mut world);
        let frame = spawn_frame(&mut world, root, 4000.0);
        let legs_entity = spawn_legs(&mut world, frame, legs(14.0, 9000.0, 0.3), 1.0);
        rebuild_mecha(&mut world, root, frame);
        run(&mut world, &map, 100, Duration::from_millis(50));

        world.get::<&mut Legs>(legs_entity).unwrap().reset_scale();
        run(&mut world, &map, 400, Duration::from_millis(50));

        assert_eq!(*world.get::<&Velocity>(root).unwrap(), Velocity::ZERO);
    }

    #[test]
    fn boost_raises_top_speed() {
        let boosted_speed = |thrust_newtons: Option<f32>| {
            let mut world = World::new();
            let map = MapStub::default();
            let root = spawn_root(&mut world);
            let frame = spawn_frame(&mut world, root, 4000.0);
            spawn_legs(&mut world, frame, legs(14.0, 9000.0, 0.3), 1.0);
            if let Some(newtons) = thrust_newtons {
                let thrust = Thrust::from(ForceMagnitude::from_newtons_f32(newtons));
                world.spawn((
                    Part::new("thruster".to_string(), Some(frame)).unwrap(),
                    Thruster::from(thrust),
                    Active,
                ));
            }
            rebuild_mecha(&mut world, root, frame);

            run(&mut world, &map, 200, Duration::from_millis(50));
            speed_of(&world, root)
        };

        assert!(boosted_speed(Some(50_000.0)) > boosted_speed(None));
    }

    #[test]
    fn slippery_surface_lowers_top_speed() {
        let settled = |surface| {
            let mut world = World::new();
            let map = MapStub(surface);
            let root = spawn_driving_mecha(&mut world, Some(legs(14.0, 9000.0, 0.3)), 4000.0);
            run(&mut world, &map, 200, Duration::from_millis(50));
            speed_of(&world, root)
        };

        assert!(Surface::Beach.traction() < Surface::Grass.traction());
        assert!(settled(Surface::Beach) < settled(Surface::Grass));
    }

    #[test]
    fn mecha_without_legs_coasts_to_a_stop() {
        let mut world = World::new();
        let map = MapStub::default();
        let root = spawn_driving_mecha(&mut world, None, 4000.0);
        world
            .insert_one(
                root,
                Velocity::from_meters_per_second_vec2(glam::Vec2::new(10.0, 0.0)),
            )
            .unwrap();

        run(&mut world, &map, 200, Duration::from_millis(50));

        assert_eq!(*world.get::<&Velocity>(root).unwrap(), Velocity::ZERO);
    }

    #[test]
    fn disabled_legs_stop_contributing_speed() {
        let mut world = World::new();
        let map = MapStub::default();
        let root = spawn_root(&mut world);
        let frame = spawn_frame(&mut world, root, 4000.0);
        let legs_entity = spawn_legs(&mut world, frame, legs(14.0, 9000.0, 0.3), 1.0);
        rebuild_mecha(&mut world, root, frame);

        run(&mut world, &map, 200, Duration::from_millis(50));
        assert!(speed_of(&world, root) > 0.0);

        world.insert_one(legs_entity, Disabled).unwrap();
        run(&mut world, &map, 400, Duration::from_millis(50));

        assert_eq!(*world.get::<&Velocity>(root).unwrap(), Velocity::ZERO);
    }
}
