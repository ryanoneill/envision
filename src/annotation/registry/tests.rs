use super::*;

#[test]
fn test_registry_register() {
    let mut registry = AnnotationRegistry::new();

    let idx = registry.register(Rect::new(0, 0, 80, 24), Annotation::container("main"));

    assert_eq!(idx, 0);
    assert_eq!(registry.len(), 1);
}

#[test]
fn test_registry_nesting() {
    let mut registry = AnnotationRegistry::new();

    // Open container
    let container = registry.open(Rect::new(0, 0, 80, 24), Annotation::container("main"));

    // Add child
    let button = registry.register(Rect::new(10, 10, 20, 3), Annotation::button("submit"));

    // Close container
    registry.close();

    assert_eq!(registry.len(), 2);

    let container_info = registry.get(container).unwrap();
    assert_eq!(container_info.children, vec![button]);

    let button_info = registry.get(button).unwrap();
    assert_eq!(button_info.parent, Some(container));
    assert_eq!(button_info.depth, 1);
}

#[test]
fn test_registry_region_at() {
    let mut registry = AnnotationRegistry::new();

    // Container
    registry.open(Rect::new(0, 0, 80, 24), Annotation::container("main"));

    // Button inside
    registry.register(Rect::new(10, 10, 20, 3), Annotation::button("submit"));

    registry.close();

    // Point inside button
    let region = registry.region_at(15, 11).unwrap();
    assert!(region.annotation.has_id("submit"));

    // Point outside button but inside container
    let region = registry.region_at(5, 5).unwrap();
    assert!(region.annotation.has_id("main"));

    // Point outside everything
    assert!(registry.region_at(100, 100).is_none());
}

#[test]
fn test_registry_find_by_id() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::input("username"));
    registry.register(Rect::new(0, 2, 10, 1), Annotation::input("password"));
    registry.register(Rect::new(0, 4, 10, 1), Annotation::button("submit"));

    let found = registry.find_by_id("password");
    assert_eq!(found.len(), 1);
    assert!(found[0].annotation.has_id("password"));

    let submit = registry.get_by_id("submit").unwrap();
    assert_eq!(submit.annotation.widget_type, WidgetType::Button);
}

#[test]
fn test_registry_find_by_type() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::input("a"));
    registry.register(Rect::new(0, 2, 10, 1), Annotation::input("b"));
    registry.register(Rect::new(0, 4, 10, 1), Annotation::button("c"));

    let inputs = registry.find_by_type(&WidgetType::Input);
    assert_eq!(inputs.len(), 2);

    let buttons = registry.find_by_type(&WidgetType::Button);
    assert_eq!(buttons.len(), 1);
}

#[test]
fn test_registry_focused() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::input("a"));
    registry.register(
        Rect::new(0, 2, 10, 1),
        Annotation::input("b").with_focus(true),
    );

    let focused = registry.focused_region().unwrap();
    assert!(focused.annotation.has_id("b"));
}

#[test]
fn test_serializable_rect() {
    let rect = SerializableRect::new(5, 10, 20, 30);

    assert!(rect.contains(5, 10));
    assert!(rect.contains(24, 39));
    assert!(!rect.contains(25, 10));
    assert!(!rect.contains(5, 40));
}

#[test]
fn test_rect_intersects() {
    let a = SerializableRect::new(0, 0, 10, 10);
    let b = SerializableRect::new(5, 5, 10, 10);
    let c = SerializableRect::new(20, 20, 10, 10);

    assert!(a.intersects(&b));
    assert!(b.intersects(&a));
    assert!(!a.intersects(&c));
}

#[test]
fn test_format_tree() {
    let mut registry = AnnotationRegistry::new();

    registry.open(Rect::new(0, 0, 80, 24), Annotation::dialog("Login"));
    registry.register(Rect::new(5, 5, 30, 1), Annotation::input("username"));
    registry.register(Rect::new(5, 7, 30, 1), Annotation::input("password"));
    registry.register(Rect::new(5, 10, 10, 1), Annotation::button("submit"));
    registry.close();

    let tree = registry.format_tree();
    assert!(tree.contains("Dialog"));
    assert!(tree.contains("Input"));
    assert!(tree.contains("Button"));
}

#[test]
fn test_registry_clear() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::button("a"));
    registry.register(Rect::new(0, 2, 10, 1), Annotation::button("b"));

    assert_eq!(registry.len(), 2);

    registry.clear();

    assert_eq!(registry.len(), 0);
    assert!(registry.is_empty());
}

#[test]
fn test_registry_is_empty() {
    let registry = AnnotationRegistry::new();
    assert!(registry.is_empty());

    let mut registry2 = AnnotationRegistry::new();
    registry2.register(Rect::new(0, 0, 10, 1), Annotation::button("btn"));
    assert!(!registry2.is_empty());
}

#[test]
fn test_registry_regions_at() {
    let mut registry = AnnotationRegistry::new();

    // Container at depth 0
    registry.open(Rect::new(0, 0, 80, 24), Annotation::container("main"));
    // Button at depth 1
    registry.register(Rect::new(10, 10, 20, 3), Annotation::button("submit"));
    registry.close();

    // Point inside button overlaps both container and button
    let regions = registry.regions_at(15, 11);
    assert_eq!(regions.len(), 2);
}

