use super::*;
use crate::component::Component;
use crate::input::{Event, KeyCode};
use ratatui::style::Color;

// =============================================================================
// FlameNode Construction
// =============================================================================

#[test]
fn test_flame_node_new() {
    let node = FlameNode::new("main()", 500);
    assert_eq!(node.label(), "main()");
    assert_eq!(node.value(), 500);
    assert_eq!(node.color(), Color::Cyan);
    assert!(node.children().is_empty());
    assert!(!node.has_children());
}

#[test]
fn test_flame_node_with_color() {
    let node = FlameNode::new("main()", 500).with_color(Color::Red);
    assert_eq!(node.color(), Color::Red);
}

#[test]
fn test_flame_node_with_child() {
    let node = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    assert_eq!(node.children().len(), 1);
    assert!(node.has_children());
    assert_eq!(node.children()[0].label(), "compute()");
}

#[test]
fn test_flame_node_with_children() {
    let children = vec![
        FlameNode::new("a()", 100),
        FlameNode::new("b()", 200),
        FlameNode::new("c()", 150),
    ];
    let node = FlameNode::new("main()", 500).with_children(children);
    assert_eq!(node.children().len(), 3);
    assert!(node.has_children());
}

#[test]
fn test_flame_node_total_value() {
    let node = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    assert_eq!(node.total_value(), 500);
}

#[test]
fn test_flame_node_self_value() {
    let node = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300))
        .with_child(FlameNode::new("io()", 100));
    assert_eq!(node.self_value(), 100); // 500 - 300 - 100
}

#[test]
fn test_flame_node_self_value_no_children() {
    let node = FlameNode::new("leaf()", 100);
    assert_eq!(node.self_value(), 100);
}

#[test]
fn test_flame_node_self_value_children_exceed_parent() {
    // Edge case: children's values exceed parent (saturating_sub)
    let node = FlameNode::new("main()", 100)
        .with_child(FlameNode::new("compute()", 80))
        .with_child(FlameNode::new("io()", 80));
    assert_eq!(node.self_value(), 0); // saturating_sub
}

// =============================================================================
// FlameGraphState Construction
// =============================================================================

#[test]
fn test_state_new() {
    let state = FlameGraphState::new();
    assert!(state.root().is_none());
    assert!(state.zoom_stack().is_empty());
    assert_eq!(state.selected_depth(), 0);
    assert_eq!(state.selected_index(), 0);
    assert_eq!(state.search_query(), "");
    assert_eq!(state.title(), None);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
}

#[test]
fn test_state_with_root() {
    let root = FlameNode::new("main()", 500);
    let state = FlameGraphState::with_root(root);
    assert!(state.root().is_some());
    assert_eq!(state.root().unwrap().label(), "main()");
}

#[test]
fn test_state_with_title() {
    let state = FlameGraphState::with_root(FlameNode::new("main()", 500)).with_title("CPU Profile");
    assert_eq!(state.title(), Some("CPU Profile"));
}

#[test]
fn test_state_with_disabled() {
    let state = FlameGraphState::with_root(FlameNode::new("main()", 500)).with_disabled(true);
    assert!(state.is_disabled());
}

#[test]
fn test_state_default() {
    let state = FlameGraphState::default();
    assert!(state.root().is_none());
}

// =============================================================================
// Set Root / Clear
// =============================================================================

#[test]
fn test_set_root() {
    let mut state = FlameGraphState::new();
    state.set_root(FlameNode::new("main()", 500));
    assert!(state.root().is_some());
    assert_eq!(state.root().unwrap().label(), "main()");
}

#[test]
fn test_set_root_resets_zoom() {
    let mut state = FlameGraphState::with_root(
        FlameNode::new("main()", 500)
            .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200))),
    );
    state.set_focused(true);
    state.select_down();
    state.zoom_in();
    assert!(!state.zoom_stack().is_empty());

    state.set_root(FlameNode::new("new_main()", 1000));
    assert!(state.zoom_stack().is_empty());
    assert_eq!(state.selected_depth(), 0);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_clear() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_search("test".to_string());
    state.clear();
    assert!(state.root().is_none());
    assert_eq!(state.search_query(), "");
    assert!(state.zoom_stack().is_empty());
}

