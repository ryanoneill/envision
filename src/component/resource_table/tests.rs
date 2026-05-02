use super::*;
use crate::component::test_utils;
use ratatui::layout::Constraint;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Test helper types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
struct PodRow {
    name: String,
    status: String,
    restarts: u32,
    age: Duration,
}

impl ResourceRow for PodRow {
    fn cells(&self) -> Vec<ResourceCell> {
        vec![
            ResourceCell::new(&self.name),
            if self.status == "Running" {
                ResourceCell::success(&self.status)
            } else if self.status == "Pending" {
                ResourceCell::warning(&self.status)
            } else {
                ResourceCell::error(&self.status)
            },
            if self.restarts > 0 {
                ResourceCell::warning(self.restarts.to_string())
            } else {
                ResourceCell::new("0")
            },
            ResourceCell::age(self.age),
        ]
    }

    fn status(&self) -> RowStatus {
        match self.status.as_str() {
            "Running" => RowStatus::Healthy,
            "Pending" => RowStatus::Warning,
            _ => RowStatus::Error,
        }
    }
}

fn make_pods() -> Vec<PodRow> {
    vec![
        PodRow {
            name: "nginx-abc".to_string(),
            status: "Running".to_string(),
            restarts: 0,
            age: Duration::from_secs(192),
        },
        PodRow {
            name: "redis-def".to_string(),
            status: "CrashLoopBackOff".to_string(),
            restarts: 5,
            age: Duration::from_secs(60),
        },
        PodRow {
            name: "worker-ghi".to_string(),
            status: "Pending".to_string(),
            restarts: 0,
            age: Duration::from_secs(10),
        },
    ]
}

fn make_columns() -> Vec<ResourceColumn> {
    vec![
        ResourceColumn::new("NAME", Constraint::Length(20)),
        ResourceColumn::new("STATUS", Constraint::Length(20)),
        ResourceColumn::new("RESTARTS", Constraint::Length(10)),
        ResourceColumn::new("AGE", Constraint::Length(8)),
    ]
}

// ---------------------------------------------------------------------------
// ResourceCell
// ---------------------------------------------------------------------------

#[test]
fn test_cell_new() {
    let cell = ResourceCell::new("hello");
    assert_eq!(cell.text(), "hello");
    assert_eq!(cell.style(), &CellStyle::Default);
}

#[test]
fn test_cell_success() {
    let cell = ResourceCell::success("OK");
    assert_eq!(cell.style(), &CellStyle::Success);
}

#[test]
fn test_cell_warning() {
    let cell = ResourceCell::warning("slow");
    assert_eq!(cell.style(), &CellStyle::Warning);
}

#[test]
fn test_cell_error() {
    let cell = ResourceCell::error("FAIL");
    assert_eq!(cell.style(), &CellStyle::Error);
}

#[test]
fn test_cell_muted() {
    let cell = ResourceCell::muted("(none)");
    assert_eq!(cell.style(), &CellStyle::Muted);
}

#[test]
fn test_cell_styled() {
    let style = Style::default().fg(Color::Cyan);
    let cell = ResourceCell::styled("cyan", style);
    assert_eq!(cell.style(), &CellStyle::Custom(style));
}

// ---------------------------------------------------------------------------
// Age formatting
// ---------------------------------------------------------------------------

#[test]
fn test_age_zero() {
    assert_eq!(format_age(Duration::from_secs(0)), "0s");
}

#[test]
fn test_age_seconds() {
    assert_eq!(format_age(Duration::from_secs(45)), "45s");
    assert_eq!(format_age(Duration::from_secs(59)), "59s");
}

#[test]
fn test_age_minutes() {
    assert_eq!(format_age(Duration::from_secs(60)), "1m");
    assert_eq!(format_age(Duration::from_secs(192)), "3m12s");
    assert_eq!(format_age(Duration::from_secs(3599)), "59m59s");
}

#[test]
fn test_age_hours() {
    assert_eq!(format_age(Duration::from_secs(3600)), "1h");
    assert_eq!(format_age(Duration::from_secs(8100)), "2h15m");
    assert_eq!(format_age(Duration::from_secs(86_399)), "23h59m");
}

#[test]
fn test_age_days() {
    assert_eq!(format_age(Duration::from_secs(86_400)), "1d");
    assert_eq!(format_age(Duration::from_secs(360_000)), "4d4h");
}

#[test]
fn test_cell_age() {
    let cell = ResourceCell::age(Duration::from_secs(192));
    assert_eq!(cell.text(), "3m12s");
    assert_eq!(cell.style(), &CellStyle::Muted);
}

// ---------------------------------------------------------------------------
// CellStyle → Style mapping
// ---------------------------------------------------------------------------

#[test]
fn test_cell_style_to_style() {
    assert_eq!(
        CellStyle::Success.to_style(),
        Style::default().fg(Color::Green)
    );
    assert_eq!(
        CellStyle::Warning.to_style(),
        Style::default().fg(Color::Yellow)
    );
    assert_eq!(CellStyle::Error.to_style(), Style::default().fg(Color::Red));
    assert_eq!(
        CellStyle::Muted.to_style(),
        Style::default().fg(Color::DarkGray)
    );
    assert_eq!(CellStyle::Default.to_style(), Style::default());
}

