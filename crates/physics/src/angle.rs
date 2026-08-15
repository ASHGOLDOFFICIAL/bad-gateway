use std::f64::consts::{PI, TAU};

use crate::{ValidationError, ValidationResult, ops::CheckedMul};

/// Minutes of arc per degree, by definition.
const MOA_PER_DEGREE: f64 = 60.0;

/// Plane angle, dimensionless.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Angle(f64);

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
            Ok(Self((value as f64 + PI).rem_euclid(TAU) - PI))
        }
    }

    /// Creates a new `Angle` from the specified [`f32`], in degrees.
    ///
    /// # Panics
    /// This constructor will panic if value is not finite.
    #[inline(always)]
    pub fn from_degrees_f32(value: f32) -> Self {
        Self::try_from_degrees_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_degrees_f32`](Self::from_degrees_f32).
    ///
    /// This constructor will return an `Err` if value is not finite.
    #[inline]
    pub fn try_from_degrees_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("angle value must be finite"))
        } else {
            Self::try_from_radians_f32(value.to_radians())
        }
    }

    /// Creates a new `Angle` from the specified [`f32`], in minutes of arc.
    ///
    /// # Panics
    /// This constructor will panic if value is not finite.
    #[inline(always)]
    pub fn from_arcminute_f32(value: f32) -> Self {
        Self::try_from_arcminute_f32(value).expect("unsafe method")
    }

    /// The checked version of [`from_arcminute_f32`](Self::from_arcminute_f32).
    ///
    /// This constructor will return an `Err` if value is not finite.
    #[inline]
    pub fn try_from_arcminute_f32(value: f32) -> ValidationResult<Self> {
        Self::try_from_degrees_f32(value / MOA_PER_DEGREE as f32)
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
        if value == glam::Vec2::ZERO || !value.is_finite() {
            Err(ValidationError("vector must be finite and non-zero"))
        } else {
            Self::try_from_radians_f32(value.to_angle())
        }
    }

    /// Returns this `Angle` as [`f32`], in radians.
    #[inline(always)]
    pub const fn as_radians_f32(&self) -> f32 {
        self.0 as f32
    }

    /// Returns this `Angle` as [`f32`], in degrees.
    #[inline(always)]
    pub fn as_degrees_f32(&self) -> f32 {
        self.0.to_degrees() as f32
    }

    /// Returns this `Angle` as [`f32`], in minutes of arc (MOA).
    #[inline(always)]
    pub fn as_arcminute_f32(&self) -> f32 {
        (self.0.to_degrees() * MOA_PER_DEGREE) as f32
    }

    /// Returns this `Angle` as [`glam::Vec2`], a unit vector pointing in its
    /// direction.
    #[inline(always)]
    pub fn as_vec2_f32(&self) -> glam::Vec2 {
        glam::Vec2::from_angle(self.0 as f32)
    }
}

impl Eq for Angle {}

impl std::ops::Add for Angle {
    type Output = Self;

    /// Sums two angles.
    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        let sum = self.as_radians_f32() + rhs.as_radians_f32();
        Self::try_from_radians_f32(sum).expect("sum of two finite angles is within [-TAU, +TAU)")
    }
}

impl std::ops::AddAssign for Angle {
    /// Sums two angles in place.
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl CheckedMul<f32> for Angle {
    type Output = Self;

    /// Scales this angle by `rhs`. Returns `None` if `rhs` isn't finite.
    #[inline]
    fn checked_mul(self, rhs: f32) -> Option<Self::Output> {
        if self.as_radians_f32() == 0.0 {
            return rhs.is_finite().then_some(Self(0.0));
        }
        let period = core::f32::consts::TAU / self.as_radians_f32().abs();
        let reduced_rhs = rhs.rem_euclid(period);
        let product = self.as_radians_f32() * reduced_rhs;
        Self::try_from_radians_f32(product).ok()
    }
}

#[cfg(test)]
mod tests {
    use core::f32;

    use super::*;

    fn approx_eq(a: Angle, b: Angle) {
        let raw_diff = (a.0 - b.0).rem_euclid(TAU);
        let diff = raw_diff.min(TAU - raw_diff);
        assert!(diff <= 4.0 * f64::from(f32::EPSILON), "{a:?} !~= {b:?}");
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
        approx_eq(
            Angle::from_radians_f32(f32::consts::TAU),
            Angle::from_radians_f32(0.0),
        );
        approx_eq(
            Angle::from_radians_f32(f32::consts::PI),
            Angle::from_radians_f32(-f32::consts::PI),
        );
        approx_eq(
            Angle::from_radians_f32(-f32::consts::PI - 0.1),
            Angle::from_radians_f32(f32::consts::PI - 0.1),
        );
    }

    #[test]
    fn try_from_degrees_rejects_non_finite() {
        assert!(Angle::try_from_degrees_f32(f32::NAN).is_err());
        assert!(Angle::try_from_degrees_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn degrees_matches_radians() {
        approx_eq(Angle::from_degrees_f32(360.0), Angle::from_radians_f32(0.0));
        approx_eq(
            Angle::from_degrees_f32(180.0),
            Angle::from_radians_f32(f32::consts::PI),
        );
    }

    #[test]
    fn as_degrees_matches_from_degrees() {
        let degrees = 15.0;
        let from_degrees = Angle::from_degrees_f32(degrees).as_degrees_f32();
        assert!((from_degrees - degrees).abs() < f32::EPSILON);
    }

    #[test]
    fn try_from_arcminute_rejects_non_finite() {
        assert!(Angle::try_from_arcminute_f32(f32::NAN).is_err());
        assert!(Angle::try_from_arcminute_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn arcminute_matches_radians() {
        approx_eq(
            Angle::from_arcminute_f32(360.0 * MOA_PER_DEGREE as f32),
            Angle::from_radians_f32(0.0),
        );
        approx_eq(
            Angle::from_arcminute_f32(180.0 * MOA_PER_DEGREE as f32),
            Angle::from_radians_f32(f32::consts::PI),
        );
    }

    #[test]
    fn as_arcminute_matches_from_arcminute() {
        let arcminutes = 15.0;
        let from_arcminutes = Angle::from_arcminute_f32(arcminutes).as_arcminute_f32();
        assert!((from_arcminutes - arcminutes).abs() < f32::EPSILON);
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
        let a = Angle::from_radians_f32(f32::consts::PI - 0.1);
        let b = Angle::from_radians_f32(0.2);
        let c = Angle::from_radians_f32(-f32::consts::PI + 0.1);
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

    #[test]
    fn checked_mul_rejects_non_finite_rhs() {
        let angle = Angle::from_radians_f32(1.0);
        assert!(angle.checked_mul(f32::NAN).is_none());
        assert!(angle.checked_mul(f32::INFINITY).is_none());
    }

    #[test]
    fn checked_mul_of_zero_angle_rejects_non_finite_rhs() {
        let zero = Angle::default();
        assert!(zero.checked_mul(f32::NAN).is_none());
        assert!(zero.checked_mul(f32::INFINITY).is_none());
    }

    #[test]
    fn checked_mul_of_zero_angle_is_always_zero() {
        let zero = Angle::default();
        assert_eq!(
            zero.checked_mul(1234.5).unwrap(),
            Angle::from_radians_f32(0.0)
        );
    }

    #[test]
    fn checked_mul_scales_within_range() {
        let angle = Angle::from_radians_f32(1.0);
        approx_eq(
            angle.checked_mul(0.5).unwrap(),
            Angle::from_radians_f32(0.5),
        );
    }

    #[test]
    fn checked_mul_wraps_result() {
        let angle = Angle::from_radians_f32(1.0);
        approx_eq(
            angle.checked_mul(4.0).unwrap(),
            Angle::from_radians_f32(4.0),
        );
    }

    #[test]
    fn checked_mul_reduces_large_positive_rhs() {
        let angle = Angle::from_radians_f32(1.0);
        let large_rhs = 10.0 * f32::consts::TAU + 0.3;
        let from_large = angle.checked_mul(large_rhs).unwrap();
        let from_small = angle.checked_mul(0.3).unwrap();
        approx_eq(from_large, from_small);
    }

    #[test]
    fn checked_mul_reduces_large_negative_rhs() {
        let angle = Angle::from_radians_f32(1.0);
        let large_rhs = 10.0 * f32::consts::TAU + 0.3;
        let from_large = angle.checked_mul(-large_rhs).unwrap();
        let from_small = angle.checked_mul(-0.3).unwrap();
        approx_eq(from_large, from_small);
    }
}
