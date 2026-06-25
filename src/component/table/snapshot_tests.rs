use super::*;

#[test]
fn table_renders_when_columns_clipped() {
    use crate::component::table::{Column, Table, TableState};
    use crate::component::test_utils::setup_render;
    use ratatui::layout::Constraint;

    #[derive(Clone, Debug, PartialEq)]
    struct R(&'static str, &'static str);
    impl TableRow for R {
        fn cells(&self) -> Vec<crate::component::cell::Cell> {
            vec![
                crate::component::cell::Cell::from(self.0.to_string()),
                crate::component::cell::Cell::from(self.1.to_string()),
            ]
        }
    }

    let columns = vec![
        Column::new("Long", Constraint::Length(20)),
        Column::new("Also Long", Constraint::Length(20)),
    ];
    let state = TableState::new(vec![R("aaa", "bbb"), R("ccc", "ddd")], columns);

    // 20-wide area can't honor 2x Length(20). Both columns clip.
    // Table must still render without panic and content must appear.
    let (mut term, theme) = setup_render(20, 6);
    term.draw(|frame| {
        let ctx = &mut crate::component::RenderContext::new(frame, frame.area(), &theme);
        <Table<R> as crate::component::Component>::view(&state, ctx);
    })
    .unwrap();

    insta::assert_snapshot!(term.backend().to_string());
}
