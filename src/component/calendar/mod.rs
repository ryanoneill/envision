//! A month-view calendar component with date selection and event markers.
//!
//! [`Calendar`] provides a navigable month view that displays a grid of days
//! with keyboard navigation, date selection, and colored event markers.
//! State is stored in [`CalendarState`], updated via [`CalendarMessage`],
//! and produces [`CalendarOutput`].
//!
//! Implements [`Focusable`] and [`Disableable`].
//!
//! # Example
//!
//! ```rust
//! use envision::component::{Calendar, CalendarMessage, CalendarOutput, CalendarState, Component, Focusable};
//! use ratatui::style::Color;
//!
//! // Create a calendar for March 2026
//! let mut state = CalendarState::new(2026, 3)
//!     .with_selected_day(20)
//!     .with_title("My Calendar");
//! Calendar::focus(&mut state);
//!
//! assert_eq!(state.year(), 2026);
//! assert_eq!(state.month(), 3);
//! assert_eq!(state.selected_day(), Some(20));
//! assert_eq!(state.month_name(), "March");
//!
//! // Navigate to next month
//! let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
//! assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
//! assert_eq!(state.month(), 4);
//!
//! // Add an event marker
//! Calendar::update(&mut state, CalendarMessage::AddEvent {
//!     year: 2026,
//!     month: 4,
//!     day: 15,
//!     color: Color::Green,
//! });
//! assert!(state.has_event(2026, 4, 15));
//! ```

use std::collections::HashMap;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Component, Disableable, Focusable, ViewContext};
use crate::input::{Event, KeyCode};
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Date math helpers (private)
// ---------------------------------------------------------------------------

/// Returns whether `year` is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Returns the number of days in the given month (1-12) of `year`.
///
/// # Panics
///
/// Panics if `month` is not in the range 1..=12.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 => 31,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => panic!("Invalid month: {month}"),
    }
}

/// Returns the day of week for the given date, 0 = Sunday .. 6 = Saturday.
///
/// Uses Tomohiko Sakamoto's algorithm.
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    let result = (y + y / 4 - y / 100 + y / 400 + T[(month - 1) as usize] + day as i32) % 7;
    ((result + 7) % 7) as u32
}

/// Returns the month name for the given month (1-12).
fn month_name_for(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

// ---------------------------------------------------------------------------
// CalendarMessage
// ---------------------------------------------------------------------------

/// Messages that can be sent to a Calendar.
#[derive(Clone, Debug, PartialEq)]
pub enum CalendarMessage {
    /// Advance to the next month.
    NextMonth,
    /// Go back to the previous month.
    PrevMonth,
    /// Advance to the next year.
    NextYear,
    /// Go back to the previous year.
    PrevYear,
    /// Select a specific day in the current month.
    SelectDay(u32),
    /// Move selection to the previous day (wraps across months).
    SelectPrevDay,
    /// Move selection to the next day (wraps across months).
    SelectNextDay,
    /// Move selection up one week (wraps across months).
    SelectPrevWeek,
    /// Move selection down one week (wraps across months).
    SelectNextWeek,
    /// Confirm the current selection (emits DateSelected).
    ConfirmSelection,
    /// Navigate to today's month/year (requires explicit date).
    Today {
        /// The current year.
        year: i32,
        /// The current month (1-12).
        month: u32,
        /// The current day.
        day: u32,
    },
    /// Navigate to a specific month.
    SetDate {
        /// Target year.
        year: i32,
        /// Target month (1-12).
        month: u32,
    },
    /// Add an event marker for a specific date.
    AddEvent {
        /// Event year.
        year: i32,
        /// Event month (1-12).
        month: u32,
        /// Event day.
        day: u32,
        /// Marker color.
        color: Color,
    },
    /// Remove all event markers.
    ClearEvents,
}

// ---------------------------------------------------------------------------
// CalendarOutput
// ---------------------------------------------------------------------------

/// Output messages from a Calendar.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CalendarOutput {
    /// A date was confirmed (Enter/Space pressed on selected day).
    DateSelected(i32, u32, u32),
    /// The displayed month changed via navigation.
    MonthChanged(i32, u32),
}

// ---------------------------------------------------------------------------
// CalendarState
// ---------------------------------------------------------------------------

