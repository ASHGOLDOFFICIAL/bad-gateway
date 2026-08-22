use std::{
    io::{self, Stdout, stdout},
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    event::{
        self, DisableFocusChange, DisableMouseCapture, EnableFocusChange, EnableMouseCapture,
        Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
        supports_keyboard_enhancement,
    },
};
use i18n_embed_fl::fl;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};

use crate::{
    App, AppSignal,
    i18n::LOADER,
    topbar::{Menu, MenuItem, TopBar, TopBarOutcome},
};

type Term = Terminal<CrosstermBackend<Stdout>>;

#[must_use]
#[derive(Debug, Clone, PartialEq, Eq)]
enum ShellAction {
    ShutDown,
    AppItem(&'static str),
}

/// Mini desktop environment that always shows exactly one `App`.
#[must_use]
pub struct Shell {
    topbar: TopBar<ShellAction>,
    default_app: Box<dyn App>,
    open: Option<Box<dyn App>>,
    frame_duration: Duration,
    should_quit: bool,
}

impl Shell {
    /// Makes new `Shell` with given `frame_rate`,
    /// showing `default_app` whenever no other app is open.
    #[inline(always)]
    pub fn new(frame_rate: u64, default_app: Box<dyn App>) -> Self {
        Self {
            topbar: TopBar::default(),
            default_app,
            open: None,
            frame_duration: Duration::from_nanos(1_000_000_000 / frame_rate),
            should_quit: false,
        }
    }

    /// Runs this `Shell`.
    pub fn run(mut self) -> io::Result<()> {
        install_panic_hook();
        let mut terminal = init_terminal()?;
        let result = self.event_loop(&mut terminal);
        restore_terminal(&mut terminal)?;
        result
    }

    fn event_loop(&mut self, terminal: &mut Term) -> io::Result<()> {
        let mut previous_instant = Instant::now();

        loop {
            let menus = self.build_menus();
            self.topbar.set_menus(menus);

            self.drain_input(terminal)?;
            if self.should_quit {
                return Ok(());
            }

            terminal.draw(|frame| {
                let (bar_area, content_area) = TopBar::<ShellAction>::split(frame.area());
                self.current_app_mut_ref().draw(frame, content_area);
                self.topbar.draw(frame, bar_area);
            })?;

            let frame_time = previous_instant.elapsed();
            if frame_time < self.frame_duration {
                thread::sleep(self.frame_duration - frame_time);
            }
            previous_instant = Instant::now();
        }
    }

    fn drain_input(&mut self, terminal: &mut Term) -> io::Result<()> {
        let area: Rect = terminal.size()?.into();
        let (bar_area, content_area) = TopBar::<ShellAction>::split(area);

        while event::poll(Duration::ZERO)? {
            let event = event::read()?;

            if self.handle_menu_event(&event, bar_area) {
                continue;
            }

            let signal = self
                .current_app_mut_ref()
                .handle_event(&event, content_area);
            self.apply_signal(signal);
        }
        Ok(())
    }

    fn run_action(&mut self, action: ShellAction) {
        match action {
            ShellAction::ShutDown => self.should_quit = true,
            ShellAction::AppItem(id) => {
                let signal = self.current_app_mut_ref().handle_menu_action(id);
                self.apply_signal(signal);
            }
        }
    }

    #[inline(always)]
    fn apply_signal(&mut self, signal: AppSignal) {
        match signal {
            AppSignal::Continue => {}
            AppSignal::Close => {
                self.open = None;
                self.topbar.reset();
            }
            AppSignal::Open(app) => {
                self.open = Some(app);
                self.topbar.reset();
            }
        }
    }

    /// Builds this frame's [`Menu`]s.
    fn build_menus(&self) -> Vec<Menu<ShellAction>> {
        let system = Menu::new("[@]", vec![MenuItem::new(
            fl!(LOADER, "menu-system-shutdown"),
            ShellAction::ShutDown,
        )]);

        let current = self.current_app_ref();
        let items = current
            .menu_actions()
            .into_iter()
            .map(|item| MenuItem::new(item.label, ShellAction::AppItem(item.action)))
            .collect();

        vec![system, Menu::new(current.name(), items)]
    }

