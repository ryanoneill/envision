use super::layout::{rects_overlap, squarified_layout};
use super::*;
use crate::component::test_utils;
use crate::input::Event;

// =============================================================================
// TreemapNode: construction
// =============================================================================

#[test]
fn test_node_new() {
    let node = TreemapNode::new("test", 42.0);
    assert_eq!(node.label, "test");
    assert_eq!(node.value, 42.0);
    assert_eq!(node.color, Color::Gray);
    assert!(node.children.is_empty());
}

#[test]
fn test_node_with_color() {
    let node = TreemapNode::new("test", 10.0).with_color(Color::Red);
    assert_eq!(node.color, Color::Red);
}

#[test]
fn test_node_with_child() {
    let node = TreemapNode::new("parent", 0.0).with_child(TreemapNode::new("child", 10.0));
    assert_eq!(node.children.len(), 1);
    assert_eq!(node.children[0].label, "child");
}

#[test]
fn test_node_with_children() {
    let children = vec![
        TreemapNode::new("a", 10.0),
        TreemapNode::new("b", 20.0),
        TreemapNode::new("c", 30.0),
    ];
    let node = TreemapNode::new("parent", 0.0).with_children(children);
    assert_eq!(node.children.len(), 3);
}

#[test]
fn test_node_chained_with_child() {
    let node = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("a", 10.0))
        .with_child(TreemapNode::new("b", 20.0))
        .with_child(TreemapNode::new("c", 30.0));
    assert_eq!(node.children.len(), 3);
    assert_eq!(node.children[2].label, "c");
}

// =============================================================================
// TreemapNode: total_value
// =============================================================================

#[test]
fn test_leaf_total_value() {
    let node = TreemapNode::new("leaf", 42.0);
    assert_eq!(node.total_value(), 42.0);
}

#[test]
fn test_parent_total_value() {
    let node = TreemapNode::new("parent", 0.0)
        .with_child(TreemapNode::new("a", 10.0))
        .with_child(TreemapNode::new("b", 20.0));
    assert_eq!(node.total_value(), 30.0);
}

#[test]
fn test_nested_total_value() {
    let node = TreemapNode::new("root", 0.0)
        .with_child(
            TreemapNode::new("parent", 0.0)
                .with_child(TreemapNode::new("a", 10.0))
                .with_child(TreemapNode::new("b", 20.0)),
        )
        .with_child(TreemapNode::new("c", 30.0));
    assert_eq!(node.total_value(), 60.0);
}

#[test]
fn test_parent_value_ignored_when_has_children() {
    // Parent's own value is ignored when it has children.
    let node = TreemapNode::new("parent", 999.0)
        .with_child(TreemapNode::new("a", 10.0))
        .with_child(TreemapNode::new("b", 20.0));
    assert_eq!(node.total_value(), 30.0);
}

// =============================================================================
// TreemapNode: is_leaf
// =============================================================================

#[test]
fn test_is_leaf_true() {
    let node = TreemapNode::new("leaf", 10.0);
    assert!(node.is_leaf());
}

#[test]
fn test_is_leaf_false() {
    let node = TreemapNode::new("parent", 0.0).with_child(TreemapNode::new("child", 5.0));
    assert!(!node.is_leaf());
}

// =============================================================================
// Layout: basic properties
// =============================================================================

#[test]
fn test_layout_empty_nodes() {
    let rects = squarified_layout(&[], Rect::new(0, 0, 20, 10), 0, &[]);
    assert!(rects.is_empty());
}

#[test]
fn test_layout_zero_area() {
    let nodes = vec![TreemapNode::new("a", 10.0)];
    let rects = squarified_layout(&nodes, Rect::new(0, 0, 0, 0), 0, &[]);
    assert!(rects.is_empty());
}

#[test]
fn test_layout_single_node() {
    let nodes = vec![TreemapNode::new("a", 10.0).with_color(Color::Red)];
    let area = Rect::new(0, 0, 20, 10);
    let rects = squarified_layout(&nodes, area, 0, &[]);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].label, "a");
    assert_eq!(rects[0].width, 20);
    assert_eq!(rects[0].height, 10);
}

