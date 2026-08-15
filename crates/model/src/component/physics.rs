use physics::ForceMagnitude;

/// Rated thrust of a reaction drive.
#[must_use]
#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, derive_more::From, derive_more::Into,
)]
pub struct Thrust(ForceMagnitude);
