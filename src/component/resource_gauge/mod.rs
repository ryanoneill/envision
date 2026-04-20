//! A triple-value gauge for displaying actual/request/limit measurements.
//!
//! `ResourceGauge` renders a single horizontal bar with three regions showing
//! how actual usage relates to a requested amount and an upper limit. This
//! is useful for CPU/memory monitoring, quota tracking, or any scenario
//! where a value has both a "normal operating range" (request) and a
//! "hard ceiling" (limit).
//!
//! # Visual
//!
//! ```text
//! CPU  [████████▓▓▓▓░░░░░░░░] 350m / 500m / 1000m
//!       actual   ↑request      ↑limit
//! ```
//!
//! - Filled (`█`): actual usage, colored by health (green/yellow/red)
//! - Request zone (`▓`): headroom between actual and request
//! - Empty (`░`): unused capacity up to the limit
//!
//! # Examples
//!
//! ```
//! use envision::component::resource_gauge::ResourceGaugeState;
//!
//! let state = ResourceGaugeState::new(350.0, 500.0, 1000.0)
//!     .with_label("CPU")
//!     .with_units("m");
//!
//! assert_eq!(state.actual(), 350.0);
//! assert_eq!(state.request(), 500.0);
//! assert_eq!(state.limit(), 1000.0);
//! assert!(!state.is_over_request());
//! assert!(!state.is_near_limit());
//! ```

use ratatui::buffer::Buffer;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::component::Component;
use crate::component::context::{EventContext, RenderContext};
use crate::input::Event;
use crate::theme::Theme;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Orientation of the gauge bar.
///
/// # Examples
///
/// ```
/// use envision::component::resource_gauge::GaugeOrientation;
///
/// let o = GaugeOrientation::default();
/// assert_eq!(o, GaugeOrientation::Horizontal);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum GaugeOrientation {
    /// Bar fills left to right.
    #[default]
    Horizontal,
    /// Bar fills bottom to top.
    Vertical,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// State for the ResourceGauge component.
///
/// Holds actual, request, and limit values plus display configuration.
///
/// # Examples
///
/// ```
/// use envision::component::resource_gauge::ResourceGaugeState;
///
/// let state = ResourceGaugeState::new(256.0, 384.0, 512.0)
///     .with_label("Memory")
///     .with_units("Mi");
///
/// assert_eq!(state.label(), Some("Memory"));
/// assert_eq!(state.units(), Some("Mi"));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ResourceGaugeState {
    actual: f64,
    request: f64,
    limit: f64,
    label: Option<String>,
    units: Option<String>,
    title: Option<String>,
    show_legend: bool,
    orientation: GaugeOrientation,
    disabled: bool,
}

impl Default for ResourceGaugeState {
    fn default() -> Self {
        Self {
            actual: 0.0,
            request: 0.0,
            limit: 0.0,
            label: None,
            units: None,
            title: None,
            show_legend: true,
            orientation: GaugeOrientation::default(),
            disabled: false,
        }
    }
}

impl ResourceGaugeState {
    /// Creates a new ResourceGauge with the given values.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(100.0, 200.0, 500.0);
    /// assert_eq!(state.actual(), 100.0);
    /// assert_eq!(state.request(), 200.0);
    /// assert_eq!(state.limit(), 500.0);
    /// ```
    pub fn new(actual: f64, request: f64, limit: f64) -> Self {
        Self {
            actual,
            request,
            limit,
            ..Self::default()
        }
    }

    // -- Builders --

