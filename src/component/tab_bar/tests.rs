use super::*;
use crate::input::{Event, Key};
use ratatui::layout::Rect;

// ========== Tab Tests ==========

#[test]
fn test_tab_new() {
    let tab = Tab::new("id1", "Label");
    assert_eq!(tab.id(), "id1");
    assert_eq!(tab.label(), "Label");
    assert!(!tab.closable());
    assert!(!tab.modified());
    assert_eq!(tab.icon(), None);
}

#[test]
fn test_tab_builder_methods() {
    let tab = Tab::new("id", "L")
        .with_closable(true)
        .with_modified(true)
        .with_icon("R");
    assert!(tab.closable());
    assert!(tab.modified());
    assert_eq!(tab.icon(), Some("R"));
}

#[test]
fn test_tab_setters() {
    let mut tab = Tab::new("id", "Old");
    tab.set_label("New");
    assert_eq!(tab.label(), "New");
    tab.set_closable(true);
    assert!(tab.closable());
    tab.set_closable(false);
    assert!(!tab.closable());
    tab.set_modified(true);
    assert!(tab.modified());
    tab.set_modified(false);
    assert!(!tab.modified());
    tab.set_icon(Some("X".to_string()));
    assert_eq!(tab.icon(), Some("X"));
    tab.set_icon(None);
    assert_eq!(tab.icon(), None);
}

#[test]
fn test_tab_rendered_width_plain() {
    let tab = Tab::new("id", "Label");
    assert_eq!(tab.rendered_width(None), 7); // " Label " => 2 + 5
}

#[test]
fn test_tab_rendered_width_with_decorations() {
    let modified = Tab::new("id", "Label").with_modified(true);
    assert_eq!(modified.rendered_width(None), 8); // + 1 for bullet

    let closable = Tab::new("id", "Label").with_closable(true);
    assert_eq!(closable.rendered_width(None), 9); // + 2 for " x"

    let icon = Tab::new("id", "Label").with_icon("R");
    assert_eq!(icon.rendered_width(None), 9); // + 2 for "R "

    let all = Tab::new("id", "Label")
        .with_icon("R")
        .with_modified(true)
        .with_closable(true);
    assert_eq!(all.rendered_width(None), 12); // 2 + 2 + 5 + 1 + 2
}

#[test]
fn test_tab_rendered_width_max_clamped() {
    let tab = Tab::new("id", "A Very Long Label");
    let unclamped = tab.rendered_width(None);
    assert!(unclamped > 10);
    assert_eq!(tab.rendered_width(Some(10)), 10);
}

// ========== TabBarState Tests ==========

#[test]
fn test_state_new() {
    let state = TabBarState::new(vec![Tab::new("a", "Alpha"), Tab::new("b", "Beta")]);
    assert_eq!(state.len(), 2);
    assert!(!state.is_empty());
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("Alpha"));
}

#[test]
fn test_state_new_empty() {
    let state = TabBarState::new(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert!(state.active_tab().is_none());
}

#[test]
fn test_state_default() {
    let state = TabBarState::default();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.scroll_offset(), 0);
    assert_eq!(state.max_tab_width(), None);
}

#[test]
fn test_state_with_selected() {
    let state = TabBarState::with_selected(
        vec![Tab::new("a", "A"), Tab::new("b", "B"), Tab::new("c", "C")],
        1,
    );
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("B"));
}

#[test]
fn test_state_with_selected_clamps() {
    let state = TabBarState::with_selected(vec![Tab::new("a", "A"), Tab::new("b", "B")], 100);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_state_with_selected_empty() {
    let state = TabBarState::with_selected(vec![], 0);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_builder_methods() {
    let s1 = TabBarState::new(vec![Tab::new("a", "A")]).with_max_tab_width(Some(15));
    assert_eq!(s1.max_tab_width(), Some(15));
}

#[test]
fn test_state_set_selected() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A"),
        Tab::new("b", "B"),
        Tab::new("c", "C"),
    ]);
    state.set_selected(Some(2));
    assert_eq!(state.selected_index(), Some(2));
    state.set_selected(Some(100));
    assert_eq!(state.selected_index(), Some(2)); // clamped
    state.set_selected(None);
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_set_selected_empty() {
    let mut state = TabBarState::new(vec![]);
    state.set_selected(Some(0));
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_state_mutators() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    state.set_scroll_offset(3);
    assert_eq!(state.scroll_offset(), 3);
    state.set_max_tab_width(Some(20));
    assert_eq!(state.max_tab_width(), Some(20));
}

