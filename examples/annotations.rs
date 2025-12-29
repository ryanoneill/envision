//! Widget Annotations example demonstrating semantic metadata.
//!
//! This example shows how to use annotations to attach semantic
//! information to widgets for:
//! - Accessibility and screen readers
//! - Automated testing and queries
//! - UI element identification
//! - State tracking (focused, disabled, selected)
//!
//! Run with: cargo run --example annotations

use envision::annotation::{with_annotations, Annotate, Annotation, WidgetType};
use envision::backend::CaptureBackend;
use envision::harness::TestHarness;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Terminal;

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     Widget Annotations Demo          ║");
    println!("╚══════════════════════════════════════╝\n");

    demo_basic_annotations();
    demo_interactive_widgets();
    demo_annotation_registry();
    demo_harness_integration();

    println!("All demos complete!");
}

fn demo_basic_annotations() {
    println!("=== Demo 1: Basic Annotations ===\n");

    let backend = CaptureBackend::new(60, 15);
    let mut terminal = Terminal::new(backend).unwrap();

    // Render with annotations using with_annotations to capture them
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks =
                    Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).split(area);

                // Annotated header
                let header = Annotate::new(
                    Paragraph::new("My Application")
                        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    Annotation::header("app-header").with_label("Application Header"),
                );
                frame.render_widget(header, chunks[0]);

                // Annotated content area
                let content = Annotate::new(
                    Paragraph::new("Welcome to the annotated UI!\n\nAnnotations add semantic meaning to widgets.")
                        .block(Block::default().borders(Borders::ALL).title("Content")),
                    Annotation::container("main-content").with_label("Main Content Area"),
                );
                frame.render_widget(content, chunks[1]);
            })
            .unwrap();
    });

    println!("Rendered UI:");
    println!("{}", terminal.backend());

    println!("Registered annotations:");
    for region in registry.regions() {
        println!(
            "  {:?} at ({}, {}) - {}x{}",
            region.annotation.widget_type,
            region.area.x,
            region.area.y,
            region.area.width,
            region.area.height
        );
        if let Some(label) = &region.annotation.label {
            println!("    Label: {}", label);
        }
        if let Some(id) = &region.annotation.id {
            println!("    ID: {}", id);
        }
    }
    println!();
}

fn demo_interactive_widgets() {
    println!("=== Demo 2: Interactive Widget Annotations ===\n");

    let backend = CaptureBackend::new(60, 12);
    let mut terminal = Terminal::new(backend).unwrap();

    // Simulate some UI state
    let _selected_item = 1;
    let focused_widget = "btn-save";
    let form_disabled = false;

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let area = frame.area();
                let chunks = Layout::vertical([
                    Constraint::Length(5), // Form fields
                    Constraint::Length(3), // Buttons
                    Constraint::Min(0),    // Status
                ])
                .split(area);

                // Input field
                let input = Annotate::new(
                    Paragraph::new("john.doe@example.com")
                        .block(Block::default().borders(Borders::ALL).title("Email")),
                    Annotation::input("email-input")
                        .with_label("Email Address")
                        .with_disabled(form_disabled),
                )
                .value("john.doe@example.com");
                frame.render_widget(input, chunks[0]);

                // Button row
                let btn_chunks =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .split(chunks[1]);

                // Save button (focused)
                let save_btn = Annotate::new(
                    Paragraph::new("[ Save ]")
                        .style(Style::default().fg(Color::Green))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    Annotation::button("btn-save").with_label("Save Changes"),
                )
                .focused(focused_widget == "btn-save")
                .disabled(form_disabled);
                frame.render_widget(save_btn, btn_chunks[0]);

                // Cancel button
                let cancel_btn = Annotate::new(
                    Paragraph::new("[ Cancel ]")
                        .style(Style::default().fg(Color::Red))
                        .alignment(Alignment::Center)
                        .block(Block::default().borders(Borders::ALL)),
                    Annotation::button("btn-cancel").with_label("Cancel"),
                )
                .focused(focused_widget == "btn-cancel")
                .disabled(form_disabled);
                frame.render_widget(cancel_btn, btn_chunks[1]);

                // Status
                let status = Annotate::new(
                    Paragraph::new("Press Tab to switch focus")
                        .style(Style::default().fg(Color::DarkGray))
                        .alignment(Alignment::Center),
                    Annotation::label("status-text"),
                );
                frame.render_widget(status, chunks[2]);
            })
            .unwrap();
    });

    println!("Rendered form:");
    println!("{}", terminal.backend());

    // Query interactive widgets
    println!("Interactive widgets:");
    for region in registry.interactive_regions() {
        let focus_marker = if region.annotation.focused {
            " (focused)"
        } else {
            ""
        };
        let disabled_marker = if region.annotation.disabled {
            " [disabled]"
        } else {
            ""
        };
        println!(
            "  {:?} - {}{}{}",
            region.annotation.widget_type,
            region.annotation.label.as_deref().unwrap_or("unlabeled"),
            focus_marker,
            disabled_marker
        );
    }
    println!();
}

