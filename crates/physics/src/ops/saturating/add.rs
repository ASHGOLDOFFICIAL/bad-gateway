use crate::traits::{NonNegative, UpperBounded, Validated};

/// The saturating version of [`std::ops::Add`].
pub trait SaturatingAdd<Rhs = Self> {
    /// The resulting type after applying the saturating addition operator.
    type Output;

    /// Performs the `+` operation. Returns the maximal value of `Output`
    /// if the result is out of valid range.
    #[must_use]
    fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

impl<T> SaturatingAdd for T
where
    T: Validated + NonNegative + UpperBounded,
    T::Repr: std::ops::Add<Output = T::Repr>,
{
    type Output = Self;

    #[inline]
    fn saturating_add(self, rhs: Self) -> Self {
        T::validate(self.as_repr() + rhs.as_repr()).unwrap_or(T::MAX)
    }
}
