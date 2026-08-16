use core::f32;

use crate::{
    Area, CalculationError, CalculationResult, Length, ValidationError, ValidationResult,
    ops::CheckedMul, shapes::Rect,
};

/// A circle, described by its radius.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    radius: Length,
}

impl Circle {
    /// Creates a `Circle` from the given `radius`, which must be positive.
    #[inline]
    pub fn new(radius: Length) -> ValidationResult<Self> {
        if radius.is_zero() {
            Err(ValidationError("circle's radius must be positive"))
        } else {
            Ok(Self { radius })
        }
    }

    /// This `Circle`'s radius.
    #[inline(always)]
    pub const fn radius(&self) -> Length {
        self.radius
    }

    /// This `Circle`'s diameter.
    #[inline]
    pub fn diameter(&self) -> CalculationResult<Length> {
        self.radius
            .checked_mul(2.0)
            .ok_or(CalculationError::Overflow(
                "diameter overflowed length's valid range",
            ))
    }

    /// This `Circle`'s [`Area`].
    #[inline]
    pub fn area(&self) -> CalculationResult<Area> {
        let squared = self.radius.area(self.radius)?;
        squared
            .checked_mul(f32::consts::PI)
            .ok_or(CalculationError::Overflow(
                "area overflowed its valid range",
            ))
    }

    /// This `Circle`'s inscribed square as a [`Rect`].
    pub fn inscribed_square(&self) -> CalculationResult<Rect> {
        let side =
            self.radius
                .checked_mul(f32::consts::SQRT_2)
                .ok_or(CalculationError::Overflow(
                    "square side length overflowed allowed range",
                ))?;
        Rect::new(side, side).map_err(CalculationError::from)
    }
}

impl Eq for Circle {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_non_positive_radius() {
        assert!(Circle::new(Length::ZERO).is_err());
    }

    #[test]
    fn diameter_is_twice_the_radius() {
        let circle = Circle::new(Length::from_meters_f32(2.0)).unwrap();
        assert_eq!(circle.diameter().unwrap(), Length::from_meters_f32(4.0));
    }

    #[test]
    fn area_uses_pi_r_squared() {
        let circle = Circle::new(Length::from_meters_f32(2.0)).unwrap();
        let expected = 2.0 * 2.0 * f32::consts::PI;
        assert!((circle.area().unwrap().as_square_meters_f32() - expected).abs() < 1e-4);
    }

    #[test]
    fn inscribed_square_side_is_radius_times_sqrt_2() {
        let radius = 2.0;
        let circle = Circle::new(Length::from_meters_f32(radius)).unwrap();
        let square = circle.inscribed_square().unwrap();
        let expected = radius * f32::consts::SQRT_2;

        assert_eq!(square.width(), square.height());
        assert!((square.width().as_meters_f32() - expected).abs() < 1e-4);
    }
}
