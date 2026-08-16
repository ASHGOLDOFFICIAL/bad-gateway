mod detection;
mod resolution;
mod search;
mod structs;

use detection::*;
use resolution::*;
use search::*;
use structs::*;

use std::time::Duration;

use crate::{
    component::{CollisionResolutionMethod, Moveable, Shape},
    map::Map,
};
use hecs::{Entity, With, World};
use log::warn;
use physics::{Displacement, Position, Velocity};

/// Clamps entity's [`Velocity`] and [`Displacement`] so movement doesn't carry
/// it through or into a colliding structure.
///
/// Reads:
/// - [`Moveable`] as only moveable objects can collide.
/// - [`Position`] to project candidate destination from.
/// - [`Velocity`] to compute that candidate destination.
/// - [`Option<Shape>`](Shape) - needed to find bounding box if present,
///   otherwise entity will be considered a point.
/// - [`Option<CollisionResolutionMethod>`](CollisionResolutionMethod) - how to
///   react to the collision.
///
/// Writes:
/// - [`Displacement`] - adjusted to avoid collisions.
/// - [`Velocity`] - reduced to match the adjusted [`Displacement`].
pub fn static_collision_system(world: &mut World, map: &dyn Map, delta_time: Duration) {
    let allowed: Vec<(Entity, Displacement, Velocity)> = world
        .query::<With<
            (
                Entity,
                &Position,
                &Velocity,
                Option<&Shape>,
                Option<&CollisionResolutionMethod>,
            ),
            &Moveable,
        >>()
        .iter()
        .map(|(entity, position, velocity, shape, resolve)| {
            let resolve = resolve.copied().unwrap_or_default();
            let (displacement, velocity) = allowed_displacement(
                map, entity, shape, *position, *velocity, resolve, delta_time,
            );

            (entity, displacement, velocity)
        })
        .collect();

    for (entity, displacement, velocity) in allowed {
        let _ = world.insert(entity, (displacement, velocity));
    }
}

/// Computes updated [`Displacement`] and [`Velocity`] that will avoid
/// colliding with a solid map cell along the way.
///
/// Entities without a [`Shape`] collide as points.
fn allowed_displacement(
    map: &dyn Map,
    entity: Entity,
    shape: Option<&Shape>,
    position: Position,
    velocity: Velocity,
    resolve: CollisionResolutionMethod,
    delta_time: Duration,
) -> (Displacement, Velocity) {
    let requested = match velocity.displacement(delta_time) {
        Ok(requested) => requested,
        Err(error) => {
            warn!(entity:? = entity, error:? = error; "displacement calculation failed");
            return (Displacement::ZERO, velocity);
        }
    };

    if requested.is_zero() {
        return (requested, velocity);
    }

    let position_vec2 = position.as_meters_vec2();
    let requested_vec2 = requested.as_meters_vec2();
    let rect = shape.and_then(|s| s.projection().ok()?.bounding_box().ok());

    let adjusted = match rect {
        Some(rect) => {
            let aabb = Aabb::from_rect(position_vec2, rect);

            let mut cells: Vec<(f32, Aabb)> = sweep_solid_cells(aabb, requested_vec2, map)
                .filter_map(|cell| {
                    dynamic_aabb_vs_aabb(requested_vec2, aabb, cell)
                        .map(|contact| (contact.time, cell))
                })
                .collect();

            if cells.is_empty() {
                return (requested, velocity);
            }
            cells.sort_by(|(a, _), (b, _)| a.total_cmp(b));

            cells
                .into_iter()
                .fold(requested_vec2, |displacement, (_, cell)| {
                    resolve_dynamic_aabb_vs_aabb(resolve, displacement, aabb, cell)
                        .unwrap_or(displacement)
                })
        }
        None => {
            let cells: Vec<Aabb> = sweep_solid_cells_point(position_vec2, requested_vec2, map)
                .filter(|&cell| {
                    dynamic_point_vs_aabb(requested_vec2, position_vec2, cell).is_some()
                })
                .collect();

            if cells.is_empty() {
                return (requested, velocity);
            }

            cells
                .into_iter()
                .fold(requested_vec2, |displacement, cell| {
                    resolve_dynamic_point_vs_aabb(resolve, displacement, position_vec2, cell)
                        .unwrap_or(displacement)
                })
        }
    };

    let displacement = match Displacement::try_from_meters_vec2(adjusted) {
        Ok(displacement) => displacement,
        Err(error) => {
            warn!(entity:? = entity, error:? = error; "adjusted displacement invalid");
            return (Displacement::ZERO, Velocity::ZERO);
        }
    };

    let velocity = match displacement.velocity(delta_time) {
        Ok(velocity) => velocity,
        Err(error) => {
            warn!(entity:? = entity, error:? = error; "adjusted velocity calculation failed");
            velocity
        }
    };

    (displacement, velocity)
}
