# Tree SHA-1 Scanner

A command-line Rust tool to traverse directories, discover duplicate files, and generate structured layout trees and logs.

## Features
* ⚡ **Byte-Size Prefiltering:** Scans and indexes file sizes from system metadata instantly, executing heavy SHA-1 hashing calculations *only* on files with matching byte sizes.
* 🛡️ **Resource-Aware Throttling:** Automatically balances system stability by scaling onto all available CPU cores minus 2 (Moderate Profile) by default, keeping your operating system responsive during operations.
* 🔢 **Rotational Logs:** Automatically indexes filenames sequentially (e.g., `_1.txt`, `_2.txt`), preventing unintended folder namespace overwrites.
* 🗂️ **Clustered Outputs:** Generates structured reports ordered by file directory hierarchies and groups detected duplicates together by size.

## Installation 
Ensure you have the Rust toolchain installed. Clone this repository and compile the release binary:

```bash
cargo build --release
```

The compiled binary will be available at `./target/release/tree_sha1_scanner`.

## Usage

Run the scanner by passing your target directory as an argument. 

```bash
# Standard Run (Uses optimized multi-threading with Moderate profile balance)
cargo run --release -- --directory "path/to/target_directory"

# Low Impact Run (Forces single-threaded execution to save system resources)
cargo run --release -- --directory "path/to/target_directory" --min

# Custom Reports Output Destination
cargo run --release -- --directory "path/to/target_directory" --output "path/to/output_folder"

# Show Dynamic Generated Terminal Interface Help
cargo run --release -- --help
```

### CLI Arguments
* `-d, --directory <DIR>` — **Required.** The target directory path to index and scan.
* `-o, --output <OUTPUT_DIR>` — *Optional.* Folder path to save the generated report output files (defaults to the execution path directory).
* `    --min` — *Optional.* Forces execution into a single-threaded minimum performance mode.

## Generated Output Profiles
Processes target file hierarchies into up to three unique plain text profiles inside your designated output folder:

1. **`<Folder>_tree_output_#.txt`**
   A structured plain text directory architecture hierarchy layout mapping duplicate `filename : sha1` values.
2. **`<Folder>_output_#.csv`**
   A tab-separated log linking file paths directly to their SHA-1 signatures. *This file is automatically suppressed and skipped if zero duplicate items are found.*
3. **`<Folder>_DupesQ_#.txt`**
   Logs duplicate file clusters ordered by group size.
