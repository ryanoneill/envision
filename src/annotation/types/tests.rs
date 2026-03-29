use super::*;

#[test]
fn test_annotation_builder() {
    let ann = Annotation::button("submit")
        .with_label("Submit Order")
        .with_focus(true);

    assert_eq!(ann.widget_type, WidgetType::Button);
    assert_eq!(ann.id, Some("submit".to_string()));
    assert_eq!(ann.label, Some("Submit Order".to_string()));
    assert!(ann.focused);
}

#[test]
fn test_annotation_has_id() {
    let ann = Annotation::input("username");
    assert!(ann.has_id("username"));
    assert!(!ann.has_id("password"));
}

#[test]
fn test_widget_type_interactive() {
    assert!(WidgetType::Button.is_interactive());
    assert!(WidgetType::Input.is_interactive());
    assert!(!WidgetType::Label.is_interactive());
    assert!(!WidgetType::Container.is_interactive());
}

#[test]
fn test_annotation_description() {
    let ann = Annotation::button("ok").with_label("OK").with_focus(true);

    let desc = ann.description();
    assert!(desc.contains("Button"));
    assert!(desc.contains("OK"));
    assert!(desc.contains("#ok"));
    assert!(desc.contains("(focused)"));
}

#[cfg(feature = "serialization")]
#[test]
fn test_annotation_serialization() {
    let ann = Annotation::input("email")
        .with_label("Email Address")
        .with_value("test@example.com");

    let json = serde_json::to_string(&ann).unwrap();
    let deserialized: Annotation = serde_json::from_str(&json).unwrap();

    assert_eq!(ann, deserialized);
}

#[test]
fn test_annotation_metadata() {
    let ann = Annotation::custom("datepicker", "birth_date")
        .with_meta("format", "YYYY-MM-DD")
        .with_meta("min_year", "1900");

    assert_eq!(ann.metadata.get("format"), Some(&"YYYY-MM-DD".to_string()));
    assert_eq!(ann.metadata.get("min_year"), Some(&"1900".to_string()));
}

#[test]
fn test_widget_type_is_container() {
    assert!(WidgetType::Container.is_container());
    assert!(WidgetType::Dialog.is_container());
    assert!(WidgetType::Scroll.is_container());
    assert!(WidgetType::Sidebar.is_container());
    assert!(!WidgetType::Button.is_container());
    assert!(!WidgetType::Input.is_container());
}

#[test]
fn test_widget_type_display() {
    assert_eq!(format!("{}", WidgetType::Button), "Button");
    assert_eq!(format!("{}", WidgetType::Input), "Input");
    assert_eq!(
        format!("{}", WidgetType::Custom("MyWidget".to_string())),
        "MyWidget"
    );
}

#[test]
fn test_annotation_new() {
    let ann = Annotation::new(WidgetType::Progress);
    assert_eq!(ann.widget_type, WidgetType::Progress);
    assert!(ann.label.is_none());
    assert!(ann.id.is_none());
    assert!(!ann.focused);
    assert!(!ann.disabled);
    assert!(!ann.selected);
    assert!(ann.expanded.is_none());
    assert!(ann.value.is_none());
    assert!(ann.metadata.is_empty());
}

#[test]
fn test_annotation_default() {
    let ann = Annotation::default();
    assert_eq!(ann.widget_type, WidgetType::Container);
}

#[test]
fn test_annotation_dialog() {
    let ann = Annotation::dialog("Confirm Delete");
    assert_eq!(ann.widget_type, WidgetType::Dialog);
    assert_eq!(ann.label, Some("Confirm Delete".to_string()));
}

#[test]
fn test_annotation_text_area() {
    let ann = Annotation::text_area("description");
    assert_eq!(ann.widget_type, WidgetType::TextArea);
    assert!(ann.has_id("description"));
}

#[test]
fn test_annotation_checkbox() {
    let ann = Annotation::checkbox("agree_tos");
    assert_eq!(ann.widget_type, WidgetType::Checkbox);
    assert!(ann.has_id("agree_tos"));
}

#[test]
fn test_annotation_list() {
    let ann = Annotation::list("items");
    assert_eq!(ann.widget_type, WidgetType::List);
    assert!(ann.has_id("items"));
}

#[test]
fn test_annotation_table() {
    let ann = Annotation::table("data_grid");
    assert_eq!(ann.widget_type, WidgetType::Table);
    assert!(ann.has_id("data_grid"));
}

#[test]
fn test_annotation_tab() {
    let ann = Annotation::tab("settings_tab");
    assert_eq!(ann.widget_type, WidgetType::Tab);
    assert!(ann.has_id("settings_tab"));
}

