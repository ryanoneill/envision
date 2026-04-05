use super::*;
use crate::component::Component;
use crate::input::{Event, KeyCode, KeyModifiers};
use ratatui::style::Color;

// =============================================================================
// SpanNode Construction
// =============================================================================

#[test]
fn test_span_node_new() {
    let node = SpanNode::new("id-1", "my-service", 10.0, 200.0);
    assert_eq!(node.id(), "id-1");
    assert_eq!(node.label(), "my-service");
    assert_eq!(node.start(), 10.0);
    assert_eq!(node.end(), 200.0);
    assert_eq!(node.color(), Color::Cyan); // default
    assert_eq!(node.status(), None);
    assert!(node.children().is_empty());
    assert!(!node.has_children());
}

#[test]
fn test_span_node_duration() {
    let node = SpanNode::new("id", "svc", 50.0, 200.0);
    assert_eq!(node.duration(), 150.0);
}

#[test]
fn test_span_node_with_color() {
    let node = SpanNode::new("id", "svc", 0.0, 10.0).with_color(Color::Red);
    assert_eq!(node.color(), Color::Red);
}

#[test]
fn test_span_node_with_status() {
    let node = SpanNode::new("id", "svc", 0.0, 10.0).with_status("200 OK");
    assert_eq!(node.status(), Some("200 OK"));
}

#[test]
fn test_span_node_with_child() {
    let node = SpanNode::new("parent", "svc", 0.0, 100.0)
        .with_child(SpanNode::new("child", "db", 10.0, 50.0));
    assert_eq!(node.children().len(), 1);
    assert!(node.has_children());
    assert_eq!(node.children()[0].id(), "child");
}

#[test]
fn test_span_node_with_children() {
    let children = vec![
        SpanNode::new("c1", "a", 0.0, 10.0),
        SpanNode::new("c2", "b", 10.0, 20.0),
        SpanNode::new("c3", "c", 20.0, 30.0),
    ];
    let node = SpanNode::new("p", "parent", 0.0, 30.0).with_children(children);
    assert_eq!(node.children().len(), 3);
    assert!(node.has_children());
}

// =============================================================================
// SpanTreeState Construction
// =============================================================================

#[test]
fn test_state_default() {
    let state = SpanTreeState::default();
    assert!(state.is_empty());
    assert!(state.roots().is_empty());
    assert_eq!(state.selected_index(), None);
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert_eq!(state.label_width(), 30);
    assert_eq!(state.title(), None);
    assert_eq!(state.global_start(), 0.0);
    assert_eq!(state.global_end(), 0.0);
}

#[test]
fn test_state_new_single_root() {
    let root = SpanNode::new("r", "root", 0.0, 100.0);
    let state = SpanTreeState::new(vec![root]);
    assert_eq!(state.roots().len(), 1);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.global_start(), 0.0);
    assert_eq!(state.global_end(), 100.0);
}

#[test]
fn test_state_new_multiple_roots() {
    let roots = vec![
        SpanNode::new("a", "svc-a", 0.0, 50.0),
        SpanNode::new("b", "svc-b", 20.0, 100.0),
    ];
    let state = SpanTreeState::new(roots);
    assert_eq!(state.roots().len(), 2);
    assert_eq!(state.global_start(), 0.0);
    assert_eq!(state.global_end(), 100.0);
}

#[test]
fn test_state_new_empty() {
    let state = SpanTreeState::new(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_with_title() {
    let state =
        SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]).with_title("My Trace");
    assert_eq!(state.title(), Some("My Trace"));
}

#[test]
fn test_state_with_label_width() {
    let state =
        SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]).with_label_width(50);
    assert_eq!(state.label_width(), 50);
}

#[test]
fn test_state_with_disabled() {
    let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]).with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// Global Time Computation
// =============================================================================

#[test]
fn test_global_time_single_span() {
    let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 10.0, 50.0)]);
    assert_eq!(state.global_start(), 10.0);
    assert_eq!(state.global_end(), 50.0);
}

#[test]
fn test_global_time_nested_spans() {
    let root = SpanNode::new("r", "root", 0.0, 100.0).with_child(
        SpanNode::new("c", "child", 10.0, 200.0), // child extends beyond root
    );
    let state = SpanTreeState::new(vec![root]);
    assert_eq!(state.global_start(), 0.0);
    assert_eq!(state.global_end(), 200.0);
}