#[test]
fn test_layout_two_equal_nodes() {
    let nodes = vec![
        TreemapNode::new("a", 50.0).with_color(Color::Red),
        TreemapNode::new("b", 50.0).with_color(Color::Blue),
    ];
    let area = Rect::new(0, 0, 20, 10);
    let rects = squarified_layout(&nodes, area, 0, &[]);
    assert_eq!(rects.len(), 2);

    // Total area should equal the input area.
    let total_area: u32 = rects.iter().map(|r| r.width as u32 * r.height as u32).sum();
    assert_eq!(total_area, 20 * 10);
}

#[test]
fn test_layout_three_nodes() {
    let nodes = vec![
        TreemapNode::new("a", 60.0).with_color(Color::Red),
        TreemapNode::new("b", 30.0).with_color(Color::Green),
        TreemapNode::new("c", 10.0).with_color(Color::Blue),
    ];
    let area = Rect::new(0, 0, 20, 10);
    let rects = squarified_layout(&nodes, area, 0, &[]);
    assert_eq!(rects.len(), 3);
}

#[test]
fn test_layout_rectangles_fill_area() {
    let nodes = vec![
        TreemapNode::new("a", 40.0).with_color(Color::Red),
        TreemapNode::new("b", 30.0).with_color(Color::Green),
        TreemapNode::new("c", 20.0).with_color(Color::Blue),
        TreemapNode::new("d", 10.0).with_color(Color::Yellow),
    ];
    let area = Rect::new(0, 0, 40, 20);
    let rects = squarified_layout(&nodes, area, 0, &[]);

    // All rectangles should be within the area.
    for rect in &rects {
        assert!(rect.x >= area.x, "rect x out of bounds");
        assert!(rect.y >= area.y, "rect y out of bounds");
        assert!(
            rect.x + rect.width <= area.x + area.width,
            "rect extends past right edge"
        );
        assert!(
            rect.y + rect.height <= area.y + area.height,
            "rect extends past bottom edge"
        );
    }
}

#[test]
fn test_layout_no_overlap() {
    let nodes = vec![
        TreemapNode::new("a", 40.0).with_color(Color::Red),
        TreemapNode::new("b", 30.0).with_color(Color::Green),
        TreemapNode::new("c", 20.0).with_color(Color::Blue),
        TreemapNode::new("d", 10.0).with_color(Color::Yellow),
    ];
    let area = Rect::new(0, 0, 40, 20);
    let rects = squarified_layout(&nodes, area, 0, &[]);

    // No two rectangles should overlap.
    for i in 0..rects.len() {
        for j in (i + 1)..rects.len() {
            assert!(
                !rects_overlap(&rects[i], &rects[j]),
                "rectangles {} ({}) and {} ({}) overlap",
                i,
                rects[i].label,
                j,
                rects[j].label,
            );
        }
    }
}

#[test]
fn test_layout_proportional_areas() {
    let nodes = vec![
        TreemapNode::new("a", 75.0).with_color(Color::Red),
        TreemapNode::new("b", 25.0).with_color(Color::Blue),
    ];
    let area = Rect::new(0, 0, 40, 20);
    let rects = squarified_layout(&nodes, area, 0, &[]);

    let area_a = rects[0].width as f64 * rects[0].height as f64;
    let area_b = rects[1].width as f64 * rects[1].height as f64;
    let ratio = area_a / area_b;

    // Should be approximately 3:1 (75/25).
    assert!(
        (ratio - 3.0).abs() < 0.5,
        "ratio was {ratio}, expected approximately 3.0"
    );
}

#[test]
fn test_layout_with_offset() {
    let nodes = vec![TreemapNode::new("a", 10.0).with_color(Color::Red)];
    let area = Rect::new(5, 3, 20, 10);
    let rects = squarified_layout(&nodes, area, 0, &[]);
    assert_eq!(rects.len(), 1);
    assert_eq!(rects[0].x, 5);
    assert_eq!(rects[0].y, 3);
}

#[test]
fn test_layout_depth() {
    let nodes = vec![TreemapNode::new("a", 10.0)];
    let rects = squarified_layout(&nodes, Rect::new(0, 0, 10, 10), 3, &[]);
    assert_eq!(rects[0].depth, 3);
}

#[test]
fn test_layout_node_index() {
    let nodes = vec![TreemapNode::new("a", 60.0), TreemapNode::new("b", 40.0)];
    let rects = squarified_layout(&nodes, Rect::new(0, 0, 20, 10), 0, &[]);
    // The nodes are sorted by value descending, so "a" (60) comes first.
    let a_rect = rects.iter().find(|r| r.label == "a").unwrap();
    assert_eq!(a_rect.node_index, vec![0]);
    let b_rect = rects.iter().find(|r| r.label == "b").unwrap();
    assert_eq!(b_rect.node_index, vec![1]);
}