#[test]
fn test_state_set_tabs() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    state.set_selected(Some(1));
    state.set_tabs(vec![Tab::new("x", "X")]);
    assert_eq!(state.len(), 1);
    assert_eq!(state.selected_index(), Some(0)); // clamped
}

#[test]
fn test_state_set_tabs_empty() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    state.set_tabs(vec![]);
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.scroll_offset(), 0);
}

#[test]
fn test_state_find_tab_by_id() {
    let state = TabBarState::new(vec![
        Tab::new("x1", "First"),
        Tab::new("x2", "Second"),
        Tab::new("x3", "Third"),
    ]);
    let (idx, tab) = state.find_tab_by_id("x2").unwrap();
    assert_eq!(idx, 1);
    assert_eq!(tab.label(), "Second");
    assert!(state.find_tab_by_id("nonexistent").is_none());
}

#[test]
fn test_state_active_tab_mut() {
    let mut state = TabBarState::new(vec![Tab::new("a", "Alpha")]);
    state.active_tab_mut().unwrap().set_label("Modified");
    assert_eq!(state.active_tab().map(|t| t.label()), Some("Modified"));
}

#[test]
fn test_state_tabs_mut() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    state.tabs_mut()[0].set_modified(true);
    assert!(state.tabs()[0].modified());
}

#[test]
fn test_state_partial_eq() {
    let s1 = TabBarState::new(vec![Tab::new("a", "A")]);
    let s2 = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(s1, s2);
    let s3 = TabBarState::new(vec![Tab::new("b", "B")]);
    assert_ne!(s1, s3);
}

// ========== Update / Navigation Tests ==========

#[test]
fn test_next_tab() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A"),
        Tab::new("b", "B"),
        Tab::new("c", "C"),
    ]);
    let output = TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_next_tab_at_last() {
    let mut state = TabBarState::with_selected(vec![Tab::new("a", "A"), Tab::new("b", "B")], 1);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::NextTab), None);
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_prev_tab() {
    let mut state = TabBarState::with_selected(
        vec![Tab::new("a", "A"), Tab::new("b", "B"), Tab::new("c", "C")],
        2,
    );
    let output = TabBar::update(&mut state, TabBarMessage::PrevTab);
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
}

#[test]
fn test_prev_tab_at_first() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::PrevTab), None);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_select_tab() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A"),
        Tab::new("b", "B"),
        Tab::new("c", "C"),
    ]);
    let output = TabBar::update(&mut state, TabBarMessage::SelectTab(2));
    assert_eq!(output, Some(TabBarOutput::TabSelected(2)));
}

#[test]
fn test_select_tab_same() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(
        TabBar::update(&mut state, TabBarMessage::SelectTab(0)),
        None
    );
}

#[test]
fn test_select_tab_clamps() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    let output = TabBar::update(&mut state, TabBarMessage::SelectTab(100));
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
}

#[test]
fn test_select_tab_empty() {
    let mut state = TabBarState::new(vec![]);
    assert_eq!(
        TabBar::update(&mut state, TabBarMessage::SelectTab(0)),
        None
    );
}

#[test]
fn test_first_and_last() {
    let mut state = TabBarState::with_selected(
        vec![Tab::new("a", "A"), Tab::new("b", "B"), Tab::new("c", "C")],
        2,
    );
    let output = TabBar::update(&mut state, TabBarMessage::First);
    assert_eq!(output, Some(TabBarOutput::TabSelected(0)));

    let output = TabBar::update(&mut state, TabBarMessage::Last);
    assert_eq!(output, Some(TabBarOutput::TabSelected(2)));
}

#[test]
fn test_first_already_first() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::First), None);
}

