use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A marker (point-in-time annotation)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Marker {
    /// Unique identifier
    pub id: Uuid,
    /// Time position in seconds
    pub time: f64,
    /// Optional label
    pub label: Option<String>,
    /// Optional color (hex string, e.g., "#f97316")
    pub color: Option<String>,
}

impl Marker {
    /// Create a new marker at the given time
    #[allow(dead_code)]
    pub fn new(time: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            time,
            label: None,
            color: None,
        }
    }

    /// Create a marker with a label
    #[allow(dead_code)]
    pub fn with_label(time: f64, label: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            time,
            label: Some(label.into()),
            color: None,
        }
    }
}
