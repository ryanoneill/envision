//! Router example -- multi-screen navigation with history.
//!
//! Demonstrates the Router component for type-safe screen navigation
//! with a back stack, history management, and screen rendering.
//!
//! Run with: cargo run --example router --features navigation-components

use envision::prelude::*;

/// The screens in our application.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Screen {
    Home,
    Settings,
    Profile,
    About,
}

/// Application marker type.
struct RouterApp;

/// Application state.
#[derive(Clone)]
struct State {
    router: RouterState<Screen>,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Navigate(Screen),
    Back,
    Reset,
    Quit,
}

impl App for RouterApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let router = RouterState::new(Screen::Home).with_max_history(10);
        (State { router }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Navigate(screen) => {
                Router::update(&mut state.router, RouterMessage::Navigate(screen));
            }
            Msg::Back => {
                Router::update(&mut state.router, RouterMessage::Back);
            }
            Msg::Reset => {
                Router::update(&mut state.router, RouterMessage::Reset(Screen::Home));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        // Render content based on current screen
        let (title, body) = match state.router.current() {
            Screen::Home => (
                "Home",
                "Welcome home!\n\n\
                 Press 's' for Settings\n\
                 Press 'p' for Profile\n\
                 Press 'a' for About",
            ),
            Screen::Settings => (
                "Settings",
                "Application Settings\n\n\
                 Theme: Dark\n\
                 Language: English\n\
                 Notifications: On\n\n\
                 Press Backspace to go back",
            ),
            Screen::Profile => (
                "Profile",
                "User Profile\n\n\
                 Name: Alice\n\
                 Email: alice@example.com\n\
                 Role: Administrator\n\n\
                 Press Backspace to go back",
            ),
            Screen::About => (
                "About",
                "Envision Framework v0.1.0\n\n\
                 A modern TUI framework\n\
                 built with Rust.\n\n\
                 Press Backspace to go back",
            ),
        };

        let widget = ratatui::widgets::Paragraph::new(body).block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title(title),
        );
        frame.render_widget(widget, chunks[0]);

        let history_len = state.router.history_len();
        let can_back = state.router.can_go_back();
        let status = format!(
            " Screen: {:?} | History: {} | Back: {}",
            state.router.current(),
            history_len,
            can_back
        );
        frame.render_widget(
            ratatui::widgets::Paragraph::new(status).style(Style::default().fg(Color::DarkGray)),
            chunks[1],
        );
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            match key.key {
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
                Key::Char('s') => Some(Msg::Navigate(Screen::Settings)),
                Key::Char('p') => Some(Msg::Navigate(Screen::Profile)),
                Key::Char('a') => Some(Msg::Navigate(Screen::About)),
                Key::Backspace => Some(Msg::Back),
                Key::Char('r') => Some(Msg::Reset),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<RouterApp, _>::virtual_terminal(50, 14)?;

    println!("=== Router Example ===\n");

    // Initial render: Home screen
    vt.tick()?;
    println!("Home screen:");
    println!("{}\n", vt.display());

    // Navigate to Settings
    vt.dispatch(Msg::Navigate(Screen::Settings));
    vt.tick()?;
    println!("After navigating to Settings:");
    println!("{}\n", vt.display());

    // Navigate to Profile
    vt.dispatch(Msg::Navigate(Screen::Profile));
    vt.tick()?;
    println!("After navigating to Profile:");
    println!("{}\n", vt.display());

    // Go back to Settings
    vt.dispatch(Msg::Back);
    vt.tick()?;
    println!("After going back (to Settings):");
    println!("{}\n", vt.display());

    // Go back to Home
    vt.dispatch(Msg::Back);
    vt.tick()?;
    println!("After going back again (to Home):");
    println!("{}\n", vt.display());

    Ok(())
}
