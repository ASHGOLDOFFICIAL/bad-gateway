use std::time::Duration;

use crate::{
    CalculationError, CalculationResult, ValidationError, ValidationResult, Velocity,
    traits::Validated,
};

/// Acceleration, dimension LT⁻² (length per time squared).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Acceleration(glam::Vec2);

impl Acceleration {
    /// `Acceleration` of zero meters per square second.
    pub const ZERO: Self = Self(glam::Vec2::ZERO);

    /// Creates a new `Acceleration` from the specified [`glam::Vec2`], in
    /// meters per second squared.
    ///
    /// # Panics
    /// This constructor will panic if value overflows `Acceleration` or not
    /// finite.
    #[inline(always)]
    pub fn from_meters_per_square_second_vec2(value: glam::Vec2) -> Self {
        Self::try_from_meters_per_square_second_vec2(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_meters_per_square_second_vec2`](Self::from_meters_per_square_second_vec2).
    ///
    /// This constructor will return an `Err` if value overflows `Acceleration`
    /// or not finite.
    #[inline]
    pub fn try_from_meters_per_square_second_vec2(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("vector must be finite"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Acceleration` as [`glam::Vec2`], in meters per second
    /// squared.
    #[inline(always)]
    pub const fn as_meters_per_square_second_vec2(&self) -> glam::Vec2 {
        self.0
    }

    /// Returns `true` if this `Acceleration` is exactly zero.
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives [`Velocity`] from this `Acceleration` and given [`Duration`].
    ///
    /// It uses `v = a * dt`, where
    /// - `v` is velocity,
    /// - `a` is acceleration.
    /// - `dt` is time passed.
    #[inline]
    pub fn velocity(self, duration: Duration) -> CalculationResult<Velocity> {
        Velocity::try_from_meters_per_second_vec2(
            self.as_meters_per_square_second_vec2() * duration.as_secs_f32(),
        )
        .map_err(CalculationError::from)
    }
}

impl std::ops::Neg for Acceleration {
    type Output = Self;

    /// Reverses this `Acceleration`'s direction.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::try_from_meters_per_square_second_vec2(-self.0)
            .expect("negation of finite vector should be finite")
    }
}

impl Validated for Acceleration {
    type Repr = glam::Vec2;

    #[inline(always)]
    fn as_repr(&self) -> glam::Vec2 {
        self.as_meters_per_square_second_vec2()
    }

    #[inline]
    fn validate(value: glam::Vec2) -> Option<Self> {
        Self::try_from_meters_per_square_second_vec2(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_meters_per_square_second_vec2_rejects_non_finite() {
        let vec: glam::prelude::Vec2 = glam::Vec2::new(f32::NAN, 0.0);
        let result = Acceleration::try_from_meters_per_square_second_vec2(vec);
        assert!(result.is_err());
    }

    #[test]
    fn velocity_scales_by_duration() {
        let acc_vec = glam::Vec2::new(1.0, 2.0);
        let time = Duration::from_secs(3);

        let acceleration = Acceleration::try_from_meters_per_square_second_vec2(acc_vec).unwrap();
        let result = acceleration.velocity(time).unwrap();
        assert_eq!(
            result.as_meters_per_second_vec2(),
            glam::Vec2::new(3.0, 6.0)
        )
    }
}
