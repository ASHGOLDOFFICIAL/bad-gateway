mod art;
mod artist;

pub use art::*;
pub use artist::*;

/// Shorthand for cell's chars and its color in RGB.
pub type ArtCell = (char, (u8, u8, u8));
