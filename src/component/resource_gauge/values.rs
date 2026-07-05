//! `ResourceValues` — named-fields carrier for the three resource_gauge values.

/// Named-fields carrier for the three resource-gauge values (`actual`, `request`,
/// `limit`).
///
/// Replaces the previous positional `(f64, f64, f64)` triple on the constructor
/// and accessor surface — struct-literal construction (`ResourceValues { actual,
/// request, limit }`) and named destructuring (`let ResourceValues { actual,
/// request, limit } = state.values();`) both eliminate the "silently transpose
/// `request` and `limit`" hazard.
///
/// # Example
///
/// ```rust
/// use envision::component::ResourceValues;
///
/// let vals = ResourceValues {
///     actual: 250.0,
///     request: 500.0,
///     limit: 1000.0,
/// };
/// assert_eq!(vals.actual, 250.0);
/// assert_eq!(vals.request, 500.0);
/// assert_eq!(vals.limit, 1000.0);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(
    feature = "serialization",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ResourceValues {
    /// Current in-use value (e.g., current CPU consumption).
    pub actual: f64,
    /// Requested value (e.g., K8s pod resource request).
    pub request: f64,
    /// Hard limit (e.g., K8s pod resource limit).
    pub limit: f64,
}
