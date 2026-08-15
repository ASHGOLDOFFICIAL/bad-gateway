use std::time::Duration;

use crate::{
    CalculationError, CalculationResult, ValidationError, ValidationResult, Velocity,
    traits::Validated,
};

/// Displacement, dimension L (change in position between two points).
#[must_use]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Displacement(glam::Vec2);

impl Displacement {
    /// `Displacement` of zero meters.
    pub const ZERO: Self = Self(glam::Vec2::ZERO);

    /// Creates a new `Displacement` from the specified [`glam::Vec2`], in
    /// meters.
    ///
    /// # Panics
    /// This constructor will panic if value overflows `Displacement` or not
    /// finite.
    #[inline]
    pub fn from_meters_vec2(value: glam::Vec2) -> Self {
        Self::try_from_meters_vec2(value).expect("unsafe method")
    }

    /// The checked version of [`from_meters_vec2`](Self::from_meters_vec2).
    ///
    /// This constructor will return an `Err` if value overflows
    /// `Displacement` or not finite.
    #[inline]
    pub fn try_from_meters_vec2(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("vector must be finite"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Displacement` as [`glam::Vec2`], in meters.
    #[inline(always)]
    pub const fn as_meters_vec2(&self) -> glam::Vec2 {
        self.0
    }

    /// Returns `true` if this `Displacement` is exactly zero.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives the [`Velocity`] that covers this `Displacement` over the given
    /// [`Duration`].
    ///
    /// It uses `v = d / dt`, where
    /// - `v` is velocity,
    /// - `d` is this displacement,
    /// - `dt` is time passed.
    ///
    /// Given `duration` must be non-zero.
    #[inline]
    pub fn velocity(self, duration: Duration) -> CalculationResult<Velocity> {
        if duration.is_zero() {
            Err(CalculationError::InvalidArgument(
                "duration must be non-zero",
            ))
        } else {
            Velocity::try_from_meters_per_second_vec2(
                self.as_meters_vec2() / duration.as_secs_f32(),
            )
            .map_err(CalculationError::from)
        }
    }
}

impl std::ops::Neg for Displacement {
    type Output = Self;

    /// Reverses this `Displacement`'s direction.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::try_from_meters_vec2(-self.0).expect("negation of finite vector should be finite")
    }
}

impl Validated for Displacement {
    type Repr = glam::Vec2;

    #[inline(always)]
    fn as_repr(&self) -> glam::Vec2 {
        self.as_meters_vec2()
    }

    #[inline]
    fn validate(value: glam::Vec2) -> Option<Self> {
        Self::try_from_meters_vec2(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_meters_vec2_rejects_non_finite() {
        assert!(Displacement::try_from_meters_vec2(glam::Vec2::new(f32::NAN, 0.0)).is_err());
    }

    #[test]
    fn only_zero_is_zero() {
        assert!(Displacement::ZERO.is_zero());
        let non_zero_vec = glam::Vec2::new(1.0, 0.0);
        let from_non_zero = Displacement::try_from_meters_vec2(non_zero_vec).unwrap();
        assert!(!from_non_zero.is_zero());
    }

    #[test]
    fn velocity_divides_by_duration() {
        let displacement = Displacement::from_meters_vec2(glam::Vec2::new(3.0, 6.0));

        let velocity = displacement.velocity(Duration::from_secs(2)).unwrap();

        assert_eq!(
            velocity.as_meters_per_second_vec2(),
            glam::Vec2::new(1.5, 3.0),
        );
    }

    #[test]
    fn velocity_round_trips_through_displacement() {
        let velocity = Velocity::from_meters_per_second_vec2(glam::Vec2::new(3.0, -4.0));
        let duration = Duration::from_millis(250);

        let round_tripped = velocity
            .displacement(duration)
            .unwrap()
            .velocity(duration)
            .unwrap();

        assert_eq!(round_tripped, velocity);
    }

    #[test]
    fn velocity_rejects_a_zero_duration() {
        let displacement = Displacement::from_meters_vec2(glam::Vec2::new(1.0, 0.0));
        assert!(displacement.velocity(Duration::ZERO).is_err());
    }
}