#[test]
fn test_last_already_last() {
    let mut state = TabBarState::with_selected(vec![Tab::new("a", "A")], 0);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::Last), None);
}

#[test]
fn test_first_last_empty() {
    let mut state = TabBarState::new(vec![]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::First), None);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::Last), None);
}

// ========== Close Tab Tests ==========

#[test]
fn test_close_tab_before_active() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
        Tab::new("c", "C").with_closable(true),
    ]);
    state.set_selected(Some(1));
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(0));
    assert_eq!(output, Some(TabBarOutput::TabClosed(0)));
    assert_eq!(state.len(), 2);
    assert_eq!(state.selected_index(), Some(0));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("B"));
}

#[test]
fn test_close_tab_active() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
        Tab::new("c", "C").with_closable(true),
    ]);
    state.set_selected(Some(1));
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(1));
    assert_eq!(output, Some(TabBarOutput::TabClosed(1)));
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("C"));
}

#[test]
fn test_close_tab_last_becomes_new_last() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
    ]);
    state.set_selected(Some(1));
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(1));
    assert_eq!(output, Some(TabBarOutput::TabClosed(1)));
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_close_tab_not_closable() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::CloseTab(0)), None);
    assert_eq!(state.len(), 1);
}

#[test]
fn test_close_tab_out_of_bounds() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A").with_closable(true)]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::CloseTab(5)), None);
}

#[test]
fn test_close_active_tab() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
    ]);
    let output = TabBar::update(&mut state, TabBarMessage::CloseActiveTab);
    assert_eq!(output, Some(TabBarOutput::TabClosed(0)));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("B"));
}

#[test]
fn test_close_active_tab_not_closable() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(
        TabBar::update(&mut state, TabBarMessage::CloseActiveTab),
        None
    );
}

#[test]
fn test_close_active_tab_empty() {
    let mut state = TabBarState::new(vec![]);
    assert_eq!(
        TabBar::update(&mut state, TabBarMessage::CloseActiveTab),
        None
    );
}

#[test]
fn test_close_last_tab_clears_state() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A").with_closable(true)]);
    let output = TabBar::update(&mut state, TabBarMessage::CloseTab(0));
    assert_eq!(output, Some(TabBarOutput::TabClosed(0)));
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
    assert_eq!(state.scroll_offset(), 0);
}

// ========== Add Tab Tests ==========

#[test]
fn test_add_tab() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A")]);
    let output = TabBar::update(&mut state, TabBarMessage::AddTab(Tab::new("b", "B")));
    assert_eq!(output, Some(TabBarOutput::TabAdded(1)));
    assert_eq!(state.len(), 2);
    assert_eq!(state.selected_index(), Some(1));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("B"));
}

#[test]
fn test_add_tab_to_empty() {
    let mut state = TabBarState::new(vec![]);
    let output = TabBar::update(&mut state, TabBarMessage::AddTab(Tab::new("a", "A")));
    assert_eq!(output, Some(TabBarOutput::TabAdded(0)));
    assert_eq!(state.selected_index(), Some(0));
}

// ========== Disabled State Tests ==========

// ========== Empty State Tests ==========

#[test]
fn test_empty_navigation() {
    let mut state = TabBarState::new(vec![]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::NextTab), None);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::PrevTab), None);
    assert_eq!(
        TabBar::update(&mut state, TabBarMessage::SelectTab(0)),
        None
    );
    assert_eq!(TabBar::update(&mut state, TabBarMessage::First), None);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::Last), None);
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_navigation_keys() {
    let state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::key(Key::Right),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::NextTab)
    );
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::key(Key::Left),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::PrevTab)
    );
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::key(Key::Home),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::First)
    );
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::key(Key::End),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::Last)
    );
}

#[test]
fn test_handle_event_vim_keys() {
    let state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::char('h'),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::PrevTab)
    );
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::char('l'),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::NextTab)
    );
}

#[test]
fn test_handle_event_close_key() {
    let state = TabBarState::new(vec![Tab::new("a", "A").with_closable(true)]);
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::char('w'),
            &EventContext::new().focused(true)
        ),
        Some(TabBarMessage::CloseActiveTab)
    );
}

