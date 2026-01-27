//! Test Harness example demonstrating testing utilities.
//!
//! This example shows different ways to test TUI applications:
//! - TestHarness for basic render testing with closures
//! - Runtime::headless for testing App implementations
//! - AsyncTestHarness for async App testing
//! - Assertions and content queries
//!
//! Run with: cargo run --example test_harness

use std::ops::Not;

use envision::harness::{Assertion, TestHarness};
use envision::prelude::*;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

// =============================================================================
// Example App for testing
// =============================================================================

struct TodoApp;

#[derive(Default, Clone)]
struct TodoState {
    items: Vec<String>,
    selected: Option<usize>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum TodoMsg {
    Add(String),
    Remove(usize),
    Select(usize),
    Clear,
}

impl App for TodoApp {
    type State = TodoState;
    type Message = TodoMsg;

    fn init() -> (TodoState, Command<TodoMsg>) {
        let state = TodoState {
            items: vec!["Buy groceries".to_string(), "Walk the dog".to_string()],
            selected: None,
        };
        (state, Command::none())
    }

    fn update(state: &mut TodoState, msg: TodoMsg) -> Command<TodoMsg> {
        match msg {
            TodoMsg::Add(item) => {
                state.items.push(item);
            }
            TodoMsg::Remove(idx) => {
                if idx < state.items.len() {
                    state.items.remove(idx);
                    state.selected = None;
                }
            }
            TodoMsg::Select(idx) => {
                if idx < state.items.len() {
                    state.selected = Some(idx);
                }
            }
            TodoMsg::Clear => {
                state.items.clear();
                state.selected = None;
            }
        }
        Command::none()
    }

    fn view(state: &TodoState, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        // Header
        let header = Paragraph::new(format!("Todo List ({} items)", state.items.len()))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title("Header"));
        frame.render_widget(header, chunks[0]);

        // Items list
        let items_text: Vec<String> = state
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let marker = if state.selected == Some(i) { ">" } else { " " };
                format!("{} {}. {}", marker, i + 1, item)
            })
            .collect();

        let items_para = Paragraph::new(items_text.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Items"));
        frame.render_widget(items_para, chunks[1]);
    }
}

// =============================================================================
// Testing demonstrations
// =============================================================================

fn demo_basic_harness() {
    println!("=== Demo 1: Basic TestHarness ===\n");

    // TestHarness works with any render closure
    let mut harness = TestHarness::new(60, 10);

    // Render directly with a closure
    harness
        .render(|frame| {
            let para = Paragraph::new("Hello, Test World!")
                .style(Style::default().fg(Color::Green))
                .block(Block::default().borders(Borders::ALL).title("Greeting"));
            frame.render_widget(para, frame.area());
        })
        .unwrap();

    println!("Rendered content:");
    println!("{}", harness.screen());

    // Use assertions
    harness.assert_contains("Hello, Test World!");
    harness.assert_contains("Greeting");
    println!("Assertions passed!\n");
}

fn demo_runtime_testing() {
    println!("=== Demo 2: Virtual Terminal for App Testing ===\n");

    // Use Runtime::virtual_terminal to test App implementations
    let mut vt = Runtime::<TodoApp, _>::virtual_terminal(60, 12).unwrap();

    // Check initial state
    vt.step().unwrap();
    println!("Initial state:");
    println!("{}", vt.display());

    assert_eq!(vt.state().items.len(), 2);
    assert!(vt.contains_text("2 items"));
    assert!(vt.contains_text("Buy groceries"));
    assert!(vt.contains_text("Walk the dog"));

    // Add a new item
    vt.dispatch(TodoMsg::Add("Learn Rust".to_string()));
    vt.step().unwrap();
    println!("After adding item:");
    println!("{}", vt.display());

    assert_eq!(vt.state().items.len(), 3);
    assert!(vt.contains_text("3 items"));
    assert!(vt.contains_text("Learn Rust"));

    // Select an item
    vt.dispatch(TodoMsg::Select(1));
    vt.step().unwrap();
    println!("After selecting item 1:");
    println!("{}", vt.display());

    assert_eq!(vt.state().selected, Some(1));
    assert!(vt.contains_text("> 2. Walk the dog"));

    // Remove the selected item
    vt.dispatch(TodoMsg::Remove(1));
    vt.step().unwrap();
    println!("After removing item:");
    println!("{}", vt.display());

    assert_eq!(vt.state().items.len(), 2);
    assert!(!vt.contains_text("Walk the dog"));

    println!("All virtual terminal tests passed!\n");
}

fn demo_assertions() {
    println!("=== Demo 3: Declarative Assertions ===\n");

    let mut harness = TestHarness::new(60, 10);

    harness
        .render(|frame| {
            let text = "Error: File not found\nWarning: Low memory\nInfo: Operation complete";
            let para =
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Logs"));
            frame.render_widget(para, frame.area());
        })
        .unwrap();

    println!("Rendered logs:");
    println!("{}", harness.screen());

    // Build complex assertions
    let assertion = Assertion::all(vec![
        Assertion::contains("Error:"),
        Assertion::contains("Warning:"),
        Assertion::contains("Info:"),
    ]);

    match assertion.check(&harness) {
        Ok(()) => println!("All log levels present!"),
        Err(e) => println!("Assertion failed: {}", e),
    }

    // Check for specific patterns
    let not_critical = Assertion::contains("CRITICAL").not();
    match not_critical.check(&harness) {
        Ok(()) => println!("No CRITICAL errors found!"),
        Err(e) => println!("Unexpected: {}", e),
    }

    // Any of multiple patterns
    let has_status = Assertion::any(vec![
        Assertion::contains("complete"),
        Assertion::contains("success"),
        Assertion::contains("done"),
    ]);
    match has_status.check(&harness) {
        Ok(()) => println!("Found a completion status!"),
        Err(e) => println!("No status found: {}", e),
    }

    println!("\nAll assertion demos complete!\n");
}

fn demo_content_queries() {
    println!("=== Demo 4: Content Queries ===\n");

    let mut harness = TestHarness::new(40, 8);

    harness
        .render(|frame| {
            let text = "Name: Alice\nAge: 30\nCity: New York";
            let para =
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Profile"));
            frame.render_widget(para, frame.area());
        })
        .unwrap();

    println!("Rendered profile:");
    println!("{}", harness.screen());

    // Check if text exists
    println!("Contains 'Alice': {}", harness.contains("Alice"));
    println!("Contains 'Bob': {}", harness.contains("Bob"));

    // Find text positions
    let positions = harness.find_all_text(":");
    println!("Found ':' at {} positions", positions.len());

    // Get specific row content
    println!("\nRow contents:");
    for y in 0..harness.height() {
        let row = harness.row(y);
        if !row.trim().is_empty() {
            println!("  Row {}: {}", y, row.trim());
        }
    }

    println!("\nContent query demo complete!\n");
}

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     Test Harness Demo                ║");
    println!("╚══════════════════════════════════════╝\n");

    demo_basic_harness();
    demo_runtime_testing();
    demo_assertions();
    demo_content_queries();

    println!("All demos complete!");
}
