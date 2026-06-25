//! Drilldown example — master+detail pattern with selection preservation.
//!
//! Demonstrates the in-state-enum approach to navigation (Screen enum)
//! as the lightweight alternative to Router for cases where you don't
//! need a history stack. The user lands on the Roster screen (a Table
//! of operations), presses Enter on a selected row to drill into the
//! PerOp detail screen, and presses Esc to return to the Roster with
//! the original selection preserved.
//!
//! Per-view KeyHints + state-aware event handling: the bottom row of
//! each screen renders a `KeyHints` bar listing only the keys active
//! on that screen, and `handle_event_with_state` gates Up/Down to the
//! Roster (no roster-selection ticks while drilled in) and Esc to the
//! PerOp (no drill-out attempts from the Roster). `q` quits from any
//! screen.
//!
//! Surface exercised:
//! - TableState<Operation> for the Roster
//! - PaneLayout::view_with for the PerOp split (header + body)
//! - styled_line + InlineStyle for emphasized metrics
//! - PaneConfig::with_title_style for the PerOp header pane
//! - KeyHints for per-view affordance hints
//! - App::handle_event_with_state for screen-gated key bindings
//!
//! Compare with examples/router.rs (history-stack navigation via
//! RouterState). See src/component/router/mod.rs module docs for
//! "When to use Router vs an in-state enum".
//!
//! Run with: cargo run --example drilldown --all-features

use envision::prelude::*;

/// One operation row in the Roster.
#[derive(Clone, Debug, PartialEq)]
struct Operation {
    id: String,
    duration_ms: f64,
    status: String,
}

impl TableRow for Operation {
    fn cells(&self) -> Vec<Cell> {
        vec![
            Cell::from(self.id.clone()),
            Cell::from(format!("{:.1} ms", self.duration_ms)),
            Cell::from(self.status.clone()),
        ]
    }
}

/// Application screen state.
#[derive(Clone, Debug)]
enum Screen {
    Roster,
    PerOp { selected: usize },
}

struct DrillApp;

#[derive(Clone)]
struct State {
    screen: Screen,
    roster: TableState<Operation>,
    operations: Vec<Operation>,
    roster_hints: KeyHintsState,
    perop_hints: KeyHintsState,
}

#[derive(Clone, Debug)]
enum Msg {
    DrillIn,
    DrillOut,
    SelectNext,
    SelectPrev,
    Quit,
}

/// Carve out the bottom row for KeyHints; return (main_area, hints_area).
fn split_hints_row(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);
    (chunks[0], chunks[1])
}

impl App for DrillApp {
    type State = State;
    type Message = Msg;
    type Args = ();

