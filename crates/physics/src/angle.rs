use std::f32::consts::{PI, TAU};

use crate::{ValidationError, ValidationResult};

/// Plane angle, dimensionless.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angle(f32);

impl Angle {
    /// Creates a new `Angle` from the specified [`f32`], in radians.
    ///
    /// # Panics
    /// This constructor will panic if value is not finite.
    #[inline(always)]
    pub fn from_radians_f32(value: f32) -> Self {
        Self::try_from_radians_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_radians_f32`](Self::from_radians_f32).
    ///
    /// This constructor will return an `Err` if value is not finite.
    #[inline]
    pub fn try_from_radians_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("angle value must be finite"))
        } else {
            Ok(Self((value + PI).rem_euclid(TAU) - PI))
        }
    }

    /// Creates a new `Angle` from the specified [`glam::Vec2`] direction.
    ///
    /// # Panics
    /// This constructor will panic if value is zero or not finite.
    #[inline(always)]
    pub fn from_vec2_f32(value: glam::Vec2) -> Self {
        Self::try_from_vec2_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_vec2_f32`](Self::from_vec2_f32).
    ///
    /// This constructor will return an `Err` if value is zero or not
    /// finite.
    #[inline]
    pub fn try_from_vec2_f32(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() || value == glam::Vec2::ZERO {
            Err(ValidationError("vector must be finite and non-zero"))
        } else {
            Self::try_from_radians_f32(value.to_angle())
        }
    }

    /// Returns this `Angle` as [`f32`], in radians.
    #[inline(always)]
    pub const fn as_radians_f32(&self) -> f32 {
        self.0
    }

    /// Returns this `Angle` as [`glam::Vec2`], a unit vector pointing in its
    /// direction.
    #[inline(always)]
    pub fn as_vec2_f32(&self) -> glam::Vec2 {
        glam::Vec2::from_angle(self.0)
    }
}

impl Eq for Angle {}

impl std::ops::Add for Angle {
    type Output = Self;

    /// Sums two angles, wrapping the result to `[-PI, +PI)`.
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from_radians_f32(self.0 + rhs.0)
            .expect("sum of two finite angles is within [-TAU, +TAU)")
    }
}

impl std::ops::AddAssign for Angle {
    /// Sums two angles in place, wrapping the result to `[-PI, +PI)`.
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: Angle, b: Angle) {
        assert!((a.0 - b.0).abs() <= 2.0 * f32::EPSILON, "{a:?} !~= {b:?}");
    }

    #[test]
    fn try_from_radians_rejects_non_finite() {
        assert!(Angle::try_from_radians_f32(f32::NAN).is_err());
        assert!(Angle::try_from_radians_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn try_from_radians_accepts_finite() {
        assert!(Angle::try_from_radians_f32(-1.0).is_ok());
        assert!(Angle::try_from_radians_f32(0.0).is_ok());
        assert!(Angle::try_from_radians_f32(1.0).is_ok());
    }

    #[test]
    fn from_radians_wraps_into_range() {
        approx_eq(Angle::from_radians_f32(TAU), Angle::from_radians_f32(0.0));
        approx_eq(Angle::from_radians_f32(PI), Angle::from_radians_f32(-PI));
        approx_eq(
            Angle::from_radians_f32(-PI - 0.1),
            Angle::from_radians_f32(PI - 0.1),
        );
    }

    #[test]
    fn try_from_vec2_f32_rejects_non_finite() {
        assert!(Angle::try_from_vec2_f32(glam::Vec2::new(f32::NAN, 0.0)).is_err());
    }

    #[test]
    fn try_from_vec2_rejects_zero_vector() {
        assert!(Angle::try_from_vec2_f32(glam::Vec2::ZERO).is_err());
    }

    #[test]
    fn from_vec2_matches_to_angle() {
        let vector = glam::Vec2::new(1.0, 1.0);
        let from_vector = Angle::from_vec2_f32(vector);
        let from_angle = Angle::from_radians_f32(vector.to_angle());
        approx_eq(from_vector, from_angle);
    }

    #[test]
    fn add_wraps_result() {
        let a = Angle::from_radians_f32(PI - 0.1);
        let b = Angle::from_radians_f32(0.2);
        let c = Angle::from_radians_f32(-PI + 0.1);
        let sum = a + b;
        approx_eq(sum, c);
    }

    #[test]
    fn add_assign_matches_add() {
        let a = Angle::from_radians_f32(1.0);
        let b = Angle::from_radians_f32(2.0);

        let mut a_mut = a;
        a_mut += b;

        assert_eq!(a_mut, a + b);
    }

    #[test]
    fn vec2_from_angle_round_trips() {
        let angle = Angle::from_radians_f32(1.0);
        let vector = angle.as_vec2_f32();
        let angle_again = Angle::from_vec2_f32(vector);
        approx_eq(angle_again, angle);
    }
}
