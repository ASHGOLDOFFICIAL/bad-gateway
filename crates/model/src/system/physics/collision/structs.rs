use physics::shapes::Rect;

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contact {
    pub time: f32,
    pub point: glam::Vec2,
    pub normal: Option<glam::Vec2>,
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub position: glam::Vec2,
    pub size: glam::Vec2,
}

impl Aabb {
    #[inline(always)]
    pub const fn new(position: glam::Vec2, size: glam::Vec2) -> Self {
        Self { position, size }
    }

    /// Builds the [`Aabb`] from the given [`Rect`]
    /// centered on the given position.
    #[inline]
    pub const fn from_rect(position: glam::Vec2, rect: Rect) -> Self {
        let size = glam::Vec2::new(rect.width().as_meters_f32(), rect.height().as_meters_f32());
        let position = glam::Vec2::new(position.x - size.x / 2.0, position.y - size.y / 2.0);
        Self { position, size }
    }

    #[inline(always)]
    pub fn center(&self) -> glam::Vec2 {
        self.position + self.size / 2.0
    }

    #[inline(always)]
    pub fn max(&self) -> glam::Vec2 {
        self.position + self.size
    }

    #[inline]
    pub fn minkowski_expand(&self, another: Self) -> Self {
        Self {
            position: self.position - another.size / 2.0,
            size: self.size + another.size,
        }
    }
}

#[cfg(test)]
mod tests {
    use physics::Length;

    use super::*;

    #[test]
    fn from_rect_centers_on_the_given_position() {
        let rect = Rect::new(Length::from_meters_f32(2.0), Length::from_meters_f32(4.0)).unwrap();
        let position = glam::Vec2::ZERO;

        let aabb = Aabb::from_rect(position, rect);

        assert_eq!(aabb.center(), position);
        assert_eq!(aabb.position, glam::Vec2::new(-1.0, -2.0));
        assert_eq!(aabb.max(), glam::Vec2::new(1.0, 2.0));
    }

    #[test]
    fn center_is_position_plus_half_size() {
        let aabb = Aabb::new(glam::Vec2::new(2.0, 4.0), glam::Vec2::new(2.0, 6.0));
        assert_eq!(aabb.center(), glam::Vec2::new(3.0, 7.0));
    }

    #[test]
    fn max_is_position_plus_size() {
        let aabb = Aabb::new(glam::Vec2::new(2.0, 4.0), glam::Vec2::new(2.0, 6.0));
        assert_eq!(aabb.max(), glam::Vec2::new(4.0, 10.0));
    }

    #[test]
    fn minkowski_expand_grows_by_others_size_centered() {
        let base = Aabb::new(glam::Vec2::new(5.0, 5.0), glam::Vec2::new(2.0, 2.0));
        let other = Aabb::new(glam::Vec2::ZERO, glam::Vec2::new(4.0, 2.0));

        let expanded = base.minkowski_expand(other);

        assert_eq!(expanded.position, glam::Vec2::new(3.0, 4.0));
        assert_eq!(expanded.size, glam::Vec2::new(6.0, 4.0));
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    pub origin: glam::Vec2,
    pub direction: glam::Vec2,
}

impl Ray {
    #[inline(always)]
    pub const fn new(origin: glam::Vec2, direction: glam::Vec2) -> Self {
        Self { origin, direction }
    }
}
