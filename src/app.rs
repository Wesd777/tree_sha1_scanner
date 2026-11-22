use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use indicatif::{ProgressBar, ProgressStyle};

use crate::crypto::compute_sha1;
use crate::reports::{generate_dupes_output, generate_tree_output, generate_tsv_output};
use crate::utils::get_unique_filename;

fn normalize_extensions(raw: &[String]) -> HashSet<String> {
    raw.iter()
        .map(|s| s.trim().trim_start_matches('.').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn run(
    target_dir: &Path,
    output_dir: Option<&Path>,
    custom_types: &[String],
) -> io::Result<()> {
    let start_time = std::time::Instant::now();

    // Determine the extension filter: use custom types if provided, otherwise defaults
    let media_extensions: HashSet<String> = if custom_types.is_empty() {
        [
            "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", // Images
            "mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", // Videos
            "mp3", "wav", "flac", "ogg", "m4a", "aac", // Audio
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    } else {
        normalize_extensions(custom_types)
    };

    let dir_name = target_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".to_string());

    println!("Scanning directory and indexing files by byte size...");

    // Collect entries and sizes in a single pass
    let size_pairs: Vec<(u64, PathBuf)> = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return false;
            }
            path.extension()
                .and_then(|s| s.to_str())
                .map(|ext| media_extensions.contains(&ext.to_lowercase()))
                .unwrap_or(false)
        })
        .filter_map(|entry| {
            let len = entry.metadata().ok()?.len();
            Some((len, entry.path().to_path_buf()))
        })
        .collect();

    let discovered_count = size_pairs.len();

    // Build both the size-grouped map and a per-path size lookup
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut size_map: HashMap<PathBuf, u64> = HashMap::new();
    for (size, path) in &size_pairs {
        files_by_size.entry(*size).or_default().push(path.clone());
        size_map.insert(path.clone(), *size);
    }

    // Filter out sizes that only belong to 1 single file
    let duplicate_candidates: Vec<PathBuf> = files_by_size
        .into_iter()
        .filter(|(_, paths)| paths.len() > 1)
        .flat_map(|(_, paths)| paths)
        .collect();

    println!(
        "Found {} total files. Matching sizes identified. Starting parallel hashing on {} candidates...",
        discovered_count,
        duplicate_candidates.len()
    );

    // Indicatif progress bar
    let progress = ProgressBar::new(duplicate_candidates.len() as u64);
    let style = ProgressStyle::with_template(
        "{bar:40.cyan/blue} {pos}/{len} ({percent}%) [{elapsed_precise}]",
    );
    progress.set_style(match style {
        Ok(s) => s.progress_chars("=>-"),
        Err(_) => ProgressStyle::with_template("{bar} {pos}/{len}")
            .unwrap_or_else(|_| ProgressStyle::default_bar()),
    });

    // LOCK-FREE EXTRACTION: Threads build localized maps and merge them
    let sha_map: HashMap<String, Vec<PathBuf>> = duplicate_candidates
        .par_iter()
        .filter_map(|path| {
            if let Ok(sha1_hex) = compute_sha1(path) {
                progress.inc(1);
                Some((sha1_hex, path.clone()))
            } else {
                None
            }
        })
        .fold(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut local_map, (sha1, path)| {
                local_map.entry(sha1).or_default().push(path);
                local_map
            },
        )
        .reduce(
            || HashMap::<String, Vec<PathBuf>>::new(),
            |mut map_a, map_b| {
                for (sha1, paths) in map_b {
                    map_a.entry(sha1).or_default().extend(paths);
                }
                map_a
            },
        );

    progress.finish_with_message("Hashing complete.");

    // Compute unique filenames while mitigating OS namespace collisions
    let tree_filename = get_unique_filename(output_dir, &dir_name, "tree_output", "txt");
    let tsv_filename = get_unique_filename(output_dir, &dir_name, "output", "csv");
    let dupes_filename = get_unique_filename(output_dir, &dir_name, "DupesQ", "txt");

    println!("Generating report files...");

    let dupes_file_count: usize = sha_map
        .values()
        .filter(|paths| paths.len() > 1)
        .map(|paths| paths.len())
        .sum();

    let dupes_group_count = sha_map.values().filter(|paths| paths.len() > 1).count();

    generate_tree_output(target_dir, &sha_map, &size_map, &tree_filename)?;
    if dupes_group_count > 0 {
        generate_tsv_output(&sha_map, &size_map, &tsv_filename)?;
    }
    generate_dupes_output(&sha_map, &size_map, &dupes_filename)?;

    let total_secs = start_time.elapsed().as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    println!(
        "\nAll operations completed successfully in {:02}:{:02}:{:02}:",
        hours, minutes, seconds
    );
    println!(
        " -> {} duplicate files in {} group(s)",
        dupes_file_count, dupes_group_count
    );
    println!(" -> Tree layout: {}", tree_filename.display());
    if dupes_group_count > 0 {
        println!(" -> TSV Registry: {}", tsv_filename.display());
    }
    println!(" -> Duplicates:  {}", dupes_filename.display());
    println!();

    Ok(())
}