#[test]
fn test_handle_event_unfocused() {
    let state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(
        TabBar::handle_event(&state, &Event::key(Key::Right), &EventContext::default()),
        None
    );
    assert_eq!(
        TabBar::handle_event(&state, &Event::char('l'), &EventContext::default()),
        None
    );
}
#[test]
fn test_handle_event_unrecognized_key() {
    let state = TabBarState::new(vec![Tab::new("a", "A")]);
    assert_eq!(
        TabBar::handle_event(
            &state,
            &Event::char('z'),
            &EventContext::new().focused(true)
        ),
        None
    );
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event_next() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);
    let output = TabBar::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.selected_index(), Some(1));
}

#[test]
fn test_dispatch_event_close() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B"),
    ]);
    let output = TabBar::dispatch_event(
        &mut state,
        &Event::char('w'),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(TabBarOutput::TabClosed(0)));
    assert_eq!(state.len(), 1);
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_methods() {
    let mut state = TabBarState::new(vec![Tab::new("a", "A"), Tab::new("b", "B")]);

    let msg = TabBar::handle_event(
        &state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(msg, Some(TabBarMessage::NextTab));

    let output = TabBar::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(TabBarOutput::TabSelected(1)));
    assert_eq!(state.selected_index(), Some(1));

    let output = state.update(TabBarMessage::PrevTab);
    assert_eq!(output, Some(TabBarOutput::TabSelected(0)));
}

// ========== Focusable / Disableable Trait Tests ==========
// ========== Init Tests ==========

