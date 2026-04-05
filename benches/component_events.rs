//! Benchmarks for component event processing hot path.
//!
//! Measures handle_event() (Event -> Message mapping) and dispatch_event()
//! (Event -> Message -> State update) for SelectableList, Table, InputField,
//! and TextArea components with various data sizes.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use envision::component::{
    Column, Component, InputField, InputFieldState, SelectableList, SelectableListState, Table,
    TableRow, TableState, TextArea, TextAreaState, ViewContext,
};
use envision::input::{Event, KeyCode};
use ratatui::layout::Constraint;

// ========================================
// Table Row Type
// ========================================

#[derive(Clone)]
struct BenchRow {
    id: u32,
    name: String,
    email: String,
}

impl TableRow for BenchRow {
    fn cells(&self) -> Vec<String> {
        vec![self.id.to_string(), self.name.clone(), self.email.clone()]
    }
}

fn make_bench_rows(count: u32) -> Vec<BenchRow> {
    (0..count)
        .map(|i| BenchRow {
            id: i,
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
        })
        .collect()
}

fn make_table_columns() -> Vec<Column> {
    vec![
        Column::new("ID", Constraint::Length(6)),
        Column::new("Name", Constraint::Length(20)),
        Column::new("Email", Constraint::Length(30)),
    ]
}

// ========================================
// handle_event Benchmarks
// ========================================

