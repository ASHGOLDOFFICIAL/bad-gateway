pub mod control;
pub mod part;
pub mod stat;

use hecs::{Entity, World};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

use crate::component::mecha::part::Part;

/// Builds a mecha's assembly rooted at `root`, following every [`Part`]'s
/// `mounted_on` relationship.
pub fn build_mecha(world: &World, root: Entity) -> Mecha {
    let mut children_map: HashMap<Entity, Vec<Entity>> = HashMap::new();

    for (entity, part) in world.query::<(Entity, &Part)>().iter() {
        if let Some(parent) = part.mounted_on() {
            children_map.entry(parent).or_default().push(entity);
        }
    }

    Mecha::from_children(root, &children_map)
}

/// Marks entity as mecha.
#[derive(Clone, Debug, PartialEq)]
pub struct Mecha {
    part_tree: PartTree,
    children: HashSet<Entity>,
}

impl Mecha {
    fn from_children(root: Entity, children_map: &HashMap<Entity, Vec<Entity>>) -> Self {
        let part_tree = build_tree(root, children_map);
        let children = part_tree.iter_bfs().collect();
        Self {
            part_tree,
            children,
        }
    }

    /// Checks if given [`Entity`] is this `Mecha`'s root or one of the parts
    /// mounted on it, transitively.
    pub fn has_part(&self, entity: &Entity) -> bool {
        self.children.contains(entity)
    }

    /// Every [`Entity`] belonging to this `Mecha`, in parent-before-child
    /// order.
    pub fn parts(&self) -> impl Iterator<Item = Entity> + '_ {
        self.part_tree.iter_bfs()
    }

    /// Every [`Entity`] belonging to this `Mecha` paired with the [`Entity`]
    /// it's mounted on ([`None`] for the root), in parent-before-child order.
    pub fn parts_with_parent(&self) -> impl Iterator<Item = (Entity, Option<Entity>)> + '_ {
        PartsWithParentIter {
            queue: VecDeque::from([(&self.part_tree, None)]),
        }
    }
}

/// Breadth-first iterator over a [`PartTree`],
/// yielding each node's entity and its parent if present.
struct PartsWithParentIter<'a> {
    queue: VecDeque<(&'a PartTree, Option<Entity>)>,
}

impl<'a> Iterator for PartsWithParentIter<'a> {
    type Item = (Entity, Option<Entity>);

    fn next(&mut self) -> Option<Self::Item> {
        let (node, parent) = self.queue.pop_front()?;
        self.queue
            .extend(node.children.iter().map(|child| (child, Some(node.root))));
        Some((node.root, parent))
    }
}

fn build_tree(root: Entity, children_map: &HashMap<Entity, Vec<Entity>>) -> PartTree {
    let mut stack = vec![(root, false)];
    let mut built: HashMap<Entity, PartTree> = HashMap::new();

    while let Some((entity, visited)) = stack.pop() {
        if visited {
            let children = children_map
                .get(&entity)
                .map(|children| {
                    children
                        .iter()
                        .map(|child| built.remove(child).unwrap())
                        .collect()
                })
                .unwrap_or_default();

            built.insert(entity, PartTree::new(entity, children));
        } else {
            stack.push((entity, true));

            if let Some(children) = children_map.get(&entity) {
                for &child in children.iter().rev() {
                    stack.push((child, false));
                }
            }
        }
    }

    built.remove(&root).unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PartTree {
    root: Entity,
    children: Arc<[PartTree]>,
}

impl PartTree {
    fn new(root: Entity, children: Vec<PartTree>) -> Self {
        Self {
            root,
            children: children.into(),
        }
    }

    fn iter_bfs(&self) -> PartTreeBfsIter<'_> {
        PartTreeBfsIter {
            queue: VecDeque::from([self]),
        }
    }
}

/// Breadth-first iterator over a [`PartTree`], yielding each node's entity.
struct PartTreeBfsIter<'a> {
    queue: VecDeque<&'a PartTree>,
}

impl<'a> Iterator for PartTreeBfsIter<'a> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.queue.pop_front()?;
        self.queue.extend(node.children.iter());
        Some(node.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::HashSet;

    fn spawn_part(world: &mut World, mounted_on: Entity) -> Entity {
        world.spawn((Part::new("part".to_string(), Some(mounted_on)).unwrap(),))
    }

    #[test]
    fn build_mecha_includes_root_and_every_mounted_part() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root);
        let weapon = spawn_part(&mut world, frame);
        let unrelated = world.spawn(());

        let mecha = build_mecha(&world, root);

        assert!(mecha.has_part(&root));
        assert!(mecha.has_part(&frame));
        assert!(mecha.has_part(&weapon));
        assert!(!mecha.has_part(&unrelated));

        let parts: HashSet<_> = mecha.parts().collect();
        assert_eq!(parts, HashSet::from([root, frame, weapon]));
    }

    #[test]
    fn parts_with_parent_reports_mount_relationships() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root);
        let weapon = spawn_part(&mut world, frame);

        let mecha = build_mecha(&world, root);
        let parents: HashMap<_, _> = mecha.parts_with_parent().collect();

        assert_eq!(parents[&root], None);
        assert_eq!(parents[&frame], Some(root));
        assert_eq!(parents[&weapon], Some(frame));
    }

    #[test]
    fn parts_with_parent_visits_parents_before_children() {
        let mut world = World::new();
        let root = world.spawn(());
        let frame = spawn_part(&mut world, root);
        let weapon = spawn_part(&mut world, frame);

        let mecha = build_mecha(&world, root);
        let order: Vec<_> = mecha
            .parts_with_parent()
            .map(|(entity, _)| entity)
            .collect();

        let root_pos = order.iter().position(|&e| e == root).unwrap();
        let frame_pos = order.iter().position(|&e| e == frame).unwrap();
        let weapon_pos = order.iter().position(|&e| e == weapon).unwrap();

        assert!(root_pos < frame_pos);
        assert!(frame_pos < weapon_pos);
    }
}