#[test]
fn test_global_time_multiple_roots() {
    let roots = vec![
        SpanNode::new("a", "a", 5.0, 50.0),
        SpanNode::new("b", "b", 20.0, 120.0),
    ];
    let state = SpanTreeState::new(roots);
    assert_eq!(state.global_start(), 5.0);
    assert_eq!(state.global_end(), 120.0);
}

#[test]
fn test_global_time_deep_nesting() {
    let root = SpanNode::new("r", "root", 10.0, 100.0).with_child(
        SpanNode::new("c1", "child", 20.0, 80.0).with_child(SpanNode::new(
            "gc",
            "grandchild",
            1.0,
            150.0,
        )),
    );
    let state = SpanTreeState::new(roots(root));
    assert_eq!(state.global_start(), 1.0);
    assert_eq!(state.global_end(), 150.0);
}

// =============================================================================
// Flatten
// =============================================================================

#[test]
fn test_flatten_single_root_no_children() {
    let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 100.0)]);
    let flat = state.flatten();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].id(), "r");
    assert_eq!(flat[0].depth(), 0);
    assert!(!flat[0].has_children());
}

#[test]
fn test_flatten_expanded_tree() {
    let root = SpanNode::new("r", "root", 0.0, 100.0)
        .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0))
        .with_child(SpanNode::new("c2", "child-2", 50.0, 90.0));
    let state = SpanTreeState::new(vec![root]);
    let flat = state.flatten();
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].id(), "r");
    assert_eq!(flat[0].depth(), 0);
    assert!(flat[0].has_children());
    assert!(flat[0].is_expanded());
    assert_eq!(flat[1].id(), "c1");
    assert_eq!(flat[1].depth(), 1);
    assert_eq!(flat[2].id(), "c2");
    assert_eq!(flat[2].depth(), 1);
}

#[test]
fn test_flatten_collapsed_tree() {
    let root = SpanNode::new("r", "root", 0.0, 100.0)
        .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0))
        .with_child(SpanNode::new("c2", "child-2", 50.0, 90.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.collapse("r");
    let flat = state.flatten();
    assert_eq!(flat.len(), 1);
    assert_eq!(flat[0].id(), "r");
    assert!(flat[0].has_children());
    assert!(!flat[0].is_expanded());
}

#[test]
fn test_flatten_deeply_nested() {
    let root = SpanNode::new("r", "root", 0.0, 100.0).with_child(
        SpanNode::new("c1", "child", 10.0, 80.0).with_child(
            SpanNode::new("gc1", "grandchild", 20.0, 60.0).with_child(SpanNode::new(
                "ggc1",
                "great-grandchild",
                30.0,
                50.0,
            )),
        ),
    );
    let state = SpanTreeState::new(vec![root]);
    let flat = state.flatten();
    assert_eq!(flat.len(), 4);
    assert_eq!(flat[0].depth(), 0);
    assert_eq!(flat[1].depth(), 1);
    assert_eq!(flat[2].depth(), 2);
    assert_eq!(flat[3].depth(), 3);
}

#[test]
fn test_flatten_multiple_roots() {
    let roots = vec![
        SpanNode::new("a", "root-a", 0.0, 50.0)
            .with_child(SpanNode::new("ac", "child-a", 10.0, 40.0)),
        SpanNode::new("b", "root-b", 20.0, 100.0),
    ];
    let state = SpanTreeState::new(roots);
    let flat = state.flatten();
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].id(), "a");
    assert_eq!(flat[1].id(), "ac");
    assert_eq!(flat[2].id(), "b");
}

#[test]
fn test_flatten_empty() {
    let state = SpanTreeState::default();
    assert!(state.flatten().is_empty());
}

// =============================================================================
// Expand / Collapse
// =============================================================================

#[test]
fn test_expand_collapse_individual() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);

    // Initially expanded
    assert!(state.expanded_ids().contains("r"));
    assert_eq!(state.flatten().len(), 2);

    // Collapse
    state.collapse("r");
    assert!(!state.expanded_ids().contains("r"));
    assert_eq!(state.flatten().len(), 1);

    // Re-expand
    state.expand("r");
    assert!(state.expanded_ids().contains("r"));
    assert_eq!(state.flatten().len(), 2);
}

#[test]
fn test_expand_all() {
    let root = SpanNode::new("r", "root", 0.0, 100.0).with_child(
        SpanNode::new("c", "child", 10.0, 80.0).with_child(SpanNode::new(
            "gc",
            "grandchild",
            20.0,
            60.0,
        )),
    );
    let mut state = SpanTreeState::new(vec![root]);
    state.collapse_all();
    assert_eq!(state.flatten().len(), 1);

    state.expand_all();
    assert_eq!(state.flatten().len(), 3);
    assert!(state.expanded_ids().contains("r"));
    assert!(state.expanded_ids().contains("c"));
}

