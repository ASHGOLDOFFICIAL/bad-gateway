use std::time::Duration;

use crate::{
    CalculationError, CalculationResult, Displacement, Speed, ValidationError, ValidationResult,
    ops,
};

/// Velocity, dimension LT⁻¹ (length per time).
#[must_use]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Velocity(glam::Vec2);

impl Velocity {
    /// `Velocity` of zero meters per second.
    pub const ZERO: Self = Self(glam::Vec2::ZERO);

    /// Creates a new `Velocity` from the specified [`glam::Vec2`], in meters
    /// per second.
    ///
    /// # Panics
    /// This constructor will panic if value overflows `Velocity` or not
    /// finite.
    #[inline]
    pub fn from_meters_per_second_vec2(value: glam::Vec2) -> Self {
        Self::try_from_meters_per_second_vec2(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_meters_per_second_vec2`](Self::from_meters_per_second_vec2).
    ///
    /// This constructor will return an `Err` if value overflows `Velocity`
    /// or not finite.
    #[inline]
    pub fn try_from_meters_per_second_vec2(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("vector must be finite"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Velocity` as [`glam::Vec2`], in meters per second.
    #[inline]
    pub const fn as_meters_per_second_vec2(&self) -> glam::Vec2 {
        self.0
    }

    /// Returns `true` if this `Velocity` is exactly zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// This `Velocity`'s magnitude, independent of direction.
    ///
    /// Errors if the magnitude overflows `Speed`'s valid range.
    #[inline]
    pub fn speed(&self) -> CalculationResult<Speed> {
        Speed::try_from_meters_per_second_f32(self.0.length()).map_err(CalculationError::from)
    }

    /// Derives [`Displacement`] from this `Velocity` and given [`Duration`].
    ///
    /// It uses `d = v * dt`, where
    /// - `d` is displacement,
    /// - `v` is velocity.
    /// - `dt` is time passed.
    #[inline]
    pub fn displacement(self, duration: Duration) -> CalculationResult<Displacement> {
        Displacement::try_from_meters_vec2(
            self.as_meters_per_second_vec2() * duration.as_secs_f32(),
        )
        .map_err(CalculationError::from)
    }
}

impl std::ops::Neg for Velocity {
    type Output = Self;

    /// Reverses this `Velocity`'s direction.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::try_from_meters_per_second_vec2(-self.0)
            .expect("negation of finite vector should be finite")
    }
}

impl ops::Validated for Velocity {
    type Repr = glam::Vec2;

    #[inline(always)]
    fn as_repr(&self) -> glam::Vec2 {
        self.as_meters_per_second_vec2()
    }

    #[inline]
    fn validate(value: glam::Vec2) -> Option<Self> {
        Self::try_from_meters_per_second_vec2(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_meters_per_second_vec2_rejects_non_finite() {
        assert!(Velocity::try_from_meters_per_second_vec2(glam::Vec2::new(f32::NAN, 0.0)).is_err());
    }

    #[test]
    fn speed_is_magnitude_of_vector() {
        let vector = glam::Vec2::new(3.0, 4.0);
        let velocity = Velocity::try_from_meters_per_second_vec2(vector).unwrap();
        assert_eq!(
            velocity.speed().unwrap().as_meters_per_second_f32(),
            vector.length()
        );
        assert!(Velocity::ZERO.speed().unwrap().is_zero());
    }

    #[test]
    fn displacement_scales_by_duration() {
        let velocity =
            Velocity::try_from_meters_per_second_vec2(glam::Vec2::new(1.0, 2.0)).unwrap();
        let time = Duration::from_secs(3);
        let displacement = velocity.displacement(time).unwrap();
        assert_eq!(displacement.as_meters_vec2(), glam::Vec2::new(3.0, 6.0));
    }
}