// =============================================================================
// Navigation: Up/Down/Left/Right
// =============================================================================

#[test]
fn test_select_down() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert_eq!(state.selected_depth(), 0);
    assert!(state.select_down());
    assert_eq!(state.selected_depth(), 1);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_down_no_children() {
    let root = FlameNode::new("main()", 500);
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert!(!state.select_down());
    assert_eq!(state.selected_depth(), 0);
}

#[test]
fn test_select_down_empty() {
    let mut state = FlameGraphState::new();
    assert!(!state.select_down());
}

#[test]
fn test_select_up() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    assert_eq!(state.selected_depth(), 1);

    assert!(state.select_up());
    assert_eq!(state.selected_depth(), 0);
}

#[test]
fn test_select_up_at_root() {
    let root = FlameNode::new("main()", 500);
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert!(!state.select_up());
    assert_eq!(state.selected_depth(), 0);
}

#[test]
fn test_select_up_empty() {
    let mut state = FlameGraphState::new();
    assert!(!state.select_up());
}

#[test]
fn test_select_left() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300))
        .with_child(FlameNode::new("io()", 100));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down(); // depth 1, index 0
    state.select_right(); // depth 1, index 1
    assert_eq!(state.selected_index(), 1);

    assert!(state.select_left());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_left_at_first() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    assert!(!state.select_left());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_left_empty() {
    let mut state = FlameGraphState::new();
    assert!(!state.select_left());
}

#[test]
fn test_select_right() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300))
        .with_child(FlameNode::new("io()", 100));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down(); // depth 1, index 0
    assert!(state.select_right());
    assert_eq!(state.selected_index(), 1);
}

#[test]
fn test_select_right_at_last() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    assert!(!state.select_right());
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_select_right_empty() {
    let mut state = FlameGraphState::new();
    assert!(!state.select_right());
}

#[test]
fn test_navigation_deep() {
    let root = FlameNode::new("main()", 500).with_child(
        FlameNode::new("compute()", 300)
            .with_child(FlameNode::new("sort()", 200).with_child(FlameNode::new("merge()", 100))),
    );
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert!(state.select_down()); // depth 1: compute()
    assert!(state.select_down()); // depth 2: sort()
    assert!(state.select_down()); // depth 3: merge()
    assert!(!state.select_down()); // no deeper
    assert_eq!(state.selected_depth(), 3);

    assert!(state.select_up()); // back to depth 2: sort()
    assert_eq!(state.selected_depth(), 2);
}

// =============================================================================
// Zoom
// =============================================================================

#[test]
fn test_zoom_in() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Select compute() and zoom in
    state.select_down();
    assert!(state.zoom_in());
    assert_eq!(state.zoom_stack(), &["compute()".to_string()]);
    assert_eq!(state.current_view_root().unwrap().label(), "compute()");
    assert_eq!(state.selected_depth(), 0);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_zoom_in_leaf_fails() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("leaf()", 100));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Select leaf and try to zoom
    state.select_down();
    assert!(!state.zoom_in());
    assert!(state.zoom_stack().is_empty());
}

#[test]
fn test_zoom_out() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    state.zoom_in();
    assert!(state.zoom_out());
    assert!(state.zoom_stack().is_empty());
    assert_eq!(state.current_view_root().unwrap().label(), "main()");
}

#[test]
fn test_zoom_out_at_root() {
    let root = FlameNode::new("main()", 500);
    let mut state = FlameGraphState::with_root(root);
    assert!(!state.zoom_out());
}

#[test]
fn test_reset_zoom() {
    let root = FlameNode::new("main()", 500).with_child(
        FlameNode::new("compute()", 300)
            .with_child(FlameNode::new("sort()", 200).with_child(FlameNode::new("merge()", 100))),
    );
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    state.zoom_in();
    state.select_down();
    state.zoom_in();
    assert_eq!(state.zoom_stack().len(), 2);

    state.reset_zoom();
    assert!(state.zoom_stack().is_empty());
    assert_eq!(state.selected_depth(), 0);
    assert_eq!(state.selected_index(), 0);
}

