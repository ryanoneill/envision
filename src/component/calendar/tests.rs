use super::*;
use crate::input::{Event, Key};

// ========== Date Math Helper Tests ==========

#[test]
fn test_is_leap_year() {
    // Divisible by 4 but not 100
    assert!(is_leap_year(2024));
    assert!(is_leap_year(2028));
    // Not divisible by 4
    assert!(!is_leap_year(2025));
    assert!(!is_leap_year(2026));
    assert!(!is_leap_year(2027));
    // Divisible by 100 but not 400
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2100));
    // Divisible by 400
    assert!(is_leap_year(2000));
    assert!(is_leap_year(2400));
    assert!(is_leap_year(1600));
}

#[test]
fn test_days_in_month_all_months() {
    assert_eq!(days_in_month(2026, 1), 31);
    assert_eq!(days_in_month(2026, 2), 28);
    assert_eq!(days_in_month(2026, 3), 31);
    assert_eq!(days_in_month(2026, 4), 30);
    assert_eq!(days_in_month(2026, 5), 31);
    assert_eq!(days_in_month(2026, 6), 30);
    assert_eq!(days_in_month(2026, 7), 31);
    assert_eq!(days_in_month(2026, 8), 31);
    assert_eq!(days_in_month(2026, 9), 30);
    assert_eq!(days_in_month(2026, 10), 31);
    assert_eq!(days_in_month(2026, 11), 30);
    assert_eq!(days_in_month(2026, 12), 31);
}

#[test]
fn test_days_in_month_february_leap_years() {
    assert_eq!(days_in_month(2024, 2), 29);
    assert_eq!(days_in_month(2028, 2), 29);
    assert_eq!(days_in_month(2025, 2), 28);
}

#[test]
fn test_day_of_week_known_dates() {
    assert_eq!(day_of_week(2026, 1, 1), 4); // Thursday
    assert_eq!(day_of_week(2026, 3, 1), 0); // Sunday
    assert_eq!(day_of_week(2026, 3, 7), 6); // Saturday
    assert_eq!(day_of_week(2026, 3, 28), 6); // Saturday
    assert_eq!(day_of_week(2024, 2, 29), 4); // Thursday
    assert_eq!(day_of_week(2000, 1, 1), 6); // Saturday
    assert_eq!(day_of_week(1970, 1, 1), 4); // Thursday
}

#[test]
fn test_month_name_all_months() {
    let expected = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    for (i, name) in expected.iter().enumerate() {
        assert_eq!(month_name_for(i as u32 + 1), *name);
    }
    assert_eq!(month_name_for(0), "Unknown");
    assert_eq!(month_name_for(13), "Unknown");
}

// ========== Construction Tests ==========

#[test]
fn test_new() {
    let state = CalendarState::new(2026, 3);
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 3);
    assert_eq!(state.selected_day(), None);
    assert_eq!(state.month_name(), "March");
}

#[test]
fn test_with_selected_day() {
    let state = CalendarState::new(2026, 3).with_selected_day(15);
    assert_eq!(state.selected_day(), Some(15));
}

#[test]
fn test_with_title() {
    let state = CalendarState::new(2026, 3).with_title("My Calendar");
    assert_eq!(state.title, Some("My Calendar".to_string()));
}

#[test]
fn test_with_title_from_string() {
    let state = CalendarState::new(2026, 3).with_title(String::from("Calendar"));
    assert_eq!(state.title, Some("Calendar".to_string()));
}

#[test]
fn test_with_event() {
    let state = CalendarState::new(2026, 3).with_event(2026, 3, 15, Color::Green);
    assert!(state.has_event(2026, 3, 15));
    assert!(!state.has_event(2026, 3, 16));
}

#[test]
fn test_builder_chaining() {
    let state = CalendarState::new(2026, 3)
        .with_selected_day(10)
        .with_title("Events")
        .with_event(2026, 3, 15, Color::Red);
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 3);
    assert_eq!(state.selected_day(), Some(10));
    assert!(state.has_event(2026, 3, 15));
}

// ========== Accessor Tests ==========

#[test]
fn test_set_selected_day() {
    let mut state = CalendarState::new(2026, 3);
    state.set_selected_day(Some(15));
    assert_eq!(state.selected_day(), Some(15));
    state.set_selected_day(None);
    assert_eq!(state.selected_day(), None);
}

#[test]
fn test_month_name_accessor() {
    for m in 1..=12 {
        let state = CalendarState::new(2026, m);
        assert_eq!(state.month_name(), month_name_for(m));
    }
}

// ========== Event Marker Tests ==========