fn bench_handle_event(c: &mut Criterion) {
    let mut group = c.benchmark_group("handle_event");

    for size in [100, 1000] {
        // ---- SelectableList ----
        let items: Vec<String> = (0..size).map(|i| format!("Item {}", i)).collect();
        let event = Event::key(KeyCode::Down);

        // Focused (processes the event)
        let state = SelectableListState::new(items.clone());
        group.bench_with_input(
            BenchmarkId::new("selectable_list/focused", size),
            &size,
            |b, _| {
                b.iter(|| {
                    SelectableList::<String>::handle_event(
                        black_box(&state),
                        black_box(&event),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // Unfocused (early return guard check)
        group.bench_with_input(
            BenchmarkId::new("selectable_list/unfocused", size),
            &size,
            |b, _| {
                b.iter(|| {
                    SelectableList::<String>::handle_event(
                        black_box(&state),
                        black_box(&event),
                        &ViewContext::default(),
                    )
                })
            },
        );

        // ---- Table ----
        let rows = make_bench_rows(size as u32);
        let columns = make_table_columns();
        let event = Event::key(KeyCode::Down);

        // Focused
        let state = TableState::new(rows.clone(), columns.clone());
        group.bench_with_input(BenchmarkId::new("table/focused", size), &size, |b, _| {
            b.iter(|| {
                Table::<BenchRow>::handle_event(
                    black_box(&state),
                    black_box(&event),
                    &ViewContext::new().focused(true),
                )
            })
        });

        // Unfocused
        group.bench_with_input(BenchmarkId::new("table/unfocused", size), &size, |b, _| {
            b.iter(|| {
                Table::<BenchRow>::handle_event(
                    black_box(&state),
                    black_box(&event),
                    &ViewContext::default(),
                )
            })
        });

        // ---- TextArea ----
        // Build multi-line content with `size` lines
        let content: String = (0..size)
            .map(|i| format!("Line {} with some text content", i))
            .collect::<Vec<_>>()
            .join("\n");
        let event = Event::key(KeyCode::Down);

        // Focused
        let state = TextAreaState::new().with_value(&content);
        group.bench_with_input(
            BenchmarkId::new("text_area/focused", size),
            &size,
            |b, _| {
                b.iter(|| {
                    TextArea::handle_event(
                        black_box(&state),
                        black_box(&event),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // Unfocused
        group.bench_with_input(
            BenchmarkId::new("text_area/unfocused", size),
            &size,
            |b, _| {
                b.iter(|| {
                    TextArea::handle_event(
                        black_box(&state),
                        black_box(&event),
                        &ViewContext::default(),
                    )
                })
            },
        );
    }

    // ---- InputField (different text lengths) ----
    for size in [100, 1000] {
        let text: String = "a".repeat(size);

        // Insert event (focused)
        let event_insert = Event::char('x');
        let mut state = InputFieldState::with_value(&text);
        // Place cursor in the middle for realistic benchmarking
        state.set_cursor_position(size / 2);

        group.bench_with_input(
            BenchmarkId::new("input_field/focused/insert", size),
            &size,
            |b, _| {
                b.iter(|| {
                    InputField::handle_event(
                        black_box(&state),
                        black_box(&event_insert),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // Backspace event (focused)
        let event_backspace = Event::key(KeyCode::Backspace);
        group.bench_with_input(
            BenchmarkId::new("input_field/focused/backspace", size),
            &size,
            |b, _| {
                b.iter(|| {
                    InputField::handle_event(
                        black_box(&state),
                        black_box(&event_backspace),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // Unfocused (early return)
        group.bench_with_input(
            BenchmarkId::new("input_field/unfocused", size),
            &size,
            |b, _| {
                b.iter(|| {
                    InputField::handle_event(
                        black_box(&state),
                        black_box(&event_insert),
                        &ViewContext::default(),
                    )
                })
            },
        );
    }

    group.finish();
}

// ========================================
// dispatch_event Benchmarks
// ========================================

fn bench_dispatch_event(c: &mut Criterion) {
    let mut group = c.benchmark_group("dispatch_event");

    for size in [100, 1000] {
        // ---- SelectableList ----
        let items: Vec<String> = (0..size).map(|i| format!("Item {}", i)).collect();
        let event = Event::key(KeyCode::Down);

        group.bench_with_input(
            BenchmarkId::new("selectable_list/focused", size),
            &size,
            |b, _| {
                let mut state = SelectableListState::new(items.clone());
                b.iter(|| {
                    SelectableList::<String>::dispatch_event(
                        black_box(&mut state),
                        black_box(&event),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // ---- Table ----
        let rows = make_bench_rows(size as u32);
        let columns = make_table_columns();
        let event = Event::key(KeyCode::Down);

        group.bench_with_input(BenchmarkId::new("table/focused", size), &size, |b, _| {
            let mut state = TableState::new(rows.clone(), columns.clone());
            b.iter(|| {
                Table::<BenchRow>::dispatch_event(
                    black_box(&mut state),
                    black_box(&event),
                    &ViewContext::new().focused(true),
                )
            })
        });

        // ---- TextArea ----
        let content: String = (0..size)
            .map(|i| format!("Line {} with some text content", i))
            .collect::<Vec<_>>()
            .join("\n");
        let event_down = Event::key(KeyCode::Down);

        group.bench_with_input(
            BenchmarkId::new("text_area/focused", size),
            &size,
            |b, _| {
                let mut state = TextAreaState::new().with_value(&content);
                // Start at top so Down always moves
                state.set_cursor_position(0, 0);
                b.iter(|| {
                    TextArea::dispatch_event(
                        black_box(&mut state),
                        black_box(&event_down),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );
    }

    // ---- InputField (insert and backspace with different text lengths) ----
    for size in [100, 1000] {
        let text: String = "a".repeat(size);

        // Insert dispatch (char insertion mutates state each iteration)
        let event_insert = Event::char('x');
        group.bench_with_input(
            BenchmarkId::new("input_field/focused/insert", size),
            &size,
            |b, _| {
                let mut state = InputFieldState::with_value(&text);
                state.set_cursor_position(size / 2);
                b.iter(|| {
                    InputField::dispatch_event(
                        black_box(&mut state),
                        black_box(&event_insert),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );

        // Backspace dispatch
        let event_backspace = Event::key(KeyCode::Backspace);
        group.bench_with_input(
            BenchmarkId::new("input_field/focused/backspace", size),
            &size,
            |b, _| {
                let mut state = InputFieldState::with_value(&text);
                state.set_cursor_position(size / 2);
                b.iter(|| {
                    InputField::dispatch_event(
                        black_box(&mut state),
                        black_box(&event_backspace),
                        &ViewContext::new().focused(true),
                    )
                })
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_handle_event, bench_dispatch_event,);

criterion_main!(benches);
