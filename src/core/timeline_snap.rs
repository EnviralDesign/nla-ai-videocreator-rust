use uuid::Uuid;

/// Category of snap target used for tie-breaking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapTargetKind {
    /// Clip start or end edge.
    ClipEdge,
    /// Playhead position.
    Playhead,
    /// Marker position.
    Marker,
}

impl SnapTargetKind {
    /// Priority for tie-breaking when distances are equal.
    pub fn priority(self) -> i32 {
        match self {
            SnapTargetKind::ClipEdge => 3,
            SnapTargetKind::Playhead => 2,
            SnapTargetKind::Marker => 1,
        }
    }
}

/// Snap target expressed in frame units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapTarget {
    /// Frame position of the target (frame boundaries).
    pub frame: f64,
    /// Target category for priority decisions.
    pub kind: SnapTargetKind,
    /// Clip id if this target comes from a clip edge.
    pub clip_id: Option<Uuid>,
    /// Marker id if this target comes from a marker.
    pub marker_id: Option<Uuid>,
}

impl SnapTarget {
    /// Build a clip-edge target.
    pub fn clip_edge(frame: f64, clip_id: Uuid) -> Self {
        Self {
            frame,
            kind: SnapTargetKind::ClipEdge,
            clip_id: Some(clip_id),
            marker_id: None,
        }
    }

    /// Build a playhead target.
    pub fn playhead(frame: f64) -> Self {
        Self {
            frame,
            kind: SnapTargetKind::Playhead,
            clip_id: None,
            marker_id: None,
        }
    }

    /// Build a marker target.
    pub fn marker(frame: f64, marker_id: Uuid) -> Self {
        Self {
            frame,
            kind: SnapTargetKind::Marker,
            clip_id: None,
            marker_id: Some(marker_id),
        }
    }
}

/// Result of a snap query in frame units.
#[derive(Clone, Copy, Debug)]
pub struct SnapMatch {
    /// Delta in frames that should be applied to the source.
    pub delta_frames: f64,
    /// The target that was snapped to.
    pub target: SnapTarget,
}

/// Convert seconds to frame units using the given fps.
pub fn frames_from_seconds(time_seconds: f64, fps: f64) -> f64 {
    time_seconds * fps.max(1.0)
}

/// Convert frame units back to seconds using the given fps.
pub fn seconds_from_frames(frames: f64, fps: f64) -> f64 {
    let fps = fps.max(1.0);
    frames / fps
}

/// Round a time value to the nearest frame boundary.
pub fn snap_time_to_frame(time_seconds: f64, fps: f64) -> f64 {
    let fps = fps.max(1.0);
    (time_seconds * fps).round() / fps
}

/// Find the best snap delta between sources and targets within a threshold.
pub fn best_snap_delta_frames(
    sources_frames: &[f64],
    targets: &[SnapTarget],
    threshold_frames: f64,
) -> Option<SnapMatch> {
    if sources_frames.is_empty() || targets.is_empty() || threshold_frames <= 0.0 {
        return None;
    }

    let mut best_match: Option<SnapMatch> = None;
    let mut best_distance = f64::INFINITY;
    let mut best_priority = i32::MIN;
    let epsilon = 1e-4;

    for &source in sources_frames {
        for &target in targets {
            let delta = target.frame - source;
            let distance = delta.abs();
            if distance > threshold_frames {
                continue;
            }
            let priority = target.kind.priority();
            let should_take = distance + epsilon < best_distance
                || ((distance - best_distance).abs() <= epsilon && priority > best_priority);
            if should_take {
                best_distance = distance;
                best_priority = priority;
                best_match = Some(SnapMatch {
                    delta_frames: delta,
                    target,
                });
            }
        }
    }

    best_match
}
