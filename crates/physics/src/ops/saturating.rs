/// The saturating version of [`std::ops::Add`].
pub trait SaturatingAdd<Rhs = Self> {
    /// The resulting type after applying the saturating addition operator.
    type Output;

    /// Performs the `+` operation. Returns the maximal value of `Output`
    /// if the result is out of valid range.
    #[must_use]
    fn saturating_add(self, rhs: Rhs) -> Self::Output;
}

/// The saturating version of [`std::ops::Sub`].
pub trait SaturatingSub<Rhs = Self> {
    /// The resulting type after applying the saturating subtraction operator.
    type Output;

    /// Performs the `-` operation. Returns the minimal value of `Output`
    /// if the result is out of valid range.
    #[must_use]
    fn saturating_sub(self, rhs: Rhs) -> Self::Output;
}
