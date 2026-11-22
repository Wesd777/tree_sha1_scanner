# Tree SHA-1 Scanner

Command-line Rust tool that walks a folder tree & finds duplicate media files.
Easy-to-read reports show where the duplicates are, how big they are, and how much space can be reclaimed.

## Features
* ⚡ **Byte-Size Prefiltering:** Scans and indexes file sizes from system metadata, executing SHA-1 hashing calculations *only* on files with matching byte sizes.
* 🛡 **Resource-Aware Throttling:** Automatically balances system stability by scaling onto all available CPU cores minus 2 by default, keeping your operating system responsive during operations.
* 🔢 **Log Rotation:** Automatically indexes filenames sequentially (e.g., `_1.txt`, `_2.txt`), preventing report logs overwrites.
* 🗂 **Report Ordering:** Reports are grouped by duplicate set and sorted from the largest group to the smallest, so the most space-hungry duplicates appear first. The summary report also shows how much disk space you would save by keeping only one copy of each group.
* 📊 **Progress Bar:** App provides a rudimentary color-coded progress bar. Displays files processed, percentage complete, and elapsed time.
* 📂 **Default File Types**
>        "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", // Images
>        "mp4", "mkv", "avi", "mov", "flv", "wmv", "webm", // Videos
>        "mp3", "wav", "flac", "ogg", "m4a", "aac", // Audio
        
## Installation
* Rust release executables are standalone native machine code and do not require Rust or Cargo to be installed on the target machine. Ensure you give it executable permissions.

## Usage
* From command line or terminal run by passing your target directory as an argument.
* Double-click will spawn a terminal asking for the directory path. **Untested on MacOS**

**Windows Command Prompt / CMD**
# Standard Run (Uses optimized multi-threading with balance)
tree_sha1_scanner.exe --directory "C:\path\to\target_directory"

# Low Impact Run (Forces single-threaded execution to save system resources)
tree_sha1_scanner.exe  --directory "C:\path\to\target_directory" --min

# Custom Reports Output Destination
tree_sha1_scanner.exe --directory "C:\path\to\target_directory" --output "C:\path\to\logs"

# Scan only specific file types
tree_sha1_scanner.exe --directory "C:\path\to\target_directory" --types mp4,mkv,avi

# Show help
tree_sha1_scanner.exe --help

```bash
# Standard Run (Uses optimized multi-threading with balance)
./tree_sha1_scanner --directory "path/to/target_directory"

# Low Impact Run (Forces single-threaded execution to save system resources)
./tree_sha1_scanner  --directory "path/to/target_directory" --min

# Custom Reports Output Destination
./tree_sha1_scanner --directory "path/to/target_directory" --output "path/to/output_folder"

# Scan only specific file types
./tree_sha1_scanner --directory "path/to/target_directory" --types mp4,mkv,avi

# Show help
./tree_sha1_scanner --help
```

### CLI Arguments
* `-d, --directory <DIR>` — **Required.** The target directory path root to index and scan.
* `--min` — *Optional.* Forces execution into a single-threaded minimum performance mode.
* `-o, --output <OUTPUT_DIR>` — *Optional.* Folder path to save the generated report output files. If omitted, reports are saved to a `dupe_scans` folder inside the target directory.
* `-t, --types <EXTENSIONS>` — *Optional.* Comma-separated list of file extensions to scan (e.g. `--types mp4,mkv` or `--types .JPG --types .PNG`). If omitted, the app scans common media files by default: images (jpg, png, gif, bmp, tiff, webp), video (mp4, mkv, avi, mov, flv, wmv, webm), and audio (mp3, wav, flac, ogg, m4a, aac).

### What You See While the App Is Working

The app prints a short status line when it starts. During the hashing phase — the part that takes the longest — a **progress bar** appears in your terminal:

```
==>----->------------------------- 1,204/5,832 (20.6%) [00:00:07.421]
```

Reading it left to right:

| Part | What it means |
|---|---|
| `==>----->` | A visual bar that fills from left to right as files are processed. |
| `1,204/5,832` | How many candidate files have been hashed so far, out of the total candidates. |
| `(20.6%)` | The percentage of the hashing work that is done. |
| `[00:00:07.421]` | Elapsed time since hashing started (hours : minutes : seconds). |

Upon hashing completion bar is replaced by **"Hashing complete."** as report files are written.

> *The progress bar only appears while hashing is in progress. The initial directory walk and the final report-writing steps are quick and do not show a bar.*

## Generated Output Profiles

App writes up to three plain-text report files into your output folder. They are always numbered sequentially (e.g. `_1`, `_2`, …) so a new run never overwrites a previous one.

### 1. `<Folder>_tree_output_#.txt` — Folder map with duplicates

A visual directory tree, one entry per line. Folders appear as `FolderName/`. Only files that were flagged as duplicate candidates are listed, each shown as:

```
  photo_copy.jpg : a3f2…9c1e (4.25 MB)
```

Folders and files are listed in alphabetical order, so you can find them as you would in file explorer.

### 2. `<Folder>_output_#.csv` — Detailed duplicate registry

A tab-separated table with a header (`Size`, `File Path`, `SHA1`). Open in any spreadsheet program.  Rows are grouped by duplicate set. Sorted from the **largest total size to the smallest**.

*This file is automatically skipped if no duplicates are found.*

### 3. `<Folder>_DupesQ_#.txt` — Duplicate summary

A human-readable summary of every duplicate group, numbered starting at #0. Each group heading shows the SHA-1 fingerprint, the **total size**, and the **total savings** you would gain by keeping only one copy. Groups are ordered from the biggest total size down to the smallest.

If the scan finds no duplicates, the file simply contains the line: *No duplicate files found.*
