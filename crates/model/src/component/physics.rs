mod shape;

pub use physics::Temperature;
pub use shape::*;

use super::utils::impl_delta_ops;
use physics::{Angle, Energy, ForceMagnitude, ops::Delta};

/// This object can change position.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moveable;

/// Direction this entity move along.
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

/// Method of collision resolution.
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CollisionResolutionMethod {
    #[default]
    Stop,
    Slide,
}