    fn handle_menu_event(&mut self, event: &Event, bar_area: Rect) -> bool {
        match self.topbar.handle_event(event, bar_area) {
            TopBarOutcome::Action(action) => {
                self.run_action(action);
                true
            }
            TopBarOutcome::Consumed => true,
            TopBarOutcome::NotConsumed => false,
        }
    }

    #[inline(always)]
    fn current_app_mut_ref(&mut self) -> &mut dyn App {
        match &mut self.open {
            Some(app) => app.as_mut(),
            None => self.default_app.as_mut(),
        }
    }

    #[inline(always)]
    fn current_app_ref(&self) -> &dyn App {
        match &self.open {
            Some(app) => app.as_ref(),
            None => self.default_app.as_ref(),
        }
    }
}

fn init_terminal() -> io::Result<Term> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(
        out,
        EnterAlternateScreen,
        EnableFocusChange,
        EnableMouseCapture
    )?;

    if supports_keyboard_enhancement().unwrap_or(false) {
        execute!(
            out,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                    | KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
            )
        )?;
    }

    Terminal::new(CrosstermBackend::new(out))
}

/// Restore terminal on panic.
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
        let _ = disable_raw_mode();
        let _ = execute!(
            stdout(),
            DisableFocusChange,
            DisableMouseCapture,
            LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        default_hook(info);
    }));
}

fn restore_terminal(terminal: &mut Term) -> io::Result<()> {
    if supports_keyboard_enhancement().unwrap_or(false) {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        DisableFocusChange,
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use ratatui::Frame;

    use super::*;

    struct DummyApp {
        close_on: &'static str,
    }

    impl DummyApp {
        fn new() -> Self {
            Self { close_on: "quit" }
        }
    }

    impl App for DummyApp {
        fn name(&self) -> String {
            "Dummy".to_string()
        }

        fn menu_actions(&self) -> Vec<MenuItem<&'static str>> {
            vec![MenuItem::new("Any", "any"), MenuItem::new("Quit", "quit")]
        }

        fn handle_menu_action(&mut self, action: &str) -> AppSignal {
            if action == "open" {
                AppSignal::Open(Box::new(DummyApp::new()))
            } else if action == self.close_on {
                AppSignal::Close
            } else {
                AppSignal::Continue
            }
        }

        fn handle_event(&mut self, _event: &Event, _area: Rect) -> AppSignal {
            AppSignal::Continue
        }

        fn draw(&mut self, _frame: &mut Frame, _area: Rect) {}
    }

    fn test_shell() -> Shell {
        Shell::new(30, Box::new(DummyApp::new()))
    }

    fn shell_with_open_dummy() -> Shell {
        let mut shell = test_shell();
        shell.open = Some(Box::new(DummyApp::new()));
        shell
    }

    #[test]
    fn shutdown_action_sets_should_quit() {
        let mut shell = test_shell();
        shell.run_action(ShellAction::ShutDown);
        assert!(shell.should_quit);
    }

    #[test]
    fn app_item_action_forwards_to_the_open_app_and_closes_on_signal() {
        let mut shell = shell_with_open_dummy();
        shell.run_action(ShellAction::AppItem("quit"));
        assert!(shell.open.is_none());
    }

    #[test]
    fn app_item_action_keeps_the_app_open_when_it_signals_continue() {
        let mut shell = shell_with_open_dummy();
        shell.run_action(ShellAction::AppItem("any"));
        assert!(shell.open.is_some());
    }

    #[test]
    fn build_menus_includes_the_default_apps_own_menu_when_nothing_is_open() {
        let shell = test_shell();
        let menus = shell.build_menus();
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[1].label, "Dummy");
        assert_eq!(menus[1].items.len(), 2);
    }

    #[test]
    fn app_item_action_reaches_the_default_app_when_nothing_is_open() {
        let mut shell = test_shell();
        shell.run_action(ShellAction::AppItem("open"));
        assert!(shell.open.is_some());
    }

    #[test]
    fn open_signal_from_the_default_app_replaces_the_shown_app() {
        let mut shell = test_shell();
        shell.apply_signal(AppSignal::Open(Box::new(DummyApp::new())));
        assert!(shell.open.is_some());
    }

    #[test]
    fn close_signal_returns_to_the_default_app() {
        let mut shell = shell_with_open_dummy();
        shell.apply_signal(AppSignal::Close);
        assert!(shell.open.is_none());
    }
}
