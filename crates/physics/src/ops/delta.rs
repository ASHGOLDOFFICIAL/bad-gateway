use crate::{
    ops::{CheckedAdd, SaturatingAdd, SaturatingSub},
    traits::NonNegative,
};

/// A signed adjustment to some [`NonNegative`] quantity `T`, computed via
/// [`between`](Delta::between) and applied via [`apply_to`](Delta::apply_to).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Delta<T: NonNegative> {
    /// No adjustment.
    #[default]
    Zero,

    /// Increase by the wrapped quantity.
    Positive(T),

    /// Decrease by the wrapped quantity.
    Negative(T),
}

impl<T: NonNegative + Eq> Eq for Delta<T> {}

impl<T: NonNegative + Ord> PartialOrd for Delta<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: NonNegative + Ord> Ord for Delta<T> {
    /// Compares two deltas as signed magnitudes: the most negative sorts
    /// first, then [`Zero`](Delta::Zero), and the most positive sorts last.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;

        match (self, other) {
            (Self::Zero, Self::Zero) => Ordering::Equal,
            (Self::Zero, Self::Positive(_)) => Ordering::Less,
            (Self::Zero, Self::Negative(_)) => Ordering::Greater,

            (Self::Positive(_), Self::Zero) => Ordering::Greater,
            (Self::Positive(a), Self::Positive(b)) => a.cmp(b),
            (Self::Positive(_), Self::Negative(_)) => Ordering::Greater,

            (Self::Negative(_), Self::Zero) => Ordering::Less,
            (Self::Negative(_), Self::Positive(_)) => Ordering::Less,
            (Self::Negative(a), Self::Negative(b)) => b.cmp(a),
        }
    }
}

impl<T: NonNegative> Delta<T> {
    /// Checks if this `Delta` is [`Zero`](Delta::Zero).
    pub fn is_zero(&self) -> bool {
        matches!(self, Delta::Zero)
    }

    /// Checks if this `Delta` is [`Positive`](Delta::Positive).
    pub fn is_positive(&self) -> bool {
        matches!(self, Delta::Positive(_))
    }

    /// Checks if this `Delta` is [`Negative`](Delta::Negative).
    pub fn is_negative(&self) -> bool {
        matches!(self, Delta::Negative(_))
    }
}

impl<T: NonNegative> Delta<T>
where
    T: PartialOrd + SaturatingSub<Output = T>,
{
    /// The signed difference between `lhs` and `rhs`. Result is
    /// [`Positive`](Delta::Positive) if `lhs` > `rhs`,
    /// [`Negative`](Delta::Negative) if `rhs` > `lhs`, and
    /// [`Zero`](Delta::Zero) otherwise.
    #[inline]
    pub fn between(lhs: T, rhs: T) -> Self {
        if lhs > rhs {
            Self::Positive(lhs.saturating_sub(rhs))
        } else if rhs > lhs {
            Self::Negative(rhs.saturating_sub(lhs))
        } else {
            Self::Zero
        }
    }
}

impl<T: NonNegative> Delta<T> {
    /// Maps a `Delta<T>` to `Delta<U>` by applying a function to a contained
    /// value (if [`Positive`](Delta::Positive) or
    /// [`Negative`](Delta::Negative)) preserving direction or returns
    /// [`Zero`](Delta::Zero) (if [`Zero`](Delta::Zero)).
    #[inline]
    pub fn map<U: NonNegative, F>(self, op: F) -> Delta<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Delta::Zero => Delta::Zero,
            Delta::Positive(value) => Delta::Positive(op(value)),
            Delta::Negative(value) => Delta::Negative(op(value)),
        }
    }
}

impl<T: NonNegative> Delta<&T> {
    /// Maps a `Delta<&T>` to a `Delta<T>` by cloning the contents of the
    /// [`Positive`](Delta::Positive) or [`Negative`](Delta::Negative) part.
    #[inline]
    pub fn cloned(self) -> Delta<T>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }

    /// Maps a `Delta<&T>` to a `Delta<T>` by copying the contents of
    /// the [`Positive`](Delta::Positive) or [`Negative`](Delta::Negative) part.
    #[inline]
    pub const fn copied(self) -> Delta<T>
    where
        T: Copy,
    {
        match self {
            Delta::Positive(&v) => Delta::Positive(v),
            Delta::Negative(&v) => Delta::Negative(v),
            Delta::Zero => Delta::Zero,
        }
    }
}

