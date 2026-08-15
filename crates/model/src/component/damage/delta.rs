use physics::ops::Delta;

use crate::component::{DamageUnit, utils::impl_delta_ops};

/// Pending damage adjustment to be applied to
/// [`Integrity`](crate::component::Integrity).
#[must_use]
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::From, derive_more::Into)]
pub struct DamageDelta(Delta<DamageUnit>);

impl_delta_ops!(DamageDelta => DamageUnit);
