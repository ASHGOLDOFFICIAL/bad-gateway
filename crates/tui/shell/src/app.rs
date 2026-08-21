use crossterm::event::Event;
use ratatui::{Frame, layout::Rect};

use crate::topbar::MenuItem;

/// What an app is telling the shell to do after handling something.
#[must_use]
pub enum AppSignal {
    /// Application continues its work.
    Continue,

    /// Application should be closed.
    Close,

    /// Replace whatever's currently shown with this app.
    Open(Box<dyn App>),
}

/// A program the shell can host.
#[must_use]
pub trait App {
    /// This app's own menu label in the top bar while it's shown.
    fn name(&self) -> String;

    /// This app's own menu items, shown in a dropdown under `name()`.
    /// Selecting an item calls [`handle_menu_action`](Self::handle_menu_action)
    /// with that item's `action`.
    fn menu_actions(&self) -> Vec<MenuItem<&'static str>>;

    /// The `action` id of whichever `menu_actions()` item was selected.
    fn handle_menu_action(&mut self, action: &str) -> AppSignal;

    /// Raw input forwarded while this app is shown,
    /// after the shell has had first refusal at the event.
    fn handle_event(&mut self, event: &Event, area: Rect) -> AppSignal;

    /// Draws the app into `area`.
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}
