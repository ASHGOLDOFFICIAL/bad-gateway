use crate::{
    component::CollisionResolutionMethod,
    system::physics::collision::{
        Aabb, dynamic_aabb_vs_aabb, dynamic_point_vs_aabb, structs::Contact,
    },
};

/// Resolves collision (if any) between moving [`Aabb`] and a static one.
/// Returns adjusted displacement of moving one.
pub fn resolve_dynamic_aabb_vs_aabb(
    resolve: CollisionResolutionMethod,
    displacement: glam::Vec2,
    dynamic: Aabb,
    r#static: Aabb,
) -> Option<glam::Vec2> {
    let contact = dynamic_aabb_vs_aabb(displacement, dynamic, r#static)?;
    Some(resolve_contact(resolve, displacement, contact))
}

/// Resolves collision (if any) between moving point and a static [`Aabb`].
/// Returns adjusted displacement of moving one.
pub fn resolve_dynamic_point_vs_aabb(
    resolve: CollisionResolutionMethod,
    displacement: glam::Vec2,
    point: glam::Vec2,
    r#static: Aabb,
) -> Option<glam::Vec2> {
    let contact = dynamic_point_vs_aabb(displacement, point, r#static)?;
    Some(resolve_contact(resolve, displacement, contact))
}

/// Applies `resolve` to a detected `contact`.
fn resolve_contact(
    resolve: CollisionResolutionMethod,
    displacement: glam::Vec2,
    contact: Contact,
) -> glam::Vec2 {
    match (resolve, contact.normal) {
        (CollisionResolutionMethod::Stop, _) | (CollisionResolutionMethod::Slide, None) => {
            displacement * contact.time
        }
        (CollisionResolutionMethod::Slide, Some(normal)) => {
            displacement + normal * displacement.abs() * (1.0 - contact.time)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aabb() -> Aabb {
        Aabb::new(glam::Vec2::new(-1.0, -1.0), glam::Vec2::new(2.0, 2.0))
    }

    #[test]
    fn stop_clamps_displacement_to_contact_time() {
        let dynamic = Aabb::new(glam::Vec2::new(-3.0, -0.5), glam::Vec2::new(1.0, 1.0));
        let displacement = glam::Vec2::new(2.0, 0.0);

        assert_eq!(
            resolve_dynamic_aabb_vs_aabb(
                CollisionResolutionMethod::Stop,
                displacement,
                dynamic,
                aabb()
            )
            .unwrap(),
            displacement * 0.5
        );
    }

    #[test]
    fn slide_on_axis_aligned_hit_preserves_the_other_axis() {
        let dynamic = Aabb::new(glam::Vec2::new(-5.0, -1.0), glam::Vec2::new(2.0, 2.0));
        let displacement = glam::Vec2::new(4.0, 3.0);

        assert_eq!(
            resolve_dynamic_aabb_vs_aabb(
                CollisionResolutionMethod::Slide,
                displacement,
                dynamic,
                aabb(),
            )
            .unwrap(),
            glam::Vec2::new(2.0, 3.0)
        );
    }

    #[test]
    fn slide_on_exact_corner_hit_falls_back_to_stops_clamp() {
        let point = glam::Vec2::new(-3.0, -3.0);
        let displacement = glam::Vec2::new(3.0, 3.0);

        let contact = dynamic_point_vs_aabb(displacement, point, aabb()).unwrap();
        assert!(contact.normal.is_none());

        assert_eq!(
            resolve_dynamic_point_vs_aabb(
                CollisionResolutionMethod::Slide,
                displacement,
                point,
                aabb(),
            )
            .unwrap(),
            displacement * contact.time
        );
    }

    #[test]
    fn resolve_returns_none_when_no_contact() {
        let point = glam::Vec2::new(-4.0, 5.0);
        let displacement = glam::Vec2::new(1.0, 0.0);

        assert!(
            resolve_dynamic_point_vs_aabb(
                CollisionResolutionMethod::Stop,
                displacement,
                point,
                aabb()
            )
            .is_none()
        );
    }
}
