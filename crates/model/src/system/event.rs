use hecs::{Entity, World};
use log::{debug, warn};
use physics::Angle;

use crate::{
    component::{
        Aim, Direction,
        mecha::control::{LegsActive, RangedWeaponsActive, ThrustersActive},
    },
    event::UserGameEvent,
};

/// Parses [`UserGameEvent`] into game components.
///
/// Writes:
/// - [`Direction`] to set it to the movement input vector.
/// - [`Aim`] to set it to the aim input vector's direction.
/// - [`LegsActive`] to set it to the length of movement input vector.
/// - [`RangedWeaponsActive`] to set it to the given fire scale clamped between
///   0 and 1.
/// - [`ThrustersActive`] to set it to the given scale clamped between 0 and 1.
///
/// Removes:
/// - [`LegsActive`] if the movement vector is zero or otherwise invalid.
/// - [`RangedWeaponsActive`] if the fire scale is at or below zero.
/// - [`ThrustersActive`] if the thrusters' scale is at or below zero.
pub fn user_event_system(world: &mut World, player: Entity, event: &UserGameEvent) {
    match event {
        UserGameEvent::SetMove(vector) => handle_set_move(world, player, *vector),
        UserGameEvent::SetAim(vector) => handle_set_aim(world, player, *vector),
        UserGameEvent::Fire(scale) => handle_fire(world, player, *scale),
        UserGameEvent::Boost(scale) => handle_thruster(world, player, *scale),
    }
}

fn handle_set_move(world: &mut World, player: Entity, vector: glam::Vec2) {
    let Ok(angle) = Angle::try_from_vec2_f32(vector) else {
        let _ = world.remove_one::<LegsActive>(player);
        return;
    };

    let scale = vector.length().min(1.0);

    match LegsActive::new(scale) {
        Ok(active) => {
            debug!(entity:? = player, scale; "legs active");
            let direction = Direction::from(angle);
            let _ = world.insert(player, (direction, active));
        }
        Err(error) => {
            warn!(entity:? = player, scale, error; "failed to activate legs");
        }
    }
}

fn handle_set_aim(world: &mut World, player: Entity, vector: glam::Vec2) {
    let Ok(angle) = Angle::try_from_vec2_f32(vector) else {
        return;
    };
    debug!(entity:? = player, angle:? = angle; "aim set");
    let _ = world.insert_one(player, Aim::from(angle));
}

fn handle_fire(world: &mut World, player: Entity, scale: f32) {
    if scale <= 0.0 {
        let _ = world.remove_one::<RangedWeaponsActive>(player);
        return;
    }

    let clamped = scale.clamp(0.0, 1.0);
    match RangedWeaponsActive::new(clamped) {
        Ok(active) => {
            debug!(entity:? = player, scale = clamped; "ranged weapons active");
            let _ = world.insert_one(player, active);
        }
        Err(error) => {
            warn!(
                entity:? = player,
                scale = clamped,
                error;
                "failed to activate ranged weapons"
            );
        }
    }
}