fn demo_annotation_registry() {
    println!("=== Demo 3: Annotation Registry Queries ===\n");

    let backend = CaptureBackend::new(60, 15);
    let mut terminal = Terminal::new(backend).unwrap();

    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                let area = frame.area();

                // Create a list of items
                let items = [
                    ("item-1", "First Item", true),
                    ("item-2", "Second Item", false),
                    ("item-3", "Third Item", false),
                ];

                let list_items: Vec<ListItem> = items
                    .iter()
                    .map(|(_, text, selected)| {
                        let style = if *selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                        };
                        let prefix = if *selected { "> " } else { "  " };
                        ListItem::new(format!("{}{}", prefix, text)).style(style)
                    })
                    .collect();

                // Annotate the list container
                let list_widget = Annotate::new(
                    List::new(list_items)
                        .block(Block::default().borders(Borders::ALL).title("Items")),
                    Annotation::list("item-list").with_label("Item Selection List"),
                );
                frame.render_widget(list_widget, area);

                // In a real app, you'd annotate each list item individually
                // Here we're just showing the container annotation
            })
            .unwrap();
    });

    println!("Rendered list:");
    println!("{}", terminal.backend());

    // Various registry queries
    println!("Registry queries:");

    // Find by type
    let lists = registry.find_by_type(&WidgetType::List);
    println!("  Lists found: {}", lists.len());

    // Find by ID
    let regions = registry.find_by_id("item-list");
    if let Some(region) = regions.first() {
        println!(
            "  Found 'item-list' at position ({}, {})",
            region.area.x, region.area.y
        );
    }

    // Check regions at a specific point
    let regions_at = registry.regions_at(5, 5);
    println!(
        "  Regions at (5, 5): {}",
        regions_at
            .iter()
            .map(|r| format!("{:?}", r.annotation.widget_type))
            .collect::<Vec<_>>()
            .join(", ")
    );

    println!();
}

fn demo_harness_integration() {
    println!("=== Demo 4: Test Harness Integration ===\n");

    let mut harness = TestHarness::new(60, 10);

    harness
        .render(|frame| {
            let area = frame.area();
            let chunks =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(area);

            // Sidebar with tabs
            let sidebar = Annotate::new(
                Paragraph::new("Tab 1\nTab 2\nTab 3")
                    .block(Block::default().borders(Borders::ALL).title("Navigation")),
                Annotation::new(WidgetType::TabBar)
                    .with_id("nav-tabs")
                    .with_label("Navigation Tabs"),
            );
            frame.render_widget(sidebar, chunks[0]);

            // Main content
            let content = Annotate::new(
                Paragraph::new("Main content area")
                    .block(Block::default().borders(Borders::ALL).title("Main")),
                Annotation::container("main-panel"),
            );
            frame.render_widget(content, chunks[1]);
        })
        .unwrap();

    println!("Rendered layout:");
    println!("{}", harness.screen());

    // Use harness annotation queries
    println!("Harness annotation queries:");

    // Find by ID
    let nav_regions = harness.find_by_id("nav-tabs");
    if let Some(region) = nav_regions.first() {
        println!("  Found 'nav-tabs': {:?}", region.annotation.widget_type);
    }

    // Get interactive regions
    let interactive = harness.interactive();
    println!("  Interactive regions: {}", interactive.len());

    // Get focused region (if any)
    if let Some(focused) = harness.focused() {
        println!(
            "  Focused: {:?}",
            focused.annotation.id.as_deref().unwrap_or("unknown")
        );
    } else {
        println!("  No focused widget");
    }

    // Custom assertions with annotations
    println!("\nAnnotation-based assertions:");
    harness.assert_widget_exists("nav-tabs");
    println!("  nav-tabs exists");
    harness.assert_widget_exists("main-panel");
    println!("  main-panel exists");

    println!();
}