impl<T: NonNegative> Delta<&mut T> {
    /// Maps a `Delta<&mut T>` to a `Delta<T>` by cloning the contents of the
    /// [`Positive`](Delta::Positive) or [`Negative`](Delta::Negative) part.
    #[inline]
    pub fn cloned(self) -> Delta<T>
    where
        T: Clone,
    {
        self.map(|t| t.clone())
    }

    /// Maps a `Delta<&mut T>` to a `Delta<T>` by copying the contents of
    /// the [`Positive`](Delta::Positive) or [`Negative`](Delta::Negative) part.
    #[inline]
    pub const fn copied(self) -> Delta<T>
    where
        T: Copy,
    {
        match self {
            Delta::Positive(&mut v) => Delta::Positive(v),
            Delta::Negative(&mut v) => Delta::Negative(v),
            Delta::Zero => Delta::Zero,
        }
    }
}

impl<T: NonNegative> Delta<Option<T>> {
    /// Transposes a `Delta` of an [`Option`] into an [`Option`] of a `Delta`.
    ///
    /// `Positive(None)` will be mapped to `None`. `Positive(Some(_))` and
    /// `Zero` will be mapped to `Some(Positive(_))` and `Some(Zero)`. Same
    /// holds for `Negative`.
    #[inline]
    pub fn transpose(self) -> Option<Delta<T>> {
        match self {
            Delta::Zero => Some(Delta::Zero),
            Delta::Positive(Some(value)) => Some(Delta::Positive(value)),
            Delta::Positive(None) => None,
            Delta::Negative(Some(value)) => Some(Delta::Negative(value)),
            Delta::Negative(None) => None,
        }
    }
}

impl<T: NonNegative, E> Delta<Result<T, E>> {
    /// Transposes a `Delta` of an [`Result`] into an [`Result`] of a `Delta`.
    ///
    /// `Positive(Err(_))` will be mapped to `Err(_)`. `Positive(Ok(_))` and
    /// `Zero` will be mapped to `Ok(Positive(_))` and `Ok(Zero)`. Same holds
    /// for `Negative`.
    #[inline]
    pub fn transpose(self) -> Result<Delta<T>, E> {
        match self {
            Delta::Zero => Ok(Delta::Zero),
            Delta::Positive(Ok(value)) => Ok(Delta::Positive(value)),
            Delta::Positive(Err(e)) => Err(e),
            Delta::Negative(Ok(value)) => Ok(Delta::Negative(value)),
            Delta::Negative(Err(e)) => Err(e),
        }
    }
}

impl<T: NonNegative> Delta<T>
where
    T: std::ops::Add<Output = T> + SaturatingSub<Output = T>,
{
    /// Applies this `Delta` to `base`, clamping at `base`'s own notion of zero
    /// when subtracting past it.
    #[must_use]
    pub fn apply_to(self, base: T) -> T {
        match self {
            Self::Zero => base,
            Self::Positive(delta) => base + delta,
            Self::Negative(delta) => base.saturating_sub(delta),
        }
    }
}

impl<T: NonNegative> Delta<T>
where
    T: CheckedAdd<Output = T> + SaturatingSub<Output = T>,
{
    /// Applies this `Delta` to `base`, clamping at `base`'s own notion of zero
    /// when subtracting past it. Returns [`None`] if the result of an addition
    /// is not valid.
    #[must_use]
    pub fn checked_apply_to(self, base: T) -> Option<T> {
        match self {
            Self::Zero => Some(base),
            Self::Positive(delta) => base.checked_add(delta),
            Self::Negative(delta) => Some(base.saturating_sub(delta)),
        }
    }
}

impl<T: NonNegative> Delta<T>
where
    T: SaturatingAdd<Output = T> + SaturatingSub<Output = T>,
{
    /// Applies this `Delta` to `base`, saturating on overflows.
    #[must_use]
    pub fn saturating_apply_to(self, base: T) -> T {
        match self {
            Self::Zero => base,
            Self::Positive(delta) => base.saturating_add(delta),
            Self::Negative(delta) => base.saturating_sub(delta),
        }
    }
}

impl<T: NonNegative> std::ops::Neg for Delta<T> {
    type Output = Self;

