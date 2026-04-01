#![cfg(feature = "full")]
//! Integration tests for new components (continued).
//!
//! This file is split from integration_new_components.rs to comply with
//! the 1000-line file limit. Tests 15-30 live here.

use envision::component::code_block::highlight::Language;
use envision::component::{
    // Observability
    AlertMetric,
    AlertPanel,
    AlertPanelMessage,
    AlertPanelState,
    AlertThreshold,
    // General purpose
    Calendar,
    CalendarMessage,
    CalendarOutput,
    CalendarState,
    // Visualization
    Chart,
    ChartMessage,
    ChartOutput,
    ChartState,
    CodeBlock,
    CodeBlockMessage,
    CodeBlockState,
    // Traits
    Component,
    // Claude Code components
    ConversationMessage,
    ConversationRole,
    ConversationView,
    ConversationViewMessage,
    ConversationViewState,
    DataSeries,
    EventLevel,
    EventStream,
    EventStreamMessage,
    EventStreamState,
    Focusable,
    Gauge,
    GaugeMessage,
    GaugeState,
    Heatmap,
    HeatmapMessage,
    HeatmapState,
    HelpPanel,
    HelpPanelMessage,
    HelpPanelState,
    KeyBinding,
    KeyBindingGroup,
    MessageBlock,
    Paginator,
    PaginatorMessage,
    PaginatorOutput,
    PaginatorState,
    Slider,
    SliderMessage,
    SliderState,
    SpanNode,
    SpanTree,
    SpanTreeMessage,
    SpanTreeState,
    Sparkline,
    SparklineMessage,
    SparklineState,
    Switch,
    SwitchState,
    Tab,
    TabBar,
    TabBarMessage,
    TabBarOutput,
    TabBarState,
    TerminalOutput,
    TerminalOutputMessage,
    TerminalOutputOutput,
    TerminalOutputState,
    ThresholdZone,
};
use envision::CaptureBackend;
use ratatui::prelude::*;
use ratatui::Terminal;

// ============================================================================
// 15. Paginator: navigate pages, verify boundary behavior
// ============================================================================

#[test]
fn test_paginator_boundary_navigation() {
    let mut state = PaginatorState::new(5);
    assert_eq!(state.current_page(), 0);
    assert_eq!(state.display_page(), 1);
    assert_eq!(state.total_pages(), 5);

    // Navigate to next page
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(1)));
    assert_eq!(state.current_page(), 1);

    // Navigate to last page
    let output = Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(4)));
    assert_eq!(state.current_page(), 4);
    assert_eq!(state.display_page(), 5);

    // NextPage at last page - should stay at last page
    let output = Paginator::update(&mut state, PaginatorMessage::NextPage);
    assert_eq!(output, None);
    assert_eq!(state.current_page(), 4);

    // Navigate to first page
    let output = Paginator::update(&mut state, PaginatorMessage::FirstPage);
    assert_eq!(output, Some(PaginatorOutput::PageChanged(0)));
    assert_eq!(state.current_page(), 0);

    // PrevPage at first page - should stay at first page
    let output = Paginator::update(&mut state, PaginatorMessage::PrevPage);
    assert_eq!(output, None);
    assert_eq!(state.current_page(), 0);

    // GoToPage
    let output = Paginator::update(&mut state, PaginatorMessage::GoToPage(3));
    assert_eq!(output, Some(PaginatorOutput::PageChanged(3)));
    assert_eq!(state.current_page(), 3);

    // Update total pages
    Paginator::update(&mut state, PaginatorMessage::SetTotalPages(10));
    assert_eq!(state.total_pages(), 10);
    assert_eq!(state.current_page(), 3); // should stay on page 3

    // Navigate to last of the new total
    Paginator::update(&mut state, PaginatorMessage::LastPage);
    assert_eq!(state.current_page(), 9);

    // Render
    let backend = CaptureBackend::new(40, 1);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            Paginator::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 16. HelpPanel: scroll through bindings
// ============================================================================

