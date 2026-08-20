use std::collections::HashMap;

use hecs::{Entity, Satisfies, World};

use crate::component::{Active, Disabled, Integrity, Unpowered, mecha::Mecha};

/// Derives each part's status from its [`Integrity`] and [`Unpowered`] marker,
/// then cascades it down each mecha's part tree so a disabled or broken part
/// takes its children with it.
///
/// Must be the only system that writes [`Disabled`].
///
/// Reads:
/// - [`Mecha`] to disable children of broken or disabled parts.
/// - [`Integrity`] to check if part is broken.
/// - [`Unpowered`] to check if a part was starved of power.
/// - [`Disabled`] to check if a part is already disabled, so the cascade
///   continues past it.
///
/// Writes:
/// - [`Disabled`] to parts that just became broken or unpowered, and to
///   every non-disabled part mounted, transitively, on one that's down.
///
/// Removes:
/// - [`Active`] from disabled components.
pub fn mecha_part_status_system(world: &mut World) {
    let mut to_disable = Vec::new();

    let mut is_down: HashMap<Entity, bool> = world
        .query::<(
            Entity,
            Satisfies<&Disabled>,
            Satisfies<&Unpowered>,
            Option<&Integrity>,
        )>()
        .iter()
        .map(|(entity, is_disabled, is_unpowered, integrity)| {
            let is_broken = integrity.is_some_and(Integrity::is_destroyed);
            let is_down = is_broken || is_unpowered || is_disabled;

            if is_down && !is_disabled {
                to_disable.push(entity);
            }
            (entity, is_down)
        })
        .collect();

    for mecha in world.query::<&Mecha>().iter() {
        for (entity, parent) in mecha.parts_with_parent() {
            let parent_is_down =
                parent.is_some_and(|parent| is_down.get(&parent).copied().unwrap_or(false));
            let child_is_down = is_down.get(&entity).copied().unwrap_or(false);

            if parent_is_down && !child_is_down {
                is_down.insert(entity, true);
                to_disable.push(entity);
            }
        }
    }

    for entity in to_disable {
        let _ = world.insert_one(entity, Disabled);
        let _ = world.remove_one::<Active>(entity);
    }
}

#[cfg(test)]
mod tests {
    use crate::component::{
        DamageUnit,
        mecha::{self, part::Part},
    };

    use super::*;

    fn integrity(current: f32, max: f32) -> Integrity {
        Integrity::new(
            DamageUnit::try_from(current).unwrap(),
            DamageUnit::try_from(max).unwrap(),
        )
        .unwrap()
    }

    fn spawn_part(world: &mut World, mounted_on: Entity, integrity: Integrity) -> Entity {
        world.spawn((
            Part::new("part".to_string(), Some(mounted_on)).unwrap(),
            integrity,
        ))
    }

    fn is_disabled(world: &World, entity: Entity) -> bool {
        world.get::<&Disabled>(entity).is_ok()
    }

    fn attach_mecha(world: &mut World, root: Entity) {
        let mecha = mecha::build_mecha(world, root);
        world.insert_one(root, mecha).unwrap();
    }

    #[test]
    fn broken_part_is_marked_disabled() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(0.0, 100.0));
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, frame));
    }

    #[test]
    fn unpowered_part_is_marked_disabled() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(100.0, 100.0));
        world.insert_one(frame, Unpowered).unwrap();
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, frame));
    }

    #[test]
    fn broken_parent_disables_its_healthy_child() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(0.0, 100.0));
        let weapon = spawn_part(&mut world, frame, integrity(100.0, 100.0));
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, weapon));
    }

    #[test]
    fn unpowered_parent_disables_its_healthy_child() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(100.0, 100.0));
        let weapon = spawn_part(&mut world, frame, integrity(100.0, 100.0));
        world.insert_one(frame, Unpowered).unwrap();
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, weapon));
    }

    #[test]
    fn disabled_parent_disables_its_healthy_child() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(100.0, 100.0));
        let weapon = spawn_part(&mut world, frame, integrity(100.0, 100.0));
        world.insert_one(frame, Disabled).unwrap();
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, weapon));
    }

    #[test]
    fn healthy_child_stays_enabled_under_a_working_parent() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(100.0, 100.0));
        let weapon = spawn_part(&mut world, frame, integrity(100.0, 100.0));
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(!is_disabled(&world, frame));
        assert!(!is_disabled(&world, weapon));
    }

    #[test]
    fn disabling_a_part_deactivates_it() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(0.0, 100.0));
        world.insert_one(frame, Active).unwrap();
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, frame));
        assert!(world.get::<&Active>(frame).is_err());
    }

    #[test]
    fn an_idle_broken_part_is_still_disabled() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(0.0, 100.0));
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, frame));
    }

    #[test]
    fn cascade_reaches_grandchildren() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, integrity(0.0, 100.0));
        let arm = spawn_part(&mut world, frame, integrity(100.0, 100.0));
        let weapon = spawn_part(&mut world, arm, integrity(100.0, 100.0));
        attach_mecha(&mut world, root);

        mecha_part_status_system(&mut world);

        assert!(is_disabled(&world, arm));
        assert!(is_disabled(&world, weapon));
    }
}
