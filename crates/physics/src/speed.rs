use crate::{
    Angle, ValidationError, ValidationResult, Velocity,
    traits::{Bounded, NonNegative, Validated},
};

/// Speed (the magnitude of a velocity), dimension LT⁻¹ (length per time).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Speed(f32);

impl Speed {
    /// `Speed` of zero meters per second.
    pub const ZERO: Self = Self(0.0);

    /// The largest representable speed.
    pub const MAX: Self = Self(f32::MAX);

    /// Creates a new `Speed` from the specified [`f32`], in meters per
    /// second.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows `Speed`
    /// or not finite.
    #[inline(always)]
    pub fn from_meters_per_second_f32(value: f32) -> Self {
        Self::try_from_meters_per_second_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_meters_per_second_f32`](Self::from_meters_per_second_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Speed` or not finite.
    #[inline]
    pub const fn try_from_meters_per_second_f32(value: f32) -> ValidationResult<Self> {
        if !value.is_finite() || value < 0.0 {
            Err(ValidationError("speed must be finite and non-negative"))
        } else {
            Ok(Self(value))
        }
    }

    /// Creates a new `Speed` from the specified [`f32`], in kilometres per
    /// hour.
    ///
    /// # Panics
    /// This constructor will panic if value is negative, overflows `Speed`
    /// or not finite.
    #[inline(always)]
    pub fn from_kilometres_per_hour_f32(value: f32) -> Self {
        Self::try_from_kilometres_per_hour_f32(value).expect("unsafe method")
    }

    /// The checked version of
    /// [`from_kilometres_per_hour_f32`](Self::from_kilometres_per_hour_f32).
    ///
    /// This constructor will return an `Err` if value is negative,
    /// overflows `Speed` or not finite.
    #[inline]
    pub const fn try_from_kilometres_per_hour_f32(value: f32) -> ValidationResult<Self> {
        let kmph = value / 3.6;
        if !kmph.is_finite() || kmph < 0.0 {
            Err(ValidationError("speed must be finite and non-negative"))
        } else {
            Ok(Self(kmph))
        }
    }

    /// Returns this `Speed` as [`f32`], in meters per second.
    #[inline(always)]
    pub const fn as_meters_per_second_f32(&self) -> f32 {
        self.0
    }

    /// Returns this `Speed` as [`f32`], in kilometres per hour.
    pub const fn as_kilometres_per_hour_f32(&self) -> f32 {
        self.0 * 3.6
    }

    /// Returns `true` if this `Speed` is exactly zero.
    #[inline(always)]
    pub const fn is_zero(&self) -> bool {
        self.0 == Self::ZERO.0
    }

    /// Derives [`Velocity`] from this `Speed` and given [`Angle`].
    ///
    /// It uses `v = s * d`, where
    /// - `v` is velocity,
    /// - `s` is this speed,
    /// - `d` is direction.
    #[inline]
    pub fn velocity(self, direction: Angle) -> Velocity {
        Velocity::try_from_meters_per_second_vec2(direction.as_vec2_f32() * self.0)
            .expect("a unit vector scaled by a finite speed is always finite")
    }
}

impl Eq for Speed {}

impl PartialOrd for Speed {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Speed {
    /// Compares two speeds.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .expect("speed is always finite, so a total order exists")
    }
}

impl Validated for Speed {
    type Repr = f32;

    #[inline(always)]
    fn as_repr(&self) -> f32 {
        self.as_meters_per_second_f32()
    }

    #[inline]
    fn validate(value: f32) -> Option<Self> {
        Self::try_from_meters_per_second_f32(value).ok()
    }
}

impl Bounded for Speed {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::ZERO;
}

impl NonNegative for Speed {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_from_meters_per_second_rejects_negative() {
        assert!(Speed::try_from_meters_per_second_f32(-1.0).is_err());
    }

    #[test]
    fn try_from_meters_per_second_rejects_non_finite() {
        assert!(Speed::try_from_meters_per_second_f32(f32::NAN).is_err());
        assert!(Speed::try_from_meters_per_second_f32(f32::INFINITY).is_err());
    }

    #[test]
    fn from_meters_per_second_accepts_non_negative() {
        let from_zero = Speed::from_meters_per_second_f32(0.0);
        assert!(from_zero.is_zero());

        let from_five = Speed::from_meters_per_second_f32(5.0);
        assert_eq!(from_five.as_meters_per_second_f32(), 5.0);
    }

    #[test]
    fn velocity_builds_along_direction() {
        let speed = Speed::from_meters_per_second_f32(5.0);
        let angle = Angle::from_radians_f32(0.0);
        let velocity = speed.velocity(angle);

        assert_eq!(velocity.speed().unwrap(), speed);
        assert_eq!(
            velocity.as_meters_per_second_vec2().normalize(),
            angle.as_vec2_f32()
        );
    }

    #[test]
    fn velocity_at_zero_speed_yields_zero_velocity() {
        let angle = Angle::from_radians_f32(1.0);
        assert!((Speed::ZERO.velocity(angle)).is_zero());
    }
}