fn handle_thruster(world: &mut World, player: Entity, scale: f32) {
    if scale <= 0.0 {
        let _ = world.remove_one::<ThrustersActive>(player);
        return;
    }
    let clamped = scale.clamp(0.0, 1.0);
    match ThrustersActive::new(clamped) {
        Ok(active) => {
            debug!(entity:? = player, scale = clamped; "boost active");
            let _ = world.insert_one(player, active);
        }
        Err(error) => {
            warn!(entity:? = player, scale = clamped, error; "failed to activate boost");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spawn_player(world: &mut World) -> Entity {
        world.spawn(())
    }

    #[test]
    fn set_move_sets_direction_and_legs_active() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(1.0, 0.0);

        user_event_system(&mut world, player, &UserGameEvent::SetMove(vector));

        let expected = Direction::from(Angle::from_vec2_f32(vector));
        assert_eq!(*world.get::<&Direction>(player).unwrap(), expected);
        assert_eq!(world.get::<&LegsActive>(player).unwrap().scale(), 1.0);
    }

    #[test]
    fn set_move_scales_legs_by_vector_length() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(0.5, 0.0);

        user_event_system(&mut world, player, &UserGameEvent::SetMove(vector));

        let expected = Direction::from(Angle::from_vec2_f32(vector));
        assert_eq!(*world.get::<&Direction>(player).unwrap(), expected);
        assert_eq!(world.get::<&LegsActive>(player).unwrap().scale(), 0.5);
    }

    #[test]
    fn set_move_clamps_legs_above_one() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(5.0, 0.0);

        user_event_system(&mut world, player, &UserGameEvent::SetMove(vector));

        let expected = Direction::from(Angle::from_vec2_f32(vector));
        assert_eq!(*world.get::<&Direction>(player).unwrap(), expected);
        assert_eq!(world.get::<&LegsActive>(player).unwrap().scale(), 1.0);
    }

    #[test]
    fn set_move_removes_legs_active_on_non_finite_vector() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(f32::NAN, 0.0);

        let direction = Direction::from(Angle::from_degrees_f32(67.0));
        let legs = LegsActive::new(0.67).unwrap();
        world.insert(player, (direction, legs)).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::SetMove(vector));

        assert_eq!(*world.get::<&Direction>(player).unwrap(), direction);
        assert!(world.get::<&LegsActive>(player).is_err());
    }

    #[test]
    fn set_move_removes_legs_active_on_zero_vector() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::ZERO;

        let direction = Direction::from(Angle::from_degrees_f32(67.0));
        let legs = LegsActive::new(0.67).unwrap();
        world.insert(player, (direction, legs)).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::SetMove(vector));

        assert_eq!(*world.get::<&Direction>(player).unwrap(), direction);
        assert!(world.get::<&LegsActive>(player).is_err());
    }

    #[test]
    fn set_aim_sets_aim_direction() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(1.0, 0.0);

        user_event_system(&mut world, player, &UserGameEvent::SetAim(vector));

        let expected = Aim::from(Angle::from_vec2_f32(vector));
        assert_eq!(*world.get::<&Aim>(player).unwrap(), expected);
    }

    #[test]
    fn set_aim_does_not_touch_ranged_weapons_active() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        user_event_system(
            &mut world,
            player,
            &UserGameEvent::SetAim(glam::Vec2::new(1.0, 0.0)),
        );

        assert!(world.get::<&RangedWeaponsActive>(player).is_err());
    }

    #[test]
    fn set_aim_leaves_aim_unchanged_on_non_finite_vector() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::new(f32::NAN, 0.0);

        let angle = Aim::from(Angle::from_degrees_f32(67.0));
        world.insert_one(player, angle).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::SetAim(vector));

        assert_eq!(*world.get::<&Aim>(player).unwrap(), angle);
    }

    #[test]
    fn set_aim_leaves_aim_unchanged_on_zero_vector() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let vector = glam::Vec2::ZERO;

        let angle = Aim::from(Angle::from_degrees_f32(67.0));
        world.insert_one(player, angle).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::SetAim(vector));

        assert_eq!(*world.get::<&Aim>(player).unwrap(), angle);
    }

    #[test]
    fn fire_sets_ranged_weapons_active() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        user_event_system(&mut world, player, &UserGameEvent::Fire(0.6));

        let scale = world.get::<&RangedWeaponsActive>(player).unwrap().scale();
        assert_eq!(scale, 0.6);
    }

    #[test]
    fn fire_clamps_scale_above_one() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        user_event_system(&mut world, player, &UserGameEvent::Fire(2.0));

        let scale = world.get::<&RangedWeaponsActive>(player).unwrap().scale();
        assert_eq!(scale, 1.0);
    }

    #[test]
    fn fire_removes_ranged_weapons_active_on_zero() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        let weapons = RangedWeaponsActive::new(0.67).unwrap();
        world.insert_one(player, weapons).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::Fire(0.0));

        assert!(world.get::<&RangedWeaponsActive>(player).is_err());
    }

    #[test]
    fn fire_does_not_touch_aim() {
        let mut world = World::new();
        let player = spawn_player(&mut world);
        let angle = Aim::from(Angle::from_degrees_f32(67.0));
        world.insert_one(player, angle).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::Fire(1.0));

        assert_eq!(*world.get::<&Aim>(player).unwrap(), angle);
    }

    #[test]
    fn boost_sets_boost_active() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        user_event_system(&mut world, player, &UserGameEvent::Boost(0.6));

        assert_eq!(world.get::<&ThrustersActive>(player).unwrap().scale(), 0.6);
    }

    #[test]
    fn boost_clamps_scale_above_one() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        user_event_system(&mut world, player, &UserGameEvent::Boost(2.0));

        assert_eq!(world.get::<&ThrustersActive>(player).unwrap().scale(), 1.0);
    }

    #[test]
    fn boost_removes_boost_active_on_zero() {
        let mut world = World::new();
        let player = spawn_player(&mut world);

        let boost = ThrustersActive::new(0.67).unwrap();
        world.insert_one(player, boost).unwrap();

        user_event_system(&mut world, player, &UserGameEvent::Boost(0.0));

        assert!(world.get::<&ThrustersActive>(player).is_err());
    }
}
