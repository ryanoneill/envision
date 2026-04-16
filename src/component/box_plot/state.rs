//! BoxPlotState constructors, builders, accessors, setters, and instance methods.
//!
//! Extracted from the main box_plot module to keep file sizes manageable.

use super::{BoxPlot, BoxPlotData, BoxPlotMessage, BoxPlotOrientation, BoxPlotState};
use crate::component::Component;

impl BoxPlotState {
    /// Creates a new box plot state with the given datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// assert_eq!(state.datasets().len(), 1);
    /// ```
    pub fn new(datasets: Vec<BoxPlotData>) -> Self {
        Self {
            datasets,
            ..Default::default()
        }
    }

    /// Sets the title (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("Test", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ])
    /// .with_title("My Box Plot");
    /// assert_eq!(state.title(), Some("My Box Plot"));
    /// ```
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets whether to show outliers (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let state = BoxPlotState::default().with_show_outliers(false);
    /// assert!(!state.show_outliers());
    /// ```
    pub fn with_show_outliers(mut self, show: bool) -> Self {
        self.show_outliers = show;
        self
    }

    /// Sets the orientation (builder pattern).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotOrientation};
    ///
    /// let state = BoxPlotState::default()
    ///     .with_orientation(BoxPlotOrientation::Horizontal);
    /// assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
    /// ```
    pub fn with_orientation(mut self, orientation: BoxPlotOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    // ---- Accessors ----

    /// Returns the datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    ///     BoxPlotData::new("B", 2.0, 3.0, 4.0, 5.0, 6.0),
    /// ]);
    /// assert_eq!(state.datasets().len(), 2);
    /// ```
    pub fn datasets(&self) -> &[BoxPlotData] {
        &self.datasets
    }

    /// Returns a mutable reference to the datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    /// use ratatui::style::Color;
    ///
    /// let mut state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// state.datasets_mut()[0].set_color(Color::Red);
    /// assert_eq!(state.datasets()[0].color(), Color::Red);
    /// ```
    pub fn datasets_mut(&mut self) -> &mut [BoxPlotData] {
        &mut self.datasets
    }

    /// Returns the dataset at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("Service A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// assert_eq!(state.get_dataset(0).unwrap().label(), "Service A");
    /// assert!(state.get_dataset(99).is_none());
    /// ```
    pub fn get_dataset(&self, index: usize) -> Option<&BoxPlotData> {
        self.datasets.get(index)
    }

    /// Returns a mutable reference to the dataset at the given index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("Service A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// if let Some(ds) = state.get_dataset_mut(0) {
    ///     ds.set_label("Service B");
    /// }
    /// assert_eq!(state.datasets()[0].label(), "Service B");
    /// ```
    pub fn get_dataset_mut(&mut self, index: usize) -> Option<&mut BoxPlotData> {
        self.datasets.get_mut(index)
    }

    /// Returns the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let state = BoxPlotState::default().with_title("Latency");
    /// assert_eq!(state.title(), Some("Latency"));
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Sets the title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let mut state = BoxPlotState::default();
    /// state.set_title(Some("Response Times".to_string()));
    /// assert_eq!(state.title(), Some("Response Times"));
    /// ```
    pub fn set_title(&mut self, title: Option<String>) {
        self.title = title;
    }

    /// Returns whether outliers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let state = BoxPlotState::default();
    /// assert!(state.show_outliers());
    /// ```
    pub fn show_outliers(&self) -> bool {
        self.show_outliers
    }

    /// Sets whether outliers are shown.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// let mut state = BoxPlotState::default();
    /// state.set_show_outliers(false);
    /// assert!(!state.show_outliers());
    /// ```
    pub fn set_show_outliers(&mut self, show: bool) {
        self.show_outliers = show;
    }

    /// Returns the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotOrientation};
    ///
    /// let state = BoxPlotState::default().with_orientation(BoxPlotOrientation::Horizontal);
    /// assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
    /// ```
    pub fn orientation(&self) -> &BoxPlotOrientation {
        &self.orientation
    }

    /// Sets the orientation.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotOrientation};
    ///
    /// let mut state = BoxPlotState::default();
    /// state.set_orientation(BoxPlotOrientation::Horizontal);
    /// assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
    /// ```
    pub fn set_orientation(&mut self, orientation: BoxPlotOrientation) {
        self.orientation = orientation;
    }

    /// Returns the currently selected dataset index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// assert_eq!(state.selected(), 0);
    /// ```
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Sets the selected dataset index.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    ///     BoxPlotData::new("B", 2.0, 3.0, 4.0, 5.0, 6.0),
    /// ]);
    /// state.set_selected(1);
    /// assert_eq!(state.selected(), 1);
    /// ```
    pub fn set_selected(&mut self, index: usize) {
        if !self.datasets.is_empty() {
            self.selected = index.min(self.datasets.len() - 1);
        }
    }

    /// Returns the number of datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    ///     BoxPlotData::new("B", 2.0, 3.0, 4.0, 5.0, 6.0),
    /// ]);
    /// assert_eq!(state.dataset_count(), 2);
    /// ```
    pub fn dataset_count(&self) -> usize {
        self.datasets.len()
    }

    /// Returns true if there are no datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::BoxPlotState;
    ///
    /// assert!(BoxPlotState::default().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty()
    }

    /// Adds a dataset.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::default();
    /// state.add_dataset(BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0));
    /// assert_eq!(state.dataset_count(), 1);
    /// ```
    pub fn add_dataset(&mut self, dataset: BoxPlotData) {
        self.datasets.push(dataset);
    }

    /// Clears all datasets.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let mut state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 1.0, 2.0, 3.0, 4.0, 5.0),
    /// ]);
    /// state.clear_datasets();
    /// assert!(state.is_empty());
    /// ```
    pub fn clear_datasets(&mut self) {
        self.datasets.clear();
        self.selected = 0;
    }

    /// Computes the global minimum value across all datasets (including outliers
    /// if show_outliers is enabled).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
    ///     BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    /// ]);
    /// assert_eq!(state.global_min(), 5.0);
    /// ```
    pub fn global_min(&self) -> f64 {
        self.datasets
            .iter()
            .map(|d| {
                if self.show_outliers {
                    d.overall_min()
                } else {
                    d.min()
                }
            })
            .reduce(f64::min)
            .unwrap_or(0.0)
    }

    /// Computes the global maximum value across all datasets (including outliers
    /// if show_outliers is enabled).
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData};
    ///
    /// let state = BoxPlotState::new(vec![
    ///     BoxPlotData::new("A", 5.0, 10.0, 20.0, 30.0, 40.0),
    ///     BoxPlotData::new("B", 8.0, 15.0, 25.0, 35.0, 50.0),
    /// ]);
    /// assert_eq!(state.global_max(), 50.0);
    /// ```
    pub fn global_max(&self) -> f64 {
        self.datasets
            .iter()
            .map(|d| {
                if self.show_outliers {
                    d.overall_max()
                } else {
                    d.max()
                }
            })
            .reduce(f64::max)
            .unwrap_or(0.0)
    }

    // ---- Focus / Disabled ----

    // ---- Instance methods ----

    /// Updates the state with a message, returning any output.
    ///
    /// # Example
    ///
    /// ```rust
    /// use envision::component::{BoxPlotState, BoxPlotData, BoxPlotMessage, BoxPlotOrientation};
    ///
    /// let mut state = BoxPlotState::default();
    /// state.update(BoxPlotMessage::SetOrientation(BoxPlotOrientation::Horizontal));
    /// assert_eq!(state.orientation(), &BoxPlotOrientation::Horizontal);
    /// ```
    pub fn update(&mut self, msg: BoxPlotMessage) -> Option<()> {
        BoxPlot::update(self, msg)
    }
}
