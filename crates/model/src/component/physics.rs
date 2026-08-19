mod shape;

pub use physics::{Displacement, Mass, Position, Power, Speed, Temperature, Velocity};
pub use shape::*;

use std::time::Duration;

use super::utils::impl_delta_ops;
use physics::{Angle, Energy, ForceMagnitude, ops::Delta};

/// This object can change position.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moveable;

/// The heading this entity is moving towards.
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::From, derive_more::Into)]
pub struct Direction(Angle);

/// Used to add heat to object to be later transformed into temperature.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::From, derive_more::Into)]
pub struct HeatDelta(Delta<Energy>);

impl_delta_ops!(HeatDelta => Energy);

/// Rated thrust of a reaction drive.
#[must_use]
#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, derive_more::From, derive_more::Into,
)]
pub struct Thrust(ForceMagnitude);

/// Time constant of the exponential approach a mecha makes towards
/// its target velocity: ~63% of the gap is closed in one `Agility`.
#[must_use]
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::From,
    derive_more::Into,
)]
pub struct Agility(Duration);

impl Agility {
    /// Returns this `Agility` as [`f32`], in seconds.
    #[inline(always)]
    pub const fn as_seconds_f32(&self) -> f32 {
        self.0.as_secs_f32()
    }

    /// Returns this `Agility` scaled by `factor`, saturating to
    /// [`Duration::ZERO`] or [`Duration::MAX`].
    #[inline]
    pub fn saturating_mul(self, factor: f32) -> Self {
        let seconds = self.as_seconds_f32() * factor;
        let bound = if seconds > 0.0 {
            Duration::MAX
        } else {
            Duration::ZERO
        };
        Self(Duration::try_from_secs_f32(seconds).unwrap_or(bound))
    }
}

/// Method of collision resolution.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CollisionResolutionMethod {
    #[default]
    Stop,
    Slide,
}
