use crate::traits::Validated;

/// The checked version of [`std::ops::Sub`].
pub trait CheckedSub<Rhs = Self> {
    /// The resulting type after applying the checked subtraction operator.
    type Output;

    /// Performs the `-` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

impl<T> CheckedSub for T
where
    T: Validated,
    T::Repr: std::ops::Sub<Output = T::Repr>,
{
    type Output = Self;

    #[inline]
    fn checked_sub(self, rhs: Self) -> Option<Self> {
        T::validate(self.as_repr() - rhs.as_repr())
    }
}
