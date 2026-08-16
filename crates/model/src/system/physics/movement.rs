use crate::component::Moveable;
use hecs::{Entity, With, World};
use log::{trace, warn};
use physics::{Displacement, Position, ops::CheckedAdd};

/// Moves each object by the [`Displacement`] allowed it this tick.
///
/// Reads:
/// - [`Moveable`] - only moveable objects should move.
/// - [`Position`] - movement origin.
/// - [`Displacement`] - how far to move.
///
/// Writes:
/// - [`Position`] - changes object's position to displacement destination.
pub fn movement_system(world: &mut World) {
    world
        .query::<With<(Entity, &mut Position, &Displacement), &Moveable>>()
        .iter()
        .for_each(|(entity, position, displacement)| {
            update_position(entity, position, *displacement)
        })
}

fn update_position(entity: Entity, current: &mut Position, displacement: Displacement) {
    if displacement.is_zero() {
        return;
    }

    match current.checked_add(displacement) {
        Some(updated) => {
            trace!(
                entity:? = entity,
                position:? = updated;
                "position updated"
            );
            *current = updated;
        }
        None => {
            warn!(
                entity:? = entity,
                position:? = current,
                displacement:? = displacement;
                "position update failed"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn displacement(x: f32, y: f32) -> Displacement {
        Displacement::from_meters_vec2(glam::Vec2::new(x, y))
    }

    #[test]
    fn ignores_entities_without_moveable() {
        let mut world = World::new();

        let start = Position::new(10.0, 5.0).unwrap();
        let entity = world.spawn((start, displacement(2.0, 0.0)));

        movement_system(&mut world);

        assert_eq!(*world.get::<&Position>(entity).unwrap(), start);
    }

    #[test]
    fn ignores_entities_without_a_displacement() {
        let mut world = World::new();

        let start = Position::new(10.0, 5.0).unwrap();
        let entity = world.spawn((Moveable, start));

        movement_system(&mut world);

        assert_eq!(*world.get::<&Position>(entity).unwrap(), start);
    }

    #[test]
    fn moves_position_by_its_displacement() {
        let mut world = World::new();

        let start = Position::new(10.0, 5.0).unwrap();
        let entity = world.spawn((Moveable, start, displacement(4.0, -2.0)));

        movement_system(&mut world);

        let expected = Position::new(14.0, 3.0).unwrap();
        assert_eq!(*world.get::<&Position>(entity).unwrap(), expected);
    }

    #[test]
    fn moves_each_moveable_entity_independently() {
        let mut world = World::new();

        let a_start = Position::new(0.0, 0.0).unwrap();
        let a = world.spawn((Moveable, a_start, displacement(2.0, 0.0)));

        let b_start = Position::new(100.0, 100.0).unwrap();
        let b = world.spawn((Moveable, b_start, displacement(0.0, -1.0)));

        movement_system(&mut world);

        assert_eq!(
            *world.get::<&Position>(a).unwrap(),
            Position::new(2.0, 0.0).unwrap()
        );
        assert_eq!(
            *world.get::<&Position>(b).unwrap(),
            Position::new(100.0, 99.0).unwrap()
        );
    }

    #[test]
    fn leaves_position_unchanged_at_zero_displacement() {
        let mut world = World::new();

        let start = Position::new(10.0, 5.0).unwrap();
        let entity = world.spawn((Moveable, start, Displacement::ZERO));

        movement_system(&mut world);

        assert_eq!(*world.get::<&Position>(entity).unwrap(), start);
    }

    #[test]
    fn leaves_position_unchanged_when_update_overflows() {
        let mut world = World::new();

        let start = Position::new(f32::MAX, 0.0).unwrap();
        let entity = world.spawn((Moveable, start, displacement(f32::MAX / 2.0, 0.0)));

        movement_system(&mut world);

        assert_eq!(*world.get::<&Position>(entity).unwrap(), start);
    }
}