/// State for a Calendar component.
///
/// # Example
///
/// ```rust
/// use envision::component::CalendarState;
///
/// let state = CalendarState::new(2026, 3)
///     .with_selected_day(15)
///     .with_title("Events");
/// assert_eq!(state.year(), 2026);
/// assert_eq!(state.month(), 3);
/// assert_eq!(state.selected_day(), Some(15));
/// assert_eq!(state.month_name(), "March");
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct CalendarState {
    year: i32,
    month: u32,
    selected_day: Option<u32>,
    events: HashMap<(i32, u32, u32), Color>,
    title: Option<String>,
    focused: bool,
    disabled: bool,
}

impl CalendarState {
    /// Creates a new calendar for the given year and month.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3);
    /// assert_eq!(state.year(), 2026);
    /// assert_eq!(state.month(), 3);
    /// assert_eq!(state.selected_day(), None);
    /// ```
    pub fn new(year: i32, month: u32) -> Self {
        Self {
            year,
            month,
            selected_day: None,
            events: HashMap::new(),
            title: None,
            focused: false,
            disabled: false,
        }
    }

    /// Sets the initially selected day (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3).with_selected_day(15);
    /// assert_eq!(state.selected_day(), Some(15));
    /// ```
    pub fn with_selected_day(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self
    }

    /// Sets the title (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3).with_title("My Calendar");
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds an event marker (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    /// use ratatui::style::Color;
    ///
    /// let state = CalendarState::new(2026, 3)
    ///     .with_event(2026, 3, 15, Color::Green);
    /// assert!(state.has_event(2026, 3, 15));
    /// ```
    pub fn with_event(mut self, year: i32, month: u32, day: u32, color: Color) -> Self {
        self.events.insert((year, month, day), color);
        self
    }

    /// Sets the disabled state (builder method).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Returns the current year.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3);
    /// assert_eq!(state.year(), 2026);
    /// ```
    pub fn year(&self) -> i32 {
        self.year
    }

    /// Returns the current month (1-12).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3);
    /// assert_eq!(state.month(), 3);
    /// ```
    pub fn month(&self) -> u32 {
        self.month
    }

    /// Returns the currently selected day, if any.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3).with_selected_day(20);
    /// assert_eq!(state.selected_day(), Some(20));
    /// ```
    pub fn selected_day(&self) -> Option<u32> {
        self.selected_day
    }

    /// Returns the title, if set.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3).with_title("My Calendar");
    /// assert_eq!(state.title(), Some("My Calendar"));
    ///
    /// let state2 = CalendarState::new(2026, 3);
    /// assert_eq!(state2.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.set_title("Events");
    /// assert_eq!(state.title(), Some("Events"));
    /// ```
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = Some(title.into());
    }

    /// Sets the selected day.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.set_selected_day(Some(15));
    /// assert_eq!(state.selected_day(), Some(15));
    /// state.set_selected_day(None);
    /// assert_eq!(state.selected_day(), None);
    /// ```
    pub fn set_selected_day(&mut self, day: Option<u32>) {
        self.selected_day = day;
    }

    /// Returns the name of the current month.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 1);
    /// assert_eq!(state.month_name(), "January");
    /// ```
    pub fn month_name(&self) -> &str {
        month_name_for(self.month)
    }

    /// Adds an event marker for the given date with the given color.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.add_event(2026, 3, 15, Color::Red);
    /// assert!(state.has_event(2026, 3, 15));
    /// ```
    pub fn add_event(&mut self, year: i32, month: u32, day: u32, color: Color) {
        self.events.insert((year, month, day), color);
    }

    /// Removes all event markers.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    /// use ratatui::style::Color;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.add_event(2026, 3, 15, Color::Red);
    /// state.clear_events();
    /// assert!(!state.has_event(2026, 3, 15));
    /// ```
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Returns whether there is an event marker for the given date.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    /// use ratatui::style::Color;
    ///
    /// let state = CalendarState::new(2026, 3)
    ///     .with_event(2026, 3, 15, Color::Green);
    /// assert!(state.has_event(2026, 3, 15));
    /// assert!(!state.has_event(2026, 3, 16));
    /// ```
    pub fn has_event(&self, year: i32, month: u32, day: u32) -> bool {
        self.events.contains_key(&(year, month, day))
    }

    /// Returns whether the component is focused.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3);
    /// assert!(!state.is_focused());
    /// ```
    pub fn is_focused(&self) -> bool {
        self.focused
    }