#[test]
fn test_help_panel_scroll_and_groups() {
    let state = HelpPanelState::new().with_groups(vec![
        KeyBindingGroup::new(
            "Navigation",
            vec![
                KeyBinding::new("Up/k", "Move up"),
                KeyBinding::new("Down/j", "Move down"),
                KeyBinding::new("PageUp", "Page up"),
                KeyBinding::new("PageDown", "Page down"),
                KeyBinding::new("Home", "Go to top"),
                KeyBinding::new("End", "Go to bottom"),
            ],
        ),
        KeyBindingGroup::new(
            "Actions",
            vec![
                KeyBinding::new("Enter", "Select item"),
                KeyBinding::new("Space", "Toggle"),
                KeyBinding::new("Tab", "Next field"),
                KeyBinding::new("q/Esc", "Quit"),
            ],
        ),
        KeyBindingGroup::new(
            "Search",
            vec![
                KeyBinding::new("/", "Start search"),
                KeyBinding::new("n", "Next result"),
                KeyBinding::new("N", "Previous result"),
            ],
        ),
    ]);

    assert_eq!(state.groups().len(), 3);
    assert_eq!(state.title(), Some("Help"));
    assert!(state.is_visible());

    // Scroll down and up
    let mut state = state;
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollDown);
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollDown);
    HelpPanel::update(&mut state, HelpPanelMessage::ScrollUp);

    // Page down
    HelpPanel::update(&mut state, HelpPanelMessage::PageDown(5));

    // Home
    HelpPanel::update(&mut state, HelpPanelMessage::Home);

    // End
    HelpPanel::update(&mut state, HelpPanelMessage::End);

    // Add a new group
    HelpPanel::update(
        &mut state,
        HelpPanelMessage::AddGroup(KeyBindingGroup::new(
            "Advanced",
            vec![KeyBinding::new("Ctrl+R", "Reload configuration")],
        )),
    );
    assert_eq!(state.groups().len(), 4);

    // Render
    let backend = CaptureBackend::new(50, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            HelpPanel::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 17. CodeBlock: set code, change language, scroll
// ============================================================================

#[test]
fn test_code_block_set_code_language_scroll() {
    let mut state = CodeBlockState::new()
        .with_code("fn main() {\n    println!(\"Hello, world!\");\n}\n")
        .with_language(Language::Rust)
        .with_title("main.rs")
        .with_line_numbers(true);

    assert_eq!(
        state.code(),
        "fn main() {\n    println!(\"Hello, world!\");\n}\n"
    );
    assert_eq!(state.language(), &Language::Rust);
    assert_eq!(state.title(), Some("main.rs"));
    assert!(state.show_line_numbers());
    assert_eq!(state.scroll_offset(), 0);

    // Change code content
    CodeBlock::update(
        &mut state,
        CodeBlockMessage::SetCode("def hello():\n    print(\"Hello\")\n\nhello()\n".to_string()),
    );
    assert_eq!(
        state.code(),
        "def hello():\n    print(\"Hello\")\n\nhello()\n"
    );

    // Change language
    CodeBlock::update(&mut state, CodeBlockMessage::SetLanguage(Language::Python));
    assert_eq!(state.language(), &Language::Python);

    // Scroll down
    CodeBlock::update(&mut state, CodeBlockMessage::ScrollDown);

    // Scroll to end
    CodeBlock::update(&mut state, CodeBlockMessage::End);

    // Scroll to beginning
    CodeBlock::update(&mut state, CodeBlockMessage::Home);
    assert_eq!(state.scroll_offset(), 0);

    // Toggle line numbers
    CodeBlock::update(&mut state, CodeBlockMessage::ToggleLineNumbers);
    assert!(!state.show_line_numbers());

    // Highlight a line
    CodeBlock::update(&mut state, CodeBlockMessage::HighlightLine(2));
    // Unhighlight
    CodeBlock::update(&mut state, CodeBlockMessage::UnhighlightLine(2));

    // Render
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            CodeBlock::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 18. TerminalOutput: append lines, verify auto-scroll, check ANSI parsing
// ============================================================================

#[test]
fn test_terminal_output_append_and_auto_scroll() {
    let mut state = TerminalOutputState::new()
        .with_title("Build Output")
        .with_auto_scroll(true)
        .with_max_lines(100);

    assert!(state.auto_scroll());
    assert_eq!(state.line_count(), 0);

    // Push lines
    state.push_line("Compiling envision v0.7.0");
    state.push_line("\x1b[32m   Finished\x1b[0m in 2.5s");
    state.push_line("\x1b[31merror[E0308]\x1b[0m: mismatched types");

    assert_eq!(state.line_count(), 3);
    assert_eq!(state.lines()[0], "Compiling envision v0.7.0");
    assert_eq!(state.lines()[1], "\x1b[32m   Finished\x1b[0m in 2.5s");

    // Push via message
    let output = TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLine("warning: unused variable".to_string()),
    );
    assert!(matches!(output, Some(TerminalOutputOutput::LineAdded(4))));
    assert_eq!(state.line_count(), 4);

    // Push multiple lines
    TerminalOutput::update(
        &mut state,
        TerminalOutputMessage::PushLines(vec!["line 5".to_string(), "line 6".to_string()]),
    );
    assert_eq!(state.line_count(), 6);

    // Toggle auto-scroll
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleAutoScroll);
    assert!(matches!(
        output,
        Some(TerminalOutputOutput::AutoScrollToggled(false))
    ));
    assert!(!state.auto_scroll());

    // Toggle line numbers
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::ToggleLineNumbers);
    assert!(matches!(
        output,
        Some(TerminalOutputOutput::LineNumbersToggled(true))
    ));

    // Set running and exit code
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetRunning(true));
    TerminalOutput::update(&mut state, TerminalOutputMessage::SetExitCode(Some(0)));

    // Clear
    let output = TerminalOutput::update(&mut state, TerminalOutputMessage::Clear);
    assert!(matches!(output, Some(TerminalOutputOutput::Cleared)));
    assert_eq!(state.line_count(), 0);

    // Verify ANSI parsing works
    let segments = envision::component::parse_ansi("\x1b[32mgreen\x1b[0m normal");
    assert!(!segments.is_empty());

    // Render
    state.push_line("Test line");
    let backend = CaptureBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            TerminalOutput::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 19. ConversationView: push messages, toggle blocks, scroll
