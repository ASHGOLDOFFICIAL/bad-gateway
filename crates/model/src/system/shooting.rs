use std::time::Duration;

use hecs::{Entity, Satisfies, World};
use log::trace;
use physics::{Angle, Length, Position, Speed};
use rand::Rng;

use crate::component::{
    Active, Aim, Direction, Lifetime, Moveable, Projectile, RangedWeapon, ShootCooldown, TextureId,
    Visual,
};

/// How far a projectile travels before despawning.
const PROJECTILE_RANGE: f32 = 400.0;

struct Shot {
    pub direction: Angle,
    pub position: Position,
    pub speed: Speed,
    pub spread: Angle,
}

/// Fires a [`Projectile`] from every [`RangedWeapon`] entity that's
/// [`Active`] and off [`ShootCooldown`].
///
/// Reads:
/// - [`Active`] as only weapons active should fire.
/// - [`Aim`] to get current shoot direction.
/// - [`Position`] to get current shoot origin.
/// - [`RangedWeapon`] to derive [`Projectile`] components.
/// - [`ShootCooldown`] to check if shooting is allowed.
///
/// Writes:
/// - [`ShootCooldown`] to reduce cooldown and/or reset to that weapon's fire
///   interval after it fires.
///
/// Spawns:
/// - A [`Projectile`] per weapon entity whose cooldown allows it.
pub fn shooting_system(world: &mut World, delta_time: Duration) {
    let shots: Vec<Shot> = world
        .query::<(
            Satisfies<&Active>,
            &Aim,
            &Position,
            &RangedWeapon,
            &mut ShootCooldown,
        )>()
        .iter()
        .filter_map(|(is_active, &aim, &position, weapon, cooldown)| {
            cooldown.reduce(delta_time);

            if !is_active || !cooldown.is_ready() {
                return None;
            }

            cooldown.reset(weapon.fire_rate().interval());

            Some(Shot {
                direction: Angle::from(aim),
                position,
                speed: weapon.muzzle_speed(),
                spread: weapon.half_spread(),
            })
        })
        .collect();

    shots.iter().for_each(|shot| {
        let jittered = jitter(shot.direction, shot.spread);
        spawn_projectile(world, shot.position, jittered, shot.speed);
    });
}

/// Randomly deviates `direction` by up to `half_spread` either way.
fn jitter(direction: Angle, half_spread: Angle) -> Angle {
    let half = half_spread.as_radians_f32().abs();
    if half <= 0.0 {
        return direction;
    }
    let offset = rand::thread_rng().gen_range(-half..=half);
    direction + Angle::from_radians_f32(offset)
}

fn spawn_projectile(
    world: &mut World,
    position: Position,
    direction: Angle,
    speed: Speed,
) -> Entity {
    let velocity = speed.velocity(direction);
    let range = Length::from_meters_f32(PROJECTILE_RANGE);
    let lifetime = range.duration(speed).unwrap();

    let entity = world.spawn((
        Projectile,
        Moveable,
        position,
        velocity,
        Direction::from(direction),
        Lifetime::from(lifetime),
        Visual::new(TextureId::Bullets),
    ));

    trace!(
        entity:? = entity,
        position:? = position,
        direction:? = direction;
        "projectile spawned"
    );

    entity
}

#[cfg(test)]
mod tests {
    use hecs::Entity;
    use physics::{Frequency, Velocity};

    use super::*;
    use crate::component::{AmmoType, FireRate};

    fn weapon() -> RangedWeapon {
        RangedWeapon::new(
            FireRate::from(Frequency::from_hertz_f32(2.0)),
            AmmoType::Bullets,
            Speed::from_meters_per_second_f32(10.0),
            Angle::default(),
        )
        .unwrap()
    }

    fn spawn_weapon(
        world: &mut World,
        active: bool,
        cooldown: ShootCooldown,
        aim: Angle,
        position: Position,
    ) -> Entity {
        let entity = world.spawn((Aim::from(aim), position, weapon(), cooldown));
        if active {
            world.insert_one(entity, Active).unwrap();
        }
        entity
    }

    fn projectiles(world: &World) -> Vec<Entity> {
        world
            .query::<(Entity, &Projectile)>()
            .iter()
            .map(|(entity, _)| entity)
            .collect()
    }

    #[test]
    fn fires_projectile_when_active_and_cooldown_ready() {
        let mut world = World::new();
        let position = Position::new(3.0, 4.0).unwrap();
        spawn_weapon(
            &mut world,
            true,
            ShootCooldown::READY,
            Angle::default(),
            position,
        );

        shooting_system(&mut world, Duration::from_millis(67));

        let fired = projectiles(&world);
        assert_eq!(fired.len(), 1);
        assert_eq!(*world.get::<&Position>(fired[0]).unwrap(), position);
    }

    #[test]
    fn does_not_fire_when_not_active() {
        let mut world = World::new();
        spawn_weapon(
            &mut world,
            false,
            ShootCooldown::READY,
            Angle::default(),
            Position::ORIGIN,
        );

        shooting_system(&mut world, Duration::from_millis(67));

        assert!(projectiles(&world).is_empty());
    }

    #[test]
    fn does_not_fire_when_cooldown_is_not_ready() {
        let mut world = World::new();
        let mut cooldown = ShootCooldown::READY;
        cooldown.reset(Duration::from_secs(1));
        spawn_weapon(
            &mut world,
            true,
            cooldown,
            Angle::default(),
            Position::ORIGIN,
        );

        shooting_system(&mut world, Duration::from_millis(67));

        assert!(projectiles(&world).is_empty());
    }

    #[test]
    fn reduces_cooldown() {
        let mut world = World::new();
        let mut cooldown = ShootCooldown::READY;
        cooldown.reset(Duration::from_secs(1));
        let weapon_entity = spawn_weapon(
            &mut world,
            false,
            cooldown,
            Angle::default(),
            Position::ORIGIN,
        );

        shooting_system(&mut world, Duration::from_millis(400));

        let mut expected = ShootCooldown::READY;
        expected.reset(Duration::from_millis(600));
        let cooldown = *world.get::<&ShootCooldown>(weapon_entity).unwrap();
        assert_eq!(cooldown, expected);
    }

    #[test]
    fn resets_cooldown_after_firing() {
        let mut world = World::new();
        let weapon_entity = spawn_weapon(
            &mut world,
            true,
            ShootCooldown::READY,
            Angle::default(),
            Position::ORIGIN,
        );

        shooting_system(&mut world, Duration::from_millis(16));

        let mut expected = ShootCooldown::READY;
        expected.reset(weapon().fire_rate().interval());
        let cooldown = *world.get::<&ShootCooldown>(weapon_entity).unwrap();
        assert_eq!(cooldown, expected);
    }

    #[test]
    fn projectiles_travels_along_aim_direction() {
        let mut world = World::new();
        let aim = Angle::from_degrees_f32(90.0);
        spawn_weapon(
            &mut world,
            true,
            ShootCooldown::READY,
            aim,
            Position::ORIGIN,
        );

        shooting_system(&mut world, Duration::from_millis(16));

        let fired = projectiles(&world);
        let velocity = *world.get::<&Velocity>(fired[0]).unwrap();
        assert_eq!(velocity, weapon().muzzle_speed().velocity(aim));
    }
}