#[test]
fn test_cell_style_custom() {
    let style = Style::default().fg(Color::Magenta);
    assert_eq!(CellStyle::Custom(style).to_style(), style);
}

// ---------------------------------------------------------------------------
// RowStatus
// ---------------------------------------------------------------------------

#[test]
fn test_row_status_default() {
    assert_eq!(RowStatus::default(), RowStatus::None);
}

#[test]
fn test_row_status_indicators() {
    assert_eq!(RowStatus::None.indicator(), None);
    assert_eq!(
        RowStatus::Healthy.indicator(),
        Some(("\u{25cf}", Color::Green))
    );
    assert_eq!(
        RowStatus::Warning.indicator(),
        Some(("\u{25b2}", Color::Yellow))
    );
    assert_eq!(RowStatus::Error.indicator(), Some(("\u{2716}", Color::Red)));
    assert_eq!(RowStatus::Unknown.indicator(), Some(("?", Color::DarkGray)));
}

#[test]
fn test_row_status_custom() {
    let status = RowStatus::Custom {
        symbol: "★",
        color: Color::Yellow,
    };
    assert_eq!(status.indicator(), Some(("★", Color::Yellow)));
}

// ---------------------------------------------------------------------------
// ResourceColumn
// ---------------------------------------------------------------------------

#[test]
fn test_column_new() {
    let col = ResourceColumn::new("NAME", Constraint::Length(20));
    assert_eq!(col.header(), "NAME");
    assert_eq!(col.width(), Constraint::Length(20));
    assert_eq!(col.alignment(), Alignment::Left);
}

#[test]
fn test_column_with_alignment() {
    let col = ResourceColumn::new("AGE", Constraint::Length(8)).with_alignment(Alignment::Right);
    assert_eq!(col.alignment(), Alignment::Right);
}

// ---------------------------------------------------------------------------
// ResourceTableState
// ---------------------------------------------------------------------------

#[test]
fn test_state_new() {
    let state: ResourceTableState<PodRow> = ResourceTableState::new(make_columns());
    assert_eq!(state.columns().len(), 4);
    assert!(state.rows().is_empty());
    assert_eq!(state.selected(), None);
    assert_eq!(state.title(), None);
}

#[test]
fn test_state_with_rows() {
    let state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    assert_eq!(state.rows().len(), 3);
}

#[test]
fn test_state_with_title() {
    let state: ResourceTableState<PodRow> =
        ResourceTableState::new(make_columns()).with_title("Pods");
    assert_eq!(state.title(), Some("Pods"));
}

#[test]
fn test_state_with_selected() {
    let state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(1));
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_state_selected_row() {
    let state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    assert_eq!(state.selected_row().unwrap().name, "worker-ghi");
}

#[test]
fn test_state_set_rows() {
    let mut state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    assert_eq!(state.rows().len(), 3);

    state.set_rows(vec![make_pods()[0].clone()]);
    assert_eq!(state.rows().len(), 1);
}

#[test]
fn test_state_set_rows_clamps_selection() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    state.set_rows(vec![make_pods()[0].clone()]);
    // Selection clamped from 2 to 0 (last remaining index)
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_state_set_rows_empty_clears_selection() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(1));
    state.set_rows(vec![]);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_state_set_selected() {
    let mut state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    state.set_selected(Some(2));
    assert_eq!(state.selected(), Some(2));
}

#[test]
fn test_state_set_selected_out_of_bounds() {
    let mut state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    state.set_selected(Some(99));
    assert_eq!(state.selected(), None);
}

#[test]
fn test_state_set_title() {
    let mut state: ResourceTableState<PodRow> = ResourceTableState::new(make_columns());
    state.set_title(Some("Updated".to_string()));
    assert_eq!(state.title(), Some("Updated"));
}

#[test]
fn test_state_has_status_column() {
    let with_status = ResourceTableState::new(make_columns()).with_rows(make_pods());
    assert!(with_status.has_status_column());

    #[derive(Clone)]
    struct Plain;
    impl ResourceRow for Plain {
        fn cells(&self) -> Vec<ResourceCell> {
            vec![ResourceCell::new("x")]
        }
    }
    let plain: ResourceTableState<Plain> =
        ResourceTableState::new(vec![ResourceColumn::new("X", Constraint::Length(5))])
            .with_rows(vec![Plain]);
    assert!(!plain.has_status_column());
}

// ---------------------------------------------------------------------------
// Navigation
// ---------------------------------------------------------------------------

#[test]
fn test_navigation_down() {
    let mut state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    assert_eq!(state.selected(), None);

    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Down);
    assert_eq!(state.selected(), Some(0));
    assert_eq!(out, Some(ResourceTableOutput::SelectionChanged(0)));

    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Down);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_navigation_down_bounded() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Down);
    assert_eq!(state.selected(), Some(2));
    assert_eq!(out, None);
}

#[test]
fn test_navigation_up() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Up);
    assert_eq!(state.selected(), Some(1));
}

