use super::*;

#[derive(Clone, Debug, PartialEq)]
enum TestField {
    A,
    B,
    C,
}

#[test]
fn test_new_unfocused() {
    let focus = FocusManager::new(vec![TestField::A, TestField::B]);
    assert_eq!(focus.focused(), None);
    assert!(!focus.is_focused(&TestField::A));
}

#[test]
fn test_with_initial_focus() {
    let focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);
    assert_eq!(focus.focused(), Some(&TestField::A));
    assert!(focus.is_focused(&TestField::A));
}

#[test]
fn test_with_initial_focus_empty() {
    let focus: FocusManager<TestField> = FocusManager::with_initial_focus(vec![]);
    assert_eq!(focus.focused(), None);
}

#[test]
fn test_focus_specific() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

    assert!(focus.focus(&TestField::B));
    assert_eq!(focus.focused(), Some(&TestField::B));
    assert!(focus.is_focused(&TestField::B));
    assert!(!focus.is_focused(&TestField::A));
}

#[test]
fn test_focus_not_found() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B]);

    // Try to focus something not in the order
    assert!(!focus.focus(&TestField::C));
    assert_eq!(focus.focused(), None);
}

#[test]
fn test_focus_not_found_preserves_current() {
    let mut focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);

    // Try to focus something not in the order
    assert!(!focus.focus(&TestField::C));
    // Should preserve current focus
    assert_eq!(focus.focused(), Some(&TestField::A));
}

#[test]
fn test_focus_next_basic() {
    let mut focus =
        FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

    assert_eq!(focus.focus_next(), Some(&TestField::B));
    assert_eq!(focus.focused(), Some(&TestField::B));

    assert_eq!(focus.focus_next(), Some(&TestField::C));
    assert_eq!(focus.focused(), Some(&TestField::C));
}

#[test]
fn test_focus_next_wraps() {
    let mut focus =
        FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

    focus.focus(&TestField::C);
    assert_eq!(focus.focus_next(), Some(&TestField::A)); // Wraps to first
}

#[test]
fn test_focus_next_from_unfocused() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B]);

    assert_eq!(focus.focus_next(), Some(&TestField::A)); // Focuses first
}

#[test]
fn test_focus_prev_basic() {
    let mut focus =
        FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

    focus.focus(&TestField::C);

    assert_eq!(focus.focus_prev(), Some(&TestField::B));
    assert_eq!(focus.focused(), Some(&TestField::B));

    assert_eq!(focus.focus_prev(), Some(&TestField::A));
    assert_eq!(focus.focused(), Some(&TestField::A));
}

#[test]
fn test_focus_prev_wraps() {
    let mut focus =
        FocusManager::with_initial_focus(vec![TestField::A, TestField::B, TestField::C]);

    assert_eq!(focus.focus_prev(), Some(&TestField::C)); // Wraps to last
}

#[test]
fn test_focus_prev_from_unfocused() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

    assert_eq!(focus.focus_prev(), Some(&TestField::C)); // Focuses last
}

#[test]
fn test_blur() {
    let mut focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);

    assert!(focus.focused().is_some());
    focus.blur();
    assert!(focus.focused().is_none());
}

#[test]
fn test_focus_first() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
    focus.focus(&TestField::C);

    assert_eq!(focus.focus_first(), Some(&TestField::A));
    assert_eq!(focus.focused(), Some(&TestField::A));
}

#[test]
fn test_focus_last() {
    let mut focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);

    assert_eq!(focus.focus_last(), Some(&TestField::C));
    assert_eq!(focus.focused(), Some(&TestField::C));
}

#[test]
fn test_empty_manager() {
    let mut focus: FocusManager<TestField> = FocusManager::default();

    assert!(focus.is_empty());
    assert_eq!(focus.len(), 0);
    assert_eq!(focus.focused(), None);
    assert_eq!(focus.focus_next(), None);
    assert_eq!(focus.focus_prev(), None);
    assert_eq!(focus.focus_first(), None);
    assert_eq!(focus.focus_last(), None);
    assert!(!focus.focus(&TestField::A));
}

#[test]
fn test_single_item() {
    let mut focus = FocusManager::with_initial_focus(vec![TestField::A]);

    assert_eq!(focus.focused(), Some(&TestField::A));

    // Next wraps to same item
    assert_eq!(focus.focus_next(), Some(&TestField::A));

    // Prev wraps to same item
    assert_eq!(focus.focus_prev(), Some(&TestField::A));
}

#[test]
fn test_order() {
    let focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
    assert_eq!(focus.order(), &[TestField::A, TestField::B, TestField::C]);
}

#[test]
fn test_len() {
    let focus = FocusManager::new(vec![TestField::A, TestField::B, TestField::C]);
    assert_eq!(focus.len(), 3);
}

#[test]
fn test_is_empty() {
    let empty: FocusManager<TestField> = FocusManager::default();
    assert!(empty.is_empty());

    let non_empty = FocusManager::new(vec![TestField::A]);
    assert!(!non_empty.is_empty());
}

#[test]
fn test_default() {
    let focus: FocusManager<TestField> = FocusManager::default();
    assert!(focus.is_empty());
    assert_eq!(focus.focused(), None);
}

#[test]
fn test_clone() {
    let focus = FocusManager::with_initial_focus(vec![TestField::A, TestField::B]);
    let cloned = focus.clone();

    assert_eq!(cloned.focused(), Some(&TestField::A));
    assert_eq!(cloned.len(), 2);
}

#[test]
fn test_with_string_ids() {
    let mut focus = FocusManager::with_initial_focus(vec![
        "username".to_string(),
        "password".to_string(),
        "submit".to_string(),
    ]);

    assert_eq!(focus.focused(), Some(&"username".to_string()));
    assert_eq!(focus.focus_next(), Some(&"password".to_string()));
    assert!(focus.focus(&"submit".to_string()));
    assert_eq!(focus.focused(), Some(&"submit".to_string()));
}
