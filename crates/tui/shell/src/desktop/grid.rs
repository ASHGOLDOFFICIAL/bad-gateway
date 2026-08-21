use ratatui::layout::{Position, Rect};

/// Config for the [`Grid`].
#[must_use]
pub(super) struct GridConfig {
    pub(super) cell_width: u16,
    pub(super) cell_height: u16,
    pub(super) inset_x: u16,
    pub(super) inset_y: u16,
}

/// Wraps raw usable area into grid.
#[must_use]
pub(super) struct Grid {
    config: GridConfig,
    area: Rect,
    width: usize,
}

impl Grid {
    /// Makes new `Grid` from the given area.
    #[inline(always)]
    pub(super) fn new(config: GridConfig, area: Rect) -> Self {
        let width = {
            let usable = area.width.saturating_sub(config.inset_x);
            ((usable / (config.cell_width + config.inset_x)).max(1)) as usize
        };
        Self {
            config,
            area,
            width,
        }
    }

    /// Spits rects for each of `len` items in this grid.
    pub(super) fn iter(&self, len: usize) -> impl Iterator<Item = Rect> + '_ {
        let width = self.width;
        let area = self.area;

        (0..len).map(move |index| {
            let col = (index % width) as u16;
            let row = (index / width) as u16;
            let x =
                area.x + self.config.inset_x + col * (self.config.cell_width + self.config.inset_x);
            let y = area.y + self.config.inset_y + row * self.config.cell_height;
            Rect::new(x, y, self.config.cell_width, self.config.cell_height).intersection(area)
        })
    }

    /// The index (if any) of the item whose cell is at screen position.
    pub(super) fn hit_test(&self, len: usize, column: u16, row: u16) -> Option<usize> {
        let point = Position::from((column, row));
        self.iter(len).position(|rect| rect.contains(point))
    }

    /// Index of the item one column to the left of `index`'s,
    /// clamped at the first column of its row.
    pub(super) const fn left(&self, index: usize) -> usize {
        let width = self.width;
        let (row, col) = self.position(index);
        row * width + col.saturating_sub(1)
    }

    /// Index of the item one column to the right of `index`'s,
    /// clamped at the last column of its row.
    pub(super) fn right(&self, index: usize) -> usize {
        let width = self.width;
        let (row, col) = self.position(index);
        row * width + (col + 1).min(width - 1)
    }

    /// Index of the item one row above `index`'s, clamped at the first row.
    pub(super) const fn above(&self, index: usize) -> usize {
        let width = self.width;
        let (row, col) = self.position(index);
        row.saturating_sub(1) * width + col
    }

    /// Index of the item one row below `index`'s.
    pub(super) const fn below(&self, index: usize) -> usize {
        let width = self.width;
        let (row, col) = self.position(index);
        (row + 1) * width + col
    }

    /// Row/column of `index` in this `Grid`.
    #[inline(always)]
    const fn position(&self, index: usize) -> (usize, usize) {
        (index / self.width, index % self.width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: GridConfig = GridConfig {
        cell_width: 8,
        cell_height: 4,
        inset_x: 2,
        inset_y: 1,
    };

    fn grid_with_columns(columns: u16) -> Grid {
        let width = columns * (CONFIG.cell_width + CONFIG.inset_x) + CONFIG.inset_x;
        Grid::new(CONFIG, Rect::new(0, 0, width, 24))
    }

    #[test]
    fn left_and_right_clamp_within_a_row() {
        let grid = grid_with_columns(2);
        assert_eq!(grid.left(1), 0);
        assert_eq!(grid.left(0), 0);
        assert_eq!(grid.right(0), 1);
        assert_eq!(grid.right(1), 1);
    }

    #[test]
    fn above_and_below_move_between_rows() {
        let grid = grid_with_columns(2);
        assert_eq!(grid.below(0), 2);
        assert_eq!(grid.above(2), 0);
        assert_eq!(grid.above(0), 0);
    }
}
