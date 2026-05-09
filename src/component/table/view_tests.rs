use super::*;

// Test row type (shared with tests.rs)
#[derive(Clone, Debug, PartialEq)]
struct TestRow {
    name: String,
    value: String,
}

impl TestRow {
    fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

impl TableRow for TestRow {
    fn cells(&self) -> Vec<crate::component::cell::Cell> {
        use crate::component::cell::Cell;
        vec![Cell::new(&self.name), Cell::new(&self.value)]
    }
}

fn test_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ]
}

fn test_rows() -> Vec<TestRow> {
    vec![
        TestRow::new("Charlie", "30"),
        TestRow::new("Alice", "10"),
        TestRow::new("Bob", "20"),
    ]
}

#[test]
fn test_view_renders() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_header() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_sort_indicator() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            );
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}
#[test]
fn test_view_empty() {
    let state: TableState<TestRow> = TableState::new(vec![], test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_descending_sort_indicator() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Set descending sort directly
    Table::<TestRow>::update(&mut state, TableMessage::SortDesc(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_unfocused() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_multi_column_sort_indicators() {
    let mut state = TableState::new(test_rows(), test_columns());
    // Sort by name ascending (primary), then add value ascending (secondary)
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::AddSortAsc(1));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// =====================================================================
// Phase 3 Task B: Render snapshot tests for Phase 2 features.
// =====================================================================
//
// These tests pin the rendered visual output of features added in Phase 2:
//   #10  Status column visibility (rendered iff any row is non-None).
//   #11  Per-cell `CellStyle` propagation (Default/Success/Warning/Error/
//        Muted/Custom + a mixed-styles row).
//   #8   Sort indicator visibility (appears on Sort{Asc,Desc,Toggle},
//        disappears on `SortClear`).
//   #15b Render-layer half of `sort_toggle_arrow_persists_on_repeated_press`
//        — the arrow character must remain present in the header row across
//        repeated `SortToggle` dispatches.

// ---------- #10: status column visibility ----------

/// Row type whose status is governed by a per-instance field, so individual
/// fixture rows can opt into a non-`None` row status without us having to
/// invent a fresh `TableRow` impl per test.
#[derive(Clone, Debug)]
struct StatusRow {
    name: String,
    value: String,
    status: crate::component::cell::RowStatus,
}

impl StatusRow {
    fn new(name: &str, value: &str, status: crate::component::cell::RowStatus) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            status,
        }
    }
}

impl TableRow for StatusRow {
    fn cells(&self) -> Vec<crate::component::cell::Cell> {
        use crate::component::cell::Cell;
        vec![Cell::new(&self.name), Cell::new(&self.value)]
    }

    fn status(&self) -> crate::component::cell::RowStatus {
        self.status.clone()
    }
}

fn status_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ]
}

