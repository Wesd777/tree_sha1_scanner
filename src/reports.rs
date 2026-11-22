use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::utils::format_size;

/// Builds a structured plain text tree where lines use the format: filename : sha1 (size)
pub fn generate_tree_output(
    root: &Path,
    sha_map: &HashMap<String, Vec<PathBuf>>,
    size_map: &HashMap<PathBuf, u64>,
    output_path: &Path,
) -> io::Result<()> {
    let mut path_to_sha: HashMap<&Path, &str> = HashMap::with_capacity(sha_map.len() * 2);
    for (sha, paths) in sha_map {
        for path in paths {
            path_to_sha.insert(path.as_path(), sha.as_str());
        }
    }

    let file = File::create(output_path)?;
    let mut output = io::BufWriter::with_capacity(64 * 1024, file);

    // WalkDir::new(...).sort_by_file_name() sorts siblings in-place;
    // no need to collect the whole tree into a Vec.
    for entry in WalkDir::new(root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let depth = entry.depth();
        let indent = "  ".repeat(depth); // "  ".repeat is cheaper than " ".repeat(depth*2)
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();

        if path.is_dir() {
            writeln!(output, "{}{}/", indent, name)?;
        } else if let Some(sha1) = path_to_sha.get(path) {
            let size_str = size_map
                .get(path)
                .map(|b| format_size(*b))
                .unwrap_or_else(|| "0 B".to_string());
            writeln!(output, "{}{} : {} ({})", indent, name, sha1, size_str)?;
        }
    }
    Ok(())
}

/// Generates a clean, tab-separated table grouped by matching duplicate sets, largest size first
pub fn generate_tsv_output(
    sha_map: &HashMap<String, Vec<PathBuf>>,
    size_map: &HashMap<PathBuf, u64>,
    output_path: &Path,
) -> io::Result<()> {
    let mut output = File::create(output_path)?;

    writeln!(output, "Size\tFile Path\tSHA1")?;

    // Collect groups that contain duplicates
    let mut grouped_entries: Vec<(&String, &Vec<PathBuf>)> = sha_map
        .iter()
        .filter(|(_, paths)| paths.len() > 1)
        .collect();

    // Sort descending by total group size (sum of all file sizes in the group)
    grouped_entries.sort_by(|a, b| {
        let size_a: u64 =
            a.1.iter()
                .map(|p| size_map.get(p).copied().unwrap_or(0))
                .sum();
        let size_b: u64 =
            b.1.iter()
                .map(|p| size_map.get(p).copied().unwrap_or(0))
                .sum();
        size_b.cmp(&size_a)
    });

    for (sha, paths) in grouped_entries {
        // Sort files within group by individual size descending
        let mut sorted_paths = paths.clone();
        sorted_paths.sort_by(|a, b| {
            let sa = size_map.get(a).copied().unwrap_or(0);
            let sb = size_map.get(b).copied().unwrap_or(0);
            sb.cmp(&sa)
        });

        for path in sorted_paths {
            let size_str = size_map
                .get(&path)
                .map(|b| format_size(*b))
                .unwrap_or_else(|| "0 B".to_string());
            writeln!(output, "{}\t{}\t{}", size_str, path.display(), sha)?;
        }
    }
    Ok(())
}

/// Filters and logs duplicate files into groups
pub fn generate_dupes_output(
    sha_map: &HashMap<String, Vec<PathBuf>>,
    size_map: &HashMap<PathBuf, u64>,
    output_path: &Path,
) -> io::Result<()> {
    let file = File::create(output_path)?;
    let mut output = io::BufWriter::with_capacity(64 * 1024, file);

    // Collect only duplicate groups (no clone of paths yet)
    let mut dup_groups: Vec<(&String, &Vec<PathBuf>)> =
        sha_map.iter().filter(|(_, p)| p.len() > 1).collect();

    dup_groups.sort_by(|a, b| {
        let ta: u64 =
            a.1.iter()
                .map(|p| size_map.get(p).copied().unwrap_or(0))
                .sum();
        let tb: u64 =
            b.1.iter()
                .map(|p| size_map.get(p).copied().unwrap_or(0))
                .sum();
        tb.cmp(&ta)
    });

    if dup_groups.is_empty() {
        writeln!(output, "No duplicate files found.")?;
        return Ok(());
    }

    for (index, (sha, paths)) in dup_groups.iter().enumerate() {
        let total: u64 = paths
            .iter()
            .map(|p| size_map.get(p).copied().unwrap_or(0))
            .sum();
        let single_size: u64 = paths
            .first()
            .and_then(|p| size_map.get(p).copied())
            .unwrap_or(0);
        let savings: u64 = total.saturating_sub(single_size);
        writeln!(
            output,
            "Duplicate Group #{}: [SHA-1: {}] (Total: {}) (Total Savings: {})",
            index,
            sha,
            format_size(total),
            format_size(savings)
        )?;

        // Sort by size desc — borrow-only, no clone
        let mut sorted: Vec<&PathBuf> = paths.iter().collect();
        sorted.sort_by(|a, b| {
            let sa = size_map.get(*a).copied().unwrap_or(0);
            let sb = size_map.get(*b).copied().unwrap_or(0);
            sb.cmp(&sa)
        });

        for path in &sorted {
            let size_str = size_map
                .get(*path)
                .map(|b| format_size(*b))
                .unwrap_or_else(|| "0 B".to_string());
            writeln!(output, " -> {} ({})", path.display(), size_str)?;
        }
        writeln!(output)?;
    }
    Ok(())
}