#[test]
fn test_registry_interactive_regions() {
    let mut registry = AnnotationRegistry::new();

    // Non-interactive
    registry.register(Rect::new(0, 0, 80, 24), Annotation::container("main"));
    registry.register(Rect::new(0, 0, 10, 1), Annotation::label("title"));

    // Interactive
    registry.register(Rect::new(0, 2, 10, 1), Annotation::button("btn"));
    registry.register(Rect::new(0, 4, 10, 1), Annotation::input("input"));
    registry.register(Rect::new(0, 6, 10, 1), Annotation::checkbox("checkbox"));

    let interactive = registry.interactive_regions();
    assert_eq!(interactive.len(), 3);
}

#[test]
fn test_registry_root_regions() {
    let mut registry = AnnotationRegistry::new();

    // Root level
    registry.open(Rect::new(0, 0, 40, 24), Annotation::container("left"));
    registry.register(Rect::new(5, 5, 10, 1), Annotation::button("btn1"));
    registry.close();

    registry.open(Rect::new(40, 0, 40, 24), Annotation::container("right"));
    registry.register(Rect::new(45, 5, 10, 1), Annotation::button("btn2"));
    registry.close();

    let roots = registry.root_regions();
    assert_eq!(roots.len(), 2);
}

#[test]
fn test_registry_children_of() {
    let mut registry = AnnotationRegistry::new();

    let parent = registry.open(Rect::new(0, 0, 80, 24), Annotation::container("parent"));
    registry.register(Rect::new(5, 5, 10, 1), Annotation::button("child1"));
    registry.register(Rect::new(5, 8, 10, 1), Annotation::button("child2"));
    registry.close();

    let children = registry.children_of(parent);
    assert_eq!(children.len(), 2);

    // Non-existent index returns empty
    let children = registry.children_of(999);
    assert!(children.is_empty());
}

#[test]
fn test_registry_regions_accessor() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::button("a"));
    registry.register(Rect::new(0, 2, 10, 1), Annotation::button("b"));

    let regions = registry.regions();
    assert_eq!(regions.len(), 2);
}

#[test]
fn test_serializable_rect_from_rect() {
    let ratatui_rect = Rect::new(10, 20, 30, 40);
    let serializable: SerializableRect = ratatui_rect.into();

    assert_eq!(serializable.x, 10);
    assert_eq!(serializable.y, 20);
    assert_eq!(serializable.width, 30);
    assert_eq!(serializable.height, 40);
}

#[test]
fn test_rect_from_serializable_rect() {
    let serializable = SerializableRect::new(10, 20, 30, 40);
    let ratatui_rect: Rect = serializable.into();

    assert_eq!(ratatui_rect.x, 10);
    assert_eq!(ratatui_rect.y, 20);
    assert_eq!(ratatui_rect.width, 30);
    assert_eq!(ratatui_rect.height, 40);
}

#[test]
fn test_registry_get_non_existent() {
    let registry = AnnotationRegistry::new();
    assert!(registry.get(0).is_none());
    assert!(registry.get(999).is_none());
}

#[test]
fn test_registry_get_by_id_not_found() {
    let mut registry = AnnotationRegistry::new();
    registry.register(Rect::new(0, 0, 10, 1), Annotation::button("exists"));

    assert!(registry.get_by_id("nonexistent").is_none());
}

#[test]
fn test_registry_focused_region_none() {
    let mut registry = AnnotationRegistry::new();

    registry.register(Rect::new(0, 0, 10, 1), Annotation::input("a"));
    registry.register(Rect::new(0, 2, 10, 1), Annotation::input("b"));

    // No focused region
    assert!(registry.focused_region().is_none());
}

#[test]
fn test_registry_close_at_zero_depth() {
    let mut registry = AnnotationRegistry::new();

    // Register without opening
    registry.register(Rect::new(0, 0, 10, 1), Annotation::button("btn"));

    // Close when already at depth 0 should not panic
    registry.close();
    registry.close(); // Extra close

    assert_eq!(registry.len(), 1);
}

#[test]
fn test_registry_default() {
    let registry = AnnotationRegistry::default();
    assert!(registry.is_empty());
}

#[test]
fn test_region_info_fields() {
    let mut registry = AnnotationRegistry::new();

    let parent_idx = registry.open(Rect::new(0, 0, 80, 24), Annotation::container("parent"));
    let child_idx = registry.register(Rect::new(5, 5, 10, 1), Annotation::button("child"));
    registry.close();

    let child = registry.get(child_idx).unwrap();
    assert_eq!(child.area.x, 5);
    assert_eq!(child.area.y, 5);
    assert_eq!(child.area.width, 10);
    assert_eq!(child.area.height, 1);
    assert_eq!(child.parent, Some(parent_idx));
    assert!(child.children.is_empty());
    assert_eq!(child.depth, 1);
}
