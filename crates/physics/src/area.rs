use crate::{ValidationError, ValidationResult, ops};

/// Area, dimension L² (length squared).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Area(f32);

impl Area {
    /// The largest representable area.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Area` from the specified [`f32`], in square meters.
    ///
    /// # Panics
    /// This constructor will panic if value is not positive, overflows
    /// `Area` or not finite.
    #[inline(always)]
    pub fn from_square_meters_f32(value: f32) -> Self {
        Self::try_from_square_meters_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_square_meters_f32`](Self::from_square_meters_f32).
    ///
    /// This constructor will return an `Err` if value is not positive,
    /// overflows `Area` or not finite.
    #[inline]
    pub const fn try_from_square_meters_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            Err(ValidationError("area must be finite and positive"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Area` as [`f32`], in square meters.
    #[inline(always)]
    pub const fn as_square_meters_f32(&self) -> f32 {
        self.0
    }
}

impl Eq for Area {}

impl PartialOrd for Area {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Area {
    /// Compares two areas.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("area is always finite, so a total order exists")
    }
}

impl ops::Validated for Area {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_square_meters_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_square_meters_f32(value).ok()
    }
}

impl ops::UpperBounded for Area {
    const MAX: Self = Self::MAX;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_square_meters_rejects_non_positive() {
        assert!(Area::try_from_square_meters_f32(0.0).is_err());
        assert!(Area::try_from_square_meters_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_square_meters_rejects_non_finite() {
        assert!(Area::try_from_square_meters_f32(f32::NAN).is_err());
        assert!(Area::try_from_square_meters_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_square_meters_accepts_positive() {
        assert_eq!(
            Area::from_square_meters_f32(12.0).as_square_meters_f32(),
            12.0
        );
    }
}