    /// Sets the label displayed before the bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_label("CPU");
    /// assert_eq!(state.label(), Some("CPU"));
    /// ```
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the units suffix for legend text (e.g., "m", "Mi", "GB").
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_units("Mi");
    /// assert_eq!(state.units(), Some("Mi"));
    /// ```
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
        self
    }

    /// Sets the title displayed in the border.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_title("Resource");
    /// assert_eq!(state.title(), Some("Resource"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Controls whether the legend text is shown.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_show_legend(false);
    /// assert!(!state.show_legend());
    /// ```
    pub fn with_show_legend(mut self, show: bool) -> Self {
        self.show_legend = show;
        self
    }

    /// Sets the gauge orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::{ResourceGaugeState, GaugeOrientation};
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0)
    ///     .with_orientation(GaugeOrientation::Vertical);
    /// assert_eq!(state.orientation(), &GaugeOrientation::Vertical);
    /// ```
    pub fn with_orientation(mut self, orientation: GaugeOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the disabled state.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0).with_disabled(true);
    /// assert!(state.is_disabled());
    /// ```
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    // -- Getters --

    /// Returns the actual value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(42.0, 100.0, 200.0);
    /// assert_eq!(state.actual(), 42.0);
    /// ```
    pub fn actual(&self) -> f64 {
        self.actual
    }

    /// Returns the request value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(42.0, 100.0, 200.0);
    /// assert_eq!(state.request(), 100.0);
    /// ```
    pub fn request(&self) -> f64 {
        self.request
    }

    /// Returns the limit value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(42.0, 100.0, 200.0);
    /// assert_eq!(state.limit(), 200.0);
    /// ```
    pub fn limit(&self) -> f64 {
        self.limit
    }

    /// Returns the label, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert_eq!(state.label(), None);
    /// ```
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the units suffix, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert_eq!(state.units(), None);
    /// ```
    pub fn units(&self) -> Option<&str> {
        self.units.as_deref()
    }

    /// Returns the title, if set.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert_eq!(state.title(), None);
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns whether the legend is shown.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert!(state.show_legend());
    /// ```
    pub fn show_legend(&self) -> bool {
        self.show_legend
    }

    /// Returns the gauge orientation.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::{ResourceGaugeState, GaugeOrientation};
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert_eq!(state.orientation(), &GaugeOrientation::Horizontal);
    /// ```
    pub fn orientation(&self) -> &GaugeOrientation {
        &self.orientation
    }

    /// Returns whether the gauge is disabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// assert!(!state.is_disabled());
    /// ```
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns the utilization ratio (actual / limit), clamped to 0.0–1.0.
    ///
    /// Returns 0.0 if limit is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(250.0, 500.0, 1000.0);
    /// assert!((state.utilization() - 0.25).abs() < 0.001);
    /// ```
    pub fn utilization(&self) -> f64 {
        if self.limit <= 0.0 {
            0.0
        } else {
            (self.actual / self.limit).clamp(0.0, 1.0)
        }
    }

    /// Returns the ratio of actual to request.
    ///
    /// Returns 0.0 if request is zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let state = ResourceGaugeState::new(250.0, 500.0, 1000.0);
    /// assert!((state.request_ratio() - 0.5).abs() < 0.001);
    /// ```
    pub fn request_ratio(&self) -> f64 {
        if self.request <= 0.0 {
            0.0
        } else {
            self.actual / self.request
        }
    }

    /// Returns true if actual exceeds the request.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let under = ResourceGaugeState::new(100.0, 500.0, 1000.0);
    /// assert!(!under.is_over_request());
    ///
    /// let over = ResourceGaugeState::new(600.0, 500.0, 1000.0);
    /// assert!(over.is_over_request());
    /// ```
    pub fn is_over_request(&self) -> bool {
        self.actual >= self.request && self.request > 0.0
    }

    /// Returns true if actual is at or above 90% of the limit.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let safe = ResourceGaugeState::new(500.0, 500.0, 1000.0);
    /// assert!(!safe.is_near_limit());
    ///
    /// let critical = ResourceGaugeState::new(950.0, 500.0, 1000.0);
    /// assert!(critical.is_near_limit());
    /// ```
    pub fn is_near_limit(&self) -> bool {
        self.limit > 0.0 && self.actual >= self.limit * 0.9
    }

    // -- Setters --

    /// Sets the actual value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 100.0, 200.0);
    /// state.set_actual(75.0);
    /// assert_eq!(state.actual(), 75.0);
    /// ```
    pub fn set_actual(&mut self, actual: f64) {
        self.actual = actual;
    }

    /// Sets the request value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(50.0, 100.0, 200.0);
    /// state.set_request(150.0);
    /// assert_eq!(state.request(), 150.0);
    /// ```
    pub fn set_request(&mut self, request: f64) {
        self.request = request;
    }

    /// Sets the limit value.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(50.0, 100.0, 200.0);
    /// state.set_limit(500.0);
    /// assert_eq!(state.limit(), 500.0);
    /// ```
    pub fn set_limit(&mut self, limit: f64) {
        self.limit = limit;
    }

    /// Sets all three values at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// state.set_values(350.0, 500.0, 1000.0);
    /// assert_eq!(state.actual(), 350.0);
    /// assert_eq!(state.request(), 500.0);
    /// assert_eq!(state.limit(), 1000.0);
    /// ```
    pub fn set_values(&mut self, actual: f64, request: f64, limit: f64) {
        self.actual = actual;
        self.request = request;
        self.limit = limit;
    }

    /// Sets the label.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// state.set_label(Some("CPU".to_string()));
    /// assert_eq!(state.label(), Some("CPU"));
    /// ```
    pub fn set_label(&mut self, label: Option<String>) {
        self.label = label;
    }

    /// Sets the units.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// state.set_units(Some("Mi".to_string()));
    /// assert_eq!(state.units(), Some("Mi"));
    /// ```
    pub fn set_units(&mut self, units: Option<String>) {
        self.units = units;
    }

    /// Sets the title.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::ResourceGaugeState;
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// state.set_title(Some("Resource".to_string()));
    /// assert_eq!(state.title(), Some("Resource"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Instance update method.
    ///
    /// # Examples
    ///
    /// ```
    /// use envision::component::resource_gauge::{ResourceGaugeState, ResourceGaugeMessage};
    ///
    /// let mut state = ResourceGaugeState::new(0.0, 0.0, 0.0);
    /// state.update(ResourceGaugeMessage::SetValues {
    ///     actual: 100.0,
    ///     request: 200.0,
    ///     limit: 500.0,
    /// });
    /// assert_eq!(state.actual(), 100.0);
    /// ```
    pub fn update(&mut self, msg: ResourceGaugeMessage) -> Option<()> {
        ResourceGauge::update(self, msg)
    }

    /// Returns the health color based on actual vs request vs limit.
    fn health_color(&self) -> Color {
        if self.limit <= 0.0 {
            Color::DarkGray
        } else if self.actual >= self.limit * 0.9 {
            Color::Red
        } else if self.request > 0.0 && self.actual >= self.request {
            Color::Yellow
        } else {
            Color::Green
        }
    }

    /// Formats the legend text.
    fn legend_text(&self) -> String {
        let units = self.units.as_deref().unwrap_or("");
        format!(
            "{}{} / {}{} / {}{}",
            format_value(self.actual),
            units,
            format_value(self.request),
            units,
            format_value(self.limit),
            units,
        )
    }
}

