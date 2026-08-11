use crate::{ValidationError, ValidationResult, ops};

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
}

impl std::ops::Neg for Displacement {
    type Output = Self;

    /// Reverses this `Displacement`'s direction.
    #[inline]
    fn neg(self) -> Self::Output {
        Self::try_from_meters_vec2(-self.0).expect("negation of finite vector should be finite")
    }
}

impl ops::Validated for Displacement {
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
}