#[test]
fn test_zoom_stack_deep() {
    let root = FlameNode::new("main()", 500).with_child(
        FlameNode::new("a()", 400)
            .with_child(FlameNode::new("b()", 300).with_child(FlameNode::new("c()", 200))),
    );
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Zoom into a()
    state.select_down();
    state.zoom_in();
    assert_eq!(state.current_view_root().unwrap().label(), "a()");

    // Zoom into b()
    state.select_down();
    state.zoom_in();
    assert_eq!(state.current_view_root().unwrap().label(), "b()");

    // Zoom out one level
    state.zoom_out();
    assert_eq!(state.current_view_root().unwrap().label(), "a()");
}

// =============================================================================
// Search
// =============================================================================

#[test]
fn test_set_search() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_search("compute".to_string());
    assert_eq!(state.search_query(), "compute");
}

#[test]
fn test_clear_search_via_set() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_search("compute".to_string());
    state.set_search(String::new());
    assert_eq!(state.search_query(), "");
}

// =============================================================================
// Current View Root
// =============================================================================

#[test]
fn test_current_view_root_no_zoom() {
    let root = FlameNode::new("main()", 500);
    let state = FlameGraphState::with_root(root);
    assert_eq!(state.current_view_root().unwrap().label(), "main()");
}

#[test]
fn test_current_view_root_empty() {
    let state = FlameGraphState::new();
    assert!(state.current_view_root().is_none());
}

#[test]
fn test_current_view_root_with_zoom() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.select_down();
    state.zoom_in();
    assert_eq!(state.current_view_root().unwrap().label(), "compute()");
}

// =============================================================================
// Selected Frame
// =============================================================================

#[test]
fn test_selected_frame_at_root() {
    let root = FlameNode::new("main()", 500);
    let state = FlameGraphState::with_root(root);
    assert_eq!(state.selected_frame().unwrap().label(), "main()");
}

#[test]
fn test_selected_frame_after_navigation() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300))
        .with_child(FlameNode::new("io()", 100));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down();
    assert_eq!(state.selected_frame().unwrap().label(), "compute()");

    state.select_right();
    assert_eq!(state.selected_frame().unwrap().label(), "io()");
}

#[test]
fn test_selected_frame_empty() {
    let state = FlameGraphState::new();
    assert!(state.selected_frame().is_none());
}

// =============================================================================
// Max Depth
// =============================================================================

#[test]
fn test_max_depth_single() {
    let state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    assert_eq!(state.max_depth(), 0);
}

#[test]
fn test_max_depth_deep() {
    let root = FlameNode::new("main()", 500).with_child(
        FlameNode::new("a()", 300)
            .with_child(FlameNode::new("b()", 200).with_child(FlameNode::new("c()", 100))),
    );
    let state = FlameGraphState::with_root(root);
    assert_eq!(state.max_depth(), 3);
}

#[test]
fn test_max_depth_empty() {
    let state = FlameGraphState::new();
    assert_eq!(state.max_depth(), 0);
}

// =============================================================================
// Handle Event
// =============================================================================

#[test]
fn test_handle_event_not_focused() {
    let state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    let msg = FlameGraph::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    state.set_disabled(true);
    let msg = FlameGraph::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_down() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Down)),
        Some(FlameGraphMessage::SelectDown)
    );
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::char('j')),
        Some(FlameGraphMessage::SelectDown)
    );
}

#[test]
fn test_handle_event_up() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Up)),
        Some(FlameGraphMessage::SelectUp)
    );
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::char('k')),
        Some(FlameGraphMessage::SelectUp)
    );
}

#[test]
fn test_handle_event_left() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Left)),
        Some(FlameGraphMessage::SelectLeft)
    );
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::char('h')),
        Some(FlameGraphMessage::SelectLeft)
    );
}

#[test]
fn test_handle_event_right() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Right)),
        Some(FlameGraphMessage::SelectRight)
    );
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::char('l')),
        Some(FlameGraphMessage::SelectRight)
    );
}

#[test]
fn test_handle_event_enter_zoom_in() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Enter)),
        Some(FlameGraphMessage::ZoomIn)
    );
}

