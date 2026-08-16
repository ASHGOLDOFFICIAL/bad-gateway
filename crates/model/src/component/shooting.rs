mod ammo;
mod cooldown;
mod fire_rate;
mod weapon;

pub use ammo::*;
pub use cooldown::*;
pub use fire_rate::*;
pub use weapon::*;

use physics::Angle;

/// Direction this entity looks at.
#[derive(Clone, Copy, Debug, Default, PartialEq, derive_more::From, derive_more::Into)]
pub struct Aim(Angle);
