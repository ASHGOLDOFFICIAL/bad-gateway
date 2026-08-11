use crate::ops::{NonNegative, SaturatingSub};

/// A signed adjustment to some non-negative quantity `T`, computed via
/// [`Delta::between`] and applied via [`Delta::apply_to`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Delta<T: NonNegative> {
    /// No adjustment.
    #[default]
    None,

    /// Increase by the wrapped quantity.
    Positive(T),

    /// Decrease by the wrapped quantity.
    Negative(T),
}

impl<T: NonNegative> Delta<T> {
    /// Maps a `Delta<T>` to `Delta<U>` by applying a function to a contained
    /// value (if `Positive` or `Negative`) preserving direction or returns
    /// `None` (if `None`).
    #[must_use]
    pub fn map<U: NonNegative, F>(self, f: F) -> Delta<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Delta::None => Delta::None,
            Delta::Positive(value) => Delta::Positive(f(value)),
            Delta::Negative(value) => Delta::Negative(f(value)),
        }
    }

    /// The fallible version of [`map`](Self::map).
    ///
    /// Applies a fallible function to a contained value, preserving direction.
    /// Returns `None` (as `Ok`) for `None`, or the first error `f` returns.
    pub fn try_map<U: NonNegative, E, F>(self, f: F) -> Result<Delta<U>, E>
    where
        F: FnOnce(T) -> Result<U, E>,
    {
        match self {
            Delta::None => Ok(Delta::None),
            Delta::Positive(value) => f(value).map(Delta::Positive),
            Delta::Negative(value) => f(value).map(Delta::Negative),
        }
    }
}

impl<T: NonNegative> Delta<T>
where
    T: PartialOrd + SaturatingSub<Output = T>,
{
    /// The signed difference between `lhs` and `rhs`. Result is `Positive` if
    /// `lhs` > `rhs`, `Negative` if `rhs` > `lhs`, and `None` otherwise.
    #[must_use]
    pub fn between(lhs: T, rhs: T) -> Self {
        if lhs > rhs {
            Self::Positive(lhs.saturating_sub(rhs))
        } else if lhs < rhs {
            Self::Negative(rhs.saturating_sub(lhs))
        } else {
            Self::None
        }
    }
}

impl<T: NonNegative> Delta<T>
where
    T: std::ops::Add<Output = T> + SaturatingSub<Output = T>,
{
    /// Applies this delta to `base`, clamping at `base`'s own notion of zero
    /// when subtracting past it.
    #[must_use]
    pub fn apply_to(self, base: T) -> T {
        match self {
            Self::None => base,
            Self::Positive(delta) => base + delta,
            Self::Negative(delta) => base.saturating_sub(delta),
        }
    }
}