/// Formats a float value for display (no trailing zeros).
fn format_value(v: f64) -> String {
    if v == v.floor() {
        format!("{}", v as i64)
    } else {
        format!("{:.1}", v)
    }
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

/// Messages for the ResourceGauge component.
///
/// # Examples
///
/// ```
/// use envision::component::resource_gauge::ResourceGaugeMessage;
///
/// let msg = ResourceGaugeMessage::SetActual(42.0);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum ResourceGaugeMessage {
    /// Set the actual value.
    SetActual(f64),
    /// Set the request value.
    SetRequest(f64),
    /// Set the limit value.
    SetLimit(f64),
    /// Set all three values.
    SetValues {
        /// Actual value.
        actual: f64,
        /// Request value.
        request: f64,
        /// Limit value.
        limit: f64,
    },
    /// Set the label.
    SetLabel(Option<String>),
    /// Set the units.
    SetUnits(Option<String>),
}

/// Output type (display-only, no output).
pub type ResourceGaugeOutput = ();

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/// A triple-value gauge displaying actual/request/limit.
pub struct ResourceGauge;

impl Component for ResourceGauge {
    type State = ResourceGaugeState;
    type Message = ResourceGaugeMessage;
    type Output = ResourceGaugeOutput;

    fn init() -> Self::State {
        ResourceGaugeState::default()
    }

    fn handle_event(
        _state: &Self::State,
        _event: &Event,
        _ctx: &EventContext,
    ) -> Option<Self::Message> {
        None // Display-only component
    }

    fn update(state: &mut Self::State, msg: Self::Message) -> Option<Self::Output> {
        match msg {
            ResourceGaugeMessage::SetActual(v) => state.actual = v,
            ResourceGaugeMessage::SetRequest(v) => state.request = v,
            ResourceGaugeMessage::SetLimit(v) => state.limit = v,
            ResourceGaugeMessage::SetValues {
                actual,
                request,
                limit,
            } => {
                state.actual = actual;
                state.request = request;
                state.limit = limit;
            }
            ResourceGaugeMessage::SetLabel(l) => state.label = l,
            ResourceGaugeMessage::SetUnits(u) => state.units = u,
        }
        None
    }

    fn view(state: &Self::State, ctx: &mut RenderContext<'_, '_>) {
        let disabled = ctx.disabled || state.disabled;

        // Border
        let border_style = if disabled {
            ctx.theme.disabled_style()
        } else if ctx.focused {
            ctx.theme.focused_border_style()
        } else {
            ctx.theme.border_style()
        };

        let mut block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style);

        if let Some(title) = &state.title {
            block = block.title(format!(" {} ", title));
        }

        let inner = block.inner(ctx.area);
        ctx.frame.render_widget(block, ctx.area);

        if inner.width < 3 || inner.height < 1 {
            return;
        }

