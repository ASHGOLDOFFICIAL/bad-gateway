use hecs::{Entity, World};

use crate::component::{DamageDelta, Integrity};

/// Applies pending [`DamageDelta`] to [`Integrity`].
///
/// Reads:
/// - [`Option<DamageDelta>`](DamageDelta) to apply, if present.
///
/// Writes:
/// - [`Integrity`] - updated by any pending damage.
///
/// Removes:
/// - [`DamageDelta`] after consumption.
pub fn integrity_system(world: &mut World) {
    let applied: Vec<Entity> = world
        .query::<(Entity, &mut Integrity, Option<&DamageDelta>)>()
        .iter()
        .filter_map(|(entity, integrity, delta)| {
            let delta = delta?;
            integrity.apply((*delta).into());
            Some(entity)
        })
        .collect();

    applied.iter().for_each(|&entity| {
        let _ = world.remove_one::<DamageDelta>(entity);
    });
}

#[cfg(test)]
mod tests {
    use physics::ops::Delta;

    use crate::component::DamageUnit;

    use super::*;

    #[test]
    fn applies_pending_damage_delta_and_removes_it() {
        let mut world = World::new();
        let max = DamageUnit::try_from(10.0).unwrap();
        let entity = world.spawn((
            Integrity::full(max).unwrap(),
            DamageDelta::from(Delta::Negative(DamageUnit::try_from(4.0).unwrap())),
        ));

        integrity_system(&mut world);

        assert_eq!(world.get::<&Integrity>(entity).unwrap().ratio(), 0.6);
        assert!(world.get::<&DamageDelta>(entity).is_err());
    }

    #[test]
    fn leaves_integrity_unchanged_without_pending_delta() {
        let mut world = World::new();
        let max = DamageUnit::try_from(10.0).unwrap();
        let entity = world.spawn((Integrity::full(max).unwrap(),));

        integrity_system(&mut world);

        assert_eq!(world.get::<&Integrity>(entity).unwrap().ratio(), 1.0);
    }
}
