//! Physical units and formulas for 2D physics simulation.
//!
//! Every quantity is a newtype validated at construction (finite, and
//! non-negative/positive where physically required), so a value that exists
//! is always safe to use in downstream arithmetic without re-checking it.
//!
//! The goal is to let calling code work in physical terms instead of raw
//! numbers, risking unit mismatching.

mod acceleration;
mod angle;
mod area;
mod density;
mod displacement;
mod energy;
mod force;
mod force_magnitude;
pub mod formulae;
mod frequency;
mod heat_capacity;
mod length;
mod mass;
pub mod ops;
mod position;
mod power;
pub mod shapes;
mod specific_heat_capacity;
mod speed;
mod temperature;
pub mod traits;
mod velocity;

pub use acceleration::*;
pub use angle::*;
pub use area::*;
pub use density::*;
pub use displacement::*;
pub use energy::*;
pub use force::*;
pub use force_magnitude::*;
pub use frequency::*;
pub use heat_capacity::*;
pub use length::*;
pub use mass::*;
pub use position::*;
pub use power::*;
pub use specific_heat_capacity::*;
pub use speed::*;
pub use temperature::*;
pub use velocity::*;

pub(crate) type ValidationResult<T> = Result<T, ValidationError>;

/// Error returned by a fallible constructor, indicating that arguments are
/// invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ValidationError(&'static str);

impl ValidationError {
    #[inline(always)]
    /// Returns the underlying message.
    pub const fn message(&self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for ValidationError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for ValidationError {}

pub(crate) type CalculationResult<T> = Result<T, CalculationError>;

/// Error returned by operations.
///
/// Describes two kind of failures:
/// - passing of invalid arguments, which is user fault,
/// - result overflow, which is not user fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalculationError {
    /// An argument doesn't satisfy this specific operation's requirements.
    InvalidArgument(&'static str),

    /// Computed result fell outside the type's valid range.
    Overflow(&'static str),
}

impl CalculationError {
    #[inline(always)]
    /// Returns the underlying message.
    pub const fn message(&self) -> &'static str {
        match self {
            CalculationError::InvalidArgument(msg) => msg,
            CalculationError::Overflow(msg) => msg,
        }
    }
}

impl From<ValidationError> for CalculationError {
    #[inline(always)]
    fn from(value: ValidationError) -> Self {
        Self::Overflow(value.0)
    }
}

impl std::fmt::Display for CalculationError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

impl std::error::Error for CalculationError {}