// ============================================================================

#[test]
fn test_conversation_view_messages_and_collapse() {
    let mut state = ConversationViewState::new().with_title("Chat Session");
    assert_eq!(state.message_count(), 0);
    assert!(state.auto_scroll());

    // Push messages using helper methods
    state.push_user("Hello, can you help me?");
    assert_eq!(state.message_count(), 1);
    assert_eq!(*state.messages()[0].role(), ConversationRole::User);

    state.push_assistant("Of course! How can I help?");
    assert_eq!(state.message_count(), 2);
    assert_eq!(*state.messages()[1].role(), ConversationRole::Assistant);

    state.push_system("System: model context loaded");
    assert_eq!(state.message_count(), 3);
    assert_eq!(*state.messages()[2].role(), ConversationRole::System);

    // Push a structured message with code blocks
    state.push_message(ConversationMessage::with_blocks(
        ConversationRole::Assistant,
        vec![
            MessageBlock::text("Here is the fix:"),
            MessageBlock::code("fn main() {}", Some("rust")),
        ],
    ));
    assert_eq!(state.message_count(), 4);
    assert_eq!(state.messages()[3].blocks().len(), 2);

    // Push tool result
    state.push_tool("Search results: 5 items found");
    assert_eq!(state.message_count(), 5);
    assert_eq!(*state.messages()[4].role(), ConversationRole::Tool);

    // Toggle collapse on a named block key
    assert!(!state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(state.is_collapsed("thinking"));
    state.toggle_collapse("thinking");
    assert!(!state.is_collapsed("thinking"));

    // Scroll via messages
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToTop);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollToBottom);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollUp);
    ConversationView::update(&mut state, ConversationViewMessage::ScrollDown);

    // Max messages cap
    let mut capped_state = ConversationViewState::new().with_max_messages(3);
    for i in 0..5 {
        capped_state.push_user(format!("Message {}", i));
    }
    assert_eq!(capped_state.message_count(), 3);

    // Render
    let backend = CaptureBackend::new(60, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            ConversationView::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 20. TabBar: add/close/navigate tabs
// ============================================================================

#[test]
fn test_tab_bar_add_close_navigate() {
    let tabs = vec![
        Tab::new("file1", "main.rs"),
        Tab::new("file2", "lib.rs").with_modified(true),
        Tab::new("file3", "test.rs").with_closable(true),
    ];
    let mut state = TabBarState::new(tabs);
    TabBar::set_focused(&mut state, true);

    assert_eq!(state.len(), 3);
    assert_eq!(state.active_index(), Some(0));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("main.rs"));

    // Navigate to next tab
    let output = TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.active_index(), Some(1));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("lib.rs"));
    assert!(state.active_tab().map(|t| t.modified()).unwrap_or(false));

    // Navigate to last tab
    TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(state.active_index(), Some(2));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("test.rs"));

    // NextTab at last position does not wrap - stays at last
    let output = TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(output, None);
    assert_eq!(state.active_index(), Some(2));

    // PrevTab at first position does not wrap - stays at first
    TabBar::update(&mut state, TabBarMessage::First);
    let output = TabBar::update(&mut state, TabBarMessage::PrevTab);
    assert_eq!(output, None);
    assert_eq!(state.active_index(), Some(0));

    // Navigate to last tab for further testing
    TabBar::update(&mut state, TabBarMessage::Last);

    // Add a new tab
    let output = TabBar::update(
        &mut state,
        TabBarMessage::AddTab(Tab::new("file4", "config.toml").with_closable(true)),
    );
    assert_eq!(output, Some(TabBarOutput::TabAdded(3)));
    assert_eq!(state.len(), 4);
    assert_eq!(state.active_index(), Some(3)); // new tab becomes active

    // Close the closable tab at index 2 (test.rs)
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(2));
    assert_eq!(output, Some(TabBarOutput::TabClosed(2)));
    assert_eq!(state.len(), 3);

    // Jump to first
    TabBar::update(&mut state, TabBarMessage::First);
    assert_eq!(state.active_index(), Some(0));

    // Jump to last
    TabBar::update(&mut state, TabBarMessage::Last);
    assert_eq!(state.active_index(), Some(2));

    // Select by index
    let output = TabBar::update(&mut state, TabBarMessage::SelectTab(1));
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.active_index(), Some(1));

    // Render
    let backend = CaptureBackend::new(60, 3);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            TabBar::view(&state, frame, frame.area(), &theme);
        })
        .unwrap();
}

