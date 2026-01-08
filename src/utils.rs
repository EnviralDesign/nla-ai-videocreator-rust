use std::path::Path;
use urlencoding;

/// Generates a URL for a local file that is compatible with the "nla" custom protocol handler.
/// This abstracts away the specific scheme (http://nla.localhost/) and encoding requirements
/// for the current Dioxus/WebView2 configuration on Windows.
pub fn get_local_file_url(path: &Path) -> String {
    // 1. Convert path separators to forward slashes (standard API for URL paths)
    let p_str = path.to_string_lossy().replace("\\", "/");
    
    // 2. Percent-encode the path to handle spaces, distinct characters, etc.
    // 3. Prefix with the configured custom protocol host mapping.
    format!("http://nla.localhost/{}", urlencoding::encode(&p_str))
}

pub fn parse_f32_input(value: &str, fallback: f32) -> f32 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed.parse::<f32>().unwrap_or(fallback)
}

pub fn parse_f64_input(value: &str, fallback: f64) -> f64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed.parse::<f64>().unwrap_or(fallback)
}

pub fn parse_i64_input(value: &str, fallback: i64) -> i64 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return fallback;
    }
    trimmed.parse::<i64>().unwrap_or(fallback)
}
