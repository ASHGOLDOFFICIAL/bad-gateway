//! Contains [`Part`] and its types.

mod battery;
mod generator;
mod legs;
mod thruster;

pub use battery::*;
pub use generator::*;
pub use legs::*;
pub use thruster::*;

use hecs::Entity;

use crate::component::ComponentResult;

/// Mecha's part. It has a name and the [`Entity`] it's mounted on, if any.
/// [`None`] marks the mecha's root, which has no parent of its own.
#[must_use]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    name: String,
    mounted_on: Option<Entity>,
}

impl Part {
    /// Makes new `Part` from given values.
    ///
    /// `name` must be non-empty.
    #[inline]
    pub fn new(name: String, mounted_on: Option<Entity>) -> ComponentResult<Self> {
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

    /// Returns the [`Entity`] this `Part` is mounted on, or [`None`] if it's
    /// the mecha's root.
    #[inline(always)]
    pub fn mounted_on(&self) -> Option<Entity> {
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
        assert!(Part::new(String::new(), Some(entity())).is_err());
    }

    #[test]
    fn new_accepts_non_empty_name() {
        let part = Part::new("leg".to_string(), Some(entity())).unwrap();
        assert_eq!(part.name(), "leg");
    }

    #[test]
    fn new_accepts_no_mounted_on_for_a_root_part() {
        let part = Part::new("frame".to_string(), None).unwrap();
        assert_eq!(part.mounted_on(), None);
    }
}
