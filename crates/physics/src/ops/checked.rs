/// The checked version of [`std::ops::Add`].
pub trait CheckedAdd<Rhs = Self> {
    /// The resulting type after applying the checked addition operator.
    type Output;

    /// Performs the `+` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_add(self, rhs: Rhs) -> Option<Self::Output>;
}

/// The checked version of [`std::ops::Sub`].
pub trait CheckedSub<Rhs = Self> {
    /// The resulting type after applying the checked subtraction operator.
    type Output;

    /// Performs the `-` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_sub(self, rhs: Rhs) -> Option<Self::Output>;
}

/// The checked version of [`std::ops::Mul`].
pub trait CheckedMul<Rhs = Self> {
    /// The resulting type after applying the checked multiplication operator.
    type Output;

    /// Performs the `*` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_mul(self, rhs: Rhs) -> Option<Self::Output>;
}

/// The checked version of [`std::ops::Div`].
pub trait CheckedDiv<Rhs = Self> {
    /// The resulting type after applying the checked division operator.
    type Output;

    /// Performs the `/` operation. Returns `None` if the result is invalid.
    #[must_use]
    fn checked_div(self, rhs: Rhs) -> Option<Self::Output>;
}
