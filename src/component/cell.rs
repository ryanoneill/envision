//! Unified cell type for tabular components.
//!
//! `Cell` is the single cell representation used by `Table` and any
//! tabular component built on `TableRow`. A cell carries display text,
//! optional `CellStyle`, and an optional `SortKey` for typed sorting.
//!
//! Constructors and full usage examples live on the individual types as
//! they're built up across Phase 1 tasks.

use compact_str::CompactString;
use ratatui::style::{Color, Style};

/// A typed sort key carried by a `Cell` for typed comparison.
///
/// **DO NOT REORDER VARIANTS** — discriminant order is part of the
/// cross-variant fallback contract used by [`SortKey::compare`].
///
/// Same-variant comparisons use the natural ordering for the type
/// (with `f64::total_cmp` for `F64`). Cross-variant comparisons
/// (which shouldn't happen in well-formed code) fall back to
/// discriminant order.
#[derive(Clone, Debug, PartialEq)]
pub enum SortKey {
    /// Lexicographic string sort.
    String(CompactString),
    /// Signed-integer sort.
    I64(i64),
    /// Unsigned-integer sort.
    U64(u64),
    /// Float sort using `f64::total_cmp`. NaN sorts after `+∞`
    /// (and `SortKey::None` sorts after NaN). Real values first,
    /// then NaN, then absent.
    F64(f64),
    /// `false` sorts before `true`.
    Bool(bool),
    /// Duration sort.
    Duration(std::time::Duration),
    /// Older sorts before newer.
    DateTime(std::time::SystemTime),
    /// Absent value. Sorts last in ascending, first in descending —
    /// always at the bottom of the visible list ("nulls last").
    None,
}

impl SortKey {
    /// Compares two sort keys per the documented rules.
    ///
    /// Same-variant comparisons use the natural ordering for the type.
    /// `F64` uses `f64::total_cmp` (not `partial_cmp`) — NaN sorts after
    /// `+∞` per IEEE 754 total ordering, so the result is always a
    /// concrete `Ordering` and the caller never sees `Option<Ordering>`.
    /// Cross-variant comparisons (which shouldn't happen in well-formed
    /// code) fall back to discriminant order and emit a `tracing::warn!`
    /// once per `(render_pass, column_index)` (deduped at the call site,
    /// not here).
    pub fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
        use SortKey::*;
        use std::cmp::Ordering;

        // None policy: any non-None always sorts before None in ascending.
        match (a, b) {
            (None, None) => return Ordering::Equal,
            (None, _) => return Ordering::Greater,
            (_, None) => return Ordering::Less,
            _ => {}
        }

        // Same-variant fast paths.
        match (a, b) {
            (String(x), String(y)) => return x.cmp(y),
            (I64(x), I64(y)) => return x.cmp(y),
            (U64(x), U64(y)) => return x.cmp(y),
            (F64(x), F64(y)) => return x.total_cmp(y),
            (Bool(x), Bool(y)) => return x.cmp(y),
            (Duration(x), Duration(y)) => return x.cmp(y),
            (DateTime(x), DateTime(y)) => return x.cmp(y),
            _ => {}
        }

        // Cross-variant: fall back to discriminant order.
        Self::discriminant(a).cmp(&Self::discriminant(b))
    }

    /// Returns the discriminant byte used as cross-variant fallback ordering.
    ///
    /// **Numeric values MUST mirror the enum's variant declaration order.**
    /// If you add a `SortKey` variant, insert it at the matching position
    /// here (renumbering subsequent values). The exhaustive match catches a
    /// missing arm at compile time, but it cannot detect *misordered* arms —
    /// a drift between this assignment and the enum order silently produces
    /// wrong cross-variant sort results.
    fn discriminant(k: &Self) -> u8 {
        use SortKey::*;
        match k {
            String(_) => 0,
            I64(_) => 1,
            U64(_) => 2,
            F64(_) => 3,
            Bool(_) => 4,
            Duration(_) => 5,
            DateTime(_) => 6,
            None => 7,
        }
    }
}

