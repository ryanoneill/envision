//! CodeBlock example -- syntax-highlighted code display.
//!
//! Demonstrates the CodeBlock component with Rust syntax highlighting,
//! line numbers, highlighted lines, and scroll navigation.
//!
//! Run with: cargo run --example code_block

use envision::component::code_block::highlight::Language;
use envision::prelude::*;
use ratatui::widgets::Paragraph;

/// Application marker type.
struct CodeBlockApp;

/// Application state wrapping a single CodeBlock.
#[derive(Clone)]
struct State {
    code: CodeBlockState,
}

/// Application messages.
#[derive(Clone, Debug)]
enum Msg {
    Code(CodeBlockMessage),
    Quit,
}

impl App for CodeBlockApp {
    type State = State;
    type Message = Msg;

    fn init() -> (State, Command<Msg>) {
        let code = r#"use std::collections::HashMap;

/// A simple key-value store.
struct Store {
    data: HashMap<String, String>,
}

impl Store {
    fn new() -> Self {
        Store {
            data: HashMap::new(),
        }
    }

    fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    fn set(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn delete(&mut self, key: &str) -> bool {
        self.data.remove(key).is_some()
    }
}

fn main() {
    let mut store = Store::new();
    store.set("hello".to_string(), "world".to_string());

    if let Some(value) = store.get("hello") {
        println!("Found: {}", value);
    }

    // Clean up
    store.delete("hello");
    println!("Done!");
}"#;

        let code_state = CodeBlockState::new()
            .with_code(code)
            .with_language(Language::Rust)
            .with_title("store.rs")
            .with_line_numbers(true)
            .with_highlight_lines(vec![4, 5, 6]);

        (State { code: code_state }, Command::none())
    }

    fn update(state: &mut State, msg: Msg) -> Command<Msg> {
        match msg {
            Msg::Code(m) => {
                state.code.update(m);
            }
            Msg::Quit => return Command::quit(),
        }
        Command::none()
    }

    fn view(state: &State, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(1)]).split(area);

        let theme = Theme::default();
        CodeBlock::view(
            &state.code,
            &mut RenderContext::new(frame, chunks[0], &theme),
        );

        let status = Paragraph::new(format!(
            " Line {} | {}/{} | Up/Down/PgUp/PgDn/Home/End | l=line nums | Esc=quit",
            state.code.scroll_offset() + 1,
            state.code.language(),
            state.code.line_count(),
        ))
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(status, chunks[1]);
    }

    fn handle_event_with_state(state: &State, event: &Event) -> Option<Msg> {
        if let Some(key) = event.as_key() {
            if matches!(key.code, Key::Char('q') | Key::Esc) {
                return Some(Msg::Quit);
            }
        }

        CodeBlock::handle_event(&state.code, event, &EventContext::new().focused(true))
            .map(Msg::Code)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut vt = Runtime::<CodeBlockApp, _>::virtual_builder(80, 24).build()?;

    println!("=== CodeBlock Example ===\n");

    // Initial render
    vt.tick()?;
    println!("Initial view (Rust with line numbers, lines 4-6 highlighted):");
    println!("{}\n", vt.display());

    // Scroll down a few lines
    vt.dispatch(Msg::Code(CodeBlockMessage::ScrollDown));
    vt.dispatch(Msg::Code(CodeBlockMessage::ScrollDown));
    vt.dispatch(Msg::Code(CodeBlockMessage::ScrollDown));
    vt.tick()?;
    println!("After scrolling down 3 lines:");
    println!("{}\n", vt.display());

    // Toggle line numbers off
    vt.dispatch(Msg::Code(CodeBlockMessage::ToggleLineNumbers));
    vt.tick()?;
    println!("With line numbers toggled off:");
    println!("{}\n", vt.display());

    // Jump to end
    vt.dispatch(Msg::Code(CodeBlockMessage::End));
    vt.tick()?;
    println!("At the end:");
    println!("{}\n", vt.display());

    println!(
        "Final scroll offset: {}, Language: {}",
        vt.state().code.scroll_offset(),
        vt.state().code.language()
    );

    Ok(())
}