// ============================================================================
// 21. Gauge with custom thresholds
// ============================================================================

#[test]
fn test_gauge_custom_thresholds() {
    let state = GaugeState::new(30.0, 100.0).with_thresholds(vec![
        ThresholdZone {
            above: 0.0,
            color: Color::Blue,
        },
        ThresholdZone {
            above: 0.5,
            color: Color::Cyan,
        },
        ThresholdZone {
            above: 0.8,
            color: Color::Magenta,
        },
    ]);

    // 30% -> Blue (above 0.0, below 0.5)
    assert_eq!(state.current_color(), Color::Blue);

    let mut state = state;
    Gauge::update(&mut state, GaugeMessage::SetValue(55.0));
    // 55% -> Cyan (above 0.5, below 0.8)
    assert_eq!(state.current_color(), Color::Cyan);

    Gauge::update(&mut state, GaugeMessage::SetValue(85.0));
    // 85% -> Magenta (above 0.8)
    assert_eq!(state.current_color(), Color::Magenta);

    // Test SetMax
    Gauge::update(&mut state, GaugeMessage::SetMax(200.0));
    assert_eq!(state.max(), 200.0);
    // 85 / 200 = 42.5%, should be Blue again
    assert_eq!(state.current_color(), Color::Blue);
}

// ============================================================================
// 22. Heatmap with labels and data replacement
// ============================================================================

