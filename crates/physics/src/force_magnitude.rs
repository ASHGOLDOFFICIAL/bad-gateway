use crate::{
    Angle, Force, ValidationError, ValidationResult,
    traits::{Bounded, NonNegative, Validated},
};

/// Force magnitude (the magnitude of a force), dimension MLT⁻² (mass times
/// length per time squared).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ForceMagnitude(f32);

impl ForceMagnitude {
    /// `ForceMagnitude` of zero newtons.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable force magnitude.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `ForceMagnitude` from the specified [`f32`], in newtons.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows
    /// `ForceMagnitude` or not finite.
    #[inline(always)]
    pub fn from_newtons_f32(value: f32) -> Self {
        Self::try_from_newtons_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_newtons_f32`](Self::from_newtons_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `ForceMagnitude` or not finite.
    #[inline]
    pub const fn try_from_newtons_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError(
                "force magnitude must be finite and non-negative",
            ))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this `ForceMagnitude` as [`f32`], in newtons.
    #[inline(always)]
    pub const fn as_newtons_f32(&self) -> f32 {
        self.0
    }

    /// Returns `true` if this `ForceMagnitude` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives [`Force`] from this `ForceMagnitude` and given [`Angle`].
    ///
    /// It uses `F = m * d`, where
    /// - `F` is force,
    /// - `m` is this magnitude,
    /// - `d` is direction.
    #[inline]
    pub fn force(self, direction: Angle) -> Force {
        Force::try_from_newtons_vec2(direction.as_vec2_f32() * self.0)
            .expect("a unit vector scaled by a finite magnitude is always finite")
    }
}

impl Eq for ForceMagnitude {}

impl PartialOrd for ForceMagnitude {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ForceMagnitude {
    /// Compares two force magnitudes.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("force magnitude is always finite, so a total order exists")
    }
}

impl Validated for ForceMagnitude {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_newtons_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_newtons_f32(value).ok()
    }
}

impl Bounded for ForceMagnitude {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::ZERO;
}

impl NonNegative for ForceMagnitude {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_newtons_rejects_negative() {
        assert!(ForceMagnitude::try_from_newtons_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_newtons_rejects_non_finite() {
        assert!(ForceMagnitude::try_from_newtons_f32(f32::NAN).is_err());
        assert!(ForceMagnitude::try_from_newtons_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_newtons_accepts_non_negative() {
        assert!(ForceMagnitude::from_newtons_f32(0.0).is_zero());
        assert_eq!(ForceMagnitude::from_newtons_f32(5.0).as_newtons_f32(), 5.0);
    }

    #[test]
    fn force_points_along_the_given_direction() {
        let magnitude = ForceMagnitude::from_newtons_f32(10.0);
        let east = Angle::from_radians_f32(0.0);

        let force = magnitude.force(east).as_newtons_vec2();

        assert!((force.x - 10.0).abs() < 1e-4);
        assert!(force.y.abs() < 1e-4);
    }

    #[test]
    fn force_preserves_magnitude_in_any_direction() {
        let magnitude = ForceMagnitude::from_newtons_f32(7.0);
        let direction = Angle::from_radians_f32(core::f32::consts::PI / 4.0);

        let length = magnitude.force(direction).as_newtons_vec2().length();

        assert!((length - 7.0).abs() < f32::EPSILON);
    }

    #[test]
    fn zero_magnitude_yields_zero_force() {
        let angle = Angle::from_radians_f32(core::f32::consts::PI / 2.0);
        let force = ForceMagnitude::ZERO.force(angle);
        assert!(force.is_zero());
    }

    #[test]
    fn ordering_compares_by_newtons() {
        let weak = ForceMagnitude::from_newtons_f32(1.0);
        let strong = ForceMagnitude::from_newtons_f32(2.0);

        assert!(weak < strong);
        assert_eq!(weak.min(strong), weak);
        assert_eq!(weak.max(strong), strong);
    }
}