    /// Sets the focus state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.set_focused(true);
    /// assert!(state.is_focused());
    /// ```
    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    /// Returns whether the component is disabled.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let state = CalendarState::new(2026, 3);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Sets the disabled state.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::CalendarState;
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// state.set_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Maps an input event to a calendar message.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Calendar, CalendarMessage, CalendarState, Component, Focusable};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    /// Calendar::focus(&mut state);
    ///
    /// let msg = state.handle_event(&Event::key(KeyCode::Right));
    /// assert_eq!(msg, Some(CalendarMessage::SelectNextDay));
    /// ```
    pub fn handle_event(&self, event: &Event) -> Option<CalendarMessage> {
        Calendar::handle_event(self, event)
    }

    /// Dispatches an event, updating state and returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{Calendar, CalendarOutput, CalendarState, Focusable};
    /// use envision::input::{Event, KeyCode};
    ///
    /// let mut state = CalendarState::new(2026, 3).with_selected_day(15);
    /// Calendar::focus(&mut state);
    ///
    /// let output = state.dispatch_event(&Event::key(KeyCode::Enter));
    /// assert_eq!(output, Some(CalendarOutput::DateSelected(2026, 3, 15)));
    /// ```
    pub fn dispatch_event(&mut self, event: &Event) -> Option<CalendarOutput> {
        Calendar::dispatch_event(self, event)
    }

    /// Updates the calendar state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{CalendarMessage, CalendarOutput, CalendarState};
    ///
    /// let mut state = CalendarState::new(2026, 3);
    /// let output = state.update(CalendarMessage::NextMonth);
    /// assert_eq!(output, Some(CalendarOutput::MonthChanged(2026, 4)));
    /// ```
    pub fn update(&mut self, msg: CalendarMessage) -> Option<CalendarOutput> {
        Calendar::update(self, msg)
    }
}

// ---------------------------------------------------------------------------
// Calendar component
// ---------------------------------------------------------------------------

/// A month-view calendar component with date selection and event markers.
///
/// The calendar renders a standard month grid with day-of-week headers,
/// supports keyboard navigation between days and months, and can display
/// colored event markers on specific dates.
///
/// # Keyboard Navigation
///
/// When focused:
/// - Left / h: previous day (wraps to previous month)
/// - Right / l: next day (wraps to next month)
/// - Up / k: same day minus 7 (previous week)
/// - Down / j: same day plus 7 (next week)
/// - PageUp: previous month
/// - PageDown: next month
/// - Enter / Space: confirm selection (emits `DateSelected`)
///
/// # Visual Layout
///
/// ```text
/// ┌─ March 2026 ─────────────────┐
/// │ Su  Mo  Tu  We  Th  Fr  Sa   │
/// │  1   2   3   4   5   6   7   │
/// │  8   9  10  11  12  13  14   │
/// │ 15  16  17  18  19 [20] 21   │
/// │ 22  23  24• 25  26  27  28   │
/// │ 29  30  31                   │
/// │ ◀ PgUp          PgDn ▶      │
/// └──────────────────────────────┘
/// ```
///
/// # Example
///
/// ```rust
/// use envision::component::{Calendar, CalendarMessage, CalendarState, Component, Focusable};
///
/// let mut state = CalendarState::new(2026, 3).with_selected_day(1);
/// Calendar::focus(&mut state);
///
/// // Navigate forward
/// let output = Calendar::update(&mut state, CalendarMessage::NextMonth);
/// assert_eq!(state.month(), 4);
/// ```
pub struct Calendar;

impl Calendar {
    /// Navigates to the previous month, adjusting year if needed.
    fn go_prev_month(state: &mut CalendarState) {
        if state.month == 1 {
            state.month = 12;
            state.year -= 1;
        } else {
            state.month -= 1;
        }
        // Clamp selected day to new month
        if let Some(day) = state.selected_day {
            let max_day = days_in_month(state.year, state.month);
            if day > max_day {
                state.selected_day = Some(max_day);
            }
        }
    }

    /// Navigates to the next month, adjusting year if needed.
    fn go_next_month(state: &mut CalendarState) {
        if state.month == 12 {
            state.month = 1;
            state.year += 1;
        } else {
            state.month += 1;
        }
        // Clamp selected day to new month
        if let Some(day) = state.selected_day {
            let max_day = days_in_month(state.year, state.month);
            if day > max_day {
                state.selected_day = Some(max_day);
            }
        }
    }
}

