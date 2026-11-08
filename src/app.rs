use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

use crate::crypto::compute_sha1;
use crate::reports::{generate_dupes_output, generate_tree_output, generate_tsv_output};
use crate::utils::get_unique_filename;

pub fn run(target_dir: &Path, output_dir: Option<&Path>) -> io::Result<()> {
    let start_time = std::time::Instant::now();

    let media_extensions: HashSet<&str> = [
        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", // Images
        "mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", // Videos
        "mp3", "wav", "flac", "ogg", "m4a", "aac", // Audio
    ]
    .iter()
    .cloned()
    .collect();

    let dir_name = target_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "root".to_string());

    println!("Scanning directory and indexing files by byte size...");

    // 1. Collect and filter entries from WalkDir into a standard Vector
    let entries: Vec<PathBuf> = WalkDir::new(target_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    return media_extensions.contains(ext.to_lowercase().as_str());
                }
            }
            false
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();

    let discovered_count = entries.len();

    // 2. Fetch metadata lengths in parallel using Rayon
    let size_pairs: Vec<(u64, PathBuf)> = entries
        .into_par_iter()
        .filter_map(|path| {
            if let Ok(metadata) = std::fs::metadata(&path) {
                Some((metadata.len(), path))
            } else {
                None
            }
        })
        .collect();

    // 3. Populate your final HashMap sequentially from the processed results
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (size, path) in size_pairs {
        files_by_size.entry(size).or_default().push(path);
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

    let file_count = Arc::new(AtomicUsize::new(0));
    let is_running = Arc::new(AtomicBool::new(true));

    let ui_file_count = Arc::clone(&file_count);
    let ui_is_running = Arc::clone(&is_running);
    let ui_handle = thread::spawn(move || {
        let spinner_frames = ['-', '\\', '|', '/'];
        let mut frame_idx = 0;

        while ui_is_running.load(Ordering::Relaxed) {
            let current_count = ui_file_count.load(Ordering::Relaxed);
            let spinner_char = spinner_frames[frame_idx % spinner_frames.len()];

            print!("\rFiles processed: {} {}", current_count, spinner_char);
            let _ = io::stdout().flush();

            frame_idx += 1;
            thread::sleep(Duration::from_millis(250));
        }
    });

    // LOCK-FREE EXTRACTION: Threads build localized maps and merge them up efficiently
    let sha_map: HashMap<String, Vec<PathBuf>> = duplicate_candidates
        .par_iter()
        .filter_map(|path| {
            if let Ok(sha1_hex) = compute_sha1(path) {
                file_count.fetch_add(1, Ordering::Relaxed);
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

    is_running.store(false, Ordering::Relaxed);
    let _ = ui_handle.join();

    print!("\r                                           ");
    let total_processed = file_count.load(Ordering::Relaxed);
    println!(
        "\rScan complete! Total files processed: {}",
        total_processed
    );

    // Compute unique filenames while mitigating OS namespace collisions
    let tree_filename = get_unique_filename(output_dir, &dir_name, "tree_output", "txt");
    let tsv_filename = get_unique_filename(output_dir, &dir_name, "output", "csv");
    let dupes_filename = get_unique_filename(output_dir, &dir_name, "DupesQ", "txt");

    println!("Generating report files...");

    let dupes_count = sha_map.values().filter(|paths| paths.len() > 1).count();

    generate_tree_output(target_dir, &sha_map, &tree_filename)?;
    if dupes_count > 0 {
        generate_tsv_output(&sha_map, &tsv_filename)?;
    }
    generate_dupes_output(&sha_map, &dupes_filename)?;

    let total_secs = start_time.elapsed().as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    println!(
        "\nAll operations completed successfully in {:02}:{:02}:{:02}:",
        hours, minutes, seconds
    );
    println!(" -> {} dupes found", dupes_count);
    println!(" -> Tree layout: {}", tree_filename.display());
    if dupes_count > 0 {
        println!(" -> TSV Registry: {}", tsv_filename.display());
    }
    println!(" -> Duplicates:  {}", dupes_filename.display());
    println!();

    Ok(())
}
