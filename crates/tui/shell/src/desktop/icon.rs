use ratatui::style::Color;

pub const ICON_ROWS: usize = 5;
pub const ICON_COLS: usize = 9;

/// A desktop icon.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Icon {
    pub glyphs: [[char; ICON_COLS]; ICON_ROWS],
    pub foreground: [[Color; ICON_COLS]; ICON_ROWS],
}

impl Icon {
    /// Builds an icon from a `glyphs` grid in one uniform foreground color.
    #[inline(always)]
    pub const fn solid(glyphs: [[char; ICON_COLS]; ICON_ROWS], foreground: Color) -> Self {
        Self {
            glyphs,
            foreground: [[foreground; ICON_COLS]; ICON_ROWS],
        }
    }
}
