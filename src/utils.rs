use std::path::{Path, PathBuf};

/// Generates a unique cross-platform filename accounting for OS-specific limits
pub fn get_unique_filename(output_dir: Option<&Path>, dir_name: &str, base_suffix: &str, extension: &str) -> PathBuf {
    // 1. Sanitize cross-platform illegal symbols (?, *, :, ", <, >, |, \, /)
    let sanitized_dir = dir_name.replace(|c: char| {
        matches!(c, '?' | '*' | ':' | '"' | '<' | '>' | '|' | '\\' | '/')
    }, "_");

    // 2. Intercept Windows reserved device naming strings 
    let base_name = match sanitized_dir.to_uppercase().as_str() {
        "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" => format!("{}_safe", sanitized_dir),
        s if s.starts_with("COM") || s.starts_with("LPT") => format!("{}_safe", sanitized_dir),
        _ => sanitized_dir,
    };

    let mut index = 1;
    loop {
        let filename = format!("{}_{}_{}.{}", base_name, base_suffix, index, extension);
        
        // Target base output dir choice if provided, otherwise resolve locally
        let path = match output_dir {
            Some(dir) => dir.join(filename),
            None => PathBuf::from(filename),
        };
        
        if !path.exists() {
            return path;
        }
        index += 1;
    }
}

