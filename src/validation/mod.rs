/// Closed-loop Validation Framework
/// Compares model predictions against observation data and reports error metrics.
/// This moves tools from "calculator" to "calibrated modeling system."

pub mod metrics;
pub mod independent;
pub mod validation_tool;

pub use validation_tool::validate_model;
