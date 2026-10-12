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

#[test]
fn serializable_style_types_are_in_the_prelude() {
    use envision::prelude::{SerializableColor, SerializableDiffOption, SerializableModifier};
    let _color = SerializableColor::Red;
    let _modifier = SerializableModifier::empty();
    let _option = SerializableDiffOption::None;
}

#[test]
fn compact_str_is_reexported_and_matches_cell_api() {
    let s: envision::compact_str::CompactString = "abc".into();
    let cell = envision::component::cell::Cell::from(s.clone());
    match cell.sort_key() {
        None => {}
        Some(envision::component::cell::SortKey::String(v)) => assert_eq!(v.as_str(), "abc"),
        other => panic!("unexpected sort key {other:?}"),
    }
    let key = envision::component::cell::SortKey::String(s);
    match key {
        envision::component::cell::SortKey::String(v) => assert_eq!(v.as_str(), "abc"),
        other => panic!("unexpected sort key {other:?}"),
    }
}

#[test]
fn string_and_str_still_construct_cells() {
    let owned = String::from("x");
    let _a = envision::component::cell::Cell::new(&owned);
    let _b = envision::component::cell::Cell::new(owned);
    let _c = envision::component::cell::Cell::new("lit");
}