#[test]
fn test_add_event() {
    let mut state = CalendarState::new(2026, 3);
    state.add_event(2026, 3, 15, Color::Red);
    assert!(state.has_event(2026, 3, 15));
}

#[test]
fn test_add_event_different_month() {
    let mut state = CalendarState::new(2026, 3);
    state.add_event(2026, 4, 1, Color::Blue);
    assert!(state.has_event(2026, 4, 1));
    assert!(!state.has_event(2026, 3, 1));
}

#[test]
fn test_clear_events() {
    let mut state = CalendarState::new(2026, 3);
    state.add_event(2026, 3, 15, Color::Red);
    state.add_event(2026, 3, 20, Color::Blue);
    state.clear_events();
    assert!(!state.has_event(2026, 3, 15));
    assert!(!state.has_event(2026, 3, 20));
}

#[test]
fn test_has_event_no_events() {
    let state = CalendarState::new(2026, 3);
    assert!(!state.has_event(2026, 3, 1));
}

#[test]
fn test_add_event_overwrites_color() {
    let mut state = CalendarState::new(2026, 3);
    state.add_event(2026, 3, 15, Color::Red);
    state.add_event(2026, 3, 15, Color::Blue);
    assert_eq!(state.events.get(&(2026, 3, 15)), Some(&Color::Blue));
}

// ========== Update Tests: Month Navigation ==========

#[test]
fn test_update_next_month() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(state.month(), 4);
    assert_eq!(state.year(), 2026);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
}

#[test]
fn test_update_next_month_december_wraps_to_january() {
    let mut state = CalendarState::new(2026, 12);
    let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(state.month(), 1);
    assert_eq!(state.year(), 2027);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 1)));
}

#[test]
fn test_update_prev_month() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(state.month(), 2);
    assert_eq!(state.year(), 2026);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 2)));
}

#[test]
fn test_update_prev_month_january_wraps_to_december() {
    let mut state = CalendarState::new(2026, 1);
    let output = Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(state.month(), 12);
    assert_eq!(state.year(), 2025);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2025, 12)));
}

#[test]
fn test_update_next_month_clamps_selected_day() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(31);
    Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(state.month(), 4);
    assert_eq!(state.selected_day(), Some(30));
}

#[test]
fn test_update_prev_month_clamps_selected_day() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(31);
    Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(state.month(), 2);
    assert_eq!(state.selected_day(), Some(28));
}

// ========== Update Tests: Year Navigation ==========

#[test]
fn test_update_next_year() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::NextYear);
    assert_eq!(state.year(), 2027);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 3)));
}

#[test]
fn test_update_prev_year() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::PrevYear);
    assert_eq!(state.year(), 2025);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2025, 3)));
}

#[test]
fn test_update_year_navigation_clamps_feb_29() {
    // 2024 leap -> 2025 non-leap
    let mut state = CalendarState::new(2024, 2).with_selected_day(29);
    Calendar::update(&mut state, CalendarMessage::NextYear);
    assert_eq!(state.year(), 2025);
    assert_eq!(state.selected_day(), Some(28));

    // 2024 leap -> 2023 non-leap
    let mut state = CalendarState::new(2024, 2).with_selected_day(29);
    Calendar::update(&mut state, CalendarMessage::PrevYear);
    assert_eq!(state.year(), 2023);
    assert_eq!(state.selected_day(), Some(28));
}

// ========== Update Tests: Day Selection ==========

#[test]
fn test_update_select_day() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::SelectDay(15));
    assert_eq!(state.selected_day(), Some(15));
    assert_eq!(output, None);
}

#[test]
fn test_update_select_day_clamps() {
    let mut state = CalendarState::new(2026, 2);
    Calendar::update(&mut state, CalendarMessage::SelectDay(31));
    assert_eq!(state.selected_day(), Some(28));

    Calendar::update(&mut state, CalendarMessage::SelectDay(0));
    assert_eq!(state.selected_day(), Some(1));
}

// ========== Update Tests: Day Navigation with Wrapping ==========

#[test]
fn test_update_select_prev_day() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevDay);
    assert_eq!(state.selected_day(), Some(14));
    assert_eq!(output, None);
}

#[test]
fn test_update_select_prev_day_wraps_to_prev_month() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(1);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevDay);
    assert_eq!(state.month(), 2);
    assert_eq!(state.selected_day(), Some(28));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 2)));
}

#[test]
fn test_update_select_prev_day_wraps_january_to_december() {
    let mut state = CalendarState::new(2026, 1).with_selected_day(1);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevDay);
    assert_eq!(state.year(), 2025);
    assert_eq!(state.month(), 12);
    assert_eq!(state.selected_day(), Some(31));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2025, 12)));
}

