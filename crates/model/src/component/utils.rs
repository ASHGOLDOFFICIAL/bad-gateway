/// Implements [`Add`](std::ops::Add) and [`AddAssign`](std::ops::AddAssign)
/// for `physics::ops::Delta<$unit>` for a newtype wrapper around it.
macro_rules! impl_delta_ops {
    ($($wrapper:ty => $unit:ty),+ $(,)?) => {
        $(
            impl std::ops::Add<physics::ops::Delta<$unit>> for $wrapper {
                type Output = Self;

                #[inline]
                fn add(self, rhs: physics::ops::Delta<$unit>) -> Self::Output {
                    use physics::ops::SaturatingAdd as _;
                    Self::from(physics::ops::Delta::from(self).saturating_add(rhs))
                }
            }

            impl std::ops::AddAssign<physics::ops::Delta<$unit>> for $wrapper {
                #[inline]
                fn add_assign(&mut self, rhs: physics::ops::Delta<$unit>) {
                    *self = *self + rhs;
                }
            }
        )+
    };
}

pub(super) use impl_delta_ops;
