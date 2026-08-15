use crate::{
    Acceleration, CalculationError, CalculationResult, ForceMagnitude, Mass, ValidationError,
    ValidationResult, traits::Validated,
};

/// Force, dimension MLT⁻² (mass times length per time squared).
#[must_use]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Force(glam::Vec2);

impl Force {
    /// `Force` of zero newtons.
    pub const ZERO: Self = Self(glam::Vec2::ZERO);

    /// Creates a new `Force` from the specified [`glam::Vec2`], in newtons.
    ///
    /// # Panics
    /// This constructor will panic if value overflows `Force` or not
    /// finite.
    #[inline(always)]
    pub fn from_newtons_vec2(value: glam::Vec2) -> Self {
        Self::try_from_newtons_vec2(value).expect("unsafe method")
    }

    /// The checked version of [`from_newtons_vec2`](Self::from_newtons_vec2).
    ///
    /// This constructor will return an `Err` if value overflows `Force` or
    /// not finite.
    #[inline]
    pub fn try_from_newtons_vec2(value: glam::Vec2) -> ValidationResult<Self> {
        if !value.is_finite() {
            Err(ValidationError("vector must be finite"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Force` as [`glam::Vec2`], in newtons.
    #[inline(always)]
    pub const fn as_newtons_vec2(&self) -> glam::Vec2 {
        self.0
    }

    /// Returns `true` if this `Force` is exactly zero.
    #[inline(always)]
    pub fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// This `Force`'s magnitude, independent of direction.
    ///
    /// Errors if the magnitude overflows `ForceMagnitude`'s valid range.
    #[inline]
    pub fn magnitude(&self) -> CalculationResult<ForceMagnitude> {
        ForceMagnitude::try_from_newtons_f32(self.0.length()).map_err(CalculationError::from)
    }

    /// Derives [`Acceleration`] from this `Force` and given [`Mass`].
    ///
    /// It uses Newton's second law: `F = ma`, where:
    /// - `F` is net force,
    /// - `m` is object's mass,
    /// - `a` is object's acceleration.
    ///
    /// Errors if mass is zero.
    #[inline]
    pub fn acceleration(self, mass: Mass) -> CalculationResult<Acceleration> {
        if mass.is_zero() {
            Err(CalculationError::InvalidArgument("division by zero mass"))
        } else {
            Acceleration::try_from_meters_per_square_second_vec2(
                self.as_newtons_vec2() / mass.as_kilograms_f32(),
            )
            .map_err(CalculationError::from)
        }
    }
}

impl std::ops::Neg for Force {
    type Output = Self;

    /// Reverses this `Force`'s direction.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::try_from_newtons_vec2(-self.0).expect("negation of finite vector should be finite")
    }
}

impl Validated for Force {
    type Repr = glam::Vec2;

    #[inline(always)]
    fn as_repr(&self) -> glam::Vec2 {
        self.as_newtons_vec2()
    }

    #[inline]
    fn validate(value: glam::Vec2) -> Option<Self> {
        Self::try_from_newtons_vec2(value).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_newtons_vec2_rejects_non_finite() {
        assert!(Force::try_from_newtons_vec2(glam::Vec2::new(f32::NAN, 0.0)).is_err());
    }

    #[test]
    fn acceleration_rejects_zero_mass() {
        let force = Force::try_from_newtons_vec2(glam::Vec2::new(10.0, 0.0)).unwrap();
        assert!(force.acceleration(Mass::ZERO).is_err());
    }

    #[test]
    fn acceleration_divides_by_mass() {
        let force = Force::try_from_newtons_vec2(glam::Vec2::new(10.0, 4.0)).unwrap();
        let mass = Mass::from_kilograms_f32(2.0);
        let acceleration = force.acceleration(mass).unwrap();
        assert_eq!(
            acceleration.as_meters_per_square_second_vec2(),
            glam::Vec2::new(5.0, 2.0)
        );
    }

    #[test]
    fn magnitude_reports_length_regardless_of_direction() {
        let force = Force::try_from_newtons_vec2(glam::Vec2::new(3.0, 4.0)).unwrap();
        assert_eq!(force.magnitude().unwrap().as_newtons_f32(), 5.0);
    }

    #[test]
    fn magnitude_round_trips_through_force() {
        let direction = crate::Angle::from_radians_f32(core::f32::consts::PI / 4.0);
        let magnitude = ForceMagnitude::from_newtons_f32(12.0);

        let round_tripped = magnitude.force(direction).magnitude().unwrap();

        assert!((round_tripped.as_newtons_f32() - 12.0).abs() < 10.0 * f32::EPSILON);
    }
}