#[test]
fn test_layout_narrow_area() {
    let nodes = vec![TreemapNode::new("a", 50.0), TreemapNode::new("b", 50.0)];
    let area = Rect::new(0, 0, 2, 20);
    let rects = squarified_layout(&nodes, area, 0, &[]);
    assert_eq!(rects.len(), 2);
    for rect in &rects {
        assert!(rect.width > 0);
        assert!(rect.height > 0);
    }
}

#[test]
fn test_layout_all_zero_values() {
    let nodes = vec![TreemapNode::new("a", 0.0), TreemapNode::new("b", 0.0)];
    let rects = squarified_layout(&nodes, Rect::new(0, 0, 20, 10), 0, &[]);
    assert!(rects.is_empty());
}

// =============================================================================
// TreemapState: construction
// =============================================================================

#[test]
fn test_state_new_empty() {
    let state = TreemapState::new();
    assert!(state.root().is_none());
    assert!(state.show_labels());
    assert!(!state.show_values());
    assert!(!state.is_focused());
    assert!(!state.is_disabled());
    assert!(state.title().is_none());
}

#[test]
fn test_state_with_root() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    assert!(state.root().is_some());
    assert_eq!(state.root().unwrap().label, "root");
}

#[test]
fn test_state_with_title() {
    let state = TreemapState::new().with_title("Test");
    assert_eq!(state.title(), Some("Test"));
}

#[test]
fn test_state_with_show_labels() {
    let state = TreemapState::new().with_show_labels(false);
    assert!(!state.show_labels());
}

#[test]
fn test_state_with_show_values() {
    let state = TreemapState::new().with_show_values(true);
    assert!(state.show_values());
}

#[test]
fn test_state_with_disabled() {
    let state = TreemapState::new().with_disabled(true);
    assert!(state.is_disabled());
}

// =============================================================================
// TreemapState: accessors
// =============================================================================

#[test]
fn test_set_root() {
    let mut state = TreemapState::new();
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    state.set_root(root);
    assert!(state.root().is_some());
}

#[test]
fn test_clear() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let mut state = TreemapState::new().with_root(root);
    state.clear();
    assert!(state.root().is_none());
}

#[test]
fn test_current_view_node_at_root() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    let view = state.current_view_node().unwrap();
    assert_eq!(view.label, "root");
}

#[test]
fn test_selected_node() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("first", 30.0))
        .with_child(TreemapNode::new("second", 20.0));
    let state = TreemapState::new().with_root(root);
    let selected = state.selected_node().unwrap();
    assert_eq!(selected.label, "first");
}

#[test]
fn test_selected_node_empty() {
    let state = TreemapState::new();
    assert!(state.selected_node().is_none());
}

// =============================================================================
// Navigation: SelectNext / SelectPrev
// =============================================================================

fn sample_state() -> TreemapState {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("a", 40.0).with_color(Color::Red))
        .with_child(TreemapNode::new("b", 30.0).with_color(Color::Green))
        .with_child(TreemapNode::new("c", 20.0).with_color(Color::Blue))
        .with_child(TreemapNode::new("d", 10.0).with_color(Color::Yellow));
    let mut state = TreemapState::new().with_root(root);
    state.set_focused(true);
    state
}

#[test]
fn test_select_next() {
    let mut state = sample_state();
    assert_eq!(state.selected_node().unwrap().label, "a");
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "b");
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "c");
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "d");
}

#[test]
fn test_select_next_at_end() {
    let mut state = sample_state();
    state.update(TreemapMessage::SelectNext);
    state.update(TreemapMessage::SelectNext);
    state.update(TreemapMessage::SelectNext);
    // Now at "d" (last).
    state.update(TreemapMessage::SelectNext);
    // Should stay at "d".
    assert_eq!(state.selected_node().unwrap().label, "d");
}

#[test]
fn test_select_prev() {
    let mut state = sample_state();
    state.update(TreemapMessage::SelectNext);
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "c");
    state.update(TreemapMessage::SelectPrev);
    assert_eq!(state.selected_node().unwrap().label, "b");
    state.update(TreemapMessage::SelectPrev);
    assert_eq!(state.selected_node().unwrap().label, "a");
}

