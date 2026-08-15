/// Marks an entity as doing work right now.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Active;

/// Marks an entity as switched off. Should be able to recover.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Disabled;

/// Marks an entity that asked for more power than it was given.
///
/// An [`Unpowered`] entity becomes [`Disabled`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Unpowered;
