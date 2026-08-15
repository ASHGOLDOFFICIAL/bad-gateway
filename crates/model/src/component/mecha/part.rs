//! Contains [`Part`] and its types.

mod battery;
mod brake;
mod engine;
mod thruster;

pub use battery::*;
pub use brake::*;
pub use engine::*;
pub use thruster::*;

use hecs::Entity;

use crate::component::ComponentResult;

/// Mecha's part. It has a name and the [`Entity`] it's mounted on.
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    name: String,
    mounted_on: Entity,
}

impl Part {
    /// Makes new `Part` from given values.
    ///
    /// `name` must be non-empty.
    #[inline]
    pub fn new(name: String, mounted_on: Entity) -> ComponentResult<Self> {
        if name.is_empty() {
            Err("name must be non-empty")
        } else {
            Ok(Self { name, mounted_on })
        }
    }

    /// Returns this `Part`'s name.
    #[inline(always)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns [`Entity`] this `Part` is mounted on.
    #[inline(always)]
    pub fn mounted_on(&self) -> Entity {
        self.mounted_on
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity() -> Entity {
        hecs::World::new().spawn(())
    }

    #[test]
    fn new_rejects_empty_name() {
        assert!(Part::new(String::new(), entity()).is_err());
    }

    #[test]
    fn new_accepts_non_empty_name() {
        let part = Part::new("leg".to_string(), entity()).unwrap();
        assert_eq!(part.name(), "leg");
    }
}
