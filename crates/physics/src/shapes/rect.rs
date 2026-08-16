use crate::{Area, CalculationResult, Length, ValidationError, ValidationResult};

/// A rectangle, described by its width and height.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    width: Length,
    height: Length,
}

impl Rect {
    /// Creates a `Rect` from the given `width` and `height`,
    /// which must both be positive.
    #[inline]
    pub fn new(width: Length, height: Length) -> ValidationResult<Self> {
        if width.is_zero() || height.is_zero() {
            Err(ValidationError("rect's width and height must be positive"))
        } else {
            Ok(Self { width, height })
        }
    }

    /// This `Rect`'s width.
    #[inline(always)]
    pub const fn width(&self) -> Length {
        self.width
    }

    /// This `Rect`'s height.
    #[inline(always)]
    pub const fn height(&self) -> Length {
        self.height
    }

    /// This `Rect`'s [`Area`].
    #[inline]
    pub fn area(&self) -> CalculationResult<Area> {
        self.width.area(self.height)
    }
}

impl Eq for Rect {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_non_positive_dimensions() {
        let positive = Length::from_meters_f32(1.0);
        assert!(Rect::new(Length::ZERO, positive).is_err());
        assert!(Rect::new(positive, Length::ZERO).is_err());
    }

    #[test]
    fn area_multiplies_width_by_height() {
        let width = Length::from_meters_f32(3.0);
        let height = Length::from_meters_f32(4.0);
        let rect = Rect::new(width, height).unwrap();

        assert_eq!(rect.area().unwrap().as_square_meters_f32(), 12.0);
    }
}