/// Semantic cell styling.
///
/// `Default` renders with no override (theme-driven). `Success`,
/// `Warning`, `Error`, `Muted` map to the theme's semantic colors.
/// `Custom(Style)` applies a raw `ratatui::style::Style` directly.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum CellStyle {
    /// No override — render with the theme's default cell style.
    #[default]
    Default,
    /// Maps to the theme's success color (typically green).
    Success,
    /// Maps to the theme's warning color (typically yellow).
    Warning,
    /// Maps to the theme's error color (typically red).
    Error,
    /// Maps to the theme's muted color (typically dark gray) for de-emphasized text.
    Muted,
    /// Applies a raw `ratatui::style::Style` directly, bypassing theme mapping.
    Custom(Style),
}

/// Optional row-level status indicator. Renders as a colored symbol
/// in a status column prepended to the table.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum RowStatus {
    /// No status column rendered for this row.
    #[default]
    None,
    /// Green dot (●) — healthy / running / passing.
    Healthy,
    /// Yellow triangle (▲) — warning / degraded.
    Warning,
    /// Red cross (✖) — error / failed.
    Error,
    /// Gray question mark (?) — unknown / pending.
    Unknown,
    /// Caller-supplied symbol and color.
    Custom {
        /// The character (or short string) to display.
        symbol: &'static str,
        /// The color applied to the symbol.
        color: Color,
    },
}

impl RowStatus {
    /// Returns `Some((symbol, color))` for non-None variants.
    pub fn indicator(&self) -> Option<(&'static str, Color)> {
        match self {
            RowStatus::None => None,
            RowStatus::Healthy => Some(("●", Color::Green)),
            RowStatus::Warning => Some(("▲", Color::Yellow)),
            RowStatus::Error => Some(("✖", Color::Red)),
            RowStatus::Unknown => Some(("?", Color::DarkGray)),
            RowStatus::Custom { symbol, color } => Some((symbol, *color)),
        }
    }
}

/// Unified cell type for tabular components.
///
/// Carries display text, optional `CellStyle`, and an optional `SortKey`
/// for typed sorting.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cell {
    text: CompactString,
    style: CellStyle,
    sort_key: Option<SortKey>,
}

impl Cell {
    /// Plain cell — default style, no typed sort key. Sort falls back to
    /// lexicographic on the cell text.
    pub fn new(text: impl Into<CompactString>) -> Self {
        Self {
            text: text.into(),
            style: CellStyle::Default,
            sort_key: Option::None,
        }
    }

