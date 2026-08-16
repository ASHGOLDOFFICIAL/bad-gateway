use std::time::Duration;

use hecs::{Entity, World};
use log::trace;

use crate::component::Lifetime;

/// Removes entities whose [`Lifetime`] has run out.
///
/// Reads:
/// - [`Lifetime`] to check if despawn is needed.
///
/// Writes:
/// - [`Lifetime`] reduces it by `delta_time`.
///
/// Despawns:
/// - [`Entities`](Entity) whose lifetime has expired.
pub fn lifetime_system(world: &mut World, delta_time: Duration) {
    let to_despawn: Vec<Entity> = world
        .query::<(Entity, &mut Lifetime)>()
        .iter()
        .filter_map(|(entity, lifetime)| {
            lifetime.reduce(delta_time);
            lifetime.is_expired().then_some(entity)
        })
        .collect();

    to_despawn.iter().for_each(|&e| {
        trace!(entity:? = e; "entity despawned due to expired lifetime");
        let _ = world.despawn(e);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_entities_without_lifetime() {
        let mut world = World::new();
        let entity = world.spawn(());

        lifetime_system(&mut world, Duration::from_secs(1));

        assert!(world.contains(entity));
    }

    #[test]
    fn despawns_entity_whose_lifetime_expired() {
        let mut world = World::new();
        let a = world.spawn((Lifetime::from(Duration::from_secs(1)),));
        let b = world.spawn((Lifetime::from(Duration::from_secs(10)),));

        lifetime_system(&mut world, Duration::from_secs(2));

        assert!(!world.contains(a));
        assert!(world.contains(b));
    }

    #[test]
    fn keeps_entity_whose_lifetime_has_not_expired() {
        let mut world = World::new();
        let entity = world.spawn((Lifetime::from(Duration::from_secs(5)),));

        lifetime_system(&mut world, Duration::from_secs(2));

        assert!(world.contains(entity));
        assert_eq!(
            *world.get::<&Lifetime>(entity).unwrap(),
            Lifetime::from(Duration::from_secs(3))
        );
    }
}
