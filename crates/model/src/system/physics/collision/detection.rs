use crate::system::physics::collision::{Aabb, Contact, Ray};

const NORMAL_TOP: glam::Vec2 = glam::Vec2::new(0.0, -1.0);
const NORMAL_BOTTOM: glam::Vec2 = glam::Vec2::new(0.0, 1.0);
const NORMAL_LEFT: glam::Vec2 = glam::Vec2::new(-1.0, 0.0);
const NORMAL_RIGHT: glam::Vec2 = glam::Vec2::new(1.0, 0.0);

/// Checks if [`Ray`] intersects [`Aabb`].
#[must_use]
pub fn ray_vs_aabb(ray: Ray, aabb: Aabb) -> Option<Contact> {
    let aabb_max = aabb.max();
    let (near_x, far_x) = axis_slab(ray.origin.x, ray.direction.x, aabb.position.x, aabb_max.x)?;
    let (near_y, far_y) = axis_slab(ray.origin.y, ray.direction.y, aabb.position.y, aabb_max.y)?;

    let t_near = glam::Vec2::new(near_x, near_y);
    let t_far = glam::Vec2::new(far_x, far_y);

    let t_hit_near = t_near.max_element();
    let t_hit_far = t_far.min_element();

    if t_hit_near > t_hit_far || t_hit_far < 0.0 {
        return None;
    }

    let contact = ray.origin + ray.direction * t_hit_near;

    let contact_normal = if t_near.x > t_near.y {
        if ray.direction.x < 0.0 {
            Some(NORMAL_RIGHT)
        } else {
            Some(NORMAL_LEFT)
        }
    } else if t_near.x < t_near.y {
        if ray.direction.y < 0.0 {
            Some(NORMAL_BOTTOM)
        } else {
            Some(NORMAL_TOP)
        }
    } else {
        None
    };

    Some(Contact {
        time: t_hit_near,
        point: contact,
        normal: contact_normal,
    })
}

/// Checks if moving [`Aabb`] collides with another static [`Aabb`] on its path.
#[must_use]
pub fn dynamic_aabb_vs_aabb(
    displacement: glam::Vec2,
    dynamic: Aabb,
    r#static: Aabb,
) -> Option<Contact> {
    if displacement == glam::Vec2::ZERO {
        return None;
    }
    let ray = Ray::new(dynamic.center(), displacement);
    let expanded_static = r#static.minkowski_expand(dynamic);
    let contact = ray_vs_aabb(ray, expanded_static)?;
    (contact.time >= 0.0 && contact.time < 1.0).then_some(contact)
}

/// Checks if moving point collides with static [`Aabb`] on its path.
#[must_use]
pub fn dynamic_point_vs_aabb(
    displacement: glam::Vec2,
    point: glam::Vec2,
    r#static: Aabb,
) -> Option<Contact> {
    if displacement == glam::Vec2::ZERO {
        return None;
    }
    let ray = Ray::new(point, displacement);
    let contact = ray_vs_aabb(ray, r#static)?;
    (contact.time >= 0.0 && contact.time < 1.0).then_some(contact)
}

/// Division that handles the case where the ray is parallel to this axis's
/// slab (`direction` is `0.0`), so the ray has a fixed coordinate on it.
/// Returns [`None`] if that fixed coordinate doesn't sit strictly inside
/// `(min, max)`.
#[must_use]
fn axis_slab(origin: f32, direction: f32, min: f32, max: f32) -> Option<(f32, f32)> {
    if direction == 0.0 {
        return (origin > min && origin < max).then_some((f32::NEG_INFINITY, f32::INFINITY));
    }

    let inv = 1.0 / direction;
    let t1 = (min - origin) * inv;
    let t2 = (max - origin) * inv;

    Some((t1.min(t2), t1.max(t2)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aabb() -> Aabb {
        Aabb::new(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(2.0, 2.0))
    }

    #[test]
    fn ray_vs_aabb_hits_left_face() {
        let ray = Ray::new(glam::Vec2::new(-5.0, 0.0), glam::Vec2::new(1.0, 0.0));
        let contact = ray_vs_aabb(ray, aabb()).unwrap();

        assert_eq!(contact.time, 4.0);
        assert_eq!(contact.point, glam::Vec2::new(-1.0, 0.0));
        assert_eq!(contact.normal, Some(NORMAL_LEFT));
    }

    #[test]
    fn ray_vs_aabb_misses_when_parallel_and_outside() {
        let ray = Ray::new(glam::Vec2::new(-5.0, 5.0), glam::Vec2::new(1.0, 0.0));
        assert!(ray_vs_aabb(ray, aabb()).is_none());
    }

    #[test]
    fn ray_vs_aabb_normal_is_none_on_exact_corner_hit() {
        let ray = Ray::new(glam::Vec2::new(-3.0, -3.0), glam::Vec2::new(1.0, 1.0));
        let contact = ray_vs_aabb(ray, aabb()).unwrap();

        assert_eq!(contact.point, glam::Vec2::new(-1.0, -1.0));
        assert_eq!(contact.normal, None);
    }

    #[test]
    fn dynamic_aabb_vs_aabb_ignores_zero_displacement() {
        let dynamic = Aabb::new(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(2.0, 2.0));
        let contact = dynamic_aabb_vs_aabb(glam::Vec2::ZERO, dynamic, aabb());
        assert!(contact.is_none());
    }

    #[test]
    fn dynamic_aabb_vs_aabb_detects_collision_before_full_displacement() {
        let dynamic = Aabb::new(glam::Vec2::new(-3.0, -0.5), glam::Vec2::new(1.0, 1.0));
        let contact = dynamic_aabb_vs_aabb(glam::Vec2::new(2.0, 0.0), dynamic, aabb()).unwrap();

        assert!((contact.time - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn dynamic_aabb_vs_aabb_ignores_contact_beyond_current_full_displacement() {
        let dynamic = Aabb::new(glam::Vec2::new(-10.0, -0.5), glam::Vec2::new(1.0, 1.0));
        assert!(dynamic_aabb_vs_aabb(glam::Vec2::new(0.5, 0.0), dynamic, aabb()).is_none());
    }

    #[test]
    fn dynamic_point_vs_aabb_ignores_zero_displacement() {
        let point = glam::Vec2::new(-1.0, 1.0);
        assert!(dynamic_point_vs_aabb(glam::Vec2::ZERO, point, aabb()).is_none());
    }

    #[test]
    fn dynamic_point_vs_aabb_detects_collision() {
        let point = glam::Vec2::new(-2.0, -0.5);
        let contact = dynamic_point_vs_aabb(glam::Vec2::new(2.0, 0.0), point, aabb()).unwrap();

        assert_eq!(contact.time, 0.5);
    }
}
