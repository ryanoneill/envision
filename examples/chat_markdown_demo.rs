//! Chat Markdown Demo — interactive chat with markdown rendering and role styles.
//!
//! This demo exercises ChatView's markdown rendering, custom role styles per theme,
//! and the LineInput component for message composition. Pre-populated with markdown
//! content to showcase heading, bold, italic, code, and list rendering.
//!
//! Controls:
//!   Enter       Submit message (rendered as markdown if enabled)
//!   Ctrl+M      Toggle markdown rendering on/off
//!   Ctrl+T      Cycle through themes
//!   Page Up/Dn  Scroll chat history
//!   Ctrl+U      Clear input line
//!   Esc/Ctrl+Q  Quit
//!
//! Run with: cargo run --example chat_markdown_demo --features "compound-components,markdown"

use envision::component::{
    ChatView, ChatViewState, Component, LineInput, LineInputMessage, LineInputOutput,
    LineInputState,
};
use envision::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

// ---------------------------------------------------------------------------
// Theme cycling
// ---------------------------------------------------------------------------

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
            Self::Default => "Default",
            Self::Nord => "Nord",
            Self::Dracula => "Dracula",
            Self::SolarizedDark => "Solarized Dark",
            Self::GruvboxDark => "Gruvbox Dark",
            Self::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::Default => Self::Nord,
            Self::Nord => Self::Dracula,
            Self::Dracula => Self::SolarizedDark,
            Self::SolarizedDark => Self::GruvboxDark,
            Self::GruvboxDark => Self::CatppuccinMocha,
            Self::CatppuccinMocha => Self::Default,
        }
    }

    fn theme(&self) -> Theme {
        match self {
            Self::Default => Theme::default(),
            Self::Nord => Theme::nord(),
            Self::Dracula => Theme::dracula(),
            Self::SolarizedDark => Theme::solarized_dark(),
            Self::GruvboxDark => Theme::gruvbox_dark(),
            Self::CatppuccinMocha => Theme::catppuccin_mocha(),
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    active_theme: ActiveTheme,
    chat: ChatViewState,
    input: LineInputState,
}

fn apply_role_styles(chat: &mut ChatViewState, theme: &Theme) {
    chat.set_role_style(
        ChatRole::User,
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD),
    );
    chat.set_role_style(
        ChatRole::Assistant,
        Style::default()
            .fg(theme.success)
            .add_modifier(Modifier::BOLD),
    );
    chat.set_role_style(
        ChatRole::System,
        Style::default()
            .fg(theme.disabled)
            .add_modifier(Modifier::ITALIC),
    );
}