impl Component for Calendar {
    type State = CalendarState;
    type Message = CalendarMessage;
    type Output = CalendarOutput;

    fn init() -> Self::State {
        CalendarState::new(2026, 1)
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        if state.disabled {
            return None;
        }

        match msg {
            CalendarMessage::NextMonth => {
                Self::go_next_month(state);
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::PrevMonth => {
                Self::go_prev_month(state);
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::NextYear => {
                state.year += 1;
                // Clamp selected day (e.g. Feb 29 -> Feb 28 on non-leap year)
                if let Some(day) = state.selected_day {
                    let max_day = days_in_month(state.year, state.month);
                    if day > max_day {
                        state.selected_day = Some(max_day);
                    }
                }
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::PrevYear => {
                state.year -= 1;
                if let Some(day) = state.selected_day {
                    let max_day = days_in_month(state.year, state.month);
                    if day > max_day {
                        state.selected_day = Some(max_day);
                    }
                }
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::SelectDay(day) => {
                let max_day = days_in_month(state.year, state.month);
                let clamped = day.min(max_day).max(1);
                state.selected_day = Some(clamped);
                None
            }
            CalendarMessage::SelectPrevDay => {
                let current_day = state.selected_day.unwrap_or(1);
                if current_day <= 1 {
                    Self::go_prev_month(state);
                    let last_day = days_in_month(state.year, state.month);
                    state.selected_day = Some(last_day);
                    Some(CalendarOutput::MonthChanged(state.year, state.month))
                } else {
                    state.selected_day = Some(current_day - 1);
                    None
                }
            }
            CalendarMessage::SelectNextDay => {
                let current_day = state.selected_day.unwrap_or(1);
                let max_day = days_in_month(state.year, state.month);
                if current_day >= max_day {
                    Self::go_next_month(state);
                    state.selected_day = Some(1);
                    Some(CalendarOutput::MonthChanged(state.year, state.month))
                } else {
                    state.selected_day = Some(current_day + 1);
                    None
                }
            }
            CalendarMessage::SelectPrevWeek => {
                let current_day = state.selected_day.unwrap_or(1);
                if current_day <= 7 {
                    let days_back = 7 - current_day;
                    Self::go_prev_month(state);
                    let prev_max = days_in_month(state.year, state.month);
                    state.selected_day = Some(prev_max - days_back);
                    Some(CalendarOutput::MonthChanged(state.year, state.month))
                } else {
                    state.selected_day = Some(current_day - 7);
                    None
                }
            }
            CalendarMessage::SelectNextWeek => {
                let current_day = state.selected_day.unwrap_or(1);
                let max_day = days_in_month(state.year, state.month);
                if current_day + 7 > max_day {
                    let overflow = current_day + 7 - max_day;
                    Self::go_next_month(state);
                    state.selected_day = Some(overflow);
                    Some(CalendarOutput::MonthChanged(state.year, state.month))
                } else {
                    state.selected_day = Some(current_day + 7);
                    None
                }
            }
            CalendarMessage::ConfirmSelection => {
                if let Some(day) = state.selected_day {
                    Some(CalendarOutput::DateSelected(state.year, state.month, day))
                } else {
                    None
                }
            }
            CalendarMessage::Today { year, month, day } => {
                state.year = year;
                state.month = month;
                let max_day = days_in_month(year, month);
                state.selected_day = Some(day.min(max_day).max(1));
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::SetDate { year, month } => {
                state.year = year;
                state.month = month;
                if let Some(day) = state.selected_day {
                    let max_day = days_in_month(year, month);
                    if day > max_day {
                        state.selected_day = Some(max_day);
                    }
                }
                Some(CalendarOutput::MonthChanged(state.year, state.month))
            }
            CalendarMessage::AddEvent {
                year,
                month,
                day,
                color,
            } => {
                state.events.insert((year, month, day), color);
                None
            }
            CalendarMessage::ClearEvents => {
                state.events.clear();
                None
            }
        }
    }

    fn handle_event(state: &Self::State, event: &Event) -> Option<Self::Message> {
        if !state.focused || state.disabled {
            return None;
        }

        if let Some(key) = event.as_key() {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => Some(CalendarMessage::SelectPrevDay),
                KeyCode::Right | KeyCode::Char('l') => Some(CalendarMessage::SelectNextDay),
                KeyCode::Up | KeyCode::Char('k') => Some(CalendarMessage::SelectPrevWeek),
                KeyCode::Down | KeyCode::Char('j') => Some(CalendarMessage::SelectNextWeek),
                KeyCode::PageUp => Some(CalendarMessage::PrevMonth),
                KeyCode::PageDown => Some(CalendarMessage::NextMonth),
                KeyCode::Enter | KeyCode::Char(' ') => Some(CalendarMessage::ConfirmSelection),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(state: &Self::State, frame: &mut Frame, area: Rect, theme: &Theme, _ctx: &ViewContext) {
        if area.height == 0 || area.width == 0 {
            return;
        }

        crate::annotation::with_registry(|reg| {
            reg.register(
                area,
                crate::annotation::Annotation::new(crate::annotation::WidgetType::Custom(
                    "Calendar".to_string(),
                ))
                .with_id("calendar")
                .with_focus(state.focused)
                .with_disabled(state.disabled),
            );
        });

        // Determine styles
        let border_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_border_style()
        } else {
            theme.border_style()
        };

        let normal_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.normal_style()
        };

        let header_style = if state.disabled {
            theme.disabled_style()
        } else if state.focused {
            theme.focused_bold_style()
        } else {
            Style::default()
                .fg(theme.foreground)
                .add_modifier(Modifier::BOLD)
        };

        let day_header_style = if state.disabled {
            theme.disabled_style()
        } else {
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD)
        };

        // Build the title
        let title_text = if let Some(ref title) = state.title {
            format!("{} - {} {}", title, state.month_name(), state.year)
        } else {
            format!("{} {}", state.month_name(), state.year)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(Span::styled(format!(" {title_text} "), header_style));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Day-of-week headers
        let dow_line = Line::from(vec![Span::styled(
            " Su  Mo  Tu  We  Th  Fr  Sa",
            day_header_style,
        )]);

        let mut lines: Vec<Line<'_>> = Vec::new();
        lines.push(dow_line);

        // Compute the calendar grid
        let first_dow = day_of_week(state.year, state.month, 1);
        let total_days = days_in_month(state.year, state.month);

        let selected_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.selected_highlight_style(state.focused)
        };

        // Build week rows
        let mut day = 1u32;

        for week in 0..6 {
            if day > total_days {
                break;
            }

            let mut spans: Vec<Span<'_>> = Vec::new();

            for dow in 0..7u32 {
                if week == 0 && dow < first_dow {
                    // Empty cell before month starts
                    spans.push(Span::styled("    ", normal_style));
                } else if day > total_days {
                    // Empty cell after month ends
                    spans.push(Span::styled("    ", normal_style));
                } else {
                    let is_selected = state.selected_day == Some(day);
                    let has_event = state.events.contains_key(&(state.year, state.month, day));
                    let event_color = state.events.get(&(state.year, state.month, day));

                    let day_str = if has_event {
                        format!("{day:>3}\u{2022}")
                    } else {
                        format!("{day:>3} ")
                    };

                    let style = if is_selected {
                        selected_style
                    } else if let Some(&color) = event_color {
                        Style::default().fg(color)
                    } else {
                        normal_style
                    };

                    spans.push(Span::styled(day_str, style));
                    day += 1;
                }
            }

            lines.push(Line::from(spans));
        }

        // Navigation hint footer
        let footer_style = if state.disabled {
            theme.disabled_style()
        } else {
            theme.placeholder_style()
        };
        lines.push(Line::from(vec![Span::styled(
            " \u{25c0} PgUp          PgDn \u{25b6}",
            footer_style,
        )]));

        let paragraph = Paragraph::new(lines).style(normal_style);
        frame.render_widget(paragraph, inner);
    }
}

impl Focusable for Calendar {
    fn is_focused(state: &Self::State) -> bool {
        state.focused
    }

    fn set_focused(state: &mut Self::State, focused: bool) {
        state.focused = focused;
    }
}

impl Disableable for Calendar {
    fn is_disabled(state: &Self::State) -> bool {
        state.disabled
    }

    fn set_disabled(state: &mut Self::State, disabled: bool) {
        state.disabled = disabled;
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;