#[test]
fn test_heatmap_labels_and_data_replacement() {
    let data = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
    let mut state = HeatmapState::with_data(data)
        .with_row_labels(vec!["Row A".into(), "Row B".into()])
        .with_col_labels(vec!["Mon".into(), "Tue".into(), "Wed".into()]);

    assert_eq!(state.rows(), 2);
    assert_eq!(state.cols(), 3);
    assert_eq!(state.row_labels(), &["Row A", "Row B"]);
    assert_eq!(state.col_labels(), &["Mon", "Tue", "Wed"]);
    assert_eq!(state.get(1, 2), Some(6.0));

    // Replace all data via message
    Heatmap::update(
        &mut state,
        HeatmapMessage::SetData(vec![vec![10.0, 20.0], vec![30.0, 40.0], vec![50.0, 60.0]]),
    );
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 2);
    assert_eq!(state.get(2, 1), Some(60.0));

    // Clear zeros all cells but preserves dimensions
    Heatmap::update(&mut state, HeatmapMessage::Clear);
    assert_eq!(state.rows(), 3);
    assert_eq!(state.cols(), 2);
    assert_eq!(state.get(0, 0), Some(0.0));
    assert_eq!(state.get(2, 1), Some(0.0));
}

// ============================================================================
// 23. Chart series cycling with multiple series
// ============================================================================

#[test]
fn test_chart_multi_series_cycling() {
    let state = ChartState::line(vec![
        DataSeries::new("Series A", vec![1.0, 2.0]),
        DataSeries::new("Series B", vec![3.0, 4.0]),
        DataSeries::new("Series C", vec![5.0, 6.0]),
    ]);

    let mut state = state;

    // Initially on series 0
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(1)));

    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(2)));

    // Wrap around
    let output = Chart::update(&mut state, ChartMessage::NextSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(0)));

    // PrevSeries
    let output = Chart::update(&mut state, ChartMessage::PrevSeries);
    assert_eq!(output, Some(ChartOutput::ActiveSeriesChanged(2)));
}

// ============================================================================
// 24. Calendar month-boundary wrapping
// ============================================================================

#[test]
fn test_calendar_month_boundary_wrapping() {
    // Start at December 2026
    let mut state = CalendarState::new(2026, 12);
    Calendar::focus(&mut state);

    // Next month wraps to January 2027
    let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 1)));
    assert_eq!(state.year(), 2027);
    assert_eq!(state.month(), 1);

    // Previous month goes back to December 2026
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 12)));
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 12);

    // January -> previous month -> December of previous year
    let mut state = CalendarState::new(2026, 1);
    Calendar::focus(&mut state);
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2025, 12)));
}

// ============================================================================
// 25. EventStream with structured fields
// ============================================================================

#[test]
fn test_event_stream_structured_fields_and_text_filter() {
    let mut state = EventStreamState::new();
    state.set_focused(true);

    state.push_event_with_fields(
        EventLevel::Info,
        "GET /api/users",
        vec![
            ("method".into(), "GET".into()),
            ("path".into(), "/api/users".into()),
            ("status".into(), "200".into()),
        ],
    );
    state.push_event_with_fields(
        EventLevel::Warning,
        "POST /api/orders",
        vec![
            ("method".into(), "POST".into()),
            ("path".into(), "/api/orders".into()),
            ("status".into(), "500".into()),
        ],
    );
    state.push_event_with_fields(
        EventLevel::Info,
        "GET /api/products",
        vec![
            ("method".into(), "GET".into()),
            ("path".into(), "/api/products".into()),
            ("status".into(), "200".into()),
        ],
    );

    assert_eq!(state.event_count(), 3);

    // Filter by text "orders"
    EventStream::update(
        &mut state,
        EventStreamMessage::SetFilter("orders".to_string()),
    );
    assert_eq!(state.visible_events().len(), 1);

    // Filter by text "GET"
    EventStream::update(&mut state, EventStreamMessage::SetFilter("GET".to_string()));
    assert_eq!(state.visible_events().len(), 2);

    // Clear filter
    EventStream::update(&mut state, EventStreamMessage::SetFilter(String::new()));
    assert_eq!(state.visible_events().len(), 3);
}

