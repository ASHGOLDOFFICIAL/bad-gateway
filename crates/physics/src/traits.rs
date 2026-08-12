//! Traits used to automatically derive the checked and saturating
//! operations in [`ops`](crate::ops).

/// Marks a quantity that can never go negative.
///
/// Allows us to assume that:
/// - result of addition is never less than its first operand, and that means
///   overflow can only happen over upper bound,
/// - result of subtraction is never greater than its first operand, and that
///   means underflow can only happen over lower bound.
pub trait NonNegative {}

impl<T: NonNegative> NonNegative for &T {}
impl<T: NonNegative> NonNegative for &mut T {}
impl<T: NonNegative> NonNegative for Option<T> {}
impl<T: NonNegative, E> NonNegative for Result<T, E> {}

/// A quantity backed by a validated representation.
pub(crate) trait Validated: Sized {
    type Repr;

    fn as_repr(&self) -> Self::Repr;
    fn validate(value: Self::Repr) -> Option<Self>;
}

/// A quantity with both smallest and largest representable values.
pub(crate) trait Bounded {
    const MAX: Self;
    const MIN: Self;
}

/// A quantity with a largest representable value.
pub(crate) trait UpperBounded {
    const MAX: Self;
}

impl<T: Bounded> UpperBounded for T {
    const MAX: Self = Self::MAX;
}

/// A quantity with a smallest representable value.
pub(crate) trait LowerBounded {
    const MIN: Self;
}

impl<T: Bounded> LowerBounded for T {
    const MIN: Self = Self::MIN;
}

/// Implements [`NonNegative`] and [`Bounded`] for the given types. Only valid
/// for types that can never represent a negative value.
macro_rules! impl_non_negative {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl NonNegative for $ty {}

            impl Bounded for $ty {
                const MAX: Self = <$ty>::MAX;
                const MIN: Self = <$ty>::MIN;
            }
        )+
    };
}

impl_non_negative!(u8, u16, u32, u64, u128, usize);