#[test]
fn test_update_select_next_day() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.selected_day(), Some(16));
    assert_eq!(output, None);
}

#[test]
fn test_update_select_next_day_wraps_to_next_month() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(31);
    let output = Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.month(), 4);
    assert_eq!(state.selected_day(), Some(1));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
}

#[test]
fn test_update_select_next_day_wraps_december_to_january() {
    let mut state = CalendarState::new(2026, 12).with_selected_day(31);
    let output = Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.year(), 2027);
    assert_eq!(state.month(), 1);
    assert_eq!(state.selected_day(), Some(1));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 1)));
}

#[test]
fn test_update_select_prev_week() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevWeek);
    assert_eq!(state.selected_day(), Some(8));
    assert_eq!(output, None);
}

#[test]
fn test_update_select_prev_week_wraps_to_prev_month() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(3);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevWeek);
    assert_eq!(state.month(), 2);
    assert_eq!(state.selected_day(), Some(24));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 2)));
}

#[test]
fn test_update_select_prev_week_from_day_7() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(7);
    let output = Calendar::update(&mut state, CalendarMessage::SelectPrevWeek);
    assert_eq!(state.month(), 2);
    assert_eq!(state.selected_day(), Some(28));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 2)));
}

#[test]
fn test_update_select_next_week() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::update(&mut state, CalendarMessage::SelectNextWeek);
    assert_eq!(state.selected_day(), Some(22));
    assert_eq!(output, None);
}

#[test]
fn test_update_select_next_week_wraps_to_next_month() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(28);
    let output = Calendar::update(&mut state, CalendarMessage::SelectNextWeek);
    assert_eq!(state.month(), 4);
    assert_eq!(state.selected_day(), Some(4));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
}

#[test]
fn test_update_navigation_defaults_to_day_1() {
    let mut state = CalendarState::new(2026, 3);
    Calendar::update(&mut state, CalendarMessage::SelectPrevDay);
    assert_eq!(state.month(), 2); // Wraps since default is day 1

    let mut state = CalendarState::new(2026, 3);
    Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.selected_day(), Some(2));
}

// ========== Update Tests: Confirm Selection ==========

#[test]
fn test_update_confirm_selection() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::update(&mut state, CalendarMessage::ConfirmSelection);
    assert_eq!(output, Some(CalendarOutput::DateSelected(2026, 3, 15)));
}

#[test]
fn test_update_confirm_selection_no_day() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(&mut state, CalendarMessage::ConfirmSelection);
    assert_eq!(output, None);
}

// ========== Update Tests: Today and SetDate ==========

#[test]
fn test_update_today() {
    let mut state = CalendarState::new(2026, 1);
    let output = Calendar::update(
        &mut state,
        CalendarMessage::Today {
            year: 2026,
            month: 3,
            day: 28,
        },
    );
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 3);
    assert_eq!(state.selected_day(), Some(28));
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 3)));
}

#[test]
fn test_update_today_clamps_day() {
    let mut state = CalendarState::new(2026, 1);
    Calendar::update(
        &mut state,
        CalendarMessage::Today {
            year: 2026,
            month: 2,
            day: 31,
        },
    );
    assert_eq!(state.selected_day(), Some(28));
}

#[test]
fn test_update_set_date() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(
        &mut state,
        CalendarMessage::SetDate {
            year: 2027,
            month: 6,
        },
    );
    assert_eq!(state.year(), 2027);
    assert_eq!(state.month(), 6);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2027, 6)));
}

#[test]
fn test_update_set_date_clamps_selected_day() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(31);
    Calendar::update(
        &mut state,
        CalendarMessage::SetDate {
            year: 2026,
            month: 2,
        },
    );
    assert_eq!(state.selected_day(), Some(28));
}

// ========== Update Tests: Events ==========

#[test]
fn test_update_add_event() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::update(
        &mut state,
        CalendarMessage::AddEvent {
            year: 2026,
            month: 3,
            day: 15,
            color: Color::Green,
        },
    );
    assert!(state.has_event(2026, 3, 15));
    assert_eq!(output, None);
}

#[test]
fn test_update_clear_events() {
    let mut state = CalendarState::new(2026, 3)
        .with_event(2026, 3, 15, Color::Red)
        .with_event(2026, 3, 20, Color::Blue);
    Calendar::update(&mut state, CalendarMessage::ClearEvents);
    assert!(!state.has_event(2026, 3, 15));
    assert!(!state.has_event(2026, 3, 20));
}

// ========== handle_event Tests ==========

