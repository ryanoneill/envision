//! Benchmarks for CaptureBackend performance.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use envision::backend::CaptureBackend;
use ratatui::backend::Backend;
use ratatui::buffer::Cell;
use ratatui::layout::Position;
use ratatui::style::{Color, Modifier, Style};

/// Benchmark creating new backends of various sizes.
fn bench_backend_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("backend_creation");

    for (width, height) in [(80, 24), (120, 40), (200, 60), (320, 100)] {
        group.throughput(Throughput::Elements((width * height) as u64));
        group.bench_with_input(
            BenchmarkId::new("new", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| CaptureBackend::new(black_box(w), black_box(h)));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_history", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| CaptureBackend::with_history(black_box(w), black_box(h), 10));
            },
        );
    }

    group.finish();
}

/// Benchmark drawing operations.
fn bench_draw(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw");

    for (width, height) in [(80, 24), (200, 60)] {
        let cell_count = (width * height) as usize;
        group.throughput(Throughput::Elements(cell_count as u64));

        // Prepare cells for drawing
        let cells: Vec<(u16, u16, Cell)> = (0..height)
            .flat_map(|y| {
                (0..width).map(move |x| {
                    let mut cell = Cell::default();
                    cell.set_char('X');
                    (x, y, cell)
                })
            })
            .collect();

        group.bench_with_input(
            BenchmarkId::new("full_screen", format!("{}x{}", width, height)),
            &cells,
            |b, cells| {
                let mut backend = CaptureBackend::new(width, height);
                b.iter(|| {
                    let content = cells.iter().map(|(x, y, c)| (*x, *y, c));
                    backend.draw(content).unwrap();
                });
            },
        );

        // Sparse update (10% of cells)
        let sparse_cells: Vec<_> = cells.iter().step_by(10).cloned().collect();
        group.bench_with_input(
            BenchmarkId::new("sparse_10pct", format!("{}x{}", width, height)),
            &sparse_cells,
            |b, cells| {
                let mut backend = CaptureBackend::new(width, height);
                b.iter(|| {
                    let content = cells.iter().map(|(x, y, c)| (*x, *y, c));
                    backend.draw(content).unwrap();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark styled drawing operations.
fn bench_draw_styled(c: &mut Criterion) {
    let mut group = c.benchmark_group("draw_styled");

    let (width, height) = (80, 24);
    let cell_count = (width * height) as usize;
    group.throughput(Throughput::Elements(cell_count as u64));

    // Plain cells
    let plain_cells: Vec<(u16, u16, Cell)> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                let mut cell = Cell::default();
                cell.set_char('A');
                (x, y, cell)
            })
        })
        .collect();

    // Colored cells
    let colored_cells: Vec<(u16, u16, Cell)> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                let mut cell = Cell::default();
                cell.set_char('C');
                cell.set_style(Style::default().fg(Color::Red).bg(Color::Blue));
                (x, y, cell)
            })
        })
        .collect();

    // Styled cells (with modifiers)
    let styled_cells: Vec<(u16, u16, Cell)> = (0..height)
        .flat_map(|y| {
            (0..width).map(move |x| {
                let mut cell = Cell::default();
                cell.set_char('S');
                cell.set_style(
                    Style::default()
                        .fg(Color::Rgb(255, 128, 64))
                        .bg(Color::Indexed(42))
                        .add_modifier(Modifier::BOLD | Modifier::ITALIC),
                );
                (x, y, cell)
            })
        })
        .collect();

    group.bench_with_input(BenchmarkId::new("plain", "80x24"), &plain_cells, |b, cells| {
        let mut backend = CaptureBackend::new(width, height);
        b.iter(|| {
            let content = cells.iter().map(|(x, y, c)| (*x, *y, c));
            backend.draw(content).unwrap();
        });
    });

    group.bench_with_input(
        BenchmarkId::new("colored", "80x24"),
        &colored_cells,
        |b, cells| {
            let mut backend = CaptureBackend::new(width, height);
            b.iter(|| {
                let content = cells.iter().map(|(x, y, c)| (*x, *y, c));
                backend.draw(content).unwrap();
            });
        },
    );

    group.bench_with_input(
        BenchmarkId::new("fully_styled", "80x24"),
        &styled_cells,
        |b, cells| {
            let mut backend = CaptureBackend::new(width, height);
            b.iter(|| {
                let content = cells.iter().map(|(x, y, c)| (*x, *y, c));
                backend.draw(content).unwrap();
            });
        },
    );

    group.finish();
}

/// Benchmark clear operations.
fn bench_clear(c: &mut Criterion) {
    let mut group = c.benchmark_group("clear");

    for (width, height) in [(80, 24), (200, 60)] {
        group.throughput(Throughput::Elements((width * height) as u64));

        group.bench_with_input(
            BenchmarkId::new("full", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                let mut backend = CaptureBackend::new(w, h);
                b.iter(|| backend.clear().unwrap());
            },
        );
    }

    group.finish();
}

/// Benchmark cursor operations.
fn bench_cursor(c: &mut Criterion) {
    let mut group = c.benchmark_group("cursor");

    let mut backend = CaptureBackend::new(80, 24);

    group.bench_function("get_position", |b| {
        b.iter(|| backend.get_cursor_position().unwrap());
    });

    group.bench_function("set_position", |b| {
        b.iter(|| backend.set_cursor_position(Position::new(40, 12)).unwrap());
    });

    group.bench_function("hide", |b| {
        b.iter(|| backend.hide_cursor().unwrap());
    });

    group.bench_function("show", |b| {
        b.iter(|| backend.show_cursor().unwrap());
    });

    group.finish();
}