    /// Builder: set the cell style.
    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }

    /// Builder: set the typed sort key.
    pub fn with_sort_key(mut self, key: SortKey) -> Self {
        self.sort_key = Some(key);
        self
    }

    /// Builder: replace the display text without changing other fields.
    /// Useful in the mixed-precision pattern:
    ///
    /// ```
    /// use envision::component::cell::Cell;
    /// let cell = Cell::number(840.16).with_text(format!("{:.2}", 840.16));
    /// assert_eq!(cell.text(), "840.16");
    /// ```
    pub fn with_text(mut self, text: impl Into<CompactString>) -> Self {
        self.text = text.into();
        self
    }

    /// Display text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Cell style.
    pub fn style(&self) -> &CellStyle {
        &self.style
    }

    /// Optional typed sort key.
    pub fn sort_key(&self) -> Option<&SortKey> {
        self.sort_key.as_ref()
    }

    /// Numeric cell with `f64` sort key. Display text is `format!("{}", value)`.
    ///
    /// For fixed-precision columns, chain `.with_text(format!(...))`:
    ///
    /// ```
    /// use envision::component::cell::Cell;
    /// let c = Cell::number(840.16).with_text(format!("{:.2} ms", 840.16));
    /// assert_eq!(c.text(), "840.16 ms");
    /// ```
    pub fn number(value: f64) -> Self {
        Self::new(format!("{}", value)).with_sort_key(SortKey::F64(value))
    }

    /// Signed-integer cell. Use only when the column's data is integer-valued —
    /// for fractional or naturally-`f64` data, use [`Cell::number`] to preserve
    /// precision in the sort key.
    ///
    /// # Example
    ///
    /// ```
    /// use envision::component::cell::Cell;
    /// let c = Cell::int(-42);
    /// assert_eq!(c.text(), "-42");
    /// ```
    pub fn int(value: i64) -> Self {
        Self::new(format!("{}", value)).with_sort_key(SortKey::I64(value))
    }

    /// Unsigned-integer cell. Use only for non-negative integer data —
    /// for naturally-`f64` data, use [`Cell::number`].
    pub fn uint(value: u64) -> Self {
        Self::new(format!("{}", value)).with_sort_key(SortKey::U64(value))
    }

    /// Boolean cell. Renders as `"true"` / `"false"`.
    pub fn bool(value: bool) -> Self {
        Self::new(format!("{}", value)).with_sort_key(SortKey::Bool(value))
    }

    /// Duration cell. Display text is a compact form (`"3m12s"`, `"2h15m"`,
    /// `"3d4h"`); sort key is the underlying `Duration`.
    pub fn duration(d: std::time::Duration) -> Self {
        Self::new(format_duration(d)).with_sort_key(SortKey::Duration(d))
    }

    /// Datetime cell. Display text is an ISO-ish string; sort key is the
    /// underlying `SystemTime`.
    pub fn datetime(t: std::time::SystemTime) -> Self {
        Self::new(format_systemtime(t)).with_sort_key(SortKey::DateTime(t))
    }

    /// Success-styled cell (theme green).
    pub fn success(text: impl Into<CompactString>) -> Self {
        Self::new(text).with_style(CellStyle::Success)
    }

    /// Warning-styled cell (theme yellow).
    pub fn warning(text: impl Into<CompactString>) -> Self {
        Self::new(text).with_style(CellStyle::Warning)
    }

    /// Error-styled cell (theme red).
    pub fn error(text: impl Into<CompactString>) -> Self {
        Self::new(text).with_style(CellStyle::Error)
    }

    /// Muted-styled cell (theme dark gray).
    pub fn muted(text: impl Into<CompactString>) -> Self {
        Self::new(text).with_style(CellStyle::Muted)
    }
}

impl From<&str> for Cell {
    fn from(s: &str) -> Self {
        Cell::new(s)
    }
}

impl From<String> for Cell {
    fn from(s: String) -> Self {
        Cell::new(s)
    }
}

impl From<CompactString> for Cell {
    fn from(s: CompactString) -> Self {
        Cell::new(s)
    }
}

fn format_duration(d: std::time::Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else if secs < 86_400 {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d{}h", secs / 86_400, (secs % 86_400) / 3600)
    }
}

fn format_systemtime(t: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    match t.duration_since(UNIX_EPOCH) {
        Ok(d) => format!("{}s", d.as_secs()),
        Err(_) => "before-epoch".to_string(),
    }
}

#[cfg(test)]
mod sort_key_tests {
    use super::*;
    use std::cmp::Ordering;
    use std::time::{Duration, SystemTime};