#[test]
fn test_navigation_up_bounded() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(0));
    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Up);
    assert_eq!(state.selected(), Some(0));
    assert_eq!(out, None);
}

#[test]
fn test_navigation_first() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::First);
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_navigation_last() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(0));
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Last);
    assert_eq!(state.selected(), Some(2));
}

#[test]
fn test_navigation_page_up() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(2));
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::PageUp(10));
    assert_eq!(state.selected(), Some(0));
}

#[test]
fn test_navigation_page_down() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(0));
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::PageDown(10));
    assert_eq!(state.selected(), Some(2));
}

#[test]
fn test_navigation_empty_table() {
    let mut state: ResourceTableState<PodRow> = ResourceTableState::new(make_columns());
    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Down);
    assert_eq!(out, None);
    assert_eq!(state.selected(), None);
}

#[test]
fn test_select_outputs_row() {
    let mut state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(1));
    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Select);
    match out {
        Some(ResourceTableOutput::Selected(row)) => assert_eq!(row.name, "redis-def"),
        _ => panic!("expected Selected output, got {:?}", out),
    }
}

#[test]
fn test_select_with_no_selection_returns_none() {
    let mut state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    let out = ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::Select);
    assert_eq!(out, None);
}

#[test]
fn test_set_rows_message() {
    let mut state = ResourceTableState::new(make_columns());
    ResourceTable::<PodRow>::update(&mut state, ResourceTableMessage::SetRows(make_pods()));
    assert_eq!(state.rows().len(), 3);
}

// ---------------------------------------------------------------------------
// handle_event
// ---------------------------------------------------------------------------

#[test]
fn test_handle_event_not_focused() {
    let state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    let ctx = EventContext::new();
    let event = Event::char('j');
    assert_eq!(
        ResourceTable::<PodRow>::handle_event(&state, &event, &ctx),
        None
    );
}

#[test]
fn test_handle_event_focused() {
    let state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    let ctx = EventContext::new().focused(true);
    let event = Event::char('j');
    assert_eq!(
        ResourceTable::<PodRow>::handle_event(&state, &event, &ctx),
        Some(ResourceTableMessage::Down)
    );
}

#[test]
fn test_handle_event_disabled() {
    let state = ResourceTableState::new(make_columns()).with_rows(make_pods());
    let ctx = EventContext::new().focused(true).disabled(true);
    let event = Event::char('j');
    assert_eq!(
        ResourceTable::<PodRow>::handle_event(&state, &event, &ctx),
        None
    );
}

// ---------------------------------------------------------------------------
// Snapshot tests
// ---------------------------------------------------------------------------

#[test]
fn test_snapshot_empty() {
    let state: ResourceTableState<PodRow> =
        ResourceTableState::new(make_columns()).with_title("Pods");
    let (mut terminal, theme) = test_utils::setup_render(80, 10);
    terminal
        .draw(|frame| {
            ResourceTable::<PodRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_mixed_status() {
    let state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_title("Pods (mixed)");
    let (mut terminal, theme) = test_utils::setup_render(80, 10);
    terminal
        .draw(|frame| {
            ResourceTable::<PodRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_with_selection() {
    let state = ResourceTableState::new(make_columns())
        .with_rows(make_pods())
        .with_selected(Some(1))
        .with_title("Pods");
    let (mut terminal, theme) = test_utils::setup_render(80, 10);
    terminal
        .draw(|frame| {
            let mut ctx = RenderContext::new(frame, frame.area(), &theme);
            ctx.focused = true;
            ResourceTable::<PodRow>::view(&state, &mut ctx);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_all_healthy() {
    let rows: Vec<PodRow> = (0..4)
        .map(|i| PodRow {
            name: format!("api-{}", i),
            status: "Running".to_string(),
            restarts: 0,
            age: Duration::from_secs(60 * (i + 1) as u64),
        })
        .collect();
    let state = ResourceTableState::new(make_columns())
        .with_rows(rows)
        .with_title("All Healthy");
    let (mut terminal, theme) = test_utils::setup_render(80, 10);
    terminal
        .draw(|frame| {
            ResourceTable::<PodRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_snapshot_no_status_column() {
    #[derive(Clone)]
    struct PlainRow {
        col1: String,
        col2: String,
    }
    impl ResourceRow for PlainRow {
        fn cells(&self) -> Vec<ResourceCell> {
            vec![ResourceCell::new(&self.col1), ResourceCell::new(&self.col2)]
        }
    }

    let cols = vec![
        ResourceColumn::new("COL1", Constraint::Length(10)),
        ResourceColumn::new("COL2", Constraint::Length(10)),
    ];
    let rows = vec![
        PlainRow {
            col1: "a".into(),
            col2: "b".into(),
        },
        PlainRow {
            col1: "c".into(),
            col2: "d".into(),
        },
    ];
    let state = ResourceTableState::new(cols)
        .with_rows(rows)
        .with_title("Plain");
    let (mut terminal, theme) = test_utils::setup_render(40, 8);
    terminal
        .draw(|frame| {
            ResourceTable::<PlainRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme),
            );
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
