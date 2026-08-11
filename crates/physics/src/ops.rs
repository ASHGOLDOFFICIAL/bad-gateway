//! Generic arithmetic traits shared across physics quantities.
//!
//! Checked traits return `None` instead of an invalid value. Saturating traits
//! clamp into range instead. Delta allows signed representation of non-negative
//! quantities.

mod checked;
mod delta;
mod saturating;
mod traits;

pub use checked::*;
pub use delta::*;
pub use saturating::*;
pub(crate) use traits::*;