impl Default for State {
    fn default() -> Self {
        let mut chat = ChatViewState::new()
            .with_markdown(true)
            .with_timestamps(true)
            .with_input_height(0);

        let theme = ActiveTheme::default().theme();
        apply_role_styles(&mut chat, &theme);

        // Pre-populate with markdown content
        chat.push_system_with_timestamp(
            "Welcome to the Chat Markdown Demo! Markdown rendering is enabled.",
            "09:00",
        );

        chat.push_user_with_timestamp("Can you show me some **markdown** features?", "09:01");

        chat.push_assistant_with_timestamp(
            "Sure! Here are the supported markdown features:\n\n\
             # Heading 1\n\
             ## Heading 2\n\
             ### Heading 3\n\n\
             **Bold text** and *italic text* and `inline code`.\n\n\
             - Bullet point one\n\
             - Bullet point two\n\
             - Bullet point three\n\n\
             ```rust\n\
             fn main() {\n\
                 println!(\"Hello, world!\");\n\
             }\n\
             ```\n\n\
             ---\n\n\
             You can also use ~~strikethrough~~ text!",
            "09:02",
        );

        chat.push_user_with_timestamp(
            "That's great! What about **nested** formatting?\n\n\
             1. First item with *emphasis*\n\
             2. Second item with `code`\n\
             3. Third item with **bold**",
            "09:03",
        );

        chat.push_assistant_with_timestamp(
            "Exactly! You can mix formatting freely. Try typing a message below \
             with markdown syntax. Toggle markdown with **Ctrl+M** to see the difference.\n\n\
             Press **Ctrl+T** to cycle through themes and see how role styles change.",
            "09:04",
        );

        let mut input = LineInputState::new().with_placeholder("Type a markdown message...");
        input.set_focused(true);

        Self {
            active_theme: ActiveTheme::default(),
            chat,
            input,
        }
    }
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    Input(LineInputMessage),
    InputOutput(LineInputOutput),
    CycleTheme,
    ToggleMarkdown,
    ScrollUp,
    ScrollDown,
    Quit,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct ChatMarkdownApp;

impl App for ChatMarkdownApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Input(m) => {
                if let Some(output) = LineInput::update(&mut state.input, m) {
                    return Self::update(state, Msg::InputOutput(output));
                }
            }
            Msg::InputOutput(output) => match output {
                LineInputOutput::Submitted(text) => {
                    if !text.trim().is_empty() {
                        state.chat.push_user(&text);
                        // Echo back with a markdown-aware response
                        let response = format!(
                            "You said:\n\n> {}\n\n*Message received and rendered with markdown {}.*",
                            text.lines().collect::<Vec<_>>().join("\n> "),
                            if state.chat.markdown_enabled() {
                                "**enabled**"
                            } else {
                                "disabled"
                            }
                        );
                        state.chat.push_assistant(&response);
                    }
                }
                LineInputOutput::Changed(_) | LineInputOutput::Copied(_) => {}
            },
            Msg::CycleTheme => {
                state.active_theme = state.active_theme.next();
                let theme = state.active_theme.theme();
                apply_role_styles(&mut state.chat, &theme);
                state.chat.push_system(format!(
                    "Theme switched to **{}**",
                    state.active_theme.name()
                ));
            }
            Msg::ToggleMarkdown => {
                let enabled = !state.chat.markdown_enabled();
                state.chat.set_markdown_enabled(enabled);
                state.chat.push_system(format!(
                    "Markdown rendering **{}**",
                    if enabled { "enabled" } else { "disabled" }
                ));
            }
            Msg::ScrollUp => {
                ChatView::update(&mut state.chat, ChatViewMessage::ScrollUp);
            }
            Msg::ScrollDown => {
                ChatView::update(&mut state.chat, ChatViewMessage::ScrollDown);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let theme = state.active_theme.theme();
        let area = frame.area();

        // Background
        frame.render_widget(Block::default().style(theme.normal_style()), area);

        let chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Min(6),    // Chat messages
            Constraint::Length(3), // Input
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        // Header
        let md_status = if state.chat.markdown_enabled() {
            Span::styled(" MD:ON ", theme.success_style())
        } else {
            Span::styled(" MD:OFF ", theme.disabled_style())
        };
        let header = Paragraph::new(Line::from(vec![
            Span::styled(
                " Chat Markdown Demo ",
                Style::default()
                    .fg(theme.primary)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | Theme: "),
            Span::styled(state.active_theme.name(), theme.focused_bold_style()),
            Span::raw(" | "),
            md_status,
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(theme.focused_border_style()),
        );
        frame.render_widget(header, chunks[0]);

        // Chat messages
        ChatView::view(&state.chat, frame, chunks[1], &theme);

        // Input
        LineInput::view(&state.input, frame, chunks[2], &theme);

        // Status bar
        let status = Paragraph::new(Line::from(vec![
            Span::styled("[Enter]", theme.info_style()),
            Span::raw(" Send  "),
            Span::styled("[Ctrl+M]", theme.info_style()),
            Span::raw(" Toggle MD  "),
            Span::styled("[Ctrl+T]", theme.info_style()),
            Span::raw(" Theme  "),
            Span::styled("[PgUp/Dn]", theme.info_style()),
            Span::raw(" Scroll  "),
            Span::styled("[Esc]", theme.error_style()),
            Span::raw(" Quit"),
        ]))
        .alignment(Alignment::Center)
        .style(theme.normal_style());
        frame.render_widget(status, chunks[3]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            // Ctrl+M toggles markdown
            if key.code == KeyCode::Char('m') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Msg::ToggleMarkdown);
            }
            // Ctrl+Q or Esc to quit
            if key.code == KeyCode::Esc {
                return Some(Msg::Quit);
            }
            if key.code == KeyCode::Char('q') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Msg::Quit);
            }
            // T for theme cycling (only when not in the middle of typing)
            if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Some(Msg::CycleTheme);
            }
            // Page Up/Down for scrolling
            if key.code == KeyCode::PageUp {
                return Some(Msg::ScrollUp);
            }
            if key.code == KeyCode::PageDown {
                return Some(Msg::ScrollDown);
            }
        }
        // Delegate to input
        state.input.handle_event(event).map(Msg::Input)
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<ChatMarkdownApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