#[test]
fn test_collapse_all() {
    let root = SpanNode::new("r", "root", 0.0, 100.0).with_child(
        SpanNode::new("c", "child", 10.0, 80.0).with_child(SpanNode::new(
            "gc",
            "grandchild",
            20.0,
            60.0,
        )),
    );
    let mut state = SpanTreeState::new(vec![root]);
    assert_eq!(state.flatten().len(), 3);

    state.collapse_all();
    assert!(state.expanded_ids().is_empty());
    assert_eq!(state.flatten().len(), 1);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_collapse_clamps_selection() {
    let root = SpanNode::new("r", "root", 0.0, 100.0)
        .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0))
        .with_child(SpanNode::new("c2", "child-2", 50.0, 90.0));
    let mut state = SpanTreeState::new(vec![root]);
    // Select the last child (index 2)
    state.selected_index = Some(2);

    // Collapse root - selection should clamp
    state.collapse("r");
    assert_eq!(state.selected_index(), Some(0));
}

// =============================================================================
// Navigation
// =============================================================================

#[test]
fn test_select_down() {
    let root = SpanNode::new("r", "root", 0.0, 100.0)
        .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0))
        .with_child(SpanNode::new("c2", "child-2", 50.0, 90.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    assert_eq!(state.selected_index(), Some(0));

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(output, Some(SpanTreeOutput::Selected("c1".into())));

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(output, Some(SpanTreeOutput::Selected("c2".into())));
}

#[test]
fn test_select_down_at_bottom() {
    let state_node = SpanNode::new("r", "root", 0.0, 100.0);
    let mut state = SpanTreeState::new(vec![state_node]);
    state.set_focused(true);
    assert_eq!(state.selected_index(), Some(0));

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_select_up() {
    let root = SpanNode::new("r", "root", 0.0, 100.0)
        .with_child(SpanNode::new("c1", "child-1", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);
    state.selected_index = Some(1);

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectUp);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, Some(SpanTreeOutput::Selected("r".into())));
}

#[test]
fn test_select_up_at_top() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 100.0)]);
    state.set_focused(true);

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectUp);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(output, None);
}

#[test]
fn test_expand_via_message() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);
    state.collapse("r");
    assert_eq!(state.flatten().len(), 1);

    let output = SpanTree::update(&mut state, SpanTreeMessage::Expand);
    assert_eq!(output, Some(SpanTreeOutput::Expanded("r".into())));
    assert_eq!(state.flatten().len(), 2);
}

#[test]
fn test_collapse_via_message() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);
    assert_eq!(state.flatten().len(), 2);

    let output = SpanTree::update(&mut state, SpanTreeMessage::Collapse);
    assert_eq!(output, Some(SpanTreeOutput::Collapsed("r".into())));
    assert_eq!(state.flatten().len(), 1);
}

#[test]
fn test_toggle_collapse() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    // Toggle: expanded -> collapsed
    let output = SpanTree::update(&mut state, SpanTreeMessage::Toggle);
    assert_eq!(output, Some(SpanTreeOutput::Collapsed("r".into())));
    assert_eq!(state.flatten().len(), 1);

    // Toggle: collapsed -> expanded
    let output = SpanTree::update(&mut state, SpanTreeMessage::Toggle);
    assert_eq!(output, Some(SpanTreeOutput::Expanded("r".into())));
    assert_eq!(state.flatten().len(), 2);
}

#[test]
fn test_toggle_leaf_node() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);
    state.selected_index = Some(1); // select leaf

    let output = SpanTree::update(&mut state, SpanTreeMessage::Toggle);
    assert_eq!(output, None); // leaf nodes can't toggle
}

// =============================================================================
// Set Roots
// =============================================================================

#[test]
fn test_set_roots() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("old", "old", 0.0, 10.0)]);
    state.set_roots(vec![
        SpanNode::new("new1", "new-1", 5.0, 50.0),
        SpanNode::new("new2", "new-2", 10.0, 100.0),
    ]);
    assert_eq!(state.roots().len(), 2);
    assert_eq!(state.global_start(), 5.0);
    assert_eq!(state.global_end(), 100.0);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_set_roots_empty() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_roots(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_set_roots_reexpands() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![SpanNode::new("old", "old", 0.0, 10.0)]);
    state.set_roots(vec![root]);
    // New tree should be fully expanded
    assert!(state.expanded_ids().contains("r"));
    assert_eq!(state.flatten().len(), 2);
}

