use crate::ArtCell;

/// A colored ASCII rendering of an image, sized
/// to exactly fill a `width` x `height` cell grid.
#[must_use]
pub struct AsciiArt {
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) cells: Vec<ArtCell>,
}

impl AsciiArt {
    /// An `AsciiArt` without any cells.
    pub const EMPTY: Self = Self {
        width: 0,
        height: 0,
        cells: vec![],
    };

    /// This art's current `(width, height)`, in cells.
    #[inline(always)]
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// The glyph and RGB color at `(col, row)`, if within bounds.
    pub fn get(&self, col: u16, row: u16) -> Option<ArtCell> {
        if col >= self.width || row >= self.height {
            return None;
        }
        Some(self.cells[row as usize * self.width as usize + col as usize])
    }
}