#[test]
fn test_annotation_menu_item() {
    let ann = Annotation::menu_item("file_open");
    assert_eq!(ann.widget_type, WidgetType::MenuItem);
    assert!(ann.has_id("file_open"));
}

#[test]
fn test_annotation_label() {
    let ann = Annotation::label("Username:");
    assert_eq!(ann.widget_type, WidgetType::Label);
    assert_eq!(ann.label, Some("Username:".to_string()));
}

#[test]
fn test_annotation_header() {
    let ann = Annotation::header("Welcome");
    assert_eq!(ann.widget_type, WidgetType::Header);
    assert_eq!(ann.label, Some("Welcome".to_string()));
}

#[test]
fn test_annotation_with_disabled() {
    let ann = Annotation::button("submit").with_disabled(true);
    assert!(ann.disabled);
}

#[test]
fn test_annotation_with_selected() {
    let ann = Annotation::checkbox("option").with_selected(true);
    assert!(ann.selected);
}

#[test]
fn test_annotation_with_expanded() {
    let ann = Annotation::container("tree_node").with_expanded(true);
    assert_eq!(ann.expanded, Some(true));

    let collapsed = Annotation::container("tree_node").with_expanded(false);
    assert_eq!(collapsed.expanded, Some(false));
}

#[test]
fn test_annotation_is_type() {
    let ann = Annotation::button("btn");
    assert!(ann.is_type(&WidgetType::Button));
    assert!(!ann.is_type(&WidgetType::Input));
}

#[test]
fn test_annotation_is_interactive() {
    // Button is interactive
    let btn = Annotation::button("btn");
    assert!(btn.is_interactive());

    // Disabled button is not interactive
    let disabled_btn = Annotation::button("btn").with_disabled(true);
    assert!(!disabled_btn.is_interactive());

    // Label is not interactive
    let lbl = Annotation::label("text");
    assert!(!lbl.is_interactive());
}

#[test]
fn test_annotation_description_with_states() {
    let ann = Annotation::button("save")
        .with_disabled(true)
        .with_selected(true);

    let desc = ann.description();
    assert!(desc.contains("(disabled)"));
    assert!(desc.contains("(selected)"));
}

#[test]
fn test_widget_type_interactive_all() {
    // Test all interactive types
    assert!(WidgetType::TextArea.is_interactive());
    assert!(WidgetType::Checkbox.is_interactive());
    assert!(WidgetType::Radio.is_interactive());
    assert!(WidgetType::Select.is_interactive());
    assert!(WidgetType::List.is_interactive());
    assert!(WidgetType::Table.is_interactive());
    assert!(WidgetType::Tab.is_interactive());
    assert!(WidgetType::MenuItem.is_interactive());
    assert!(WidgetType::Tree.is_interactive());
    assert!(WidgetType::LineInput.is_interactive());
    assert!(WidgetType::Dropdown.is_interactive());
    assert!(WidgetType::LoadingList.is_interactive());
    assert!(WidgetType::Accordion.is_interactive());
    assert!(WidgetType::RadioGroup.is_interactive());
    assert!(WidgetType::SearchableList.is_interactive());

    // Non-interactive types
    assert!(!WidgetType::Header.is_interactive());
    assert!(!WidgetType::Footer.is_interactive());
    assert!(!WidgetType::Sidebar.is_interactive());
    assert!(!WidgetType::Toolbar.is_interactive());
    assert!(!WidgetType::StatusBar.is_interactive());
    assert!(!WidgetType::Progress.is_interactive());
    assert!(!WidgetType::Scroll.is_interactive());
    assert!(!WidgetType::TabBar.is_interactive());
    assert!(!WidgetType::Menu.is_interactive());
    assert!(!WidgetType::Custom("test".to_string()).is_interactive());
    assert!(!WidgetType::Spinner.is_interactive());
    assert!(!WidgetType::Toast.is_interactive());
    assert!(!WidgetType::Tooltip.is_interactive());
    assert!(!WidgetType::Breadcrumb.is_interactive());
    assert!(!WidgetType::KeyHints.is_interactive());
    assert!(!WidgetType::MultiProgress.is_interactive());
    assert!(!WidgetType::StatusLog.is_interactive());
    assert!(!WidgetType::TitleCard.is_interactive());
    assert!(!WidgetType::ScrollableText.is_interactive());
    assert!(!WidgetType::Form.is_interactive());
    assert!(!WidgetType::SplitPanel.is_interactive());
    assert!(!WidgetType::BigText.is_interactive());
}

