//! Sort benchmark: 10k × 10 column primary sort wall-time gate.
//!
//! Records a baseline before merge so post-merge regressions are visible.
//! The benchmark builds 10,000 rows of 10 columns (one `f64` plus nine
//! `i64`) and times a single primary-column sort. Setup time (cloning
//! rows + columns into fresh `TableState`) is excluded from the
//! measurement via `iter_with_setup`.

use criterion::{Criterion, criterion_group, criterion_main};
use envision::component::cell::Cell;
use envision::component::{Column, Component, Table, TableMessage, TableRow, TableState};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use ratatui::layout::Constraint;

#[derive(Clone)]
struct Row(f64, [i64; 9]);

impl TableRow for Row {
    fn cells(&self) -> Vec<Cell> {
        let mut v = Vec::with_capacity(10);
        v.push(Cell::number(self.0));
        for x in &self.1 {
            v.push(Cell::int(*x));
        }
        v
    }
}

fn bench_sort_10k_rows(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let rows: Vec<Row> = (0..10_000)
        .map(|_| {
            Row(
                rng.random_range(0.0_f64..1000.0),
                std::array::from_fn(|_| rng.random()),
            )
        })
        .collect();
    let columns: Vec<Column> = (0..10)
        .map(|i| Column::new(format!("C{}", i), Constraint::Length(8)).sortable())
        .collect();

    c.bench_function("sort_10k_rows_f64", |b| {
        b.iter_with_setup(
            || TableState::new(rows.clone(), columns.clone()),
            |mut state| {
                let _ = Table::<Row>::update(&mut state, TableMessage::SortAsc(0));
            },
        );
    });
}

criterion_group!(benches, bench_sort_10k_rows);
criterion_main!(benches);