// ============================================================================
// 26. Slider with dispatch_event
// ============================================================================

#[test]
fn test_slider_dispatch_event_keyboard() {
    let mut state = SliderState::new(0.0, 100.0).with_step(5.0);
    state.set_focused(true);

    // Right arrow should increment
    let event = envision::input::Event::key(crossterm::event::KeyCode::Right);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 5.0);

    // Left arrow should decrement
    let event = envision::input::Event::key(crossterm::event::KeyCode::Left);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 0.0);

    // Home should go to min
    Slider::update(&mut state, SliderMessage::SetValue(50.0));
    let event = envision::input::Event::key(crossterm::event::KeyCode::Home);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 0.0);

    // End should go to max
    let event = envision::input::Event::key(crossterm::event::KeyCode::End);
    Slider::dispatch_event(&mut state, &event);
    assert_eq!(state.value(), 100.0);
}

// ============================================================================
// 27. Zero-size rendering for new components
// ============================================================================

fn assert_view_zero_size<F>(name: &str, render_fn: F)
where
    F: FnOnce(&mut ratatui::Frame, Rect, &envision::Theme),
{
    let backend = CaptureBackend::new(0, 0);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            let area = Rect::default();
            render_fn(frame, area, &theme);
        })
        .unwrap_or_else(|e| panic!("{} panicked on zero-size area: {}", name, e));
}

#[test]
fn test_new_components_handle_zero_size_area() {
    // Sparkline
    assert_view_zero_size("Sparkline", |frame, area, theme| {
        let state = SparklineState::with_data(vec![1, 2, 3]);
        Sparkline::view(&state, frame, area, theme);
    });

    // Gauge
    assert_view_zero_size("Gauge", |frame, area, theme| {
        let state = GaugeState::new(50.0, 100.0);
        Gauge::view(&state, frame, area, theme);
    });

    // Calendar
    assert_view_zero_size("Calendar", |frame, area, theme| {
        let state = CalendarState::new(2026, 3);
        Calendar::view(&state, frame, area, theme);
    });

    // Slider
    assert_view_zero_size("Slider", |frame, area, theme| {
        let state = SliderState::new(0.0, 100.0);
        Slider::view(&state, frame, area, theme);
    });

    // Switch
    assert_view_zero_size("Switch", |frame, area, theme| {
        let state = SwitchState::new();
        Switch::view(&state, frame, area, theme);
    });

    // Paginator
    assert_view_zero_size("Paginator", |frame, area, theme| {
        let state = PaginatorState::new(5);
        Paginator::view(&state, frame, area, theme);
    });

    // CodeBlock
    assert_view_zero_size("CodeBlock", |frame, area, theme| {
        let state = CodeBlockState::new().with_code("fn main() {}");
        CodeBlock::view(&state, frame, area, theme);
    });

    // TabBar
    assert_view_zero_size("TabBar", |frame, area, theme| {
        let state = TabBarState::new(vec![Tab::new("a", "Tab A")]);
        TabBar::view(&state, frame, area, theme);
    });
}

// ============================================================================
// 28. SpanTree with deep nesting
// ============================================================================

#[test]
fn test_span_tree_deep_nesting() {
    let root = SpanNode::new("root", "service", 0.0, 1000.0).with_child(
        SpanNode::new("l1", "handler", 10.0, 900.0).with_child(
            SpanNode::new("l2", "middleware", 20.0, 880.0)
                .with_child(SpanNode::new("l3", "db-query", 30.0, 500.0))
                .with_child(SpanNode::new("l4", "cache-set", 510.0, 600.0)),
        ),
    );

    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    // Selected at root
    assert_eq!(state.selected_index(), Some(0));

    // Navigate all the way down
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(1)); // handler
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(2)); // middleware
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(3)); // db-query
    SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(4)); // cache-set

    // Navigate back up
    SpanTree::update(&mut state, SpanTreeMessage::SelectUp);
    assert_eq!(state.selected_index(), Some(3)); // db-query
}

