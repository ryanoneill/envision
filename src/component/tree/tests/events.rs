use super::*;

fn make_tree_state() -> TreeState<&'static str> {
    let mut root = TreeNode::new_expanded("Root", "root");
    root.add_child(TreeNode::new("Child 1", "child1"));
    root.add_child(TreeNode::new("Child 2", "child2"));
    TreeState::new(vec![root])
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_up_when_focused() {
    let mut state = make_tree_state();
    state.selected_index = Some(1);

    let event = Event::key(Key::Up);
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Up));
}

#[test]
fn test_handle_event_down_when_focused() {
    let state = make_tree_state();

    let event = Event::key(Key::Down);
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Down));
}

#[test]
fn test_handle_event_expand_when_focused() {
    let state = make_tree_state();

    let event = Event::key(Key::Right);
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Expand));
}

#[test]
fn test_handle_event_collapse_when_focused() {
    let state = make_tree_state();

    let event = Event::key(Key::Left);
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Collapse));
}

#[test]
fn test_handle_event_toggle_when_focused() {
    let state = make_tree_state();

    let event = Event::char(' ');
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Toggle));
}

#[test]
fn test_handle_event_select_when_focused() {
    let state = make_tree_state();

    let event = Event::key(Key::Enter);
    let msg = Tree::<&str>::handle_event(&state, &event, &EventContext::new().focused(true));
    assert_eq!(msg, Some(TreeMessage::Select));
}

#[test]
fn test_handle_event_vim_keys() {
    let state = make_tree_state();

    let msg_k = Tree::<&str>::handle_event(
        &state,
        &Event::char('k'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg_k, Some(TreeMessage::Up));

    let msg_j = Tree::<&str>::handle_event(
        &state,
        &Event::char('j'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg_j, Some(TreeMessage::Down));

    let msg_h = Tree::<&str>::handle_event(
        &state,
        &Event::char('h'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg_h, Some(TreeMessage::Collapse));

    let msg_l = Tree::<&str>::handle_event(
        &state,
        &Event::char('l'),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg_l, Some(TreeMessage::Expand));
}

#[test]
fn test_handle_event_ignored_when_unfocused() {
    let state = make_tree_state();
    // focused is false by default

    let msg = Tree::<&str>::handle_event(&state, &Event::key(Key::Down), &EventContext::default());
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(&state, &Event::key(Key::Enter), &EventContext::default());
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(&state, &Event::char('j'), &EventContext::default());
    assert_eq!(msg, None);
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event() {
    let mut state = make_tree_state();

    // Dispatch Down: should move selection from 0 to 1
    let output = Tree::<&str>::dispatch_event(
        &mut state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, None); // Down returns None but updates state
    assert_eq!(state.selected_index(), Some(1));

    // Dispatch Enter: should select the current node
    let output = Tree::<&str>::dispatch_event(
        &mut state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_methods() {
    let mut state = make_tree_state();

    // dispatch_event via static method
    let output = Tree::<&str>::dispatch_event(
        &mut state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, None); // Down returns None but updates state
    assert_eq!(state.selected_index(), Some(1));

    // update via instance method
    let output = state.update(TreeMessage::Select);
    assert_eq!(output, Some(TreeOutput::Selected(vec![0, 0])));

    // handle_event via static method
    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Up),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(TreeMessage::Up));
}

#[test]
fn test_selected_item() {
    let state = TreeState::new(vec![TreeNode::new("root", "data")]);
    assert_eq!(state.selected_item().unwrap().data(), &"data");
    assert_eq!(
        state.selected_item().unwrap().label(),
        state.selected_node().unwrap().label()
    );
}

// ========== Disabled State Tests ==========

#[test]
fn test_handle_event_ignored_when_disabled() {
    let state = make_tree_state();

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Up),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::char(' '),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);

    let msg = Tree::<&str>::handle_event(
        &state,
        &Event::char('j'),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(msg, None);
}

#[test]
fn test_dispatch_event_ignored_when_disabled() {
    let mut state = make_tree_state();

    let output = Tree::<&str>::dispatch_event(
        &mut state,
        &Event::key(Key::Down),
        &EventContext::new().focused(true).disabled(true),
    );
    assert_eq!(output, None);
    assert_eq!(state.selected_index(), Some(0)); // Should not have moved
}
