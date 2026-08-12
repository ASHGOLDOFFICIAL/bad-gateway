use crate::traits::Validated;

/// The checked version of [`std::ops::Mul`].
pub trait CheckedMul<Rhs = Self> {
    /// The resulting type after applying the checked multiplication operator.
    type Output;

    /// Performs the `*` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

impl<T> CheckedMul<f32> for T
where
    T: Validated,
    T::Repr: std::ops::Mul<f32, Output = T::Repr>,
{
    type Output = Self;

    #[inline]
    fn checked_mul(self, rhs: f32) -> Option<Self> {
        T::validate(self.as_repr() * rhs)
    }
}
