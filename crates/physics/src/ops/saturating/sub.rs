use crate::traits::{LowerBounded, NonNegative, Validated};

/// The saturating version of [`std::ops::Sub`].
pub trait SaturatingSub<Rhs = Self> {
    /// The resulting type after applying the saturating subtraction operator.
    type Output;

    /// Performs the `-` operation. Returns the minimal value of `Output`
    /// if the result is out of valid range.
    #[must_use]
    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}

impl<T> SaturatingSub for T
where
    T: Validated + NonNegative + LowerBounded,
    T::Repr: std::ops::Sub<Output = T::Repr>,
{
    type Output = Self;

    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        T::validate(self.as_repr() - rhs.as_repr()).unwrap_or(T::MIN)
    }
}