// =============================================================================
// Set Label Width
// =============================================================================

#[test]
fn test_set_label_width() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    SpanTree::update(&mut state, SpanTreeMessage::SetLabelWidth(50));
    assert_eq!(state.label_width(), 50);
}

#[test]
fn test_set_label_width_clamped_low() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    SpanTree::update(&mut state, SpanTreeMessage::SetLabelWidth(5));
    assert_eq!(state.label_width(), 10); // minimum is 10
}

#[test]
fn test_set_label_width_clamped_high() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    SpanTree::update(&mut state, SpanTreeMessage::SetLabelWidth(200));
    assert_eq!(state.label_width(), 100); // maximum is 100
}

// =============================================================================
// Handle Event
// =============================================================================

#[test]
fn test_handle_event_not_focused() {
    let state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    let msg = SpanTree::handle_event(&state, &Event::key(KeyCode::Down), &ViewContext::default());
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_disabled() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    state.set_disabled(true);
    let msg = SpanTree::handle_event(
        &state,
        &Event::key(KeyCode::Down),
        &ViewContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_handle_event_down() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    assert_eq!(
        SpanTree::handle_event(
            &state,
            &Event::key(KeyCode::Down),
            &ViewContext::new().focused(true)
        ),
        Some(SpanTreeMessage::SelectDown)
    );
    assert_eq!(
        SpanTree::handle_event(&state, &Event::char('j'), &ViewContext::new().focused(true)),
        Some(SpanTreeMessage::SelectDown)
    );
}

#[test]
fn test_handle_event_up() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    assert_eq!(
        SpanTree::handle_event(
            &state,
            &Event::key(KeyCode::Up),
            &ViewContext::new().focused(true)
        ),
        Some(SpanTreeMessage::SelectUp)
    );
    assert_eq!(
        SpanTree::handle_event(&state, &Event::char('k'), &ViewContext::new().focused(true)),
        Some(SpanTreeMessage::SelectUp)
    );
}

#[test]
fn test_handle_event_expand() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    assert_eq!(
        SpanTree::handle_event(
            &state,
            &Event::key(KeyCode::Right),
            &ViewContext::new().focused(true)
        ),
        Some(SpanTreeMessage::Expand)
    );
    assert_eq!(
        SpanTree::handle_event(&state, &Event::char('l'), &ViewContext::new().focused(true)),
        Some(SpanTreeMessage::Expand)
    );
}

#[test]
fn test_handle_event_collapse() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    assert_eq!(
        SpanTree::handle_event(
            &state,
            &Event::key(KeyCode::Left),
            &ViewContext::new().focused(true)
        ),
        Some(SpanTreeMessage::Collapse)
    );
    assert_eq!(
        SpanTree::handle_event(&state, &Event::char('h'), &ViewContext::new().focused(true)),
        Some(SpanTreeMessage::Collapse)
    );
}

#[test]
fn test_handle_event_toggle() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    assert_eq!(
        SpanTree::handle_event(&state, &Event::char(' '), &ViewContext::new().focused(true)),
        Some(SpanTreeMessage::Toggle)
    );
    assert_eq!(
        SpanTree::handle_event(
            &state,
            &Event::key(KeyCode::Enter),
            &ViewContext::new().focused(true)
        ),
        Some(SpanTreeMessage::Toggle)
    );
}

#[test]
fn test_handle_event_shift_right() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    let msg = SpanTree::handle_event(
        &state,
        &Event::key_with(KeyCode::Right, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SpanTreeMessage::SetLabelWidth(32)));
}

#[test]
fn test_handle_event_shift_left() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.set_focused(true);
    let msg = SpanTree::handle_event(
        &state,
        &Event::key_with(KeyCode::Left, KeyModifiers::SHIFT),
        &ViewContext::new().focused(true),
    );
    assert_eq!(msg, Some(SpanTreeMessage::SetLabelWidth(28)));
}

// =============================================================================
// Dispatch Event
// =============================================================================

#[test]
fn test_dispatch_event() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);

    let output = state.dispatch_event(&Event::key(KeyCode::Down));
    assert_eq!(output, Some(SpanTreeOutput::Selected("c".into())));
    assert_eq!(state.selected_index(), Some(1));
}

// =============================================================================
// Component Trait
// =============================================================================

#[test]
fn test_init() {
    let state = SpanTree::init();
    assert!(state.is_empty());
}

// =============================================================================
// Selected Span
// =============================================================================