/// Benchmark flush and history operations.
fn bench_flush(c: &mut Criterion) {
    let mut group = c.benchmark_group("flush");

    group.bench_function("no_history", |b| {
        let mut backend = CaptureBackend::new(80, 24);
        b.iter(|| backend.flush().unwrap());
    });

    group.bench_function("with_history_10", |b| {
        let mut backend = CaptureBackend::with_history(80, 24, 10);
        b.iter(|| backend.flush().unwrap());
    });

    group.bench_function("with_history_100", |b| {
        let mut backend = CaptureBackend::with_history(80, 24, 100);
        b.iter(|| backend.flush().unwrap());
    });

    // Benchmark with history at capacity (requires eviction)
    group.bench_function("history_eviction", |b| {
        let mut backend = CaptureBackend::with_history(80, 24, 5);
        // Fill history to capacity
        for _ in 0..5 {
            backend.flush().unwrap();
        }
        b.iter(|| backend.flush().unwrap());
    });

    group.finish();
}

/// Benchmark snapshot creation.
fn bench_snapshot(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot");

    for (width, height) in [(80, 24), (200, 60)] {
        group.throughput(Throughput::Elements((width * height) as u64));

        let backend = CaptureBackend::new(width, height);

        group.bench_with_input(
            BenchmarkId::new("create", format!("{}x{}", width, height)),
            &backend,
            |b, backend| {
                b.iter(|| backend.snapshot());
            },
        );
    }

    group.finish();
}

/// Benchmark diff computation.
fn bench_diff(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff");

    for (width, height) in [(80, 24), (200, 60)] {
        group.throughput(Throughput::Elements((width * height) as u64));

        // Create two snapshots with some differences
        let backend1 = CaptureBackend::new(width, height);
        let mut backend2 = CaptureBackend::new(width, height);

        // Modify ~10% of cells in backend2
        for y in 0..height {
            for x in (0..width).step_by(10) {
                if let Some(cell) = backend2.cell_mut(x, y) {
                    cell.set_char('X');
                }
            }
        }

        let snapshot1 = backend1.snapshot();
        let _snapshot2 = backend2.snapshot();

        group.bench_with_input(
            BenchmarkId::new("identical", format!("{}x{}", width, height)),
            &snapshot1,
            |b, s1| {
                b.iter(|| backend1.diff_from(s1));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("10pct_changed", format!("{}x{}", width, height)),
            &snapshot1,
            |b, s1| {
                b.iter(|| backend2.diff_from(s1));
            },
        );
    }

    group.finish();
}

/// Benchmark output rendering.
fn bench_output(c: &mut Criterion) {
    let mut group = c.benchmark_group("output");

    for (width, height) in [(80, 24), (200, 60)] {
        group.throughput(Throughput::Elements((width * height) as u64));

        // Create a backend with some content
        let mut backend = CaptureBackend::new(width, height);
        for y in 0..height {
            for (i, ch) in "Hello, World! ".chars().cycle().take(width as usize).enumerate() {
                if let Some(cell) = backend.cell_mut(i as u16, y) {
                    cell.set_char(ch);
                }
            }
        }

        group.bench_with_input(
            BenchmarkId::new("to_string", format!("{}x{}", width, height)),
            &backend,
            |b, backend| {
                b.iter(|| backend.to_string());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("to_ansi", format!("{}x{}", width, height)),
            &backend,
            |b, backend| {
                b.iter(|| backend.to_ansi());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("to_json", format!("{}x{}", width, height)),
            &backend,
            |b, backend| {
                b.iter(|| backend.to_json());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("to_json_pretty", format!("{}x{}", width, height)),
            &backend,
            |b, backend| {
                b.iter(|| backend.to_json_pretty());
            },
        );
    }

    group.finish();
}

/// Benchmark text search operations.
fn bench_text_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("text_search");

    let (width, height) = (80, 24);

    // Create a backend with repeating content
    let mut backend = CaptureBackend::new(width, height);
    for y in 0..height {
        let text = if y % 5 == 0 {
            "Button Submit Cancel Reset "
        } else {
            "Lorem ipsum dolor sit amet "
        };
        for (i, ch) in text.chars().cycle().take(width as usize).enumerate() {
            if let Some(cell) = backend.cell_mut(i as u16, y) {
                cell.set_char(ch);
            }
        }
    }

    group.bench_function("contains_text_found", |b| {
        b.iter(|| backend.contains_text(black_box("Button")));
    });

    group.bench_function("contains_text_not_found", |b| {
        b.iter(|| backend.contains_text(black_box("NotPresent")));
    });

    group.bench_function("find_text_multiple", |b| {
        b.iter(|| backend.find_text(black_box("Button")));
    });

    group.bench_function("row_content", |b| {
        b.iter(|| backend.row_content(black_box(10)));
    });

    group.bench_function("content_lines", |b| {
        b.iter(|| backend.content_lines());
    });

    group.finish();
}

/// Benchmark cell access patterns.
fn bench_cell_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("cell_access");

    let mut backend = CaptureBackend::new(80, 24);

    group.bench_function("cell_read", |b| {
        b.iter(|| backend.cell(black_box(40), black_box(12)));
    });

    group.bench_function("cell_write", |b| {
        b.iter(|| {
            if let Some(cell) = backend.cell_mut(black_box(40), black_box(12)) {
                cell.set_char('X');
            }
        });
    });

    group.bench_function("cells_slice", |b| {
        b.iter(|| backend.cells());
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_backend_creation,
    bench_draw,
    bench_draw_styled,
    bench_clear,
    bench_cursor,
    bench_flush,
    bench_snapshot,
    bench_diff,
    bench_output,
    bench_text_search,
    bench_cell_access,
);

criterion_main!(benches);
