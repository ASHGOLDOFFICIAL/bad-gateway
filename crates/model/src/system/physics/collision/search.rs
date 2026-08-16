use crate::{map::Map, system::physics::collision::Aabb};

/// Every solid map [`Cell`](crate::map::Cell) a `dynamic` [`Aabb`] could reach
/// while moving by `displacement`, yields 1x1 cell [`Aabb`] per cell with solid
/// structure.
pub fn sweep_solid_cells(
    dynamic: Aabb,
    displacement: glam::Vec2,
    map: &dyn Map,
) -> impl Iterator<Item = Aabb> + '_ {
    let min = dynamic
        .position
        .min(dynamic.position + displacement)
        .floor();
    let max = dynamic.max().max(dynamic.max() + displacement).floor();

    let min_x = min.x as i32;
    let min_y = min.y as i32;
    let max_x = max.x as i32;
    let max_y = max.y as i32;

    (min_y..=max_y)
        .flat_map(move |cy| (min_x..=max_x).map(move |cx| (cx, cy)))
        .filter_map(move |(cx, cy)| is_solid(cx, cy, map))
}

/// Every solid map [`Cell`](crate::map::Cell) a moving `point` could reach
/// while moving by `displacement`.
pub fn sweep_solid_cells_point(
    point: glam::Vec2,
    displacement: glam::Vec2,
    map: &dyn Map,
) -> impl Iterator<Item = Aabb> + '_ {
    cells_on_path(point, displacement)
        .into_iter()
        .filter_map(move |(cx, cy)| is_solid(cx, cy, map))
}

fn is_solid(cx: i32, cy: i32, map: &dyn Map) -> Option<Aabb> {
    let is_solid = map
        .cell_at(cx as f32, cy as f32)
        .structure
        .is_some_and(|structure| structure.collides());

    is_solid.then(|| Aabb::new(glam::Vec2::new(cx as f32, cy as f32), glam::Vec2::ONE))
}

/// Grid cells a point visits while moving in a straight line by
/// `displacement`, in path order.
fn cells_on_path(point: glam::Vec2, displacement: glam::Vec2) -> Vec<(i32, i32)> {
    let mut cell = point.floor().as_ivec2();

    let step = displacement.signum();
    let step_i = step.as_ivec2();
    let t_delta = (glam::Vec2::ONE / displacement).abs();

    let next_boundary = cell.as_vec2() + step.max(glam::Vec2::ZERO);
    let mut t_max = (next_boundary - point) / displacement;

    let mut cells = vec![(cell.x, cell.y)];

    while t_max.x < 1.0 || t_max.y < 1.0 {
        if t_max.x < t_max.y {
            cell.x += step_i.x;
            t_max.x += t_delta.x;
        } else if t_max.y < t_max.x {
            cell.y += step_i.y;
            t_max.y += t_delta.y;
        } else {
            cells.push((cell.x + step_i.x, cell.y));
            cells.push((cell.x, cell.y + step_i.y));
            cell += step_i;
            t_max += t_delta;
        }
        cells.push((cell.x, cell.y));
    }

    cells
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use physics::{Density, Length, Temperature};

    use super::*;
    use crate::map::{Cell, Structure, Surface};

    struct TestMap {
        solid: HashSet<(i32, i32)>,
    }

    impl Map for TestMap {
        fn cell_at(&self, x: f32, y: f32) -> Cell {
            let structure = self
                .solid
                .contains(&(x.floor() as i32, y.floor() as i32))
                .then_some(Structure::Tree);

            Cell {
                elevation: Length::ZERO,
                surface: Surface::Grass,
                structure,
                liquid: None,
            }
        }

        fn temperature(&self) -> Temperature {
            Temperature::from_celsius_f32(20.0)
        }

        fn air_density(&self) -> Density {
            Density::from_kilograms_per_cubic_meter_f32(1.2255)
        }

        fn gravity(&self) -> f32 {
            9.8
        }
    }

    #[test]
    fn sweep_solid_cells_finds_solid_cell_in_swept_region() {
        let map = TestMap {
            solid: [(3, 1)].into(),
        };
        let dynamic = Aabb::new(glam::Vec2::new(0.0, 1.0), glam::Vec2::new(1.0, 1.0));

        let hits: Vec<Aabb> = sweep_solid_cells(dynamic, glam::Vec2::new(3.0, 0.0), &map).collect();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].position, glam::Vec2::new(3.0, 1.0));
    }

    #[test]
    fn sweep_solid_cells_point_finds_solid_cell_on_path() {
        let map = TestMap {
            solid: [(2, 0)].into(),
        };

        let hits: Vec<Aabb> =
            sweep_solid_cells_point(glam::Vec2::new(0.5, 0.5), glam::Vec2::new(3.0, 0.0), &map)
                .collect();

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].position, glam::Vec2::new(2.0, 0.0));
    }

    #[test]
    fn sweep_solid_cells_point_finds_nothing_when_path_is_clear() {
        let map = TestMap {
            solid: HashSet::new(),
        };

        let hits: Vec<Aabb> =
            sweep_solid_cells_point(glam::Vec2::new(0.5, 0.5), glam::Vec2::new(3.0, 0.0), &map)
                .collect();

        assert!(hits.is_empty());
    }

    #[test]
    fn cells_on_path_horizontal_line_visits_every_crossed_cell() {
        let visited = cells_on_path(glam::Vec2::new(0.5, 0.5), glam::Vec2::new(3.0, 0.0));
        let expected: HashSet<(i32, i32)> = [(0, 0), (1, 0), (2, 0), (3, 0)].into();

        assert_eq!(visited.into_iter().collect::<HashSet<_>>(), expected);
    }

    #[test]
    fn cells_on_path_diagonal_line_includes_corner_adjacent_cells() {
        let visited = cells_on_path(glam::Vec2::new(0.5, 0.5), glam::Vec2::new(2.0, 2.0));
        let expected: HashSet<(i32, i32)> =
            [(0, 0), (1, 0), (0, 1), (1, 1), (2, 1), (1, 2), (2, 2)].into();

        assert_eq!(visited.into_iter().collect::<HashSet<_>>(), expected);
    }
}