    /// Negates this `Delta` by reversing its direction.
    fn neg(self) -> Self::Output {
        match self {
            Delta::Zero => Delta::Zero,
            Delta::Positive(v) => Delta::Negative(v),
            Delta::Negative(v) => Delta::Positive(v),
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
            (Self::Zero, other) | (other, Self::Zero) => other,
            (Self::Positive(e1), Self::Positive(e2)) => Self::Positive(e1 + e2),
            (Self::Negative(e1), Self::Negative(e2)) => Self::Negative(e1 + e2),
            (Self::Positive(e1), Self::Negative(e2)) | (Self::Negative(e2), Self::Positive(e1)) => {
                if e1 < e2 {
                    Self::Negative(e2.saturating_sub(e1))
                } else if e1 > e2 {
                    Self::Positive(e1.saturating_sub(e2))
                } else {
                    Self::Zero
                }
            }
        }
    }
}

impl<T: NonNegative> CheckedAdd for Delta<T>
where
    T: Copy + PartialOrd + CheckedAdd<Output = T> + SaturatingSub<Output = T>,
{
    type Output = Self;

    /// Combines two deltas, netting out opposing directions. Returns [`None`]
    /// if the result of an addition is not valid.
    fn checked_add(self, rhs: Self) -> Option<Self::Output> {
        match (self, rhs) {
            (Self::Zero, other) | (other, Self::Zero) => Some(other),
            (Self::Positive(e1), Self::Positive(e2)) => e1.checked_add(e2).map(Self::Positive),
            (Self::Negative(e1), Self::Negative(e2)) => e1.checked_add(e2).map(Self::Negative),
            (Self::Positive(e1), Self::Negative(e2)) | (Self::Negative(e2), Self::Positive(e1)) => {
                if e1 < e2 {
                    Some(Self::Negative(e2.saturating_sub(e1)))
                } else if e1 > e2 {
                    Some(Self::Positive(e1.saturating_sub(e2)))
                } else {
                    Some(Self::Zero)
                }
            }
        }
    }
}