#[test]
fn test_select_prev_at_start() {
    let mut state = sample_state();
    state.update(TreemapMessage::SelectPrev);
    // Should stay at "a".
    assert_eq!(state.selected_node().unwrap().label, "a");
}

// =============================================================================
// Zoom: in / out / reset
// =============================================================================

fn nested_state() -> TreemapState {
    let root = TreemapNode::new("root", 0.0)
        .with_child(
            TreemapNode::new("src", 0.0)
                .with_color(Color::Blue)
                .with_child(TreemapNode::new("main.rs", 30.0).with_color(Color::Cyan))
                .with_child(TreemapNode::new("lib.rs", 20.0).with_color(Color::LightBlue)),
        )
        .with_child(TreemapNode::new("README.md", 10.0).with_color(Color::Green));
    let mut state = TreemapState::new().with_root(root);
    state.set_focused(true);
    state
}

#[test]
fn test_zoom_in() {
    let mut state = nested_state();
    // Selected: "src" (has children).
    let output = state.update(TreemapMessage::ZoomIn);
    assert_eq!(output, Some(TreemapOutput::ZoomedIn("src".to_string())));

    // Now viewing "src"'s children.
    let view = state.current_view_node().unwrap();
    assert_eq!(view.label, "src");
    assert_eq!(state.selected_node().unwrap().label, "main.rs");
}

#[test]
fn test_zoom_in_leaf() {
    let mut state = nested_state();
    // Select "README.md" (leaf node).
    state.update(TreemapMessage::SelectNext);
    let output = state.update(TreemapMessage::ZoomIn);
    assert_eq!(
        output,
        Some(TreemapOutput::NodeSelected {
            label: "README.md".to_string(),
            value: 10.0,
        })
    );
}

#[test]
fn test_zoom_out() {
    let mut state = nested_state();
    state.update(TreemapMessage::ZoomIn); // Zoom into "src".
    let output = state.update(TreemapMessage::ZoomOut);
    assert_eq!(output, Some(TreemapOutput::ZoomedOut));

    // Back at root, "src" should be re-selected.
    let view = state.current_view_node().unwrap();
    assert_eq!(view.label, "root");
    assert_eq!(state.selected_node().unwrap().label, "src");
}

#[test]
fn test_zoom_out_at_root() {
    let mut state = nested_state();
    let output = state.update(TreemapMessage::ZoomOut);
    assert_eq!(output, None); // Already at root.
}

#[test]
fn test_reset_zoom() {
    let mut state = nested_state();
    state.update(TreemapMessage::ZoomIn); // Zoom into "src".
    let output = state.update(TreemapMessage::ResetZoom);
    assert_eq!(output, Some(TreemapOutput::ZoomedOut));
    let view = state.current_view_node().unwrap();
    assert_eq!(view.label, "root");
}

#[test]
fn test_reset_zoom_already_at_root() {
    let mut state = nested_state();
    let output = state.update(TreemapMessage::ResetZoom);
    assert_eq!(output, None);
}

// =============================================================================
// Navigation: SelectChild / SelectParent
// =============================================================================

#[test]
fn test_select_child() {
    let mut state = nested_state();
    // "src" is selected; it has children.
    let output = state.update(TreemapMessage::SelectChild);
    assert_eq!(output, Some(TreemapOutput::ZoomedIn("src".to_string())));
    assert_eq!(state.selected_node().unwrap().label, "main.rs");
}

#[test]
fn test_select_child_on_leaf() {
    let mut state = nested_state();
    state.update(TreemapMessage::SelectNext); // Select "README.md".
    let output = state.update(TreemapMessage::SelectChild);
    assert_eq!(output, None); // Leaf has no children.
}

#[test]
fn test_select_parent() {
    let mut state = nested_state();
    state.update(TreemapMessage::ZoomIn); // Zoom into "src".
    let output = state.update(TreemapMessage::SelectParent);
    assert_eq!(output, Some(TreemapOutput::ZoomedOut));
    assert_eq!(state.selected_node().unwrap().label, "src");
}

#[test]
fn test_select_parent_at_root() {
    let mut state = nested_state();
    let output = state.update(TreemapMessage::SelectParent);
    assert_eq!(output, None);
}

// =============================================================================
// Event mapping
// =============================================================================

