use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Cap for sequential filename numbering before switching to timestamps
const MAX_FILENAME_INDEX: u32 = 999;
const IO_RETRY_LIMIT: u32 = 3;

/// Formats a byte count into a human-readable string (B, KB, MB, GB, TB).
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    const TB: u64 = 1024 * 1024 * 1024 * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn get_unique_filename(
    output_dir: Option<&Path>,
    dir_name: &str,
    base_suffix: &str,
    extension: &str,
) -> PathBuf {
    let sanitized_dir = dir_name
        .replace(
            |c: char| matches!(c, '?' | '*' | ':' | '"' | '<' | '>' | '|' | '\\' | '/'),
            "_",
        )
        // Windows forbids trailing spaces and periods in file/folder names
        .trim_end_matches([' ', '.'])
        .to_string();

    let fallback_dir = if sanitized_dir.is_empty() {
        "output"
    } else {
        sanitized_dir.as_str()
    };

    let base_name = if is_windows_reserved_name(fallback_dir) {
        format!("{}_safe", fallback_dir)
    } else {
        fallback_dir.to_string()
    };

    let base_path = output_dir.unwrap_or_else(|| Path::new(""));
    let clean_ext = extension.trim_start_matches('.');

    for index in 1..=MAX_FILENAME_INDEX {
        let filename = format!("{}_{}_{}.{}", base_name, base_suffix, index, clean_ext);
        let path = base_path.join(&filename);

        if !entry_exists_resilient(&path) {
            return path;
        }
    }

    // Fallback: Use millisecond timestamp to prevent same-second overwrites
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);

    base_path.join(format!(
        "{}_{}_{}.{}",
        base_name, base_suffix, ts, clean_ext
    ))
}

/// Checks if a path entry exists (including broken symlinks) with brief retries
/// for temporary antivirus locks.
fn entry_exists_resilient(path: &Path) -> bool {
    for attempt in 0..IO_RETRY_LIMIT {
        match fs::symlink_metadata(path) {
            Ok(_) => return true, // File, directory, or symlink exists
            Err(e) if e.kind() == ErrorKind::NotFound => return false,
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                // Assume occupied if access is denied so we don't collide
                return true;
            }
            Err(_) => {
                // Transient I/O error pause briefly and retry
                if attempt + 1 < IO_RETRY_LIMIT {
                    thread::sleep(Duration::from_millis(200 * (attempt as u64 + 1)));
                }
            }
        }
    }
    // If the drive keeps failing I/O checks, assume occupied to avoid overwriting
    true
}

/// Checks Windows reserved DOS device names
fn is_windows_reserved_name(name: &str) -> bool {
    // Strip any extension for the check (Windows reserves CON, CON.txt, CON.whatever)
    let stem = name.split('.').next().unwrap_or(name);
    let upper = stem.to_uppercase();

    matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$")
        || (upper.len() == 4
            && (upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.as_bytes()[3].is_ascii_digit()
            && upper.as_bytes()[3] != b'0')
}
