use hecs::{Entity, Without, World};
use physics::Position;

use crate::component::{
    Active, Aim, ComponentResult, Disabled, RangedWeapon,
    mecha::{
        Mecha,
        control::{LegsActive, RangedWeaponsActive, ThrustersActive},
        part::{Legs, Thruster},
    },
};

/// A part with the ability to scale its efficiency based on user input.
trait Scalable {
    fn set_scale(&mut self, scale: f32) -> ComponentResult<()>;
    fn reset_scale(&mut self);
}

impl Scalable for RangedWeapon {
    fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        RangedWeapon::set_scale(self, scale)
    }

    fn reset_scale(&mut self) {
        RangedWeapon::reset_scale(self);
    }
}

impl Scalable for Thruster {
    fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        Thruster::set_scale(self, scale)
    }

    fn reset_scale(&mut self) {
        Thruster::reset_scale(self);
    }
}

impl Scalable for Legs {
    fn set_scale(&mut self, scale: f32) -> ComponentResult<()> {
        Legs::set_scale(self, scale)
    }

    fn reset_scale(&mut self) {
        Legs::reset_scale(self);
    }
}

/// State that must be propagated from root to its children.
#[derive(Clone, Copy)]
struct RootState {
    legs_scale: Option<f32>,
    thruster_scale: Option<f32>,
    weapon_scale: Option<f32>,

    /// For ranged weapons only.
    transform: Option<(Aim, Position)>,
}

/// The mecha's downward pass: carries each mecha root's state to the parts
/// mounted under it.
///
/// Activates every non-[`Disabled`] part whose kind the root asks for by
/// marking it [`Active`], and copies the root's [`Aim`] and [`Position`] onto
/// its [`RangedWeapon`] so they can fire without knowing they're part of a
/// mecha.
///
/// Reads:
/// - [`Mecha`] to find its parts.
/// - [`Disabled`] - only non-disabled parts can be activated.
/// - [`Option<LegsActive>`](LegsActive),
///   [`Option<ThrustersActive>`](ThrustersActive),
///   [`Option<RangedWeaponsActive>`](RangedWeaponsActive) - which part kinds
///   the root wants activated, and how hard.
/// - [`Option<Legs>`](Legs), [`Option<Thruster>`](Thruster),
///   [`Option<RangedWeapon>`](RangedWeapon) - the parts to activate.
/// - [`Aim`] and [`Position`] to copy onto [`RangedWeapon`]s.
///
/// Writes:
/// - [`Legs`], [`Thruster`], [`RangedWeapon`] - scaled to the requested
///   amount, or reset when not requested.
/// - [`Active`] on non-[`Disabled`] activated parts.
/// - [`Aim`], [`Position`] on ranged weapon parts, received from their root.
///
/// Removes:
/// - [`Active`] from non-[`Disabled`] parts that are disactivated during this
///   call.
pub fn mecha_propagation_system(world: &mut World) {
    let mechas: Vec<(Mecha, RootState)> = world
        .query::<(
            &Mecha,
            Option<&LegsActive>,
            Option<&ThrustersActive>,
            Option<&RangedWeaponsActive>,
            Option<&Aim>,
            Option<&Position>,
        )>()
        .iter()
        .map(|(mecha, legs, thruster, weapon, aim, position)| {
            let state = RootState {
                legs_scale: legs.map(LegsActive::scale),
                thruster_scale: thruster.map(ThrustersActive::scale),
                weapon_scale: weapon.map(RangedWeaponsActive::scale),
                transform: aim.zip(position).map(|(&aim, &position)| (aim, position)),
            };
            (mecha.clone(), state)
        })
        .collect();

    for (mecha, state) in mechas {
        for entity in mecha.parts() {
            activate_part(world, entity, state);

            if let Some((aim, position)) = state.transform {
                aim_weapon(world, entity, aim, position);
            }
        }
    }
}

