use std::path::Path;
use std::process::Command;

/// Probe media duration in seconds using ffprobe.
pub fn probe_duration_seconds(path: &Path) -> Option<f64> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let duration_str = stdout.trim();
    if duration_str.is_empty() {
        return None;
    }

    duration_str.parse::<f64>().ok()
}