#[test]
fn test_widget_type_hash() {
    use std::collections::HashSet;

    let mut set = HashSet::new();
    set.insert(WidgetType::Button);
    set.insert(WidgetType::Input);
    set.insert(WidgetType::Button); // Duplicate

    assert_eq!(set.len(), 2);
}

// =============================================================================
// New WidgetType variants
// =============================================================================

#[test]
fn test_widget_type_is_container_expanded() {
    assert!(WidgetType::Form.is_container());
    assert!(WidgetType::SplitPanel.is_container());
    assert!(!WidgetType::Spinner.is_container());
    assert!(!WidgetType::LineInput.is_container());
}

#[test]
fn test_annotation_spinner() {
    let ann = Annotation::spinner("loading");
    assert_eq!(ann.widget_type, WidgetType::Spinner);
    assert!(ann.has_id("loading"));
}

#[test]
fn test_annotation_toast_constructor() {
    let ann = Annotation::toast("notification");
    assert_eq!(ann.widget_type, WidgetType::Toast);
    assert!(ann.has_id("notification"));
}

#[test]
fn test_annotation_tooltip_constructor() {
    let ann = Annotation::tooltip("help_tip");
    assert_eq!(ann.widget_type, WidgetType::Tooltip);
    assert!(ann.has_id("help_tip"));
}

#[test]
fn test_annotation_accordion_constructor() {
    let ann = Annotation::accordion("settings");
    assert_eq!(ann.widget_type, WidgetType::Accordion);
    assert!(ann.has_id("settings"));
}

#[test]
fn test_annotation_breadcrumb_constructor() {
    let ann = Annotation::breadcrumb("nav");
    assert_eq!(ann.widget_type, WidgetType::Breadcrumb);
    assert!(ann.has_id("nav"));
}

#[test]
fn test_annotation_loading_list_constructor() {
    let ann = Annotation::loading_list("tasks");
    assert_eq!(ann.widget_type, WidgetType::LoadingList);
    assert!(ann.has_id("tasks"));
}

#[test]
fn test_annotation_key_hints_constructor() {
    let ann = Annotation::key_hints("hints");
    assert_eq!(ann.widget_type, WidgetType::KeyHints);
    assert!(ann.has_id("hints"));
}

#[test]
fn test_annotation_multi_progress_constructor() {
    let ann = Annotation::multi_progress("downloads");
    assert_eq!(ann.widget_type, WidgetType::MultiProgress);
    assert!(ann.has_id("downloads"));
}

#[test]
fn test_annotation_status_log_constructor() {
    let ann = Annotation::status_log("log");
    assert_eq!(ann.widget_type, WidgetType::StatusLog);
    assert!(ann.has_id("log"));
}

#[test]
fn test_annotation_title_card_constructor() {
    let ann = Annotation::title_card("app_title");
    assert_eq!(ann.widget_type, WidgetType::TitleCard);
    assert!(ann.has_id("app_title"));
}

#[test]
fn test_annotation_line_input_constructor() {
    let ann = Annotation::line_input("chat_input");
    assert_eq!(ann.widget_type, WidgetType::LineInput);
    assert!(ann.has_id("chat_input"));
}

#[test]
fn test_annotation_dropdown_constructor() {
    let ann = Annotation::dropdown("color_picker");
    assert_eq!(ann.widget_type, WidgetType::Dropdown);
    assert!(ann.has_id("color_picker"));
}

#[test]
fn test_annotation_scrollable_text_constructor() {
    let ann = Annotation::scrollable_text("preview");
    assert_eq!(ann.widget_type, WidgetType::ScrollableText);
    assert!(ann.has_id("preview"));
}

#[test]
fn test_annotation_form_constructor() {
    let ann = Annotation::form("login_form");
    assert_eq!(ann.widget_type, WidgetType::Form);
    assert!(ann.has_id("login_form"));
}

#[test]
fn test_annotation_split_panel_constructor() {
    let ann = Annotation::split_panel("editor");
    assert_eq!(ann.widget_type, WidgetType::SplitPanel);
    assert!(ann.has_id("editor"));
}

#[test]
fn test_annotation_searchable_list_constructor() {
    let ann = Annotation::searchable_list("file_list");
    assert_eq!(ann.widget_type, WidgetType::SearchableList);
    assert!(ann.has_id("file_list"));
}

#[test]
fn test_annotation_radio_group_constructor() {
    let ann = Annotation::radio_group("options");
    assert_eq!(ann.widget_type, WidgetType::RadioGroup);
    assert!(ann.has_id("options"));
}
