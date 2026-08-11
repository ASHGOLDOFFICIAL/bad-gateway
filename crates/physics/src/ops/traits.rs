use crate::ops::{CheckedAdd, CheckedMul, CheckedSub, SaturatingAdd, SaturatingSub};

/// A quantity backed by a validated representation.
pub(crate) trait Validated: Copy {
    type Repr: Copy;

    fn as_repr(&self) -> Self::Repr;
    fn validate(value: Self::Repr) -> Option<Self>;
}

/// A quantity with a largest representable value.
pub(crate) trait UpperBounded: Copy {
    const MAX: Self;
}

/// A quantity with a smallest representable value.
pub(crate) trait LowerBounded: Copy {
    const MIN: Self;
}

/// Marks a quantity that can never go negative.
///
/// Allows us to assume that:
/// - result of addition is never lesser than its first operand, and that means
///   overflow can only happen over upper bound,
/// - result of subtraction is never greater than its first operand, and that
///   means underflow can only happen over lower bound.
pub trait NonNegative {}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Quantity(f32);

    impl Validated for Quantity {
        type Repr = f32;

        fn as_repr(&self) -> f32 {
            self.0
        }

        fn validate(value: f32) -> Option<Self> {
            (value.is_finite() && (0.0..=100.0).contains(&value)).then_some(Self(value))
        }
    }

    impl UpperBounded for Quantity {
        const MAX: Self = Self(100.0);
    }

    impl LowerBounded for Quantity {
        const MIN: Self = Self(0.0);
    }

    impl NonNegative for Quantity {}

    #[test]
    fn checked_add_rejects_invalid_result() {
        assert!(Quantity(f32::MAX).checked_add(Quantity(f32::MAX)).is_none());
    }

    #[test]
    fn checked_add_sums_valid_result() {
        assert_eq!(
            Quantity(1.0).checked_add(Quantity(2.0)),
            Some(Quantity(3.0))
        );
    }

    #[test]
    fn checked_sub_rejects_invalid_result() {
        assert!(Quantity(1.0).checked_sub(Quantity(2.0)).is_none());
    }

    #[test]
    fn checked_sub_accepts_valid_result() {
        assert_eq!(
            Quantity(5.0).checked_sub(Quantity(2.0)),
            Some(Quantity(3.0))
        );
    }

    #[test]
    fn checked_mul_f32_rejects_invalid_result() {
        assert!(Quantity(10.0).checked_mul(-1.0).is_none());
    }

    #[test]
    fn checked_mul_f32_scales_by_coefficient() {
        assert_eq!(Quantity(10.0).checked_mul(2.0), Some(Quantity(20.0)));
    }

    #[test]
    fn saturating_add_sums_valid_result() {
        assert_eq!(Quantity(1.0).saturating_add(Quantity(2.0)), Quantity(3.0));
    }

    #[test]
    fn saturating_add_clamps_at_max() {
        assert_eq!(Quantity::MAX.saturating_add(Quantity(1.0)), Quantity::MAX);
    }

    #[test]
    fn saturating_sub_clamps_at_min() {
        assert_eq!(Quantity(1.0).saturating_sub(Quantity(2.0)), Quantity::MIN);
    }
}