#[test]
fn test_handle_event_navigation_keys() {
    let state = CalendarState::new(2026, 3).with_selected_day(15);

    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Left),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectPrevDay)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char('h'),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectPrevDay)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Right),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectNextDay)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char('l'),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectNextDay)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Up),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectPrevWeek)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char('k'),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectPrevWeek)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Down),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectNextWeek)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char('j'),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::SelectNextWeek)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::PageUp),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::PrevMonth)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::PageDown),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::NextMonth)
    );
}

#[test]
fn test_handle_event_confirm_keys() {
    let state = CalendarState::new(2026, 3).with_selected_day(15);

    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Enter),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::ConfirmSelection)
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char(' '),
            &EventContext::new().focused(true)
        ),
        Some(CalendarMessage::ConfirmSelection)
    );
}

#[test]
fn test_handle_event_unfocused_ignores_events() {
    let state = CalendarState::new(2026, 3).with_selected_day(15);
    assert_eq!(
        Calendar::handle_event(&state, &Event::key(Key::Left), &EventContext::default()),
        None
    );
    assert_eq!(
        Calendar::handle_event(&state, &Event::key(Key::Enter), &EventContext::default()),
        None
    );
    assert_eq!(
        Calendar::handle_event(&state, &Event::key(Key::PageUp), &EventContext::default()),
        None
    );
}

#[test]
fn test_handle_event_disabled_ignores_events() {
    let state = CalendarState::new(2026, 3).with_selected_day(15);
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Left),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::key(Key::Enter),
            &EventContext::new().focused(true).disabled(true)
        ),
        None
    );
}

#[test]
fn test_handle_event_unrecognized_key_returns_none() {
    let state = CalendarState::new(2026, 3);
    assert_eq!(
        Calendar::handle_event(
            &state,
            &Event::char('x'),
            &EventContext::new().focused(true)
        ),
        None
    );
}

// ========== dispatch_event Tests ==========

#[test]
fn test_dispatch_event_navigation() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.selected_day(), Some(16));
    assert_eq!(output, None);

    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::Left),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.selected_day(), Some(15));
    assert_eq!(output, None);
}

#[test]
fn test_dispatch_event_enter_confirms() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::Enter),
        &EventContext::new().focused(true),
    );
    assert_eq!(output, Some(CalendarOutput::DateSelected(2026, 3, 15)));
}

#[test]
fn test_dispatch_event_page_navigation() {
    let mut state = CalendarState::new(2026, 3);
    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::PageUp),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.month(), 2);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 2)));

    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::PageDown),
        &EventContext::new().focused(true),
    );
    assert_eq!(state.month(), 3);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 3)));
}

#[test]
fn test_dispatch_event_unfocused_returns_none() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    let output = Calendar::dispatch_event(
        &mut state,
        &Event::key(Key::Right),
        &EventContext::default(),
    );
    assert_eq!(output, None);
    assert_eq!(state.selected_day(), Some(15));
}

// ========== Instance Method Tests ==========

#[test]
fn test_instance_update() {
    let mut state = CalendarState::new(2026, 3).with_selected_day(15);

    let output = state.update(CalendarMessage::NextMonth);
    assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
}

// ========== Init Test ==========

#[test]
fn test_init() {
    let state = Calendar::init();
    assert_eq!(state.year(), 2026);
    assert_eq!(state.month(), 1);
    assert_eq!(state.selected_day(), None);
}

// ========== Edge Case Tests ==========

#[test]
fn test_navigate_full_year_forward() {
    let mut state = CalendarState::new(2026, 1);
    for expected_month in 2..=12 {
        Calendar::update(&mut state, CalendarMessage::NextMonth);
        assert_eq!(state.month(), expected_month);
    }
    Calendar::update(&mut state, CalendarMessage::NextMonth);
    assert_eq!(state.month(), 1);
    assert_eq!(state.year(), 2027);
}

#[test]
fn test_navigate_full_year_backward() {
    let mut state = CalendarState::new(2026, 12);
    for expected_month in (1..=11).rev() {
        Calendar::update(&mut state, CalendarMessage::PrevMonth);
        assert_eq!(state.month(), expected_month);
    }
    Calendar::update(&mut state, CalendarMessage::PrevMonth);
    assert_eq!(state.month(), 12);
    assert_eq!(state.year(), 2025);
}

#[test]
fn test_leap_year_february_navigation() {
    let mut state = CalendarState::new(2024, 2).with_selected_day(28);
    Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.selected_day(), Some(29));
    assert_eq!(state.month(), 2);

    Calendar::update(&mut state, CalendarMessage::SelectNextDay);
    assert_eq!(state.month(), 3);
    assert_eq!(state.selected_day(), Some(1));
}
