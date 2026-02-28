//! Themed App example demonstrating the theming system.
//!
//! This example shows how to use themes with Envision components:
//! - Switching between Default and Nord themes
//! - Components adapting to theme colors
//! - Creating a cohesive themed UI
//!
//! Run with: cargo run --example themed_app

use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

/// The application marker type
struct ThemedApp;

/// Which theme is currently active
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ActiveTheme {
    #[default]
    Default,
    Nord,
}

impl ActiveTheme {
    fn name(&self) -> &'static str {
        match self {
            ActiveTheme::Default => "Default",
            ActiveTheme::Nord => "Nord",
        }
    }

    fn toggle(&self) -> Self {
        match self {
            ActiveTheme::Default => ActiveTheme::Nord,
            ActiveTheme::Nord => ActiveTheme::Default,
        }
    }
}

/// Application state
#[derive(Clone)]
struct State {
    active_theme: ActiveTheme,
    button_state: ButtonState,
    checkbox_state: CheckboxState,
    progress_state: ProgressBarState,
    list_state: SelectableListState<String>,
}

impl Default for State {
    fn default() -> Self {
        let items = vec![
            "First item".to_string(),
            "Second item".to_string(),
            "Third item".to_string(),
            "Fourth item".to_string(),
        ];
        let mut list_state = SelectableListState::with_items(items);
        SelectableList::<String>::set_focused(&mut list_state, true);
        list_state.select(Some(0));

        let mut button_state = ButtonState::new("Click Me");
        Button::set_focused(&mut button_state, true);

        Self {
            active_theme: ActiveTheme::default(),
            button_state,
            checkbox_state: CheckboxState::new("Enable feature"),
            progress_state: ProgressBarState::with_progress(0.65),
            list_state,
        }
    }
}

/// Messages that can modify state
#[derive(Clone, Debug)]
enum Msg {
    ToggleTheme,
    ButtonPressed,
    CheckboxToggled,
    IncreaseProgress,
    DecreaseProgress,
    NextItem,
    PrevItem,
    Quit,
}

impl App for ThemedApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::ToggleTheme => {
                state.active_theme = state.active_theme.toggle();
            }
            Msg::ButtonPressed => {
                // Toggle button focused state for visual feedback
            }
            Msg::CheckboxToggled => {
                Checkbox::update(&mut state.checkbox_state, CheckboxMessage::Toggle);
            }
            Msg::IncreaseProgress => {
                let current = state.progress_state.progress();
                state.progress_state.set_progress((current + 0.1).min(1.0));
            }
            Msg::DecreaseProgress => {
                let current = state.progress_state.progress();
                state.progress_state.set_progress((current - 0.1).max(0.0));
            }
            Msg::NextItem => {
                SelectableList::<String>::update(&mut state.list_state, ListMessage::Down);
            }
            Msg::PrevItem => {
                SelectableList::<String>::update(&mut state.list_state, ListMessage::Up);
            }
            Msg::Quit => {
                return Command::quit();
            }
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        // Get the current theme
        let theme = match state.active_theme {
            ActiveTheme::Default => Theme::default(),
            ActiveTheme::Nord => Theme::nord(),
        };

        let area = frame.area();

        // Main layout
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Theme indicator
            Constraint::Length(4), // Button and checkbox row
            Constraint::Length(3), // Progress bar
            Constraint::Min(5),    // List
            Constraint::Length(3), // Controls
        ])
        .split(area);

        // Title
        let title_style = Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD);
        let title = Paragraph::new("Themed App - Envision Theming Demo")
            .style(title_style)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(theme.border_style()),
            );
        frame.render_widget(title, chunks[0]);

        // Theme indicator
        let theme_indicator = Paragraph::new(Line::from(vec![
            Span::raw("Current Theme: "),
            Span::styled(state.active_theme.name(), theme.focused_bold_style()),
            Span::raw("  |  Press "),
            Span::styled("[T]", theme.focused_style()),
            Span::raw(" to toggle"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title("Theme"),
        );
        frame.render_widget(theme_indicator, chunks[1]);

        // Button and checkbox row
        let button_checkbox_chunks =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);

        // Render button using Component trait
        Button::view(
            &state.button_state,
            frame,
            button_checkbox_chunks[0],
            &theme,
        );

        // Render checkbox using Component trait
        Checkbox::view(
            &state.checkbox_state,
            frame,
            button_checkbox_chunks[1],
            &theme,
        );

        // Progress bar
        ProgressBar::view(&state.progress_state, frame, chunks[3], &theme);

        // Selectable list with block wrapper
        let list_area = chunks[4];
        let is_list_focused = SelectableList::<String>::is_focused(&state.list_state);
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if is_list_focused {
                theme.focused_border_style()
            } else {
                theme.border_style()
            })
            .title("Items");
        let inner_area = list_block.inner(list_area);
        frame.render_widget(list_block, list_area);
        SelectableList::view(&state.list_state, frame, inner_area, &theme);

        // Controls help
        let controls = Paragraph::new(Line::from(vec![
            Span::styled("[T]", theme.info_style()),
            Span::raw(" Theme  "),
            Span::styled("[Space]", theme.info_style()),
            Span::raw(" Toggle  "),
            Span::styled("[+/-]", theme.info_style()),
            Span::raw(" Progress  "),
            Span::styled("[Up/Dn]", theme.info_style()),
            Span::raw(" Navigate  "),
            Span::styled("[Q]", theme.error_style()),
            Span::raw(" Quit"),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.border_style())
                .title("Controls"),
        );
        frame.render_widget(controls, chunks[5]);
    }

    fn handle_event(event: &Event) -> Option<Msg> {
        use crossterm::event::KeyCode;

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Char('t') | KeyCode::Char('T') => Some(Msg::ToggleTheme),
                KeyCode::Char(' ') => Some(Msg::CheckboxToggled),
                KeyCode::Char('+') | KeyCode::Char('=') => Some(Msg::IncreaseProgress),
                KeyCode::Char('-') => Some(Msg::DecreaseProgress),
                KeyCode::Up | KeyCode::Char('k') => Some(Msg::PrevItem),
                KeyCode::Down | KeyCode::Char('j') => Some(Msg::NextItem),
                KeyCode::Enter => Some(Msg::ButtonPressed),
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Msg::Quit),
                _ => None,
            }
        } else {
            None
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a virtual terminal for demonstration
    let mut vt = Runtime::<ThemedApp, _>::virtual_terminal(60, 24)?;

    println!("=== Themed App Demo ===\n");
    println!("This example demonstrates Envision's theming system.\n");

    // Render with default theme
    vt.step()?;
    println!("Default Theme:");
    println!("{}\n", vt.display_ansi());

    // Toggle to Nord theme
    vt.dispatch(Msg::ToggleTheme);
    vt.step()?;
    println!("Nord Theme:");
    println!("{}\n", vt.display_ansi());

    // Demonstrate some interactions
    vt.dispatch(Msg::CheckboxToggled);
    vt.dispatch(Msg::IncreaseProgress);
    vt.dispatch(Msg::NextItem);
    vt.step()?;
    println!("Nord Theme (after interactions):");
    println!("{}\n", vt.display_ansi());

    // Show theme comparison
    println!("=== Theme Comparison ===");
    println!("Default theme uses: Yellow focus, DarkGray disabled, Cyan primary");
    println!("Nord theme uses: Light Blue focus (#88C0D0), Muted gray disabled, Dark blue primary");
    println!("\nThe Nord theme provides a cohesive, eye-friendly color palette");
    println!("inspired by the Arctic's colors - ideal for extended coding sessions.");

    Ok(())
}