impl<T: NonNegative> std::ops::Add for Delta<T>
where
    T: Copy + PartialOrd + std::ops::Add<Output = T> + SaturatingSub<Output = T>,
{
    type Output = Self;

    /// Combines two deltas, netting out opposing directions.
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (_, Self::None) => self,
            (Self::None, _) => rhs,

            (Self::Positive(e1), Self::Positive(e2)) => Self::Positive(e1 + e2),
            (Self::Negative(e1), Self::Negative(e2)) => Self::Negative(e1 + e2),

            (Self::Positive(e1), Self::Negative(e2)) | (Self::Negative(e2), Self::Positive(e1)) => {
                if e1 < e2 {
                    Self::Negative(e2.saturating_sub(e1))
                } else if e1 > e2 {
                    Self::Positive(e1.saturating_sub(e2))
                } else {
                    Self::None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
    struct Quantity(i32);

    impl std::ops::Add for Quantity {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    impl NonNegative for Quantity {}

    impl SaturatingSub for Quantity {
        type Output = Self;

        fn saturating_sub(self, rhs: Self) -> Self::Output {
            Self((self.0 - rhs.0).max(0))
        }
    }

    #[test]
    fn between_is_positive_when_lhs_is_greater() {
        assert_eq!(
            Delta::between(Quantity(5), Quantity(2)),
            Delta::Positive(Quantity(3))
        );
    }

    #[test]
    fn between_is_negative_when_rhs_is_greater() {
        assert_eq!(
            Delta::between(Quantity(2), Quantity(5)),
            Delta::Negative(Quantity(3))
        );
    }

    #[test]
    fn between_is_none_when_lhs_equals_rhs() {
        assert_eq!(Delta::between(Quantity(5), Quantity(5)), Delta::None);
    }

    #[test]
    fn map_transforms_wrapped_value_preserving_direction() {
        assert_eq!(
            Delta::Positive(Quantity(1)).map(|q| q + Quantity(1)),
            Delta::Positive(Quantity(2))
        );
        assert_eq!(
            Delta::Negative(Quantity(1)).map(|q| q + Quantity(1)),
            Delta::Negative(Quantity(2))
        );
        assert_eq!(Delta::None.map(|q: Quantity| q + Quantity(1)), Delta::None);
    }

    #[test]
    fn try_map_transforms_wrapped_value_preserving_direction() {
        assert_eq!(
            Delta::Positive(Quantity(1)).try_map(|q| Ok::<_, &str>(q + Quantity(1))),
            Ok(Delta::Positive(Quantity(2)))
        );
        assert_eq!(
            Delta::Negative(Quantity(1)).try_map(|q| Ok::<_, &str>(q + Quantity(1))),
            Ok(Delta::Negative(Quantity(2)))
        );
        assert_eq!(
            Delta::None.try_map(|q: Quantity| Ok::<_, &str>(q + Quantity(1))),
            Ok(Delta::None)
        );
    }

    #[test]
    fn try_map_propagates_error() {
        assert_eq!(
            Delta::Positive(Quantity(1)).try_map(|_| Err::<Quantity, _>("failed")),
            Err("failed")
        );
    }

    #[test]
    fn applied_none_leaves_base_unchanged() {
        assert_eq!(Delta::None.apply_to(Quantity(5)), Quantity(5));
    }

    #[test]
    fn applied_positive_increases_base() {
        assert_eq!(
            Delta::Positive(Quantity(3)).apply_to(Quantity(5)),
            Quantity(8)
        );
    }

    #[test]
    fn applied_negative_decreases_base_within_bounds() {
        assert_eq!(
            Delta::Negative(Quantity(3)).apply_to(Quantity(5)),
            Quantity(2)
        );
    }

    #[test]
    fn applied_negative_saturates_at_zero() {
        assert_eq!(
            Delta::Negative(Quantity(10)).apply_to(Quantity(5)),
            Quantity(0)
        );
    }

    #[test]
    fn add_combines_same_direction() {
        assert_eq!(
            Delta::Positive(Quantity(1)) + Delta::Positive(Quantity(2)),
            Delta::Positive(Quantity(3))
        );
        assert_eq!(
            Delta::Negative(Quantity(1)) + Delta::Negative(Quantity(2)),
            Delta::Negative(Quantity(3))
        );
    }

    #[test]
    fn add_nets_out_opposing_directions() {
        assert_eq!(
            Delta::Positive(Quantity(5)) + Delta::Negative(Quantity(2)),
            Delta::Positive(Quantity(3))
        );
        assert_eq!(
            Delta::Positive(Quantity(2)) + Delta::Negative(Quantity(5)),
            Delta::Negative(Quantity(3))
        );
        assert_eq!(
            Delta::Positive(Quantity(2)) + Delta::Negative(Quantity(2)),
            Delta::None
        );
    }

    #[test]
    fn none_is_identity_of_addition() {
        assert_eq!(
            Delta::Positive(Quantity(1)) + Delta::None,
            Delta::Positive(Quantity(1))
        );
        assert_eq!(
            Delta::None + Delta::Negative(Quantity(1)),
            Delta::Negative(Quantity(1))
        );
    }
}
