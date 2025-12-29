//! CaptureBackend example demonstrating headless TUI rendering.
//!
//! This example shows how to use CaptureBackend to render a TUI
//! without a real terminal, and inspect the output in various formats.
//!
//! Run with: cargo run --example capture_backend

use envision::backend::CaptureBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Terminal;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a CaptureBackend - this captures all rendering without a real terminal
    let backend = CaptureBackend::new(60, 20);
    let mut terminal = Terminal::new(backend)?;

    // Draw a simple UI
    terminal.draw(|frame| {
        let area = frame.area();

        // Create a layout with two sections
        let chunks = Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

        // Header
        let header = Paragraph::new(Line::from(vec![
            Span::styled("Envision ", Style::default().fg(Color::Cyan)),
            Span::raw("- Collaborative TUI Development"),
        ]))
        .block(Block::default().borders(Borders::ALL).title("Header"));

        frame.render_widget(header, chunks[0]);

        // Content
        let content = Paragraph::new(vec![
            Line::raw(""),
            Line::raw("This is rendered using CaptureBackend."),
            Line::raw("The output can be inspected programmatically!"),
            Line::raw(""),
            Line::styled(
                "• Supports colors and styling",
                Style::default().fg(Color::Green),
            ),
            Line::styled("• Full frame capture", Style::default().fg(Color::Yellow)),
            Line::styled(
                "• Multiple output formats",
                Style::default().fg(Color::Magenta),
            ),
        ])
        .block(Block::default().borders(Borders::ALL).title("Content"));

        frame.render_widget(content, chunks[1]);
    })?;

    // Now we can inspect the output in various formats:

    println!("=== Plain Text Output ===\n");
    println!("{}", terminal.backend());

    println!("\n=== ANSI Colored Output ===\n");
    println!("{}", terminal.backend().to_ansi());

    println!("\n=== JSON Output ===\n");
    println!("{}", terminal.backend().to_json_pretty());

    // Demonstrate text searching
    println!("\n=== Text Search ===\n");
    if terminal.backend().contains_text("CaptureBackend") {
        let positions = terminal.backend().find_text("CaptureBackend");
        println!("Found 'CaptureBackend' at positions: {:?}", positions);
    }

    // Show frame info
    println!("\n=== Frame Info ===\n");
    println!("Current frame: {}", terminal.backend().current_frame());
    println!(
        "Dimensions: {}x{}",
        terminal.backend().width(),
        terminal.backend().height()
    );
    println!("Cursor visible: {}", terminal.backend().is_cursor_visible());

    Ok(())
}
