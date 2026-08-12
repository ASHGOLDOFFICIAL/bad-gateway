use crate::traits::Validated;

/// The checked version of [`std::ops::Add`].
pub trait CheckedAdd<Rhs = Self> {
    /// The resulting type after applying the checked addition operator.
    type Output;

    /// Performs the `+` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

impl<T> CheckedAdd for T
where
    T: Validated,
    T::Repr: std::ops::Add<Output = T::Repr>,
{
    type Output = Self;

    #[inline]
    fn checked_add(self, rhs: Self) -> Option<Self> {
        T::validate(self.as_repr() + rhs.as_repr())
    }
}