#[test]
fn test_selected_span() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let state = SpanTreeState::new(vec![root]);
    let span = state.selected_span().unwrap();
    assert_eq!(span.id(), "r");
    assert_eq!(span.label(), "root");
    assert_eq!(span.start(), 0.0);
    assert_eq!(span.end(), 100.0);
    assert_eq!(span.duration(), 100.0);
}

#[test]
fn test_selected_span_empty() {
    let state = SpanTreeState::default();
    assert!(state.selected_span().is_none());
}

// =============================================================================
// Focusable / Disableable
// =============================================================================

#[test]
fn test_focusable() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    assert!(!SpanTree::is_focused(&state));
    SpanTree::set_focused(&mut state, true);
    assert!(SpanTree::is_focused(&state));
    SpanTree::blur(&mut state);
    assert!(!SpanTree::is_focused(&state));
    SpanTree::focus(&mut state);
    assert!(SpanTree::is_focused(&state));
}

#[test]
fn test_disableable() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    assert!(!SpanTree::is_disabled(&state));
    SpanTree::set_disabled(&mut state, true);
    assert!(SpanTree::is_disabled(&state));
    SpanTree::enable(&mut state);
    assert!(!SpanTree::is_disabled(&state));
    SpanTree::disable(&mut state);
    assert!(SpanTree::is_disabled(&state));
}

#[test]
fn test_disabled_ignores_navigation() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.set_focused(true);
    state.set_disabled(true);

    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0));
}

// =============================================================================
// FlatSpan accessors
// =============================================================================

#[test]
fn test_flat_span_accessors() {
    let root = SpanNode::new("r", "root", 10.0, 90.0)
        .with_color(Color::Yellow)
        .with_status("OK")
        .with_child(SpanNode::new("c", "child", 20.0, 60.0));
    let state = SpanTreeState::new(vec![root]);
    let flat = state.flatten();

    assert_eq!(flat[0].id(), "r");
    assert_eq!(flat[0].label(), "root");
    assert_eq!(flat[0].start(), 10.0);
    assert_eq!(flat[0].end(), 90.0);
    assert_eq!(flat[0].color(), Color::Yellow);
    assert_eq!(flat[0].status(), Some("OK"));
    assert_eq!(flat[0].depth(), 0);
    assert!(flat[0].has_children());
    assert!(flat[0].is_expanded());
    assert_eq!(flat[0].duration(), 80.0);

    assert_eq!(flat[1].id(), "c");
    assert_eq!(flat[1].depth(), 1);
    assert!(!flat[1].has_children());
}

// =============================================================================
// Edge Cases
// =============================================================================

#[test]
fn test_expand_nonexistent_id() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.expand("nonexistent");
    // Should not panic, just adds the id to the set
    assert!(state.expanded_ids().contains("nonexistent"));
}

#[test]
fn test_collapse_nonexistent_id() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("r", "root", 0.0, 10.0)]);
    state.collapse("nonexistent");
    // Should not panic
}

#[test]
fn test_expand_all_collapse_all_empty() {
    let mut state = SpanTreeState::default();
    state.expand_all();
    state.collapse_all();
    assert!(state.is_empty());
}

#[test]
fn test_navigate_empty_tree() {
    let mut state = SpanTreeState::default();
    state.set_focused(true);
    let output = SpanTree::update(&mut state, SpanTreeMessage::SelectDown);
    assert_eq!(output, None);
}

#[test]
fn test_set_roots_via_message() {
    let mut state = SpanTreeState::new(vec![SpanNode::new("old", "old", 0.0, 10.0)]);
    let new_roots = vec![SpanNode::new("new", "new", 5.0, 50.0)];
    SpanTree::update(&mut state, SpanTreeMessage::SetRoots(new_roots));
    assert_eq!(state.roots().len(), 1);
    assert_eq!(state.roots()[0].id(), "new");
}

#[test]
fn test_expand_all_via_message() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);
    state.collapse_all();

    SpanTree::update(&mut state, SpanTreeMessage::ExpandAll);
    assert!(state.expanded_ids().contains("r"));
}

#[test]
fn test_collapse_all_via_message() {
    let root =
        SpanNode::new("r", "root", 0.0, 100.0).with_child(SpanNode::new("c", "child", 10.0, 50.0));
    let mut state = SpanTreeState::new(vec![root]);

    SpanTree::update(&mut state, SpanTreeMessage::CollapseAll);
    assert!(state.expanded_ids().is_empty());
}

// Helper
fn roots(root: SpanNode) -> Vec<SpanNode> {
    vec![root]
}