    fn init(_args: ()) -> (State, Command<Msg>) {
        let operations = vec![
            Operation {
                id: "op-001".into(),
                duration_ms: 12.4,
                status: "ok".into(),
            },
            Operation {
                id: "op-002".into(),
                duration_ms: 837.2,
                status: "slow".into(),
            },
            Operation {
                id: "op-003".into(),
                duration_ms: 4.1,
                status: "ok".into(),
            },
        ];
        let columns = vec![
            Column::new("ID", Constraint::Length(12)),
            Column::new("Duration", Constraint::Length(14)),
            Column::new("Status", Constraint::Min(10)),
        ];
        let mut roster = TableState::new(operations.clone(), columns);
        roster.set_selected(Some(0));

        let roster_hints = KeyHintsState::new()
            .hint("↑/↓", "select")
            .hint("Enter", "open")
            .hint("q", "quit");
        let perop_hints = KeyHintsState::new().hint("Esc", "back").hint("q", "quit");

        (
            State {
                screen: Screen::Roster,
                roster,
                operations,
                roster_hints,
                perop_hints,
            },
            Command::none(),
        )
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::DrillIn => {
                if let Some(idx) = state.roster.selected() {
                    state.screen = Screen::PerOp { selected: idx };
                }
            }
            Msg::DrillOut => {
                if let Screen::PerOp { selected } = state.screen {
                    state.roster.set_selected(Some(selected));
                    state.screen = Screen::Roster;
                }
            }
            Msg::SelectNext => {
                let next = state
                    .roster
                    .selected()
                    .map(|i| i + 1)
                    .unwrap_or(0)
                    .min(state.operations.len().saturating_sub(1));
                state.roster.set_selected(Some(next));
            }
            Msg::SelectPrev => {
                let prev = state.roster.selected().unwrap_or(0).saturating_sub(1);
                state.roster.set_selected(Some(prev));
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        use envision::component::pane_layout::{
            PaneConfig, PaneDirection, PaneLayout, PaneLayoutState,
        };
        let area = frame.area();
        let theme = Theme::default();
        let (main_area, hints_area) = split_hints_row(area);

        match &state.screen {
            Screen::Roster => {
                let mut ctx = RenderContext::new(frame, main_area, &theme).focused(true);
                <Table<Operation> as Component>::view(&state.roster, &mut ctx);
                let mut hints_ctx = RenderContext::new(frame, hints_area, &theme).focused(true);
                <KeyHints as Component>::view(&state.roster_hints, &mut hints_ctx);
            }
            Screen::PerOp { selected } => {
                use envision::component::styled_text::StyledInline;
                use envision::render::styled_line;

                let op = &state.operations[*selected];
                let layout = PaneLayoutState::new(
                    PaneDirection::Vertical,
                    vec![
                        PaneConfig::new("header")
                            .with_title(format!(" {} ", op.id))
                            .with_title_style(
                                Style::default()
                                    .add_modifier(Modifier::BOLD)
                                    .fg(Color::Cyan),
                            )
                            .with_proportion(0.25),
                        PaneConfig::new("body")
                            .with_title(" Details ")
                            .with_proportion(0.75),
                    ],
                );

                PaneLayout::view_with(
                    &layout,
                    &mut RenderContext::new(frame, main_area, &theme).focused(true),
                    |pane_id, child_ctx| match pane_id {
                        "header" => {
                            let inlines = vec![
                                StyledInline::Plain("duration: ".to_string()),
                                StyledInline::bold(format!("{:.1} ms", op.duration_ms)),
                            ];
                            styled_line(child_ctx.frame, child_ctx.area, &inlines, child_ctx.theme);
                        }
                        "body" => {
                            let inlines = vec![
                                StyledInline::Plain("status: ".to_string()),
                                StyledInline::colored(op.status.clone(), Color::Green),
                            ];
                            styled_line(child_ctx.frame, child_ctx.area, &inlines, child_ctx.theme);
                        }
                        _ => {}
                    },
                );

                let mut hints_ctx = RenderContext::new(frame, hints_area, &theme).focused(true);
                <KeyHints as Component>::view(&state.perop_hints, &mut hints_ctx);
            }
        }
    }

    /// State-aware key handling: each screen owns only the keys it
    /// advertises. Up/Down moves the Roster selection only when the
    /// Roster is the active screen; Esc returns to the Roster only
    /// from PerOp. `q` quits from any screen.
    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        let key = event.as_key()?;
        if matches!(key.code, Key::Char('q')) {
            return Some(Msg::Quit);
        }
        match state.screen {
            Screen::Roster => match key.code {
                Key::Down => Some(Msg::SelectNext),
                Key::Up => Some(Msg::SelectPrev),
                Key::Enter => Some(Msg::DrillIn),
                _ => None,
            },
            Screen::PerOp { .. } => match key.code {
                Key::Esc => Some(Msg::DrillOut),
                _ => None,
            },
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 16 rows: ~13 for the main content + 1 for hints + table chrome.
    let mut vt = Runtime::<DrillApp, _>::virtual_builder(60, 16).build()?;

    println!("=== Drilldown Example ===\n");

    vt.tick()?;
    println!("Roster (initial):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::SelectNext);
    vt.tick()?;
    println!("After selecting row 1:");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::DrillIn);
    vt.tick()?;
    println!("After Enter (PerOp detail for op-002, PerOp hints active):");
    println!("{}\n", vt.display());

    vt.dispatch(Msg::DrillOut);
    vt.tick()?;
    println!("After Esc (back to Roster, selection preserved, Roster hints restored):");
    println!("{}\n", vt.display());

    Ok(())
}
