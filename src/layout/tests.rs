use super::*;

mod vertical_tests {
    use super::*;

    #[test]
    fn splits_into_two_parts() {
        let area = Rect::new(0, 0, 80, 24);
        let [top, bottom] = vertical(area, [Constraint::Length(4), Constraint::Min(0)]);

        assert_eq!(top, Rect::new(0, 0, 80, 4));
        assert_eq!(bottom, Rect::new(0, 4, 80, 20));
    }

    #[test]
    fn splits_into_three_parts() {
        let area = Rect::new(0, 0, 80, 24);
        let [header, body, footer] = vertical(
            area,
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        );

        assert_eq!(header.height, 3);
        assert_eq!(body.height, 20);
        assert_eq!(footer.height, 1);
    }

    #[test]
    fn handles_single_part() {
        let area = Rect::new(0, 0, 80, 24);
        let [full] = vertical(area, [Constraint::Min(0)]);

        assert_eq!(full, area);
    }

    #[test]
    fn preserves_x_and_width() {
        let area = Rect::new(10, 5, 60, 20);
        let [top, bottom] = vertical(area, [Constraint::Length(5), Constraint::Min(0)]);

        assert_eq!(top.x, 10);
        assert_eq!(top.width, 60);
        assert_eq!(bottom.x, 10);
        assert_eq!(bottom.width, 60);
    }

    #[test]
    fn handles_percentage_constraints() {
        let area = Rect::new(0, 0, 80, 100);
        let [top, bottom] = vertical(
            area,
            [Constraint::Percentage(30), Constraint::Percentage(70)],
        );

        assert_eq!(top.height, 30);
        assert_eq!(bottom.height, 70);
    }
}

mod horizontal_tests {
    use super::*;

    #[test]
    fn splits_into_two_parts() {
        let area = Rect::new(0, 0, 80, 24);
        let [left, right] = horizontal(area, [Constraint::Length(20), Constraint::Min(0)]);

        assert_eq!(left, Rect::new(0, 0, 20, 24));
        assert_eq!(right, Rect::new(20, 0, 60, 24));
    }

    #[test]
    fn splits_into_three_parts() {
        let area = Rect::new(0, 0, 90, 24);
        let [left, center, right] = horizontal(
            area,
            [
                Constraint::Length(20),
                Constraint::Min(0),
                Constraint::Length(20),
            ],
        );

        assert_eq!(left.width, 20);
        assert_eq!(center.width, 50);
        assert_eq!(right.width, 20);
    }

    #[test]
    fn preserves_y_and_height() {
        let area = Rect::new(5, 10, 60, 20);
        let [left, right] = horizontal(area, [Constraint::Length(30), Constraint::Min(0)]);

        assert_eq!(left.y, 10);
        assert_eq!(left.height, 20);
        assert_eq!(right.y, 10);
        assert_eq!(right.height, 20);
    }

    #[test]
    fn handles_percentage_constraints() {
        let area = Rect::new(0, 0, 100, 24);
        let [left, right] = horizontal(
            area,
            [Constraint::Percentage(40), Constraint::Percentage(60)],
        );

        assert_eq!(left.width, 40);
        assert_eq!(right.width, 60);
    }
}

mod centered_tests {
    use super::*;

    #[test]
    fn centers_in_area() {
        let area = Rect::new(0, 0, 80, 24);
        let result = centered(area, 40, 10);

        assert_eq!(result.x, 20);
        assert_eq!(result.y, 7);
        assert_eq!(result.width, 40);
        assert_eq!(result.height, 10);
    }

    #[test]
    fn clamps_to_area_bounds() {
        let area = Rect::new(0, 0, 30, 10);
        let result = centered(area, 50, 20);

        assert_eq!(result.width, 30);
        assert_eq!(result.height, 10);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn handles_offset_area() {
        let area = Rect::new(10, 5, 80, 24);
        let result = centered(area, 40, 10);

        assert_eq!(result.x, 30); // 10 + (80 - 40) / 2
        assert_eq!(result.y, 12); // 5 + (24 - 10) / 2
    }

    #[test]
    fn exact_fit() {
        let area = Rect::new(0, 0, 40, 10);
        let result = centered(area, 40, 10);

        assert_eq!(result, area);
    }

    #[test]
    fn zero_size() {
        let area = Rect::new(0, 0, 80, 24);
        let result = centered(area, 0, 0);

        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
        assert_eq!(result.x, 40);
        assert_eq!(result.y, 12);
    }
}

mod centered_percent_tests {
    use super::*;

    #[test]
    fn centers_by_percentage() {
        let area = Rect::new(0, 0, 100, 50);
        let result = centered_percent(area, 60, 40);

        assert_eq!(result.width, 60);
        assert_eq!(result.height, 20);
        assert_eq!(result.x, 20);
        assert_eq!(result.y, 15);
    }

    #[test]
    fn full_percentage() {
        let area = Rect::new(0, 0, 80, 24);
        let result = centered_percent(area, 100, 100);

        assert_eq!(result, area);
    }

    #[test]
    fn zero_percentage() {
        let area = Rect::new(0, 0, 80, 24);
        let result = centered_percent(area, 0, 0);

        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn clamps_percentage_over_100() {
        let area = Rect::new(0, 0, 80, 24);
        let result = centered_percent(area, 150, 200);

        assert_eq!(result, area);
    }

    #[test]
    fn handles_offset_area() {
        let area = Rect::new(10, 5, 100, 50);
        let result = centered_percent(area, 50, 50);

        assert_eq!(result.width, 50);
        assert_eq!(result.height, 25);
        assert_eq!(result.x, 35); // 10 + (100 - 50) / 2
        assert_eq!(result.y, 17); // 5 + (50 - 25) / 2 = 5 + 12
    }
}

mod re_export_tests {
    use super::*;

    #[test]
    fn rect_is_accessible() {
        let rect = Rect::new(0, 0, 80, 24);
        assert_eq!(rect.width, 80);
        assert_eq!(rect.height, 24);
    }

    #[test]
    fn constraint_variants_are_accessible() {
        let _ = Constraint::Length(10);
        let _ = Constraint::Min(5);
        let _ = Constraint::Max(20);
        let _ = Constraint::Percentage(50);
        let _ = Constraint::Ratio(1, 3);
        let _ = Constraint::Fill(1);
    }

    #[test]
    fn direction_is_accessible() {
        let _ = Direction::Vertical;
        let _ = Direction::Horizontal;
    }

    #[test]
    fn alignment_is_accessible() {
        let _ = Alignment::Left;
        let _ = Alignment::Center;
        let _ = Alignment::Right;
    }

    #[test]
    fn margin_is_accessible() {
        let _ = Margin::new(1, 2);
    }

    #[test]
    fn position_is_accessible() {
        let pos = Position::new(5, 10);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn size_is_accessible() {
        let size = Size::new(80, 24);
        assert_eq!(size.width, 80);
        assert_eq!(size.height, 24);
    }
}
