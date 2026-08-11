use crate::{Displacement, ValidationError, ValidationResult, ops};

/// Position, dimension L (distance from some point of origin).
#[must_use]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Position(glam::Vec2);

impl Position {
    // Point of origin for all other positions.
    pub const ORIGIN: Self = Self(glam::Vec2::ZERO);

    /// The largest representable position.
    pub const MAX: Self = Self(glam::Vec2::MAX);

    /// Creates a position from `x`/`y` coordinates, in meters, which must
    /// be finite.
    #[inline]
    pub const fn new(x: f32, y: f32) -> ValidationResult<Self> {
        if !x.is_finite() || !y.is_finite() {
            Err(ValidationError("x and y must be finite"))
        } else {
            Ok(Self(glam::Vec2 { x, y }))
        }
    }

    /// Creates a new `Position` from the specified [`glam::Vec2`], in meters.
    ///
    /// # Panics
    /// This constructor will panic if value overflows `Position` or not
    /// finite.
    #[inline]
    pub fn from_meters_vec2(value: glam::Vec2) -> Self {
        Self::try_from_meters_vec2(value).expect("unsafe method")
    }

    /// The checked version of [`from_meters_vec2`](Self::from_meters_vec2).
    ///
    /// This constructor will return an `Err` if value overflows `Position`
    /// or not finite.
    #[inline]
    pub fn try_from_meters_vec2(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("vector must be finite"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Position` as [`glam::Vec2`], in meters.
    #[inline(always)]
    pub const fn as_meters_vec2(&self) -> glam::Vec2 {
        self.0
    }
}

impl ops::CheckedAdd<Displacement> for Position {
    type Output = Self;

    /// Offsets this `Position` by given [`Displacement`]. `None` if the
    /// result is invalid.
    #[inline]
    fn checked_add(self, rhs: Displacement) -> Option<Self> {
        Self::try_from_meters_vec2(self.0 + rhs.as_meters_vec2()).ok()
    }
}

impl ops::CheckedSub<Displacement> for Position {
    type Output = Self;

    /// Offsets this `Position` by the negation of given [`Displacement`].
    /// `None` if the result is invalid.
    #[inline]
    fn checked_sub(self, rhs: Displacement) -> Option<Self> {
        Self::try_from_meters_vec2(self.0 - rhs.as_meters_vec2()).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ops::CheckedAdd;

    #[test]
    fn new_rejects_non_finite() {
        assert!(Position::new(f32::NAN, 0.0).is_err());
        assert!(Position::new(0.0, f32::INFINITY).is_err());
    }

    #[test]
    fn new_accepts_finite() {
        let position = Position::new(1.0, 2.0).unwrap();
        assert_eq!(position.as_meters_vec2(), glam::Vec2::new(1.0, 2.0));
    }

    #[test]
    fn from_meters_vec2_rejects_non_finite() {
        assert!(Position::try_from_meters_vec2(glam::Vec2::new(f32::NAN, 0.0)).is_err());
    }

    #[test]
    fn checked_add_offsets_position() {
        let position = Position::new(1.0, 2.0).unwrap();
        let displacement = Displacement::from_meters_vec2(glam::Vec2::new(3.0, -1.0));
        let moved = position.checked_add(displacement).unwrap();
        assert_eq!(moved.as_meters_vec2(), glam::Vec2::new(4.0, 1.0));
    }
}
