/// The checked version of [`std::ops::Div`].
pub trait CheckedDiv<Rhs = Self> {
    /// The resulting type after applying the checked division operator.
    type Output;

    /// Performs the `/` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}
