use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::topbar::Menu;

/// What handling an event against the bar resulted in.
#[must_use]
pub(crate) enum TopBarOutcome<Action> {
    /// An item was chosen; the shell should carry out `Action`.
    Action(Action),

    /// The bar consumed the event itself, must not be propagated.
    Consumed,

    /// The event wasn't aimed at the bar at all.
    NotConsumed,
}

/// The persistent top menu bar.
#[must_use]
pub(crate) struct TopBar<Action> {
    menus: Vec<Menu<Action>>,
    open: Option<usize>,
    hovered_label: Option<usize>,
    highlighted: Option<usize>,
}

impl<Action> Default for TopBar<Action> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            menus: Vec::new(),
            open: None,
            hovered_label: None,
            highlighted: None,
        }
    }
}

impl<Action: Clone> TopBar<Action> {
    /// Splits `area` into the 1-row bar strip
    /// and the remaining content area below it.
    #[inline]
    pub fn split(area: Rect) -> (Rect, Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);
        (chunks[0], chunks[1])
    }

    /// Replaces this frame's menu contents.
    #[inline(always)]
    pub fn set_menus(&mut self, menus: Vec<Menu<Action>>) {
        self.menus = menus;
    }

    /// Closes any open dropdown and clears all highlight/hover state.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.close();
        self.hovered_label = None;
    }

    /// Draws this `TopBar` inside the given area.
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let rects = self.menu_rects(area);

        for (index, (menu, rect)) in self.menus.iter().zip(rects).enumerate() {
            let style = if self.open == Some(index) || self.hovered_label == Some(index) {
                Style::default().add_modifier(Modifier::REVERSED)
            } else {
                Style::default()
            };
            let label = format!("  {}  ", menu.label);
            frame.render_widget(Paragraph::new(label).style(style), rect);
        }

        if let Some(index) = self.open {
            self.draw_dropdown(frame, area, index);
        }
    }

    /// Resolves a raw input event against the bar.
    pub fn handle_event(&mut self, event: &Event, bar_area: Rect) -> TopBarOutcome<Action> {
        match event {
            Event::Mouse(mouse)
                if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) =>
            {
                self.handle_click(bar_area, mouse.column, mouse.row)
            }
            Event::Mouse(mouse) if matches!(mouse.kind, MouseEventKind::Moved) => {
                self.handle_hover(bar_area, mouse.column, mouse.row)
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key.code),
            _ => TopBarOutcome::NotConsumed,
        }
    }

    fn handle_click(&mut self, area: Rect, column: u16, row: u16) -> TopBarOutcome<Action> {
        if self.open.is_some()
            && let Some(action) = self.hit_test_dropdown(area, column, row)
        {
            self.close();
            return TopBarOutcome::Action(action);
        }

        if let Some(index) = self.hit_test_bar(area, column, row) {
            if self.open == Some(index) {
                self.close();
            } else {
                self.open(index);
            }
            return TopBarOutcome::Consumed;
        }

        if self.open.is_some() {
            self.close();
            return TopBarOutcome::Consumed;
        }
        TopBarOutcome::NotConsumed
    }

    fn handle_hover(&mut self, area: Rect, column: u16, row: u16) -> TopBarOutcome<Action> {
        if self.open.is_some() {
            if let Some(item_index) = self.hovered_item(area, column, row) {
                self.highlighted = Some(item_index);
                return TopBarOutcome::Consumed;
            }
            self.highlighted = None;
        }

        if let Some(label_index) = self.hit_test_bar(area, column, row) {
            self.hovered_label = Some(label_index);
            return TopBarOutcome::Consumed;
        }

        self.hovered_label = None;
        TopBarOutcome::NotConsumed
    }

    fn handle_key(&mut self, code: KeyCode) -> TopBarOutcome<Action> {
        let Some(index) = self.open else {
            return TopBarOutcome::NotConsumed;
        };

        match code {
            KeyCode::Up => self.move_highlight(-1, self.item_count(index)),
            KeyCode::Down => self.move_highlight(1, self.item_count(index)),
            KeyCode::Enter => {
                let action = self
                    .highlighted
                    .and_then(|item_index| self.action_at(index, item_index));
                self.close();
                if let Some(action) = action {
                    return TopBarOutcome::Action(action);
                }
            }
            KeyCode::Esc => self.close(),
            _ => {}
        }
        TopBarOutcome::Consumed
    }

    fn draw_dropdown(&self, frame: &mut Frame, bar: Rect, index: usize) {
        let Some(menu) = self.menus.get(index) else {
            return;
        };
        let Some(popup) = self.dropdown_rect(bar, index) else {
            return;
        };

        frame.render_widget(Clear, popup);
        let block = Block::default().borders(Borders::ALL);
        let items: Vec<ListItem> = menu
            .items
            .iter()
            .enumerate()
            .map(|(item_index, item)| {
                let style = if Some(item_index) == self.highlighted {
                    Style::default().add_modifier(Modifier::BOLD | Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(format!(" {} ", item.label)).style(style)
            })
            .collect();
        frame.render_widget(List::new(items).block(block), popup);
    }

    #[inline(always)]
    fn open(&mut self, index: usize) {
        self.open = Some(index);
        self.highlighted = None;
    }

    #[inline(always)]
    fn close(&mut self) {
        self.open = None;
        self.highlighted = None;
    }

    /// Moves the highlight by `delta` through `len` items, wrapping.
    fn move_highlight(&mut self, delta: isize, len: usize) {
        if len == 0 {
            self.highlighted = None;
            return;
        }

        self.highlighted = Some(match self.highlighted {
            Some(index) => (index as isize + delta).rem_euclid(len as isize) as usize,
            None if delta > 0 => 0,
            None => len - 1,
        });
    }

    #[inline]
    fn item_count(&self, menu_index: usize) -> usize {
        self.menus
            .get(menu_index)
            .map_or(0, |menu| menu.items.len())
    }

    /// Returns the action triggered by the item `item_index`
    /// within menu `menu_index`. Returns [`None`] if no action
    /// can be found for the given values.
    #[inline]
    fn action_at(&self, menu_index: usize, item_index: usize) -> Option<Action> {
        self.menus
            .get(menu_index)?
            .items
            .get(item_index)
            .map(|item| item.action.clone())
    }

    /// Rect of each top-level label within the bar strip, left-to-right.
    #[inline]
    fn menu_rects(&self, bar: Rect) -> Vec<Rect> {
        let mut x = bar.x;
        self.menus
            .iter()
            .map(|menu| {
                let width = menu.label.chars().count() as u16 + 4;
                let rect = Rect::new(x, bar.y, width, 1).intersection(bar);
                x += width;
                rect
            })
            .collect()
    }

    /// [`Rect`] of the open dropdown for menu `index`,
    /// anchored directly under its label.
    fn dropdown_rect(&self, bar: Rect, index: usize) -> Option<Rect> {
        let menu = self.menus.get(index)?;
        let label_rects = self.menu_rects(bar);
        let anchor = label_rects.get(index)?;
        let width = menu
            .items
            .iter()
            .map(|item| item.label.chars().count() as u16)
            .max()
            .unwrap_or(0)
            + 4;
        let height = menu.items.len() as u16 + 2;
        Some(Rect::new(anchor.x, anchor.y + 1, width, height))
    }

    /// Which top-level label (by index) is at `(column, row)`, if any.
    #[inline]
    fn hit_test_bar(&self, bar: Rect, column: u16, row: u16) -> Option<usize> {
        let point = Position::from((column, row));
        self.menu_rects(bar)
            .into_iter()
            .position(|rect| rect.contains(point))
    }

    /// Which item index (if any) is at `(column, row)` inside the currently
    /// open dropdown - shared by click and hover resolution.
    fn hovered_item(&self, bar: Rect, column: u16, row: u16) -> Option<usize> {
        let index = self.open?;
        let menu = self.menus.get(index)?;
        let popup = self.dropdown_rect(bar, index)?;
        if !popup.contains(Position::from((column, row))) {
            return None;
        }
        // Row 0 of `popup` is the top border; items start at row 1.
        let item_row = row.checked_sub(popup.y + 1)? as usize;
        (item_row < menu.items.len()).then_some(item_row)
    }

    /// Which item's action (if any) was clicked inside the open dropdown.
    #[inline]
    fn hit_test_dropdown(&self, bar: Rect, column: u16, row: u16) -> Option<Action> {
        let index = self.open?;
        let item_index = self.hovered_item(bar, column, row)?;
        self.action_at(index, item_index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topbar::MenuItem;

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TestAction {
        A,
        B,
    }

    fn bar() -> TopBar<TestAction> {
        let mut bar = TopBar::default();
        bar.set_menus(vec![
            Menu::new("File", vec![MenuItem::new("Open", TestAction::A)]),
            Menu::new("Edit", vec![
                MenuItem::new("Cut", TestAction::A),
                MenuItem::new("Paste", TestAction::B),
            ]),
        ]);
        bar
    }

    fn bar_area() -> Rect {
        Rect::new(0, 0, 80, 1)
    }

    fn press(code: KeyCode) -> Event {
        use crossterm::event::{KeyEvent, KeyEventState, KeyModifiers};
        Event::Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    fn left_click(column: u16, row: u16) -> Event {
        use crossterm::event::{KeyModifiers, MouseEvent};
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    fn moved(column: u16, row: u16) -> Event {
        use crossterm::event::{KeyModifiers, MouseEvent};
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Moved,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        })
    }

    #[test]
    fn split_gives_the_bar_exactly_one_row() {
        let (bar, content) = TopBar::<TestAction>::split(Rect::new(0, 0, 80, 24));
        assert_eq!(bar.height, 1);
        assert_eq!(content.height, 23);
    }

    #[test]
    fn clicking_a_label_opens_its_dropdown() {
        let mut bar = bar();
        let area = bar_area();
        let rects = bar.menu_rects(area);

        let outcome = bar.handle_event(&left_click(rects[1].x, 0), area);
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert_eq!(bar.open, Some(1));
    }

    #[test]
    fn clicking_the_same_label_again_closes_it() {
        let mut bar = bar();
        let area = bar_area();
        let rects = bar.menu_rects(area);

        let _ = bar.handle_event(&left_click(rects[1].x, 0), area);
        let outcome = bar.handle_event(&left_click(rects[1].x, 0), area);
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert!(bar.open.is_none());
    }

    #[test]
    fn clicking_a_dropdown_item_runs_its_action_and_closes() {
        let mut bar = bar();
        let area = bar_area();
        bar.open(1);

        let popup = bar.dropdown_rect(area, 1).unwrap();
        let outcome = bar.handle_event(&left_click(popup.x, popup.y + 1), area);

        assert!(matches!(outcome, TopBarOutcome::Action(TestAction::A)));
        assert!(bar.open.is_none());
    }

    #[test]
    fn clicking_away_closes_and_is_consumed() {
        let mut bar = bar();
        bar.open(1);

        let outcome = bar.handle_event(&left_click(79, 20), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert!(bar.open.is_none());
    }

    #[test]
    fn click_outside_with_nothing_open_is_not_consumed() {
        let mut bar = bar();
        let outcome = bar.handle_event(&left_click(79, 20), bar_area());
        assert!(matches!(outcome, TopBarOutcome::NotConsumed));
    }

    #[test]
    fn esc_does_nothing_when_no_menu_is_open() {
        let mut bar = bar();
        let outcome = bar.handle_event(&press(KeyCode::Esc), bar_area());
        assert!(matches!(outcome, TopBarOutcome::NotConsumed));
        assert!(bar.open.is_none());
    }

    #[test]
    fn esc_closes_an_open_menu() {
        let mut bar = bar();
        bar.open(1);
        let outcome = bar.handle_event(&press(KeyCode::Esc), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert!(bar.open.is_none());
    }

    #[test]
    fn nothing_is_highlighted_when_a_menu_first_opens() {
        let mut bar = bar();
        bar.open(1);
        assert!(bar.highlighted.is_none());
    }

    #[test]
    fn down_from_nothing_highlights_the_first_item_and_enter_selects_it() {
        let mut bar = bar();
        bar.open(1);
        let _ = bar.handle_event(&press(KeyCode::Down), bar_area());
        assert_eq!(bar.highlighted, Some(0));

        let outcome = bar.handle_event(&press(KeyCode::Enter), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Action(TestAction::A)));
        assert!(bar.open.is_none());
    }

    #[test]
    fn up_from_nothing_highlights_the_last_item() {
        let mut bar = bar();
        bar.open(1);
        let _ = bar.handle_event(&press(KeyCode::Up), bar_area());
        assert_eq!(bar.highlighted, Some(1));
    }

    #[test]
    fn down_twice_then_enter_selects_the_second_item() {
        let mut bar = bar();
        bar.open(1);
        let _ = bar.handle_event(&press(KeyCode::Down), bar_area());
        let outcome = bar.handle_event(&press(KeyCode::Down), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert_eq!(bar.highlighted, Some(1));

        let outcome = bar.handle_event(&press(KeyCode::Enter), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Action(TestAction::B)));
        assert!(bar.open.is_none());
    }

    #[test]
    fn enter_with_nothing_highlighted_just_closes() {
        let mut bar = bar();
        bar.open(1);
        let outcome = bar.handle_event(&press(KeyCode::Enter), bar_area());
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert!(bar.open.is_none());
    }

    #[test]
    fn reset_closes_an_open_menu() {
        let mut bar = bar();
        bar.open(1);
        bar.reset();
        assert!(bar.open.is_none());
    }

    #[test]
    fn hovering_a_label_highlights_it_without_opening_its_dropdown() {
        let mut bar = bar();
        let area = bar_area();
        let rects = bar.menu_rects(area);

        let outcome = bar.handle_event(&moved(rects[0].x, 0), area);
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert_eq!(bar.hovered_label, Some(0));
        assert!(bar.open.is_none());
    }

    #[test]
    fn moving_off_the_bar_clears_the_label_hover() {
        let mut bar = bar();
        let area = bar_area();
        let rects = bar.menu_rects(area);

        let _ = bar.handle_event(&moved(rects[0].x, 0), area);
        let outcome = bar.handle_event(&moved(79, 20), area);
        assert!(matches!(outcome, TopBarOutcome::NotConsumed));
        assert!(bar.hovered_label.is_none());
    }

    #[test]
    fn hovering_a_dropdown_item_highlights_it() {
        let mut bar = bar();
        let area = bar_area();
        bar.open(1);

        let popup = bar.dropdown_rect(area, 1).unwrap();
        let outcome = bar.handle_event(&moved(popup.x, popup.y + 2), area);
        assert!(matches!(outcome, TopBarOutcome::Consumed));
        assert_eq!(bar.highlighted, Some(1));
    }

    #[test]
    fn moving_off_the_dropdown_clears_its_highlight_but_leaves_it_open() {
        let mut bar = bar();
        let area = bar_area();
        bar.open(1);

        let popup = bar.dropdown_rect(area, 1).unwrap();
        let _ = bar.handle_event(&moved(popup.x, popup.y + 1), area);
        assert_eq!(bar.highlighted, Some(0));

        let outcome = bar.handle_event(&moved(79, 20), area);
        assert!(matches!(outcome, TopBarOutcome::NotConsumed));
        assert!(bar.highlighted.is_none());
        assert_eq!(bar.open, Some(1));
    }

    #[test]
    fn hovering_overrides_a_highlight_set_by_the_keyboard() {
        let mut bar = bar();
        let area = bar_area();
        bar.open(1);

        let _ = bar.handle_event(&press(KeyCode::Down), area);
        assert_eq!(bar.highlighted, Some(0));

        let popup = bar.dropdown_rect(area, 1).unwrap();
        let _ = bar.handle_event(&moved(popup.x, popup.y + 2), area);
        assert_eq!(bar.highlighted, Some(1));
    }

    #[test]
    fn moving_outside_bar_area_is_not_consumed() {
        let mut bar = bar();
        let outcome = bar.handle_event(&moved(10, 15), bar_area());
        assert!(matches!(outcome, TopBarOutcome::NotConsumed));
    }
}
