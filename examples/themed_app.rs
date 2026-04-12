//! Themed App example demonstrating the theming system.
//!
//! This example shows how to use themes with Envision components:
//! - Switching between Default and Nord themes
//! - Components adapting to theme colors
//! - Creating a cohesive themed UI
//!
//! Run with: cargo run --example themed_app

use envision::component::{
    Button, ButtonState, Checkbox, CheckboxMessage, CheckboxState, ProgressBar, ProgressBarState,
    SelectableList, SelectableListMessage, SelectableListState,
};
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
    Dracula,
    SolarizedDark,
    GruvboxDark,
    CatppuccinMocha,
}

impl ActiveTheme {
    fn name(&self) -> &'static str {
        match self {
            ActiveTheme::Default => "Default",
            ActiveTheme::Nord => "Nord",
            ActiveTheme::Dracula => "Dracula",
            ActiveTheme::SolarizedDark => "Solarized Dark",
            ActiveTheme::GruvboxDark => "Gruvbox Dark",
            ActiveTheme::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    fn next(&self) -> Self {
        match self {
            ActiveTheme::Default => ActiveTheme::Nord,
            ActiveTheme::Nord => ActiveTheme::Dracula,
            ActiveTheme::Dracula => ActiveTheme::SolarizedDark,
            ActiveTheme::SolarizedDark => ActiveTheme::GruvboxDark,
            ActiveTheme::GruvboxDark => ActiveTheme::CatppuccinMocha,
            ActiveTheme::CatppuccinMocha => ActiveTheme::Default,
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
        list_state.select(Some(0));

        let button_state = ButtonState::new("Click Me");

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
                state.active_theme = state.active_theme.next();
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
                SelectableList::<String>::update(
                    &mut state.list_state,
                    SelectableListMessage::Down,
                );
            }
            Msg::PrevItem => {
                SelectableList::<String>::update(&mut state.list_state, SelectableListMessage::Up);
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
            ActiveTheme::Dracula => Theme::dracula(),
            ActiveTheme::SolarizedDark => Theme::solarized_dark(),
            ActiveTheme::GruvboxDark => Theme::gruvbox_dark(),
            ActiveTheme::CatppuccinMocha => Theme::catppuccin_mocha(),
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
            &mut RenderContext::new(frame, button_checkbox_chunks[0], &theme),
        );

        // Render checkbox using Component trait
        Checkbox::view(
            &state.checkbox_state,
            &mut RenderContext::new(frame, button_checkbox_chunks[1], &theme),
        );

        // Progress bar
        ProgressBar::view(
            &state.progress_state,
            &mut RenderContext::new(frame, chunks[3], &theme),
        );

        // Selectable list with block wrapper
        let list_area = chunks[4];
        let list_block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme.focused_border_style())
            .title("Items");
        let inner_area = list_block.inner(list_area);
        frame.render_widget(list_block, list_area);
        SelectableList::view(
            &state.list_state,
            &mut RenderContext::new(frame, inner_area, &theme).focused(true),
        );

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
        use envision::input::Key;

        if let Some(key) = event.as_key() {
            match key.code {
                Key::Char('t') => Some(Msg::ToggleTheme),
                Key::Char(' ') => Some(Msg::CheckboxToggled),
                Key::Char('+') | Key::Char('=') => Some(Msg::IncreaseProgress),
                Key::Char('-') => Some(Msg::DecreaseProgress),
                Key::Up | Key::Char('k') => Some(Msg::PrevItem),
                Key::Down | Key::Char('j') => Some(Msg::NextItem),
                Key::Enter => Some(Msg::ButtonPressed),
                Key::Char('q') | Key::Esc => Some(Msg::Quit),
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
    vt.tick()?;
    println!("Default Theme:");
    println!("{}\n", vt.display_ansi());

    // Toggle to Nord theme
    vt.dispatch(Msg::ToggleTheme);
    vt.tick()?;
    println!("Nord Theme:");
    println!("{}\n", vt.display_ansi());

    // Demonstrate some interactions
    vt.dispatch(Msg::CheckboxToggled);
    vt.dispatch(Msg::IncreaseProgress);
    vt.dispatch(Msg::NextItem);
    vt.tick()?;
    println!("Nord Theme (after interactions):");
    println!("{}\n", vt.display_ansi());

    // Cycle through remaining themes
    for _ in 0..4 {
        vt.dispatch(Msg::ToggleTheme);
        vt.tick()?;
        println!("{} Theme:", vt.state().active_theme.name());
        println!("{}\n", vt.display_ansi());
    }

    // Show theme comparison
    println!("=== Theme Comparison ===");
    println!("Default:        Yellow focus, DarkGray disabled, Cyan primary");
    println!("Nord:           Light Blue focus (#88C0D0), Muted gray disabled, Dark blue primary");
    println!("Dracula:        Purple focus (#BD93F9), Comment gray disabled, Cyan primary");
    println!("Solarized Dark: Blue focus (#268BD2), Base01 disabled, Blue primary");
    println!("Gruvbox Dark:   Yellow focus (#FABD2F), Gray disabled, Aqua primary");
    println!("Catppuccin:     Lavender focus (#B4BEFE), Surface2 disabled, Blue primary");

    Ok(())
}