        // Layout: [label] [bar] [legend]
        let label_width = state
            .label
            .as_ref()
            .map(|l| l.len() as u16 + 1)
            .unwrap_or(0);

        let legend = if state.show_legend {
            state.legend_text()
        } else {
            String::new()
        };
        let legend_width = if legend.is_empty() {
            0
        } else {
            legend.len() as u16 + 1
        };

        let bar_width = inner
            .width
            .saturating_sub(label_width)
            .saturating_sub(legend_width);

        if bar_width < 2 {
            // Too narrow for a bar — just show legend
            let text = if let Some(label) = &state.label {
                format!("{} {}", label, legend)
            } else {
                legend
            };
            let style = if disabled {
                ctx.theme.disabled_style()
            } else {
                Style::default().fg(state.health_color())
            };
            ctx.frame
                .render_widget(Paragraph::new(text).style(style), inner);
            return;
        }

        let bar_y = inner.y;
        let buf = ctx.frame.buffer_mut();
        let buf_area = buf.area;

        // Render label
        if let Some(label) = &state.label {
            let label_style = if disabled {
                ctx.theme.disabled_style()
            } else {
                Style::default().add_modifier(Modifier::BOLD)
            };
            for (i, ch) in label.chars().enumerate() {
                let x = inner.x + i as u16;
                if x < inner.x + label_width {
                    set_cell(buf, x, bar_y, &ch.to_string(), label_style, buf_area);
                }
            }
        }

        // Render bar
        let bar_x = inner.x + label_width;
        let bar_params = BarParams {
            state,
            disabled,
            theme: ctx.theme,
        };
        render_bar(buf, bar_x, bar_y, bar_width, &bar_params);

        // Render legend
        if !legend.is_empty() {
            let legend_x = bar_x + bar_width + 1;
            let legend_style = if disabled {
                ctx.theme.disabled_style()
            } else {
                Style::default().fg(state.health_color())
            };
            for (i, ch) in legend.chars().enumerate() {
                let x = legend_x + i as u16;
                if x < inner.x + inner.width {
                    set_cell(buf, x, bar_y, &ch.to_string(), legend_style, buf_area);
                }
            }
        }
    }
}

struct BarParams<'a> {
    state: &'a ResourceGaugeState,
    disabled: bool,
    theme: &'a Theme,
}

/// Renders the triple-value bar.
fn render_bar(buf: &mut Buffer, x: u16, y: u16, width: u16, params: &BarParams<'_>) {
    let state = params.state;
    let disabled = params.disabled;
    let buf_area = buf.area;
    if state.limit <= 0.0 || width == 0 {
        // Empty bar
        for i in 0..width {
            set_cell(
                buf,
                x + i,
                y,
                "\u{2591}",
                Style::default().fg(Color::DarkGray),
                buf_area,
            );
        }
        return;
    }

    let actual_ratio = (state.actual / state.limit).clamp(0.0, 1.0);
    let request_ratio = if state.request > 0.0 {
        (state.request / state.limit).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let actual_pos = ((actual_ratio * width as f64) as u16).min(width);
    let request_pos = ((request_ratio * width as f64) as u16).min(width);

    let fill_color = if disabled {
        params.theme.disabled_style().fg.unwrap_or(Color::DarkGray)
    } else {
        state.health_color()
    };

    let fill_style = Style::default().fg(fill_color);
    let request_zone_style = Style::default().fg(Color::DarkGray);
    let empty_style = Style::default().fg(Color::DarkGray);

    for i in 0..width {
        let cx = x + i;
        if i < actual_pos {
            // Filled — actual usage
            set_cell(buf, cx, y, "\u{2588}", fill_style, buf_area);
        } else if i < request_pos {
            // Request headroom zone
            set_cell(buf, cx, y, "\u{2593}", request_zone_style, buf_area);
        } else {
            // Empty — unused capacity
            set_cell(buf, cx, y, "\u{2591}", empty_style, buf_area);
        }
    }

    // Request marker (if request is between 0 and limit)
    if request_pos > 0 && request_pos < width && state.request > 0.0 {
        let marker_style = if disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        };
        set_cell(buf, x + request_pos, y, "\u{2502}", marker_style, buf_area);
    }
}

/// Writes a character to the buffer at (x, y) if within bounds.
fn set_cell(buf: &mut Buffer, x: u16, y: u16, ch: &str, style: Style, area: Rect) {
    if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
        if let Some(cell) = buf.cell_mut(Position::new(x, y)) {
            cell.set_symbol(ch);
            cell.set_style(style);
        }
    }
}

#[cfg(test)]
mod tests;
