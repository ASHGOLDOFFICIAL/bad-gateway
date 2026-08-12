mod add;
mod div;
mod mul;
mod sub;

pub use add::*;
pub use div::*;
pub use mul::*;
pub use sub::*;

/// Implements checked arithmetic traits for the given types by delegating to
/// their inherent `checked_*` methods.
macro_rules! impl_checked {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl CheckedAdd for $ty {
                type Output = Self;

                #[inline(always)]
                fn checked_add(self, rhs: Self) -> Option<Self::Output> {
                    <$ty>::checked_add(self, rhs)
                }
            }

            impl CheckedSub for $ty {
                type Output = Self;

                #[inline(always)]
                fn checked_sub(self, rhs: Self) -> Option<Self::Output> {
                    <$ty>::checked_sub(self, rhs)
                }
            }

            impl CheckedMul for $ty {
                type Output = Self;

                #[inline(always)]
                fn checked_mul(self, rhs: Self) -> Option<Self::Output> {
                    <$ty>::checked_mul(self, rhs)
                }
            }

            impl CheckedDiv for $ty {
                type Output = Self;

                #[inline(always)]
                fn checked_div(self, rhs: Self) -> Option<Self::Output> {
                    <$ty>::checked_div(self, rhs)
                }
            }
        )+
    };
}

impl_checked!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
