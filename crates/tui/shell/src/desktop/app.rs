use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use i18n_embed_fl::fl;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

use crate::{
    App, AppSignal, Entry, MenuItem,
    desktop::{
        ICON_COLS, ICON_ROWS,
        grid::{Grid, GridConfig},
    },
    i18n::LOADER,
};

const GRID_CONFIG: GridConfig = GridConfig {
    cell_width: ICON_COLS as u16 + 2,
    cell_height: ICON_ROWS as u16 + 2,
    inset_x: 2,
    inset_y: 1,
};

#[inline(always)]
fn grid(area: Rect) -> Grid {
    Grid::new(GRID_CONFIG, area)
}

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Desktop with launchable icons.
#[must_use]
#[derive(Default)]
pub(crate) struct Desktop {
    entries: Vec<Entry>,
    selected: Option<usize>,
}

impl Desktop {
    /// Adds new [`Entry`] to this `Desktop`.
    #[inline(always)]
    pub(crate) fn add_entry(&mut self, program: Entry) {
        self.entries.push(program);
    }

    /// Moves the selection by one step in `direction` through the grid.
    fn move_selection(&mut self, direction: Direction, area: Rect) {
        let items = self.entries.len();
        if items == 0 {
            self.selected = None;
            return;
        }

        let grid = grid(area);
        let selected = self.selected.unwrap_or(0);
        let candidate = match direction {
            Direction::Left => grid.left(selected),
            Direction::Right => grid.right(selected),
            Direction::Up => grid.above(selected),
            Direction::Down => grid.below(selected),
        };

        self.selected = if candidate < items {
            Some(candidate)
        } else {
            Some(items - 1)
        }
    }

    /// Launches selected app if any.
    fn open_selected(&mut self) -> AppSignal {
        match self.selected.and_then(|index| self.entries.get(index)) {
            Some(program) => AppSignal::Open(program.launch()),
            None => AppSignal::Continue,
        }
    }
}

impl App for Desktop {
    #[inline(always)]
    fn name(&self) -> String {
        fl!(LOADER, "menu-desktop-label")
    }

    #[inline(always)]
    fn menu_actions(&self) -> Vec<MenuItem<&'static str>> {
        vec![]
    }

    #[inline(always)]
    fn handle_menu_action(&mut self, _action: &str) -> AppSignal {
        AppSignal::Continue
    }

    fn handle_event(&mut self, event: &Event, area: Rect) -> AppSignal {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Esc => {
                    self.selected = None;
                    AppSignal::Continue
                }
                KeyCode::Up => {
                    self.move_selection(Direction::Up, area);
                    AppSignal::Continue
                }
                KeyCode::Down => {
                    self.move_selection(Direction::Down, area);
                    AppSignal::Continue
                }
                KeyCode::Left => {
                    self.move_selection(Direction::Left, area);
                    AppSignal::Continue
                }
                KeyCode::Right => {
                    self.move_selection(Direction::Right, area);
                    AppSignal::Continue
                }
                KeyCode::Enter => self.open_selected(),
                _ => AppSignal::Continue,
            },
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                match grid(area).hit_test(self.entries.len(), mouse.column, mouse.row) {
                    Some(index) => {
                        self.selected = Some(index);
                        self.open_selected()
                    }
                    None => AppSignal::Continue,
                }
            }
            _ => AppSignal::Continue,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        frame
            .buffer_mut()
            .set_style(area, Style::default().bg(Color::White));

        for (index, rect) in grid(area).iter(self.entries.len()).enumerate() {
            draw_icon(
                frame,
                rect,
                &self.entries[index],
                Some(index) == self.selected,
            );
        }
    }
}

/// Draws icon inside given `area`.
fn draw_icon(frame: &mut Frame, area: Rect, entry: &Entry, selected: bool) {
    let icon = entry.icon();
    let buffer = frame.buffer_mut();

    if selected {
        buffer.set_style(area, Style::default().bg(Color::LightBlue));
    }

    for row in 0..ICON_ROWS {
        let y = area.y + 1 + row as u16;

        if y >= area.bottom() {
            break;
        }

        for col in 0..ICON_COLS {
            let x = area.x + 1 + col as u16;
            if x >= area.right() {
                break;
            }

            let style = Style::default().fg(icon.foreground[row][col]);

            if let Some(cell) = buffer.cell_mut((x, y)) {
                cell.set_char(icon.glyphs[row][col]).set_style(style);
            }
        }
    }

    let name_area =
        Rect::new(area.x, area.y + 1 + ICON_ROWS as u16, area.width, 1).intersection(area);
    if let Some(name) = truncate_name(entry.name(), name_area.width as usize) {
        frame.render_widget(
            Paragraph::new(name)
                .style(Style::default().fg(Color::Black))
                .alignment(Alignment::Center),
            name_area,
        );
    }
}

/// Shortens `name` to fit within `max_width` columns,
/// replacing the tail with `…` when it doesn't fit. Returns
/// [`None`] when `max_width` is less than or equal to 1.
fn truncate_name(name: &str, max_width: usize) -> Option<String> {
    let len = name.chars().count();
    if len <= max_width {
        return Some(name.to_string());
    }
    if max_width <= 1 {
        return None;
    }

    let mut truncated: String = name.chars().take(max_width - 1).collect();
    truncated.push('…');
    Some(truncated)
}
