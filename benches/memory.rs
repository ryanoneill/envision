//! Memory allocation benchmarks for component rendering.
//!
//! Measures heap allocation count and bytes per render cycle for
//! SelectableList, Table, and Tree components with 1000 items, plus
//! state creation scaling (1000/5000/10000 items).
//!
//! # Approach
//!
//! A counting `GlobalAlloc` wrapper intercepts every allocation and
//! records the count and cumulative byte total in atomics. Because
//! this is a standalone benchmark binary the `#[global_allocator]`
//! declaration does not conflict with the library or other bench
//! binaries.
//!
//! # Unsafe
//!
//! The `CountingAllocator` implementation contains the only `unsafe`
//! code in this project. It is acceptable here because:
//!
//! 1. It lives exclusively in a benchmark binary, never in library
//!    code shipped to users.
//! 2. The implementation is trivially safe: it delegates every call
//!    directly to the standard `System` allocator with no pointer
//!    arithmetic of its own.
//! 3. `GlobalAlloc` is an `unsafe trait` by design — the contract
//!    (return a valid pointer or null, dealloc the same pointer) is
//!    upheld by forwarding to `System`.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use envision::backend::CaptureBackend;
use envision::component::ViewContext;
use envision::component::{
    Column, Component, SelectableList, SelectableListState, Table, TableRow, TableState, Tree,
    TreeNode, TreeState,
};
use envision::theme::Theme;
use ratatui::Terminal;
use ratatui::layout::Constraint;

// ============================================================
// Counting allocator
// ============================================================

static ALLOC_COUNT: AtomicUsize = AtomicUsize::new(0);
static ALLOC_BYTES: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

// SAFETY: `CountingAllocator` delegates every call directly to the
// standard `System` allocator, which upholds all `GlobalAlloc`
// safety invariants. The only additional work performed is updating
// two `Relaxed` atomics, which is safe from any thread.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_COUNT.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        // SAFETY: delegated directly to System; caller upholds Layout validity.
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: delegated directly to System; caller provides the same ptr/layout
        // pair returned by our `alloc`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

fn reset_counts() {
    ALLOC_COUNT.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
}

fn alloc_count() -> usize {
    ALLOC_COUNT.load(Ordering::Relaxed)
}

fn alloc_bytes() -> usize {
    ALLOC_BYTES.load(Ordering::Relaxed)
}

// ============================================================
// Helper types (mirror component_view.rs)
// ============================================================

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

fn make_table_columns() -> Vec<Column> {
    vec![
        Column::new("ID", Constraint::Length(6)),
        Column::new("Name", Constraint::Length(20)),
        Column::new("Email", Constraint::Length(30)),
    ]
}

fn make_flat_tree(child_count: usize) -> Vec<TreeNode<String>> {
    let mut root = TreeNode::new_expanded("Root", "root".to_string());
    for i in 0..child_count {
        root.add_child(TreeNode::new(format!("Item {}", i), format!("item_{}", i)));
    }
    vec![root]
}

// ============================================================
// Render-allocation benchmarks (1000 items)
// ============================================================

/// Measures allocation count and bytes for a single SelectableList
/// render with 1000 items at 80×24.
fn bench_selectable_list_1000_view_allocs(c: &mut Criterion) {
    let items: Vec<String> = (0..1000).map(|i| format!("Item {}", i)).collect();
    let state = SelectableListState::new(items);
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default();

    c.bench_function("selectable_list_1000_view_allocs", |b| {
        b.iter(|| {
            reset_counts();
            terminal
                .draw(|frame| {
                    SelectableList::<String>::view(
                        black_box(&state),
                        frame,
                        frame.area(),
                        &theme,
                        &ViewContext::default(),
                    );
                })
                .unwrap();
            black_box((alloc_count(), alloc_bytes()));
        });
    });
}

/// Measures allocation count and bytes for a single Table render
/// with 1000 rows at 80×24.
fn bench_table_1000_view_allocs(c: &mut Criterion) {
    let rows: Vec<BenchRow> = (0..1000)
        .map(|i| BenchRow {
            id: i,
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();
    let columns = make_table_columns();
    let state = TableState::new(rows, columns);
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default();

    c.bench_function("table_1000_view_allocs", |b| {
        b.iter(|| {
            reset_counts();
            terminal
                .draw(|frame| {
                    Table::<BenchRow>::view(
                        black_box(&state),
                        frame,
                        frame.area(),
                        &theme,
                        &ViewContext::default(),
                    );
                })
                .unwrap();
            black_box((alloc_count(), alloc_bytes()));
        });
    });
}

/// Measures allocation count and bytes for a single Tree render with
/// 1000 nodes (flat layout) at 80×24.
fn bench_tree_1000_view_allocs(c: &mut Criterion) {
    let roots = make_flat_tree(1000);
    let state = TreeState::new(roots);
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = Theme::default();

    c.bench_function("tree_1000_view_allocs", |b| {
        b.iter(|| {
            reset_counts();
            terminal
                .draw(|frame| {
                    Tree::<String>::view(
                        black_box(&state),
                        frame,
                        frame.area(),
                        &theme,
                        &ViewContext::default(),
                    );
                })
                .unwrap();
            black_box((alloc_count(), alloc_bytes()));
        });
    });
}

// ============================================================
// State-creation benchmarks (1000 / 5000 / 10000 items)
// ============================================================

/// Measures allocations during SelectableListState construction at
/// varying item counts (1 000 / 5 000 / 10 000).
fn bench_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("state_creation");

    for item_count in [1000usize, 5000, 10000] {
        let items: Vec<String> = (0..item_count).map(|i| format!("Item {}", i)).collect();

        group.bench_with_input(
            BenchmarkId::new("selectable_list", item_count),
            &items,
            |b, items| {
                b.iter(|| {
                    reset_counts();
                    let s = SelectableListState::new(black_box(items.clone()));
                    black_box((s, alloc_count(), alloc_bytes()));
                });
            },
        );
    }

    group.finish();
}

// ============================================================
// Criterion wiring
// ============================================================

criterion_group!(
    benches,
    bench_selectable_list_1000_view_allocs,
    bench_table_1000_view_allocs,
    bench_tree_1000_view_allocs,
    bench_state_creation,
);

criterion_main!(benches);