fn activate_part(world: &mut World, entity: Entity, state: RootState) {
    let mut query = world.query_one::<Without<
        (
            Option<&mut Legs>,
            Option<&mut Thruster>,
            Option<&mut RangedWeapon>,
        ),
        &Disabled,
    >>(entity);

    let Ok((legs, thruster, weapon)) = query.get() else {
        return;
    };

    let has_state_changed = [
        try_activate(legs, state.legs_scale),
        try_activate(thruster, state.thruster_scale),
        try_activate(weapon, state.weapon_scale),
    ]
    .into_iter()
    .flatten()
    .reduce(|left, right| left | right);

    drop(query);

    let Some(activated) = has_state_changed else {
        return;
    };

    if activated {
        let _ = world.insert_one(entity, Active);
    } else {
        let _ = world.remove_one::<Active>(entity);
    }
}

/// Scales `scalable` to `intent`, reporting whether it ended up engaged.
/// [`None`] when the part has no scalable of this kind at all.
fn try_activate<A: Scalable>(scalable: Option<&mut A>, intent: Option<f32>) -> Option<bool> {
    let scalable = scalable?;

    match intent {
        Some(scale) => match scalable.set_scale(scale) {
            Ok(()) => Some(true),
            Err(_) => {
                scalable.reset_scale();
                Some(false)
            }
        },
        None => {
            scalable.reset_scale();
            Some(false)
        }
    }
}

