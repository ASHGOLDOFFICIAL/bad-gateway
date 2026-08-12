mod add;
mod sub;

pub use add::*;
pub use sub::*;

/// Implements saturating arithmetic for the given types by delegating to their
/// inherent `saturating_*` methods.
macro_rules! impl_saturating {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl SaturatingAdd for $ty {
                type Output = Self;

                #[inline]
                fn saturating_add(self, rhs: Self) -> Self::Output {
                    <$ty>::saturating_add(self, rhs)
                }
            }

            impl SaturatingSub for $ty {
                type Output = Self;

                #[inline]
                fn saturating_sub(self, rhs: Self) -> Self::Output {
                    <$ty>::saturating_sub(self, rhs)
                }
            }
        )+
    };
}

impl_saturating!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);