#[test]
fn test_right_maps_to_select_next() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, Some(TreemapMessage::SelectNext));
}

#[test]
fn test_left_maps_to_select_prev() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Left));
    assert_eq!(msg, Some(TreemapMessage::SelectPrev));
}

#[test]
fn test_down_maps_to_select_child() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Down));
    assert_eq!(msg, Some(TreemapMessage::SelectChild));
}

#[test]
fn test_up_maps_to_select_parent() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Up));
    assert_eq!(msg, Some(TreemapMessage::SelectParent));
}

#[test]
fn test_enter_maps_to_zoom_in() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Enter));
    assert_eq!(msg, Some(TreemapMessage::ZoomIn));
}

#[test]
fn test_esc_maps_to_zoom_out() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Esc));
    assert_eq!(msg, Some(TreemapMessage::ZoomOut));
}

#[test]
fn test_backspace_maps_to_zoom_out() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Backspace));
    assert_eq!(msg, Some(TreemapMessage::ZoomOut));
}

#[test]
fn test_home_maps_to_reset_zoom() {
    let state = sample_state();
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Home));
    assert_eq!(msg, Some(TreemapMessage::ResetZoom));
}

#[test]
fn test_hjkl_keys() {
    let state = sample_state();
    assert_eq!(
        Treemap::handle_event(&state, &Event::char('h')),
        Some(TreemapMessage::SelectPrev)
    );
    assert_eq!(
        Treemap::handle_event(&state, &Event::char('l')),
        Some(TreemapMessage::SelectNext)
    );
    assert_eq!(
        Treemap::handle_event(&state, &Event::char('j')),
        Some(TreemapMessage::SelectChild)
    );
    assert_eq!(
        Treemap::handle_event(&state, &Event::char('k')),
        Some(TreemapMessage::SelectParent)
    );
}

// =============================================================================
// Disabled / unfocused state
// =============================================================================

#[test]
fn test_disabled_ignores_events() {
    let mut state = sample_state();
    state.set_disabled(true);
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, None);
}

#[test]
fn test_unfocused_ignores_events() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    let msg = Treemap::handle_event(&state, &Event::key(KeyCode::Right));
    assert_eq!(msg, None);
}

// =============================================================================
// Instance methods
// =============================================================================

#[test]
fn test_instance_handle_event() {
    let state = sample_state();
    let msg = state.handle_event(&Event::key(KeyCode::Right));
    assert_eq!(msg, Some(TreemapMessage::SelectNext));
}

#[test]
fn test_instance_update() {
    let mut state = sample_state();
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "b");
}

#[test]
fn test_instance_dispatch_event() {
    let mut state = sample_state();
    state.dispatch_event(&Event::key(KeyCode::Right));
    assert_eq!(state.selected_node().unwrap().label, "b");
}

// =============================================================================
// Focus and disabled
// =============================================================================

#[test]
fn test_focus_methods() {
    let mut state = TreemapState::new();
    assert!(!state.is_focused());
    state.set_focused(true);
    assert!(state.is_focused());
    state.set_focused(false);
    assert!(!state.is_focused());
}

#[test]
fn test_disabled_methods() {
    let mut state = TreemapState::new();
    assert!(!state.is_disabled());
    state.set_disabled(true);
    assert!(state.is_disabled());
    state.set_disabled(false);
    assert!(!state.is_disabled());
}

// =============================================================================
// Focusable / Disableable trait
// =============================================================================

#[test]
fn test_focusable_trait() {
    let mut state = TreemapState::new();
    assert!(!Treemap::is_focused(&state));
    Treemap::set_focused(&mut state, true);
    assert!(Treemap::is_focused(&state));
}

#[test]
fn test_disableable_trait() {
    let mut state = TreemapState::new();
    assert!(!Treemap::is_disabled(&state));
    Treemap::set_disabled(&mut state, true);
    assert!(Treemap::is_disabled(&state));
}

// =============================================================================
// Message handling: SetRoot / Clear
// =============================================================================

#[test]
fn test_set_root_message() {
    let mut state = TreemapState::new();
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    state.update(TreemapMessage::SetRoot(root));
    assert!(state.root().is_some());
}

#[test]
fn test_clear_message() {
    let mut state = sample_state();
    state.update(TreemapMessage::Clear);
    assert!(state.root().is_none());
}