// ============================================================================
// 29. Paginator from items
// ============================================================================

#[test]
fn test_paginator_from_items_calculation() {
    let state = PaginatorState::from_items(247, 25);
    assert_eq!(state.total_pages(), 10);
    assert_eq!(state.total_items(), 247);
    assert_eq!(state.page_size(), 25);
    assert_eq!(state.current_page(), 0);

    // Exact division
    let state = PaginatorState::from_items(100, 10);
    assert_eq!(state.total_pages(), 10);

    // Single item
    let state = PaginatorState::from_items(1, 10);
    assert_eq!(state.total_pages(), 1);
}

// ============================================================================
// 30. Full workflow: dashboard with multiple new components
// ============================================================================

#[test]
fn test_dashboard_workflow_with_mixed_new_components() {
    // Simulate a monitoring dashboard that uses several new components together

    // CPU gauge
    let mut cpu_gauge = GaugeState::new(0.0, 100.0)
        .with_units("%")
        .with_title("CPU");

    // Memory sparkline
    let mut mem_sparkline = SparklineState::new().with_title("Memory");

    // Alert panel
    let mut alerts = AlertPanelState::new().with_metrics(vec![
        AlertMetric::new("cpu", "CPU", AlertThreshold::new(70.0, 90.0))
            .with_units("%")
            .with_value(0.0),
        AlertMetric::new("mem", "Memory", AlertThreshold::new(80.0, 95.0))
            .with_units("%")
            .with_value(0.0),
    ]);

    // Paginator for log pages
    let mut paginator = PaginatorState::new(1);

    // Simulate metric updates over time
    let cpu_values = [25.0, 45.0, 60.0, 72.0, 85.0, 91.0, 78.0, 55.0];
    let mem_values = [40u64, 42, 45, 50, 55, 60, 62, 65];

    for (i, (&cpu, &mem)) in cpu_values.iter().zip(mem_values.iter()).enumerate() {
        // Update gauge
        Gauge::update(&mut cpu_gauge, GaugeMessage::SetValue(cpu));
        assert_eq!(cpu_gauge.value(), cpu);

        // Push sparkline data
        Sparkline::update(&mut mem_sparkline, SparklineMessage::Push(mem));
        assert_eq!(mem_sparkline.len(), i + 1);

        // Update alert metric
        AlertPanel::update(
            &mut alerts,
            AlertPanelMessage::UpdateMetric {
                id: "cpu".into(),
                value: cpu,
            },
        );
        AlertPanel::update(
            &mut alerts,
            AlertPanelMessage::UpdateMetric {
                id: "mem".into(),
                value: mem as f64,
            },
        );
    }

    // After all updates, verify final states
    assert_eq!(cpu_gauge.value(), 55.0);
    assert_eq!(cpu_gauge.current_color(), Color::Green); // 55% is below 70%
    assert_eq!(mem_sparkline.len(), 8);
    assert_eq!(mem_sparkline.last(), Some(65));

    // All metrics should be OK at these values
    assert_eq!(alerts.ok_count(), 2);

    // Update paginator for new data
    Paginator::update(&mut paginator, PaginatorMessage::SetTotalPages(5));
    Paginator::update(&mut paginator, PaginatorMessage::NextPage);
    assert_eq!(paginator.current_page(), 1);

    // Render all components
    let backend = CaptureBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let theme = envision::Theme::default();
    terminal
        .draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(10),
                    Constraint::Length(1),
                ])
                .split(frame.area());

            Gauge::view(&cpu_gauge, frame, chunks[0], &theme);
            Sparkline::view(&mem_sparkline, frame, chunks[1], &theme);
            AlertPanel::view(&alerts, frame, chunks[2], &theme);
            Paginator::view(&paginator, frame, chunks[3], &theme);
        })
        .unwrap();
}