#[test]
fn test_handle_event_escape_zoom_out() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Esc)),
        Some(FlameGraphMessage::ZoomOut)
    );
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Backspace)),
        Some(FlameGraphMessage::ZoomOut)
    );
}

#[test]
fn test_handle_event_home_reset_zoom() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::key(KeyCode::Home)),
        Some(FlameGraphMessage::ResetZoom)
    );
}

#[test]
fn test_handle_event_slash_search() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(
        FlameGraph::handle_event(&state, &Event::char('/')),
        Some(FlameGraphMessage::SetSearch(String::new()))
    );
}

#[test]
fn test_handle_event_unknown_key() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    assert_eq!(FlameGraph::handle_event(&state, &Event::char('x')), None);
}

// =============================================================================
// Component Update Messages
// =============================================================================

#[test]
fn test_update_set_root() {
    let mut state = FlameGraphState::new();
    let output = FlameGraph::update(
        &mut state,
        FlameGraphMessage::SetRoot(FlameNode::new("main()", 500)),
    );
    assert_eq!(output, None);
    assert!(state.root().is_some());
}

#[test]
fn test_update_clear() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    let output = FlameGraph::update(&mut state, FlameGraphMessage::Clear);
    assert_eq!(output, None);
    assert!(state.root().is_none());
}

#[test]
fn test_update_set_search() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    FlameGraph::update(&mut state, FlameGraphMessage::SetSearch("test".to_string()));
    assert_eq!(state.search_query(), "test");
}

#[test]
fn test_update_clear_search() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_search("test".to_string());
    FlameGraph::update(&mut state, FlameGraphMessage::ClearSearch);
    assert_eq!(state.search_query(), "");
}

#[test]
fn test_update_select_down_output() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
    assert_eq!(
        output,
        Some(FlameGraphOutput::FrameSelected {
            label: "compute()".to_string(),
            value: 300,
            self_value: 300,
        })
    );
}

#[test]
fn test_update_select_up_output() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.select_down();

    let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectUp);
    assert_eq!(
        output,
        Some(FlameGraphOutput::FrameSelected {
            label: "main()".to_string(),
            value: 500,
            self_value: 200,
        })
    );
}

#[test]
fn test_update_zoom_in_output() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.select_down();

    let output = FlameGraph::update(&mut state, FlameGraphMessage::ZoomIn);
    assert_eq!(
        output,
        Some(FlameGraphOutput::ZoomedIn("compute()".to_string()))
    );
}

#[test]
fn test_update_zoom_out_output() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("compute()", 300).with_child(FlameNode::new("sort()", 200)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.select_down();
    state.zoom_in();

    let output = FlameGraph::update(&mut state, FlameGraphMessage::ZoomOut);
    assert_eq!(output, Some(FlameGraphOutput::ZoomedOut));
}

#[test]
fn test_update_reset_zoom() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.select_down();
    state.zoom_in();

    let output = FlameGraph::update(&mut state, FlameGraphMessage::ResetZoom);
    assert_eq!(output, None);
    assert!(state.zoom_stack().is_empty());
}

#[test]
fn test_update_disabled_ignores_navigation() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);
    state.set_disabled(true);

    let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
    assert_eq!(output, None);
    assert_eq!(state.selected_depth(), 0);
}

#[test]
fn test_update_empty_graph_ignores_navigation() {
    let mut state = FlameGraphState::new();
    let output = FlameGraph::update(&mut state, FlameGraphMessage::SelectDown);
    assert_eq!(output, None);
}

// =============================================================================
// Dispatch Event
// =============================================================================

#[test]
fn test_dispatch_event() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(
        output,
        Some(FlameGraphOutput::FrameSelected {
            label: "compute()".to_string(),
            value: 300,
            self_value: 300,
        })
    );
}

// =============================================================================
// Instance Methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    state.set_focused(true);
    let msg = state.handle_event(&Event::key(KeyCode::Down));
    assert_eq!(msg, Some(FlameGraphMessage::SelectDown));
}

#[test]
fn test_instance_update() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("compute()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    let output = state.update(FlameGraphMessage::SelectDown);
    assert!(output.is_some());
}

// =============================================================================
// Component Trait
// =============================================================================

