use physics::ForceMagnitude;

mod shape;

pub use shape::*;

/// This object can change position.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moveable;

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
