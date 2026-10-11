//! Public re-exports that let consumers stay on envision's exact ratatui.

#[test]
fn ratatui_crate_is_reexported() {
    let area = envision::ratatui::layout::Rect::new(0, 0, 4, 2);
    let buffer = envision::ratatui::buffer::Buffer::empty(area);
    assert_eq!(buffer.area, area);
}

#[test]
fn crossterm_is_reachable_through_ratatui() {
    let _code = envision::ratatui::crossterm::event::KeyCode::Enter;
}

#[test]
fn backend_trait_is_reexported() {
    fn assert_backend<B: envision::backend::Backend>() {}
    assert_backend::<envision::CaptureBackend>();
}

#[test]
fn serializable_style_types_are_nameable() {
    let _color = envision::backend::SerializableColor::Red;
    let _modifier: envision::SerializableModifier = envision::SerializableModifier::empty();
    let _root_color: envision::SerializableColor = envision::SerializableColor::Reset;
}
