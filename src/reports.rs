use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Builds a structured plain text tree where lines use the format: filename : sha1
pub fn generate_tree_output(root: &Path, sha_map: &HashMap<String, Vec<PathBuf>>, output_path: &Path) -> io::Result<()> {
    let mut path_to_sha: HashMap<&Path, &str> = HashMap::new();
    for (sha, paths) in sha_map {
        for path in paths {
            path_to_sha.insert(path.as_path(), sha);
        }
    }

    let mut output = File::create(output_path)?;

    // Preserve original layout formatting by sorting directory structure components
    let mut entries: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by(|a, b| a.path().cmp(b.path()));

    for entry in entries {
        let path = entry.path();
        let depth = entry.depth();
        let indent = " ".repeat(depth * 2);
        let name = path.file_name().map(|n| n.to_string_lossy()).unwrap_or_default();

        if path.is_dir() {
            writeln!(output, "{}{}/", indent, name)?;
        } else if let Some(sha1) = path_to_sha.get(path) {
            writeln!(output, "{}{} : {}", indent, name, sha1)?;
        }
    }
    Ok(())
}

/// Generates a clean, tab-separated table matching paths to hashes sorted by file paths
pub fn generate_tsv_output(sha_map: &HashMap<String, Vec<PathBuf>>, output_path: &Path) -> io::Result<()> {
    let mut output = File::create(output_path)?;

    writeln!(output, "File Path\tSHA1")?;

    let mut sorted_entries: Vec<(&PathBuf, &String)> = Vec::new();
    for (sha, paths) in sha_map {
        for path in paths {
            sorted_entries.push((path, sha));
        }
    }

    sorted_entries.sort_by(|a, b| a.0.cmp(b.0));

    for (path, sha) in sorted_entries {
        writeln!(output, "{}\t{}", path.display(), sha)?;
    }
    Ok(())
}

/// Filters and logs duplicate files into clusters within the resolved DupesQ file
pub fn generate_dupes_output(sha_map: &HashMap<String, Vec<PathBuf>>, output_path: &Path) -> io::Result<()> {
    let mut output = File::create(output_path)?;

    let mut duplicate_groups: Vec<(String, Vec<PathBuf>)> = sha_map
        .iter()
        .filter(|(_, paths)| paths.len() > 1)
        .map(|(sha, paths)| {
            let mut sorted_paths = paths.clone();
            sorted_paths.sort();
            (sha.clone(), sorted_paths)
        })
        .collect();

    duplicate_groups.sort_by(|a, b| a.1.cmp(&b.1));

    let mut index = 1;
    for (sha, paths) in duplicate_groups {
        writeln!(output, "Duplicate Group #{}: [SHA-1: {}]", index, sha)?;
        for path in paths {
            writeln!(output, " -> {}", path.display())?;
        }
        writeln!(output, "")?;
        index += 1;
    }

    if index == 1 {
        writeln!(output, "No duplicate files found.")?;
    }
    Ok(())
}