    #[test]
    fn string_compare_lexicographic() {
        let a = SortKey::String("alice".into());
        let b = SortKey::String("bob".into());
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn i64_compare_numeric() {
        let a = SortKey::I64(7);
        let b = SortKey::I64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn u64_compare_numeric() {
        let a = SortKey::U64(7);
        let b = SortKey::U64(11);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_compare_total_order() {
        // total_cmp: 3.5 < 7.0
        let a = SortKey::F64(3.5);
        let b = SortKey::F64(7.0);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn f64_nan_sorts_after_positive_infinity() {
        let inf = SortKey::F64(f64::INFINITY);
        let nan = SortKey::F64(f64::NAN);
        // total_cmp: +∞ < NaN
        assert_eq!(SortKey::compare(&inf, &nan), Ordering::Less);
    }

    #[test]
    fn bool_false_lt_true() {
        let f = SortKey::Bool(false);
        let t = SortKey::Bool(true);
        assert_eq!(SortKey::compare(&f, &t), Ordering::Less);
    }

    #[test]
    fn duration_compare() {
        let a = SortKey::Duration(Duration::from_secs(1));
        let b = SortKey::Duration(Duration::from_secs(2));
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn datetime_older_lt_newer() {
        let older = SystemTime::UNIX_EPOCH;
        let newer = older + Duration::from_secs(1);
        assert_eq!(
            SortKey::compare(&SortKey::DateTime(older), &SortKey::DateTime(newer)),
            Ordering::Less,
        );
    }

    #[test]
    fn none_eq_none() {
        assert_eq!(
            SortKey::compare(&SortKey::None, &SortKey::None),
            Ordering::Equal
        );
    }

    #[test]
    fn cross_variant_falls_back_to_discriminant_order() {
        // I64 < F64 in the enum; in ascending, I64(7) appears before F64(3.5)
        // even though numerically 3.5 < 7.
        let a = SortKey::I64(7);
        let b = SortKey::F64(3.5);
        assert_eq!(SortKey::compare(&a, &b), Ordering::Less);
    }

    #[test]
    fn none_sorts_last_in_ascending_against_real_value() {
        // None > Some(real) — None sorts last in ascending order.
        let real = SortKey::F64(0.0);
        assert_eq!(SortKey::compare(&real, &SortKey::None), Ordering::Less);
        assert_eq!(SortKey::compare(&SortKey::None, &real), Ordering::Greater);
    }

    #[test]
    fn none_sorts_after_nan() {
        let nan = SortKey::F64(f64::NAN);
        assert_eq!(SortKey::compare(&nan, &SortKey::None), Ordering::Less);
    }
}

#[cfg(test)]
mod cell_style_tests {
    use super::*;
    use ratatui::style::{Color, Style};

    #[test]
    fn default_is_default_variant() {
        assert_eq!(CellStyle::default(), CellStyle::Default);
    }

    #[test]
    fn custom_carries_style() {
        let style = Style::default().fg(Color::Red);
        assert_eq!(CellStyle::Custom(style), CellStyle::Custom(style));
    }
}

#[cfg(test)]
mod row_status_tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn none_has_no_indicator() {
        assert_eq!(RowStatus::None.indicator(), None);
    }

    #[test]
    fn healthy_indicator_green_dot() {
        assert_eq!(RowStatus::Healthy.indicator(), Some(("●", Color::Green)));
    }

    #[test]
    fn warning_indicator_yellow_triangle() {
        assert_eq!(RowStatus::Warning.indicator(), Some(("▲", Color::Yellow)));
    }

    #[test]
    fn error_indicator_red_cross() {
        assert_eq!(RowStatus::Error.indicator(), Some(("✖", Color::Red)));
    }

    #[test]
    fn unknown_indicator_gray_question() {
        assert_eq!(RowStatus::Unknown.indicator(), Some(("?", Color::DarkGray)));
    }

    #[test]
    fn custom_indicator_passes_through() {
        let custom = RowStatus::Custom {
            symbol: "★",
            color: Color::Magenta,
        };
        assert_eq!(custom.indicator(), Some(("★", Color::Magenta)));
    }
}

#[cfg(test)]
mod cell_struct_tests {
    use super::*;

    #[test]
    fn new_default_style_no_sort_key() {
        let c = Cell::new("alice");
        assert_eq!(c.text(), "alice");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }

    #[test]
    fn with_style_round_trips() {
        let c = Cell::new("ok").with_style(CellStyle::Success);
        assert_eq!(*c.style(), CellStyle::Success);
    }

    #[test]
    fn with_sort_key_round_trips() {
        let c = Cell::new("7").with_sort_key(SortKey::I64(7));
        assert_eq!(c.sort_key(), Some(&SortKey::I64(7)));
    }

    #[test]
    fn with_text_replaces_text_only() {
        let c = Cell::new("v1")
            .with_style(CellStyle::Warning)
            .with_sort_key(SortKey::I64(1))
            .with_text("v2");
        assert_eq!(c.text(), "v2");
        assert_eq!(*c.style(), CellStyle::Warning);
        assert_eq!(c.sort_key(), Some(&SortKey::I64(1)));
    }

    #[test]
    fn default_cell_is_empty() {
        let c = Cell::default();
        assert_eq!(c.text(), "");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }
}

#[cfg(test)]
mod cell_typed_constructor_tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn number_uses_display_fmt_and_f64_sort_key() {
        let c = Cell::number(840.0);
        assert_eq!(c.text(), "840");
        assert_eq!(c.sort_key(), Some(&SortKey::F64(840.0)));
    }

    #[test]
    fn number_with_text_overrides_display_for_mixed_precision() {
        let c = Cell::number(840.16).with_text(format!("{:.2}", 840.16));
        assert_eq!(c.text(), "840.16");
        assert_eq!(c.sort_key(), Some(&SortKey::F64(840.16)));
    }

    #[test]
    fn int_renders_and_sorts_as_i64() {
        let c = Cell::int(-42);
        assert_eq!(c.text(), "-42");
        assert_eq!(c.sort_key(), Some(&SortKey::I64(-42)));
    }

    #[test]
    fn uint_renders_and_sorts_as_u64() {
        let c = Cell::uint(7);
        assert_eq!(c.text(), "7");
        assert_eq!(c.sort_key(), Some(&SortKey::U64(7)));
    }

    #[test]
    fn bool_renders_and_sorts() {
        let t = Cell::bool(true);
        assert_eq!(t.text(), "true");
        assert_eq!(t.sort_key(), Some(&SortKey::Bool(true)));
    }

    #[test]
    fn duration_renders_compact_and_sorts() {
        let c = Cell::duration(Duration::from_secs(192));
        assert_eq!(
            c.sort_key(),
            Some(&SortKey::Duration(Duration::from_secs(192)))
        );
        assert!(c.text().contains("3m"));
        assert!(c.text().contains("12s"));
    }

    #[test]
    fn datetime_carries_systemtime_sort_key() {
        let t = SystemTime::UNIX_EPOCH;
        let c = Cell::datetime(t);
        assert_eq!(c.sort_key(), Some(&SortKey::DateTime(t)));
        assert!(!c.text().is_empty());
    }
}

#[cfg(test)]
mod cell_style_constructor_tests {
    use super::*;

    #[test]
    fn success_sets_style() {
        let c = Cell::success("running");
        assert_eq!(c.text(), "running");
        assert_eq!(*c.style(), CellStyle::Success);
    }

    #[test]
    fn warning_sets_style() {
        assert_eq!(*Cell::warning("retry").style(), CellStyle::Warning);
    }

    #[test]
    fn error_sets_style() {
        assert_eq!(*Cell::error("crash").style(), CellStyle::Error);
    }

    #[test]
    fn muted_sets_style() {
        assert_eq!(*Cell::muted("idle").style(), CellStyle::Muted);
    }
}

#[cfg(test)]
mod cell_from_impls_tests {
    use super::*;
    use compact_str::CompactString;

    #[test]
    fn from_str_round_trips() {
        let c: Cell = "alice".into();
        assert_eq!(c.text(), "alice");
        assert_eq!(*c.style(), CellStyle::Default);
        assert_eq!(c.sort_key(), None);
    }

    #[test]
    fn from_string_round_trips() {
        let c: Cell = String::from("bob").into();
        assert_eq!(c.text(), "bob");
    }

    #[test]
    fn from_compact_string_round_trips() {
        let c: Cell = CompactString::from("carol").into();
        assert_eq!(c.text(), "carol");
    }
}
