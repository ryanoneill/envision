//! Styling Showcase — interactive exploration of all theme styles and rich text formatting.
//!
//! This demo exercises every theme style helper method and every StyledInline variant,
//! letting you switch between all 6 built-in themes to see how each one renders.
//!
//! Controls:
//!   Ctrl+T      Cycle through themes (Default → Nord → Dracula → Solarized → Gruvbox → Catppuccin)
//!   Tab         Switch between Style Palette and Rich Text panels
//!   Up/k        Scroll up (in Rich Text panel)
//!   Down/j      Scroll down (in Rich Text panel)
//!   Page Up     Scroll up one page
//!   Page Down   Scroll down one page
//!   Home        Jump to top
//!   End         Jump to bottom
//!   Esc         Quit
//!
//! Run with: cargo run --example styling_showcase --features full

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
// Panel focus
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Panel {
    #[default]
    Palette,
    RichText,
}

impl Panel {
    fn toggle(&self) -> Self {
        match self {
            Self::Palette => Self::RichText,
            Self::RichText => Self::Palette,
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct State {
    active_theme: ActiveTheme,
    panel: Panel,
    styled_text: StyledTextState,
}

impl Default for State {
    fn default() -> Self {
        let content = build_rich_text_content();
        let styled_text = StyledTextState::new()
            .with_content(content)
            .with_title("Rich Text Formatting");

        Self {
            active_theme: ActiveTheme::default(),
            panel: Panel::default(),
            styled_text,
        }
    }
}

fn build_rich_text_content() -> styled_text::StyledContent {
    styled_text::StyledContent::new()
        .heading(1, "StyledInline Variants")
        .paragraph(vec![styled_text::StyledInline::Plain(
            "This is Plain text — the default inline style.".to_string(),
        )])
        .paragraph(vec![styled_text::StyledInline::Bold(
            "This is Bold text — for emphasis and headings.".to_string(),
        )])
        .paragraph(vec![styled_text::StyledInline::Italic(
            "This is Italic text — for subtle emphasis or titles.".to_string(),
        )])
        .paragraph(vec![styled_text::StyledInline::Underline(
            "This is Underline text — for links or key terms.".to_string(),
        )])
        .paragraph(vec![styled_text::StyledInline::Strikethrough(
            "This is Strikethrough text — for deprecated or removed items.".to_string(),
        )])
        .paragraph(vec![styled_text::StyledInline::Code(
            "This is Code text — for inline code snippets.".to_string(),
        )])
        .blank_line()
        .heading(2, "Colored Inline Text")
        .paragraph(vec![
            styled_text::StyledInline::Colored {
                text: "Red foreground".to_string(),
                fg: Some(Color::Red),
                bg: None,
            },
            styled_text::StyledInline::Plain(" | ".to_string()),
            styled_text::StyledInline::Colored {
                text: "Green foreground".to_string(),
                fg: Some(Color::Green),
                bg: None,
            },
            styled_text::StyledInline::Plain(" | ".to_string()),
            styled_text::StyledInline::Colored {
                text: "Blue foreground".to_string(),
                fg: Some(Color::Blue),
                bg: None,
            },
        ])
        .paragraph(vec![
            styled_text::StyledInline::Colored {
                text: "Cyan foreground".to_string(),
                fg: Some(Color::Cyan),
                bg: None,
            },
            styled_text::StyledInline::Plain(" | ".to_string()),
            styled_text::StyledInline::Colored {
                text: "Magenta foreground".to_string(),
                fg: Some(Color::Magenta),
                bg: None,
            },
            styled_text::StyledInline::Plain(" | ".to_string()),
            styled_text::StyledInline::Colored {
                text: "Yellow foreground".to_string(),
                fg: Some(Color::Yellow),
                bg: None,
            },
        ])
        .paragraph(vec![styled_text::StyledInline::Colored {
            text: " Highlighted text with background ".to_string(),
            fg: Some(Color::White),
            bg: Some(Color::Blue),
        }])
        .blank_line()
        .heading(2, "Mixed Inline Styles")
        .paragraph(vec![
            styled_text::StyledInline::Plain("You can ".to_string()),
            styled_text::StyledInline::Bold("mix".to_string()),
            styled_text::StyledInline::Plain(" and ".to_string()),
            styled_text::StyledInline::Italic("match".to_string()),
            styled_text::StyledInline::Plain(" different styles within a ".to_string()),
            styled_text::StyledInline::Code("single paragraph".to_string()),
            styled_text::StyledInline::Plain(". Here's ".to_string()),
            styled_text::StyledInline::Colored {
                text: "colored".to_string(),
                fg: Some(Color::Cyan),
                bg: None,
            },
            styled_text::StyledInline::Plain(" text too.".to_string()),
        ])
        .blank_line()
        .heading(2, "Block-Level Elements")
        .text("Bullet lists group related items:")
        .bullet_list(vec![
            vec![
                styled_text::StyledInline::Bold("First item".to_string()),
                styled_text::StyledInline::Plain(" — with bold label".to_string()),
            ],
            vec![
                styled_text::StyledInline::Italic("Second item".to_string()),
                styled_text::StyledInline::Plain(" — with italic label".to_string()),
            ],
            vec![styled_text::StyledInline::Plain(
                "Third item — plain text".to_string(),
            )],
        ])
        .text("Numbered lists for ordered content:")
        .numbered_list(vec![
            vec![styled_text::StyledInline::Plain(
                "Install envision".to_string(),
            )],
            vec![styled_text::StyledInline::Plain(
                "Create your App".to_string(),
            )],
            vec![styled_text::StyledInline::Plain(
                "Run and enjoy!".to_string(),
            )],
        ])
        .text("Code blocks for source code:")
        .code_block(
            Some("rust"),
            "use envision::prelude::*;\n\nfn main() -> envision::Result<()> {\n    let _state = TerminalRuntime::<MyApp>::new_terminal()?\n        .run_terminal()\n        .await?;\n    Ok(())\n}",
        )
        .horizontal_rule()
        .text("Use Up/Down to scroll. Press Tab to switch panels. Press T to change themes.")
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
enum Msg {
    CycleTheme,
    TogglePanel,
    StyledText(StyledTextMessage),
    Quit,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct StylingShowcaseApp;

impl App for StylingShowcaseApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        (State::default(), Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::CycleTheme => {
                state.active_theme = state.active_theme.next();
            }
            Msg::TogglePanel => {
                state.panel = state.panel.toggle();
            }
            Msg::StyledText(m) => {
                StyledText::update(&mut state.styled_text, m);
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

        let main_chunks = Layout::vertical([
            Constraint::Length(3), // Header
            Constraint::Min(10),   // Content
            Constraint::Length(1), // Footer
        ])
        .split(area);

        // Header — theme name
        render_header(state, frame, main_chunks[0], &theme);

        // Content — two panels side by side
        let panels = Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(main_chunks[1]);

        // Left panel: Style Palette
        render_style_palette(state, frame, panels[0], &theme);

        // Right panel: Rich Text
        StyledText::view(
            &state.styled_text,
            &mut RenderContext::new(frame, panels[1], &theme),
        );

        // Footer — key hints
        render_footer(frame, main_chunks[2], &theme);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if key.key == Key::Char('t') && key.modifiers.ctrl() {
                return Some(Msg::CycleTheme);
            }
            match key.key {
                Key::Char('q') | Key::Esc => return Some(Msg::Quit),
                Key::Tab => return Some(Msg::TogglePanel),
                _ => {}
            }
        }
        if state.panel == Panel::RichText {
            StyledText::handle_event(
                &state.styled_text,
                event,
                &EventContext::new().focused(true),
            )
            .map(Msg::StyledText)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Rendering helpers
// ---------------------------------------------------------------------------

fn render_header(state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let header = Paragraph::new(Line::from(vec![
        Span::styled(
            " Styling Showcase ",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" — Theme: "),
        Span::styled(state.active_theme.name(), theme.focused_bold_style()),
    ]))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.focused_border_style())
            .title(" envision "),
    );
    frame.render_widget(header, area);
}

fn render_style_palette(_state: &State, frame: &mut Frame, area: Rect, theme: &Theme) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border_style())
        .title(" Style Palette ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Build lines showing each theme style method
    let lines: Vec<Line> = vec![
        Line::from(vec![
            Span::styled("  normal_style():          ", theme.normal_style()),
            Span::styled("Sample Text", theme.normal_style()),
        ]),
        Line::from(vec![
            Span::styled("  focused_style():         ", theme.normal_style()),
            Span::styled("Sample Text", theme.focused_style()),
        ]),
        Line::from(vec![
            Span::styled("  focused_bold_style():    ", theme.normal_style()),
            Span::styled("Sample Text", theme.focused_bold_style()),
        ]),
        Line::from(vec![
            Span::styled("  focused_border_style():  ", theme.normal_style()),
            Span::styled("Sample Text", theme.focused_border_style()),
        ]),
        Line::from(vec![
            Span::styled("  selected_style(true):    ", theme.normal_style()),
            Span::styled("Sample Text", theme.selected_style(true)),
        ]),
        Line::from(vec![
            Span::styled("  selected_style(false):   ", theme.normal_style()),
            Span::styled("Sample Text", theme.selected_style(false)),
        ]),
        Line::from(vec![
            Span::styled("  selection_style():       ", theme.normal_style()),
            Span::styled("Sample Text", theme.selection_style()),
        ]),
        Line::from(vec![
            Span::styled("  disabled_style():        ", theme.normal_style()),
            Span::styled("Sample Text", theme.disabled_style()),
        ]),
        Line::from(vec![
            Span::styled("  placeholder_style():     ", theme.normal_style()),
            Span::styled("Sample Text", theme.placeholder_style()),
        ]),
        Line::from(vec![
            Span::styled("  border_style():          ", theme.normal_style()),
            Span::styled("Sample Text", theme.border_style()),
        ]),
        Line::from(vec![
            Span::styled("  primary_style():         ", theme.normal_style()),
            Span::styled("Sample Text", theme.primary_style()),
        ]),
        Line::from(vec![
            Span::styled("  success_style():         ", theme.normal_style()),
            Span::styled("Sample Text", theme.success_style()),
        ]),
        Line::from(vec![
            Span::styled("  warning_style():         ", theme.normal_style()),
            Span::styled("Sample Text", theme.warning_style()),
        ]),
        Line::from(vec![
            Span::styled("  error_style():           ", theme.normal_style()),
            Span::styled("Sample Text", theme.error_style()),
        ]),
        Line::from(vec![
            Span::styled("  info_style():            ", theme.normal_style()),
            Span::styled("Sample Text", theme.info_style()),
        ]),
        Line::from(vec![
            Span::styled("  progress_filled_style(): ", theme.normal_style()),
            Span::styled("Sample Text", theme.progress_filled_style()),
        ]),
        // Color swatch section
        Line::from(""),
        Line::from(Span::styled(
            "  Color Fields:",
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(vec![
            Span::styled(
                "  background  ",
                Style::default().bg(theme.background).fg(theme.foreground),
            ),
            Span::styled("  foreground  ", Style::default().fg(theme.foreground)),
            Span::styled("  border  ", Style::default().fg(theme.border)),
        ]),
        Line::from(vec![
            Span::styled("  focused  ", Style::default().fg(theme.focused)),
            Span::styled("  selected  ", Style::default().fg(theme.selected)),
            Span::styled("  disabled  ", Style::default().fg(theme.disabled)),
        ]),
        Line::from(vec![
            Span::styled("  primary  ", Style::default().fg(theme.primary)),
            Span::styled("  success  ", Style::default().fg(theme.success)),
            Span::styled("  warning  ", Style::default().fg(theme.warning)),
            Span::styled("  error  ", Style::default().fg(theme.error)),
        ]),
        Line::from(vec![
            Span::styled("  info  ", Style::default().fg(theme.info)),
            Span::styled("  placeholder  ", Style::default().fg(theme.placeholder)),
            Span::styled("  progress  ", Style::default().fg(theme.progress_filled)),
        ]),
    ];

    let palette = Paragraph::new(lines).style(theme.normal_style());
    frame.render_widget(palette, inner);
}

fn render_footer(frame: &mut Frame, area: Rect, theme: &Theme) {
    let footer = Paragraph::new(Line::from(vec![
        Span::styled("[Ctrl+T]", theme.info_style()),
        Span::styled(" Theme  ", theme.normal_style()),
        Span::styled("[Tab]", theme.info_style()),
        Span::styled(" Panel  ", theme.normal_style()),
        Span::styled("[Up/Dn]", theme.info_style()),
        Span::styled(" Scroll  ", theme.normal_style()),
        Span::styled("[PgUp/PgDn]", theme.info_style()),
        Span::styled(" Page  ", theme.normal_style()),
        Span::styled("[Esc]", theme.error_style()),
        Span::styled(" Quit", theme.normal_style()),
    ]))
    .alignment(Alignment::Center)
    .style(theme.normal_style());
    frame.render_widget(footer, area);
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> envision::Result<()> {
    let _final_state = TerminalRuntime::<StylingShowcaseApp>::new_terminal()?
        .run_terminal()
        .await?;
    Ok(())
}