impl<T: NonNegative> SaturatingAdd for Delta<T>
where
    T: Copy + PartialOrd + SaturatingAdd<Output = T> + SaturatingSub<Output = T>,
{
    type Output = Self;

    /// Combines two deltas, netting out opposing directions. Addition of the
    /// same directions saturates inner value.
    fn saturating_add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Zero, other) | (other, Self::Zero) => other,
            (Self::Positive(e1), Self::Positive(e2)) => Self::Positive(e1.saturating_add(e2)),
            (Self::Negative(e1), Self::Negative(e2)) => Self::Negative(e1.saturating_add(e2)),
            (Self::Positive(e1), Self::Negative(e2)) | (Self::Negative(e2), Self::Positive(e1)) => {
                if e1 < e2 {
                    Self::Negative(e2.saturating_sub(e1))
                } else if e1 > e2 {
                    Self::Positive(e1.saturating_sub(e2))
                } else {
                    Self::Zero
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    struct Quantity(i32);

    impl std::ops::Add for Quantity {
        type Output = Self;

        fn add(self, rhs: Self) -> Self::Output {
            Self(self.0 + rhs.0)
        }
    }

    impl NonNegative for Quantity {}

    impl CheckedAdd for Quantity {
        type Output = Self;

        fn checked_add(self, rhs: Self) -> Option<Self::Output> {
            Some(Self(self.0 + rhs.0))
        }
    }

    impl SaturatingAdd for Quantity {
        type Output = Self;

        fn saturating_add(self, rhs: Self) -> Self::Output {
            Self((self.0 + rhs.0).min(100))
        }
    }

    impl SaturatingSub for Quantity {
        type Output = Self;

        fn saturating_sub(self, rhs: Self) -> Self::Output {
            Self((self.0 - rhs.0).max(0))
        }
    }

    #[test]
    fn ord_orders_negatives_before_zero_before_positives() {
        assert!(Delta::Negative(Quantity(2)) < Delta::Zero);
        assert!(Delta::Zero < Delta::Positive(Quantity(2)));
        assert!(Delta::Negative(Quantity(1)) < Delta::Positive(Quantity(1)));
    }

    #[test]
    fn ord_orders_same_direction_by_magnitude() {
        assert!(Delta::Positive(Quantity(2)) < Delta::Positive(Quantity(10)));
        assert!(Delta::Negative(Quantity(10)) < Delta::Negative(Quantity(2)));
        assert_eq!(
            Delta::Positive(Quantity(3)).cmp(&Delta::Positive(Quantity(3))),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn is_zero_detects_zero_variant() {
        assert!(Delta::<Quantity>::Zero.is_zero());
        assert!(!Delta::Positive(Quantity(1)).is_zero());
        assert!(!Delta::Negative(Quantity(1)).is_zero());
    }

    #[test]
    fn is_positive_detects_positive_variant() {
        assert!(!Delta::<Quantity>::Zero.is_positive());
        assert!(Delta::Positive(Quantity(1)).is_positive());
        assert!(!Delta::Negative(Quantity(1)).is_positive());
    }

    #[test]
    fn is_negative_detects_negative_variant() {
        assert!(!Delta::<Quantity>::Zero.is_negative());
        assert!(!Delta::Positive(Quantity(1)).is_negative());
        assert!(Delta::Negative(Quantity(1)).is_negative());
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
    fn between_is_zero_when_lhs_equals_rhs() {
        assert_eq!(Delta::between(Quantity(5), Quantity(5)), Delta::Zero);
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
        assert_eq!(Delta::Zero.map(|q: Quantity| q + Quantity(1)), Delta::Zero);
    }

    #[test]
    fn cloned_clones_shared_reference_contents() {
        let positive = Quantity(3);
        assert_eq!(
            Delta::Positive(&positive).cloned(),
            Delta::Positive(Quantity(3))
        );
        let negative = Quantity(3);
        assert_eq!(
            Delta::Negative(&negative).cloned(),
            Delta::Negative(Quantity(3))
        );
        let zero: Delta<&Quantity> = Delta::Zero;
        assert_eq!(zero.cloned(), Delta::Zero);
    }

    #[test]
    fn copied_copies_shared_reference_contents() {
        let positive = Quantity(3);
        assert_eq!(
            Delta::Positive(&positive).copied(),
            Delta::Positive(Quantity(3))
        );
        let negative = Quantity(3);
        assert_eq!(
            Delta::Negative(&negative).copied(),
            Delta::Negative(Quantity(3))
        );
        let zero: Delta<&Quantity> = Delta::Zero;
        assert_eq!(zero.copied(), Delta::Zero);
    }

    #[test]
    fn cloned_clones_mutable_reference_contents() {
        let mut positive = Quantity(3);
        assert_eq!(
            Delta::Positive(&mut positive).cloned(),
            Delta::Positive(Quantity(3))
        );
        let mut negative = Quantity(3);
        assert_eq!(
            Delta::Negative(&mut negative).cloned(),
            Delta::Negative(Quantity(3))
        );
        let zero: Delta<&mut Quantity> = Delta::Zero;
        assert_eq!(zero.cloned(), Delta::Zero);
    }

    #[test]
    fn copied_copies_mutable_reference_contents() {
        let mut positive = Quantity(3);
        assert_eq!(
            Delta::Positive(&mut positive).copied(),
            Delta::Positive(Quantity(3))
        );
        let mut negative = Quantity(3);
        assert_eq!(
            Delta::Negative(&mut negative).copied(),
            Delta::Negative(Quantity(3))
        );
        let zero: Delta<&mut Quantity> = Delta::Zero;
        assert_eq!(zero.copied(), Delta::Zero);
    }

    #[test]
    fn transpose_option_maps_some_preserving_direction() {
        assert_eq!(
            Delta::Positive(Some(Quantity(1))).transpose(),
            Some(Delta::Positive(Quantity(1)))
        );
        assert_eq!(
            Delta::Negative(Some(Quantity(1))).transpose(),
            Some(Delta::Negative(Quantity(1)))
        );
    }

    #[test]
    fn transpose_option_maps_none_payload_to_none() {
        assert_eq!(Delta::Positive(None::<Quantity>).transpose(), None);
        assert_eq!(Delta::Negative(None::<Quantity>).transpose(), None);
    }

    #[test]
    fn transpose_option_maps_zero_to_some_zero() {
        let none: Delta<Option<Quantity>> = Delta::Zero;
        assert_eq!(none.transpose(), Some(Delta::Zero));
    }

    #[test]
    fn transpose_result_maps_ok_preserving_direction() {
        assert_eq!(
            Delta::Positive(Ok::<_, &str>(Quantity(1))).transpose(),
            Ok(Delta::Positive(Quantity(1)))
        );
        assert_eq!(
            Delta::Negative(Ok::<_, &str>(Quantity(1))).transpose(),
            Ok(Delta::Negative(Quantity(1)))
        );
    }

    #[test]
    fn transpose_result_propagates_err() {
        assert_eq!(
            Delta::Positive(Err::<Quantity, _>("failed")).transpose(),
            Err("failed")
        );
        assert_eq!(
            Delta::Negative(Err::<Quantity, _>("failed")).transpose(),
            Err("failed")
        );
    }

    #[test]
    fn transpose_result_maps_zero_to_ok_zero() {
        let none: Delta<Result<Quantity, &str>> = Delta::Zero;
        assert_eq!(none.transpose(), Ok(Delta::Zero));
    }

    #[test]
    fn applied_zero_leaves_base_unchanged() {
        assert_eq!(Delta::Zero.apply_to(Quantity(5)), Quantity(5));
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
    fn checked_applied_zero_leaves_base_unchanged() {
        assert_eq!(
            Delta::Zero.checked_apply_to(Quantity(5)).unwrap(),
            Quantity(5)
        );
    }

    #[test]
    fn checked_applied_positive_increases_base() {
        assert_eq!(
            Delta::Positive(Quantity(3))
                .checked_apply_to(Quantity(5))
                .unwrap(),
            Quantity(8)
        );
    }

    #[test]
    fn checked_applied_negative_decreases_base_within_bounds() {
        assert_eq!(
            Delta::Negative(Quantity(3))
                .checked_apply_to(Quantity(5))
                .unwrap(),
            Quantity(2)
        );
    }

    #[test]
    fn checked_applied_negative_saturates_at_zero() {
        assert_eq!(
            Delta::Negative(Quantity(10))
                .checked_apply_to(Quantity(5))
                .unwrap(),
            Quantity(0)
        );
    }

    #[test]
    fn saturating_applied_zero_leaves_base_unchanged() {
        assert_eq!(Delta::Zero.saturating_apply_to(Quantity(5)), Quantity(5));
    }

    #[test]
    fn saturating_applied_positive_increases_base() {
        assert_eq!(
            Delta::Positive(Quantity(3)).saturating_apply_to(Quantity(5)),
            Quantity(8)
        );
    }

    #[test]
    fn saturating_applied_negative_decreases_base_within_bounds() {
        assert_eq!(
            Delta::Negative(Quantity(3)).saturating_apply_to(Quantity(5)),
            Quantity(2)
        );
    }

    #[test]
    fn saturating_applied_negative_saturates_at_zero() {
        assert_eq!(
            Delta::Negative(Quantity(10)).saturating_apply_to(Quantity(5)),
            Quantity(0)
        );
    }

    #[test]
    fn saturating_applied_saturates_on_addition_overflow() {
        assert_eq!(
            Delta::Positive(Quantity(100)).saturating_apply_to(Quantity(5)),
            Quantity(100)
        );
    }

    #[test]
    fn neg_reverses_direction() {
        assert_eq!(-Delta::Positive(Quantity(3)), Delta::Negative(Quantity(3)));
        assert_eq!(-Delta::Negative(Quantity(3)), Delta::Positive(Quantity(3)));
        assert_eq!(-Delta::<Quantity>::Zero, Delta::Zero);
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
            Delta::Zero
        );
    }

    #[test]
    fn zero_is_identity_of_addition() {
        assert_eq!(
            Delta::Positive(Quantity(1)) + Delta::Zero,
            Delta::Positive(Quantity(1))
        );
        assert_eq!(
            Delta::Zero + Delta::Negative(Quantity(1)),
            Delta::Negative(Quantity(1))
        );
    }

    #[test]
    fn checked_add_combines_same_direction() {
        assert_eq!(
            Delta::Positive(Quantity(1))
                .checked_add(Delta::Positive(Quantity(2)))
                .unwrap(),
            Delta::Positive(Quantity(3))
        );
        assert_eq!(
            Delta::Negative(Quantity(1))
                .checked_add(Delta::Negative(Quantity(2)))
                .unwrap(),
            Delta::Negative(Quantity(3))
        );
    }

    #[test]
    fn checked_add_nets_out_opposing_directions() {
        assert_eq!(
            Delta::Positive(Quantity(5))
                .checked_add(Delta::Negative(Quantity(2)))
                .unwrap(),
            Delta::Positive(Quantity(3))
        );
        assert_eq!(
            Delta::Positive(Quantity(2))
                .checked_add(Delta::Negative(Quantity(5)))
                .unwrap(),
            Delta::Negative(Quantity(3))
        );
        assert_eq!(
            Delta::Positive(Quantity(2))
                .checked_add(Delta::Negative(Quantity(2)))
                .unwrap(),
            Delta::Zero
        );
    }

    #[test]
    fn zero_is_identity_of_checked_addition() {
        assert_eq!(
            Delta::Positive(Quantity(1))
                .checked_add(Delta::Zero)
                .unwrap(),
            Delta::Positive(Quantity(1))
        );
        assert_eq!(
            Delta::Zero
                .checked_add(Delta::Negative(Quantity(1)))
                .unwrap(),
            Delta::Negative(Quantity(1))
        );
    }
}