/// Sets [`Aim`] and [`Position`] of the entity if it's a [`RangedWeapon`].
fn aim_weapon(world: &mut World, entity: Entity, aim: Aim, position: Position) {
    if world.get::<&RangedWeapon>(entity).is_err() {
        return;
    }
    let _ = world.insert(entity, (aim, position));
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use physics::{Angle, ForceMagnitude, Frequency, Mass, Speed};

    use super::*;
    use crate::component::{
        Agility, AmmoType, FireRate, ShootCooldown, Thrust, mecha, mecha::part::Part,
    };

    fn weapon() -> RangedWeapon {
        RangedWeapon::new(
            FireRate::from(Frequency::from_hertz_f32(2.0)),
            AmmoType::Bullets,
            Speed::from_meters_per_second_f32(10.0),
            Angle::default(),
        )
        .unwrap()
    }

    fn spawn_part(world: &mut World, mounted_on: Entity, disabled: bool) -> Entity {
        let entity = world.spawn((Part::new("part".to_string(), Some(mounted_on)).unwrap(),));
        if disabled {
            world.insert_one(entity, Disabled).unwrap();
        }
        entity
    }

    fn attach_mecha(world: &mut World, root: Entity, frame: Entity) {
        let mecha = mecha::build_mecha(world, frame);
        world.insert_one(root, mecha).unwrap();
    }

    /// A mecha with a single weapon part, and the root's aim/position set.
    fn mecha_with_weapon(world: &mut World, disabled: bool) -> (Entity, Entity) {
        let root = world.spawn((Aim::default(), Position::ORIGIN));
        let frame = spawn_part(world, root, false);

        let weapon_entity = spawn_part(world, frame, disabled);
        world
            .insert(weapon_entity, (weapon(), ShootCooldown::READY))
            .unwrap();

        attach_mecha(world, root, frame);

        (root, weapon_entity)
    }

    fn spawn_thruster(world: &mut World, mounted_on: Entity, newtons: f32) -> Entity {
        let entity = spawn_part(world, mounted_on, false);
        let thrust = Thrust::from(ForceMagnitude::from_newtons_f32(newtons));
        world.insert_one(entity, Thruster::from(thrust)).unwrap();
        entity
    }

    fn thruster_newtons(world: &World, entity: Entity) -> f32 {
        let thruster = *world.get::<&Thruster>(entity).unwrap();
        ForceMagnitude::from(thruster.thrust()).as_newtons_f32()
    }

    #[test]
    fn try_activate_resets_and_reports_inactive_on_an_invalid_scale() {
        let mut legs = Legs::new(
            Speed::from_meters_per_second_f32(10.0),
            Mass::from_kilograms_f32(1000.0),
            Agility::from(Duration::from_secs_f32(0.3)),
        )
        .unwrap();
        legs.set_scale(0.5).unwrap();

        let activated = try_activate(Some(&mut legs), Some(2.0));

        assert_eq!(activated, Some(false));
        assert_eq!(legs.scale(), 0.0);
    }

    #[test]
    fn propagates_aim_and_position_onto_ranged_weapons() {
        let mut world = World::new();
        let (root, weapon) = mecha_with_weapon(&mut world, false);

        let aim = Aim::from(Angle::from_degrees_f32(45.0));
        let position = Position::new(1.0, 2.0).unwrap();
        world.insert(root, (aim, position)).unwrap();

        mecha_propagation_system(&mut world);

        assert_eq!(*world.get::<&Aim>(weapon).unwrap(), aim);
        assert_eq!(*world.get::<&Position>(weapon).unwrap(), position);
    }

    #[test]
    fn marks_weapon_active_when_the_root_asks_for_weapons() {
        let mut world = World::new();
        let (root, weapon) = mecha_with_weapon(&mut world, false);
        world
            .insert_one(root, RangedWeaponsActive::new(1.0).unwrap())
            .unwrap();

        mecha_propagation_system(&mut world);

        assert!(world.get::<&Active>(weapon).is_ok());
    }

    #[test]
    fn leaves_weapon_inactive_when_the_root_asks_for_nothing() {
        let mut world = World::new();
        let (_, weapon) = mecha_with_weapon(&mut world, false);

        mecha_propagation_system(&mut world);

        assert!(world.get::<&Active>(weapon).is_err());
    }

    #[test]
    fn does_not_engage_a_disabled_part_the_root_asks_for() {
        let mut world = World::new();
        let (root, weapon_entity) = mecha_with_weapon(&mut world, true);
        world
            .insert_one(root, RangedWeaponsActive::new(1.0).unwrap())
            .unwrap();

        mecha_propagation_system(&mut world);

        assert!(world.get::<&Active>(weapon_entity).is_err());
        assert_eq!(
            world
                .get::<&RangedWeapon>(weapon_entity)
                .unwrap()
                .fire_rate(),
            weapon().fire_rate(),
        );
    }

    #[test]
    fn leaves_parts_without_a_ranged_weapon_without_aim_or_position() {
        let mut world = World::new();
        let root = world.spawn((Aim::default(), Position::ORIGIN));
        let frame = spawn_part(&mut world, root, false);
        attach_mecha(&mut world, root, frame);

        mecha_propagation_system(&mut world);

        assert!(world.get::<&Aim>(frame).is_err());
        assert!(world.get::<&Position>(frame).is_err());
        assert!(world.get::<&Active>(frame).is_err());
    }

    #[test]
    fn propagates_boost_active_scale_to_thruster_parts() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, false);
        let thruster_entity = spawn_thruster(&mut world, frame, 1000.0);
        attach_mecha(&mut world, root, frame);
        world
            .insert_one(root, ThrustersActive::new(0.5).unwrap())
            .unwrap();

        mecha_propagation_system(&mut world);

        assert_eq!(thruster_newtons(&world, thruster_entity), 500.0);
        assert!(world.get::<&Active>(thruster_entity).is_ok());
    }

    #[test]
    fn leaves_thruster_inactive_when_boost_not_requested() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, false);
        let thruster_entity = spawn_thruster(&mut world, frame, 1000.0);
        attach_mecha(&mut world, root, frame);

        mecha_propagation_system(&mut world);

        assert_eq!(thruster_newtons(&world, thruster_entity), 0.0);
        assert!(world.get::<&Active>(thruster_entity).is_err());
    }

    #[test]
    fn does_not_engage_a_disabled_thruster() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root, false);
        let thruster_entity = spawn_thruster(&mut world, frame, 1000.0);
        world.insert_one(thruster_entity, Disabled).unwrap();
        attach_mecha(&mut world, root, frame);
        world
            .insert_one(root, ThrustersActive::new(1.0).unwrap())
            .unwrap();

        mecha_propagation_system(&mut world);

        assert!(world.get::<&Active>(thruster_entity).is_err());
    }
}
