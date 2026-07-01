use chrono::Utc;
use std::path::Path;
use uuid::Uuid;

/// Helper to collect query rows, logging any deserialization errors
/// rather than silently dropping them.
pub fn collect_ok<T>(rows: impl Iterator<Item = Result<T, impl std::fmt::Display>>) -> Vec<T> {
    rows.filter_map(|r| match r {
        Ok(v) => Some(v),
        Err(e) => {
            log::warn!("Row skipped during query: {}", e);
            None
        }
    })
    .collect()
}

/// Generate a new UUID v4 string.
pub fn new_id() -> String {
    Uuid::new_v4().to_string()
}

/// Return the current time as an RFC 3339 timestamp.
pub fn now_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Resolve a raw path string to its canonical (absolute, symlink-resolved) form.
///
/// Validates the path exists and returns a descriptive error if not.
pub fn resolve_canonical_path(raw: &str) -> Result<String, String> {
    let path = Path::new(raw);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", raw));
    }
    path.canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to resolve path: {}", e))
}
