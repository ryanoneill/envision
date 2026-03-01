//! Benchmarks for component view() rendering performance.
//!
//! Tests rendering of SelectableList, Table, and Tree components
//! with varying data sizes (100 and 1000 items) and terminal sizes.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use envision::backend::CaptureBackend;
use envision::component::{
    Column, Component, SelectableList, SelectableListState, Table, TableRow, TableState, Tree,
    TreeNode, TreeState,
};
use envision::theme::Theme;
use ratatui::layout::Constraint;
use ratatui::Terminal;

// ========================================
// SelectableList Benchmarks
// ========================================

fn bench_selectable_list_view(c: &mut Criterion) {
    let mut group = c.benchmark_group("selectable_list_view");

    for item_count in [100, 1000] {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();

        for (width, height) in [(80, 24), (120, 40)] {
            let label = format!("{}_items/{}x{}", item_count, width, height);

            group.bench_with_input(
                BenchmarkId::new("render", &label),
                &(&items, width, height),
                |b, &(items, w, h)| {
                    let mut state = SelectableListState::new(items.clone());
                    state.set_focused(true);
                    let backend = CaptureBackend::new(w, h);
                    let mut terminal = Terminal::new(backend).unwrap();
                    let theme = Theme::default();

                    b.iter(|| {
                        terminal
                            .draw(|frame| {
                                SelectableList::<String>::view(
                                    black_box(&state),
                                    frame,
                                    frame.area(),
                                    &theme,
                                );
                            })
                            .unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

// ========================================
// Table Benchmarks
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

fn bench_table_view(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_view");

    let columns = vec![
        Column::new("ID", Constraint::Length(6)),
        Column::new("Name", Constraint::Length(20)),
        Column::new("Email", Constraint::Length(30)),
    ];

    for row_count in [100, 1000] {
        let rows: Vec<BenchRow> = (0..row_count)
            .map(|i| BenchRow {
                id: i,
                name: format!("User {}", i),
                email: format!("user{}@example.com", i),
            })
            .collect();

        for (width, height) in [(80, 24), (120, 40)] {
            let label = format!("{}_rows/{}x{}", row_count, width, height);

            group.bench_with_input(
                BenchmarkId::new("render", &label),
                &(&rows, &columns, width, height),
                |b, &(rows, columns, w, h)| {
                    let mut state = TableState::new(rows.clone(), columns.clone());
                    state.set_focused(true);
                    let backend = CaptureBackend::new(w, h);
                    let mut terminal = Terminal::new(backend).unwrap();
                    let theme = Theme::default();

                    b.iter(|| {
                        terminal
                            .draw(|frame| {
                                Table::<BenchRow>::view(
                                    black_box(&state),
                                    frame,
                                    frame.area(),
                                    &theme,
                                );
                            })
                            .unwrap();
                    });
                },
            );
        }
    }

    group.finish();
}

// ========================================
// Tree Benchmarks
// ========================================

/// Creates a tree with the specified total node count.
/// Distributes nodes as a flat list of children under one root for consistent sizing.
fn make_flat_tree(child_count: usize) -> Vec<TreeNode<String>> {
    let mut root = TreeNode::new_expanded("Root", "root".to_string());
    for i in 0..child_count {
        root.add_child(TreeNode::new(format!("Item {}", i), format!("item_{}", i)));
    }
    vec![root]
}

/// Creates a tree with nested structure for deeper traversal benchmarks.
/// depth=3, breadth=10 gives ~1111 nodes; depth=2, breadth=10 gives ~111 nodes.
fn make_nested_tree(depth: usize, breadth: usize) -> Vec<TreeNode<String>> {
    fn build(depth: usize, breadth: usize, prefix: &str) -> TreeNode<String> {
        let label = format!("Node {}", prefix);
        let data = prefix.to_string();
        let mut node = TreeNode::new_expanded(label, data);
        if depth > 0 {
            for i in 0..breadth {
                let child_prefix = format!("{}.{}", prefix, i);
                node.add_child(build(depth - 1, breadth, &child_prefix));
            }
        }
        node
    }
    vec![build(depth, breadth, "0")]
}

fn bench_tree_view(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_view");

    // Flat tree benchmarks (100 and 1000 children)
    for child_count in [100, 1000] {
        let roots = make_flat_tree(child_count);

        for (width, height) in [(80, 24), (120, 40)] {
            let label = format!("{}_flat/{}x{}", child_count, width, height);

            group.bench_with_input(
                BenchmarkId::new("render", &label),
                &(&roots, width, height),
                |b, &(roots, w, h)| {
                    let mut state = TreeState::new(roots.clone());
                    state.set_focused(true);
                    let backend = CaptureBackend::new(w, h);
                    let mut terminal = Terminal::new(backend).unwrap();
                    let theme = Theme::default();

                    b.iter(|| {
                        terminal
                            .draw(|frame| {
                                Tree::<String>::view(
                                    black_box(&state),
                                    frame,
                                    frame.area(),
                                    &theme,
                                );
                            })
                            .unwrap();
                    });
                },
            );
        }
    }

    // Nested tree benchmark (depth=3, breadth=10 ≈ 1111 nodes)
    let nested_roots = make_nested_tree(3, 10);
    group.bench_with_input(
        BenchmarkId::new("render", "nested_1111/80x24"),
        &(&nested_roots, 80u16, 24u16),
        |b, &(roots, w, h)| {
            let mut state = TreeState::new(roots.clone());
            state.set_focused(true);
            let backend = CaptureBackend::new(w, h);
            let mut terminal = Terminal::new(backend).unwrap();
            let theme = Theme::default();

            b.iter(|| {
                terminal
                    .draw(|frame| {
                        Tree::<String>::view(black_box(&state), frame, frame.area(), &theme);
                    })
                    .unwrap();
            });
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_selectable_list_view,
    bench_table_view,
    bench_tree_view,
);

criterion_main!(benches);