#[test]
fn test_init() {
    let state = FlameGraph::init();
    assert!(state.root().is_none());
}

// =============================================================================
// Focusable / Disableable
// =============================================================================

#[test]
fn test_focusable() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    assert!(!FlameGraph::is_focused(&state));
    FlameGraph::set_focused(&mut state, true);
    assert!(FlameGraph::is_focused(&state));
    FlameGraph::blur(&mut state);
    assert!(!FlameGraph::is_focused(&state));
    FlameGraph::focus(&mut state);
    assert!(FlameGraph::is_focused(&state));
}

#[test]
fn test_disableable() {
    let mut state = FlameGraphState::with_root(FlameNode::new("main()", 500));
    assert!(!FlameGraph::is_disabled(&state));
    FlameGraph::set_disabled(&mut state, true);
    assert!(FlameGraph::is_disabled(&state));
    FlameGraph::enable(&mut state);
    assert!(!FlameGraph::is_disabled(&state));
    FlameGraph::disable(&mut state);
    assert!(FlameGraph::is_disabled(&state));
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_zoom_in_on_root_with_children() {
    let root = FlameNode::new("main()", 500).with_child(FlameNode::new("a()", 300));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Zoom into root (which has children)
    assert!(state.zoom_in());
    assert_eq!(state.current_view_root().unwrap().label(), "main()");
    // Note: zooming into root pushes "main()" onto zoom stack,
    // but current_view_root follows the stack from self.root
    // and "main()" won't be found as a child of root.
    // This means zoom_in on the root itself shouldn't work the same way.
    // Let's verify the actual behavior.
}

#[test]
fn test_select_down_multiple_children() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("a()", 200))
        .with_child(FlameNode::new("b()", 150))
        .with_child(FlameNode::new("c()", 100));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    state.select_down(); // depth 1
    assert_eq!(state.selected_frame().unwrap().label(), "a()");

    state.select_right();
    assert_eq!(state.selected_frame().unwrap().label(), "b()");

    state.select_right();
    assert_eq!(state.selected_frame().unwrap().label(), "c()");

    // Can't go right past last
    assert!(!state.select_right());
}

#[test]
fn test_navigation_sibling_at_depth_zero() {
    // At depth 0 there's only one frame (the root)
    let root = FlameNode::new("main()", 500);
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    assert!(!state.select_left());
    assert!(!state.select_right());
}

#[test]
fn test_deep_nesting_navigation() {
    let root = FlameNode::new("d0", 1000).with_child(
        FlameNode::new("d1", 800).with_child(
            FlameNode::new("d2", 600)
                .with_child(FlameNode::new("d3", 400).with_child(FlameNode::new("d4", 200))),
        ),
    );
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Navigate all the way down
    for depth in 1..=4 {
        assert!(state.select_down());
        assert_eq!(state.selected_depth(), depth);
    }
    // Can't go deeper
    assert!(!state.select_down());

    // Navigate all the way back up
    for depth in (0..4).rev() {
        assert!(state.select_up());
        assert_eq!(state.selected_depth(), depth);
    }
    assert!(!state.select_up());
}

#[test]
fn test_zoom_in_no_children_returns_false() {
    let root = FlameNode::new("leaf()", 100);
    let mut state = FlameGraphState::with_root(root);
    assert!(!state.zoom_in());
}

#[test]
fn test_select_up_finds_correct_parent() {
    let root = FlameNode::new("main()", 500)
        .with_child(FlameNode::new("a()", 200).with_child(FlameNode::new("a1()", 100)))
        .with_child(FlameNode::new("b()", 200).with_child(FlameNode::new("b1()", 100)));
    let mut state = FlameGraphState::with_root(root);
    state.set_focused(true);

    // Go to depth 1, select b()
    state.select_down();
    state.select_right();
    assert_eq!(state.selected_frame().unwrap().label(), "b()");

    // Go to depth 2 (b1())
    state.select_down();
    assert_eq!(state.selected_frame().unwrap().label(), "b1()");

    // Go back up - should return to b()'s position
    state.select_up();
    assert_eq!(state.selected_frame().unwrap().label(), "b()");
}