// =============================================================================
// Rendering
// =============================================================================

#[test]
fn test_render_empty() {
    let state = TreemapState::new();
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_simple() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("src", 60.0).with_color(Color::Blue))
        .with_child(TreemapNode::new("docs", 30.0).with_color(Color::Green))
        .with_child(TreemapNode::new("tests", 10.0).with_color(Color::Yellow));
    let state = TreemapState::new().with_root(root);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_focused() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("a", 60.0).with_color(Color::Red))
        .with_child(TreemapNode::new("b", 40.0).with_color(Color::Blue));
    let mut state = TreemapState::new().with_root(root).with_title("Test");
    state.set_focused(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().focused(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_disabled() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new()
        .with_root(root)
        .with_disabled(true)
        .with_title("Disabled");
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(
                &state,
                frame,
                frame.area(),
                &theme,
                &ViewContext::new().disabled(true),
            );
        })
        .unwrap();
}

#[test]
fn test_render_with_values() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("a", 60.0).with_color(Color::Red))
        .with_child(TreemapNode::new("b", 40.0).with_color(Color::Blue));
    let state = TreemapState::new().with_root(root).with_show_values(true);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    terminal
        .draw(|frame| {
            Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_small_area() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    let (mut terminal, theme) = test_utils::setup_render(5, 4);
    terminal
        .draw(|frame| {
            Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

#[test]
fn test_render_too_small() {
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    let (mut terminal, theme) = test_utils::setup_render(2, 2);
    terminal
        .draw(|frame| {
            Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
        })
        .unwrap();
}

// =============================================================================
// Annotation
// =============================================================================

#[test]
fn test_annotation_emitted() {
    use crate::annotation::with_annotations;
    let root = TreemapNode::new("root", 0.0).with_child(TreemapNode::new("a", 10.0));
    let state = TreemapState::new().with_root(root);
    let (mut terminal, theme) = test_utils::setup_render(40, 10);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                Treemap::view(&state, frame, frame.area(), &theme, &ViewContext::default());
            })
            .unwrap();
    });
    assert!(registry.get_by_id("treemap").is_some());
}

// =============================================================================
// Edge cases
// =============================================================================

#[test]
fn test_navigation_on_empty_treemap() {
    let mut state = TreemapState::new();
    state.set_focused(true);
    let output = state.update(TreemapMessage::SelectNext);
    assert_eq!(output, None);
    let output = state.update(TreemapMessage::SelectPrev);
    assert_eq!(output, None);
    let output = state.update(TreemapMessage::ZoomIn);
    assert_eq!(output, None);
}

#[test]
fn test_single_child() {
    let root = TreemapNode::new("root", 0.0)
        .with_child(TreemapNode::new("only", 100.0).with_color(Color::Red));
    let mut state = TreemapState::new().with_root(root);
    state.set_focused(true);
    assert_eq!(state.selected_node().unwrap().label, "only");
    // Can't go next or prev.
    state.update(TreemapMessage::SelectNext);
    assert_eq!(state.selected_node().unwrap().label, "only");
    state.update(TreemapMessage::SelectPrev);
    assert_eq!(state.selected_node().unwrap().label, "only");
}

#[test]
fn test_deep_zoom() {
    let root = TreemapNode::new("root", 0.0).with_child(
        TreemapNode::new("level1", 0.0).with_child(
            TreemapNode::new("level2", 0.0)
                .with_child(TreemapNode::new("leaf", 10.0).with_color(Color::Red)),
        ),
    );
    let mut state = TreemapState::new().with_root(root);
    state.set_focused(true);

    // Zoom in three levels.
    state.update(TreemapMessage::ZoomIn); // Into level1.
    assert_eq!(state.current_view_node().unwrap().label, "level1");
    state.update(TreemapMessage::ZoomIn); // Into level2.
    assert_eq!(state.current_view_node().unwrap().label, "level2");

    // Leaf node -- ZoomIn should emit NodeSelected.
    let output = state.update(TreemapMessage::ZoomIn);
    assert_eq!(
        output,
        Some(TreemapOutput::NodeSelected {
            label: "leaf".to_string(),
            value: 10.0,
        })
    );

    // Reset zoom.
    let output = state.update(TreemapMessage::ResetZoom);
    assert_eq!(output, Some(TreemapOutput::ZoomedOut));
    assert_eq!(state.current_view_node().unwrap().label, "root");
}
