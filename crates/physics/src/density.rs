use crate::{ValidationError, ValidationResult, ops};

/// Density, dimension ML⁻³ (mass per length cubed).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Density(f32);

impl Density {
    /// The largest representable density.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Density` from the specified [`f32`], in kilograms per
    /// cubic meter.
    ///
    /// # Panics
    /// This constructor will panic if value is not positive, overflows
    /// `Density` or not finite.
    #[inline(always)]
    pub fn from_kilograms_per_cubic_meter_f32(value: f32) -> Self {
        Self::try_from_kilograms_per_cubic_meter_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_kilograms_per_cubic_meter_f32`](Self::from_kilograms_per_cubic_meter_f32).
    ///
    /// This constructor will return an `Err` if value is not positive,
    /// overflows `Density` or not finite.
    #[inline]
    pub const fn try_from_kilograms_per_cubic_meter_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value <= 0.0 {
            Err(ValidationError("density must be finite and positive"))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `Density` as [`f32`], in kilograms per cubic meter.
    #[inline(always)]
    pub const fn as_kilograms_per_cubic_meter_f32(&self) -> f32 {
        self.0
    }
}

impl Eq for Density {}

impl PartialOrd for Density {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Density {
    /// Compares two densities.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("density is always finite, so a total order exists")
    }
}

impl ops::Validated for Density {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_kilograms_per_cubic_meter_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_kilograms_per_cubic_meter_f32(value).ok()
    }
}

impl ops::UpperBounded for Density {
    const MAX: Self = Self::MAX;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_kilograms_per_cubic_meter_rejects_non_positive() {
        assert!(Density::try_from_kilograms_per_cubic_meter_f32(0.0).is_err());
        assert!(Density::try_from_kilograms_per_cubic_meter_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_kilograms_per_cubic_meter_rejects_non_finite() {
        assert!(Density::try_from_kilograms_per_cubic_meter_f32(f32::NAN).is_err());
        assert!(Density::try_from_kilograms_per_cubic_meter_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_kilograms_per_cubic_meter_accepts_positive() {
        let density = Density::from_kilograms_per_cubic_meter_f32(1.2255);
        assert_eq!(density.as_kilograms_per_cubic_meter_f32(), 1.2255);
    }
}