#[test]
fn snapshot_table_no_status_column_when_all_rows_status_none() {
    use crate::component::cell::RowStatus;

    // Every row returns `RowStatus::None` ⇒ no status column should render.
    // Layout must match the no-status-column variant exactly.
    let rows = vec![
        StatusRow::new("Alice", "10", RowStatus::None),
        StatusRow::new("Bob", "20", RowStatus::None),
        StatusRow::new("Carol", "30", RowStatus::None),
    ];
    let state = TableState::new(rows, status_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<StatusRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn snapshot_table_renders_status_column_when_any_row_non_none() {
    use crate::component::cell::RowStatus;

    // Mixed: one Healthy, one Warning, one None. The status column must
    // appear for ALL rows (with an empty cell for the None row).
    let rows = vec![
        StatusRow::new("Alice", "10", RowStatus::Healthy),
        StatusRow::new("Bob", "20", RowStatus::Warning),
        StatusRow::new("Carol", "30", RowStatus::None),
    ];
    let state = TableState::new(rows, status_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<StatusRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    insta::assert_snapshot!(terminal.backend().to_string());
}

// ---------- #11: per-cell `CellStyle` ----------

/// A row carrying a single user-supplied `Cell` so each per-style snapshot
/// test can construct exactly the styled cell it wants without inventing a
/// new `TableRow` impl per variant.
#[derive(Clone, Debug)]
struct StyledRow {
    cells: Vec<crate::component::cell::Cell>,
}

impl TableRow for StyledRow {
    fn cells(&self) -> Vec<crate::component::cell::Cell> {
        self.cells.clone()
    }
}

fn styled_columns() -> Vec<Column> {
    vec![
        Column::new("Name", Constraint::Length(10)).sortable(),
        Column::new("Value", Constraint::Length(10)).sortable(),
    ]
}

/// Renders a single-row table containing the given pre-built cells and
/// returns both the plain-text output (for snapshotting) and the ANSI
/// output (for color-level assertions). Centralizes the per-style
/// boilerplate.
///
/// Selection is explicitly cleared via `set_selected(None)` because
/// ratatui's row-highlight style overrides per-cell fg colors on the
/// selected row — leaving the default `Some(0)` selection in place would
/// mask the very styling these tests are meant to pin.
///
/// Plain text alone cannot distinguish a `Cell::success` from a
/// `Cell::warning` because color is stripped — the ANSI string is the
/// only place where the per-cell style is observable in test output.
fn render_styled_single_row(cells: Vec<crate::component::cell::Cell>) -> (String, String) {
    let mut state = TableState::new(vec![StyledRow { cells }], styled_columns());
    state.set_selected(None);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<StyledRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();
    (plain, ansi)
}

#[test]
fn snapshot_table_cells_with_default_style() {
    use crate::component::cell::Cell;

    // Baseline — no style overrides. Compared against the styled variants
    // below to confirm the layout is invariant under per-cell styling.
    let cells = vec![Cell::new("alpha"), Cell::new("beta")];
    let (plain, _ansi) = render_styled_single_row(cells);
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_with_success_style() {
    use crate::component::cell::Cell;

    let cells = vec![Cell::success("alpha"), Cell::new("beta")];
    let (plain, ansi) = render_styled_single_row(cells);
    // Plain-text snapshot pins layout; ANSI assertion pins the green color
    // applied by `CellStyle::Success` (default theme: success = green).
    assert!(
        ansi.contains("\x1b[32m"),
        "expected green ANSI fg for Success-styled cell, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_with_warning_style() {
    use crate::component::cell::Cell;

    let cells = vec![Cell::warning("alpha"), Cell::new("beta")];
    let (plain, ansi) = render_styled_single_row(cells);
    // Default theme: warning = yellow.
    assert!(
        ansi.contains("\x1b[33m"),
        "expected yellow ANSI fg for Warning-styled cell, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_with_error_style() {
    use crate::component::cell::Cell;

    let cells = vec![Cell::error("alpha"), Cell::new("beta")];
    let (plain, ansi) = render_styled_single_row(cells);
    // Default theme: error = red.
    assert!(
        ansi.contains("\x1b[31m"),
        "expected red ANSI fg for Error-styled cell, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_with_muted_style() {
    use crate::component::cell::Cell;

    let cells = vec![Cell::muted("alpha"), Cell::new("beta")];
    let (plain, ansi) = render_styled_single_row(cells);
    // Muted is hardcoded to dark gray in render.rs (the theme has no
    // dedicated muted accessor at present).
    assert!(
        ansi.contains("\x1b[90m"),
        "expected dark-gray ANSI fg for Muted-styled cell, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_with_custom_style() {
    use crate::component::cell::{Cell, CellStyle};
    use ratatui::style::{Color, Style};

    let custom =
        Cell::new("alpha").with_style(CellStyle::Custom(Style::default().fg(Color::Magenta)));
    let cells = vec![custom, Cell::new("beta")];
    let (plain, ansi) = render_styled_single_row(cells);
    // Custom(Style::default().fg(Magenta)) → magenta ANSI fg = \x1b[35m.
    assert!(
        ansi.contains("\x1b[35m"),
        "expected magenta ANSI fg for Custom-styled cell, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

#[test]
fn snapshot_table_cells_mixed_styles_in_one_row() {
    use crate::component::cell::Cell;

    // First column uses Cell::success, second uses Cell::error — and we
    // override the column count to fit three styled cells in a single row
    // for full coverage.
    let columns = vec![
        Column::new("S", Constraint::Length(8)),
        Column::new("E", Constraint::Length(8)),
        Column::new("M", Constraint::Length(8)),
    ];
    let row = StyledRow {
        cells: vec![
            Cell::success("ok"),
            Cell::error("fail"),
            Cell::muted("idle"),
        ],
    };
    let mut state = TableState::new(vec![row], columns);
    // See `render_styled_single_row` — clearing selection prevents the
    // row-highlight style from overriding per-cell fg colors.
    state.set_selected(None);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<StyledRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();
    // All three semantic colors must appear in the rendered ANSI output.
    assert!(
        ansi.contains("\x1b[32m"),
        "expected green (Success) in mixed row, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[31m"),
        "expected red (Error) in mixed row, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[90m"),
        "expected dark-gray (Muted) in mixed row, got:\n{ansi}",
    );
    insta::assert_snapshot!(plain);
}

// ---------- #8: sort indicator visibility ----------

#[test]
fn snapshot_table_sort_indicator_appears_on_sort_asc() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let rendered = terminal.backend().to_string();
    // Sanity assert above the snapshot: the ascending arrow MUST appear
    // somewhere in the rendered output. Pinning by name protects this
    // against future cleanup that strips the indicator silently.
    assert!(
        rendered.contains('\u{2191}'),
        "expected ascending sort indicator (↑) to appear in rendered output:\n{rendered}",
    );
    insta::assert_snapshot!(rendered);
}

#[test]
fn snapshot_table_sort_indicator_disappears_on_sort_clear() {
    let mut state = TableState::new(test_rows(), test_columns());
    Table::<TestRow>::update(&mut state, TableMessage::SortAsc(0));
    Table::<TestRow>::update(&mut state, TableMessage::SortClear);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let rendered = terminal.backend().to_string();
    // Sanity assert: NEITHER arrow may appear after `SortClear`.
    assert!(
        !rendered.contains('\u{2191}') && !rendered.contains('\u{2193}'),
        "expected no sort indicator after SortClear, but rendered output contained one:\n{rendered}",
    );
    insta::assert_snapshot!(rendered);
}

// ---------- #15b: render-layer assertion for repeated SortToggle ----------

/// The render-layer half of test #15. Dispatches `SortToggle(0)` ten times
/// and, after each dispatch, asserts that the rendered header row contains
/// a sort indicator character. This is an assertion-based test (no snapshot)
/// because the *direction* alternates between iterations, so a single
/// pinned snapshot would not cover all states; what we want is the
/// invariant "the indicator never disappears".
///
/// Pinned by name so future cleanup of the cell-unification work can't
/// silently regress this end-user-visible behavior.
#[test]
fn sort_toggle_arrow_persists_on_repeated_press() {
    let mut state = TableState::new(test_rows(), test_columns());

    for i in 0..10 {
        let _ = Table::<TestRow>::update(&mut state, TableMessage::SortToggle(0));

        let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
        terminal
            .draw(|frame| {
                Table::<TestRow>::view(
                    &state,
                    &mut RenderContext::new(frame, frame.area(), &theme),
                );
            })
            .unwrap();

        let rendered = terminal.backend().to_string();
        let has_arrow = rendered.contains('\u{2191}') || rendered.contains('\u{2193}');
        assert!(
            has_arrow,
            "iteration {i}: sort indicator missing from rendered output:\n{rendered}",
        );
    }
}

#[test]
fn test_view_no_outer_border_when_chrome_owned() {
    let state = TableState::new(test_rows(), test_columns());

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 8);

    terminal
        .draw(|frame| {
            Table::<TestRow>::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).chrome_owned(true),
            );
        })
        .unwrap();

    let display = terminal.backend().to_string();
    insta::assert_snapshot!("view_chrome_owned_no_outer_border", display);
}

#[test]
fn snapshot_table_cells_with_severity_style() {
    use crate::component::cell::{Cell, CellStyle};
    use crate::theme::Severity;

    // Render four severity bands in one row. Default theme: Good=Green,
    // Mild=Yellow, Bad=Yellow (collapses with Mild on Default — documented
    // behavior from D6+D9), Critical=Red+BOLD.
    let columns = vec![
        Column::new("G", Constraint::Length(8)),
        Column::new("M", Constraint::Length(8)),
        Column::new("B", Constraint::Length(8)),
        Column::new("C", Constraint::Length(8)),
    ];
    let cells = vec![
        Cell::new("good").with_style(CellStyle::Severity(Severity::Good)),
        Cell::new("mild").with_style(CellStyle::Severity(Severity::Mild)),
        Cell::new("bad").with_style(CellStyle::Severity(Severity::Bad)),
        Cell::new("crit").with_style(CellStyle::Severity(Severity::Critical)),
    ];
    let mut state = TableState::new(vec![StyledRow { cells }], columns);
    state.set_selected(None);

    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Table::<StyledRow>::view(&state, &mut RenderContext::new(frame, frame.area(), &theme));
        })
        .unwrap();

    let plain = terminal.backend().to_string();
    let ansi = terminal.backend().to_ansi();

    // ANSI assertions pin per-band coloring. Default theme: Mild and Bad
    // both render as Color::Yellow (\x1b[33m) — the documented collapse
    // from D6+D9 means severity bands degrade from four to three on
    // Default. Critical stays distinguishable via BOLD.
    assert!(
        ansi.contains("\x1b[32m"),
        "expected green (32m) for Severity::Good, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[33m"),
        "expected yellow (33m) for Severity::Mild and Severity::Bad, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[31m"),
        "expected red (31m) for Severity::Critical, got:\n{ansi}",
    );
    assert!(
        ansi.contains("\x1b[1m"),
        "expected BOLD (1m) for Severity::Critical, got:\n{ansi}",
    );

    insta::assert_snapshot!(plain);
}