#[test]
fn test_init() {
    let state = TabBar::init();
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

// ========== View Tests ==========

#[test]
fn test_view_renders_basic() {
    let state = TabBarState::new(vec![
        Tab::new("a", "Alpha"),
        Tab::new("b", "Beta"),
        Tab::new("c", "Gamma"),
    ]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_focused() {
    let state = TabBarState::new(vec![Tab::new("a", "Tab1"), Tab::new("b", "Tab2")]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            TabBar::view(
                &state,
                &mut RenderContext::new(frame, frame.area(), &theme).focused(true),
            )
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}
#[test]
fn test_view_empty() {
    let state = TabBarState::new(vec![]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_modified() {
    let state = TabBarState::new(vec![
        Tab::new("a", "main.rs").with_modified(true),
        Tab::new("b", "lib.rs"),
    ]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_closable() {
    let state = TabBarState::new(vec![
        Tab::new("a", "main.rs").with_closable(true),
        Tab::new("b", "lib.rs").with_closable(true),
    ]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_icon() {
    let state = TabBarState::new(vec![
        Tab::new("a", "main.rs").with_icon("R"),
        Tab::new("b", "style.css").with_icon("C"),
    ]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_with_all_decorations() {
    let state = TabBarState::new(vec![
        Tab::new("a", "main.rs")
            .with_icon("R")
            .with_modified(true)
            .with_closable(true),
        Tab::new("b", "lib.rs").with_closable(true),
    ]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(50, 1);
    terminal
        .draw(|frame| TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme)))
        .unwrap();
    insta::assert_snapshot!(terminal.backend().to_string());
}

#[test]
fn test_view_zero_area() {
    let state = TabBarState::new(vec![Tab::new("a", "A")]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    terminal
        .draw(|frame| {
            TabBar::view(
                &state,
                &mut RenderContext::new(frame, Rect::new(0, 0, 0, 0), &theme),
            )
        })
        .unwrap();
    // Should not panic
}

// ========== Truncation Tests ==========

#[test]
fn test_truncate_label() {
    assert_eq!(truncate_label("Hello", 10), "Hello");
    assert_eq!(truncate_label("Hello", 5), "Hello");
    assert_eq!(truncate_label("Hello World", 6), "Hello\u{2026}");
    assert_eq!(truncate_label("Hello", 0), "");
    assert_eq!(truncate_label("Hello", 1), "\u{2026}");
}

// ========== Full Workflow / Integration Tests ==========

#[test]
fn test_full_workflow() {
    let mut state = TabBarState::new(vec![
        Tab::new("f1", "main.rs").with_closable(true),
        Tab::new("f2", "lib.rs")
            .with_closable(true)
            .with_modified(true),
        Tab::new("f3", "test.rs").with_closable(true),
    ]);
    assert_eq!(state.selected_index(), Some(0));

    TabBar::update(&mut state, TabBarMessage::NextTab);
    TabBar::update(&mut state, TabBarMessage::NextTab);
    assert_eq!(state.selected_index(), Some(2));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("test.rs"));

    TabBar::update(&mut state, TabBarMessage::PrevTab);
    assert_eq!(state.selected_index(), Some(1));

    let output = TabBar::update(&mut state, TabBarMessage::CloseActiveTab);
    assert_eq!(output, Some(TabBarOutput::TabClosed(1)));
    assert_eq!(state.len(), 2);
    assert_eq!(state.active_tab().map(|t| t.label()), Some("test.rs"));

    TabBar::update(&mut state, TabBarMessage::First);
    assert_eq!(state.selected_index(), Some(0));

    let output = TabBar::update(
        &mut state,
        TabBarMessage::AddTab(Tab::new("f4", "new.rs").with_closable(true)),
    );
    assert_eq!(output, Some(TabBarOutput::TabAdded(2)));
    assert_eq!(state.active_tab().map(|t| t.label()), Some("new.rs"));
}

#[test]
fn test_close_all_tabs_one_by_one() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
        Tab::new("c", "C").with_closable(true),
    ]);
    TabBar::update(&mut state, TabBarMessage::CloseTab(2));
    assert_eq!(state.len(), 2);
    TabBar::update(&mut state, TabBarMessage::CloseTab(1));
    assert_eq!(state.len(), 1);
    TabBar::update(&mut state, TabBarMessage::CloseTab(0));
    assert!(state.is_empty());
    assert_eq!(state.selected_index(), None);
}

#[test]
fn test_single_tab() {
    let mut state = TabBarState::new(vec![Tab::new("only", "Only")]);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::NextTab), None);
    assert_eq!(TabBar::update(&mut state, TabBarMessage::PrevTab), None);
    assert_eq!(state.selected_index(), Some(0));
}

// ========== Annotation Test ==========

#[test]
fn test_annotation_emitted() {
    use crate::annotation::{WidgetType, with_annotations};
    let state = TabBarState::new(vec![Tab::new("a", "Tab1"), Tab::new("b", "Tab2")]);
    let (mut terminal, theme) = crate::component::test_utils::setup_render(40, 1);
    let registry = with_annotations(|| {
        terminal
            .draw(|frame| {
                TabBar::view(&state, &mut RenderContext::new(frame, frame.area(), &theme))
            })
            .unwrap();
    });
    assert_eq!(registry.len(), 1);
    let regions = registry.find_by_type(&WidgetType::TabBar);
    assert_eq!(regions.len(), 1);
}

// ========== Scroll Offset Tests ==========

#[test]
fn test_ensure_active_visible_scrolls_left() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A"),
        Tab::new("b", "B"),
        Tab::new("c", "C"),
    ]);
    state.scroll_offset = 2;
    state.active = Some(0);
    state.ensure_active_visible();
    assert_eq!(state.scroll_offset, 0);
}

#[test]
fn test_scroll_offset_reset_on_first() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A"),
        Tab::new("b", "B"),
        Tab::new("c", "C"),
    ]);
    state.scroll_offset = 2;
    state.active = Some(2);
    TabBar::update(&mut state, TabBarMessage::First);
    assert_eq!(state.scroll_offset, 0);
    assert_eq!(state.selected_index(), Some(0));
}

#[test]
fn test_close_tab_clamps_scroll_offset() {
    let mut state = TabBarState::new(vec![
        Tab::new("a", "A").with_closable(true),
        Tab::new("b", "B").with_closable(true),
    ]);
    state.scroll_offset = 1;
    state.active = Some(1);
    TabBar::update(&mut state, TabBarMessage::CloseTab(1));
    assert_eq!(state.len(), 1);
    assert!(state.scroll_offset < state.len());
}
