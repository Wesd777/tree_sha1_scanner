# Tree SHA-1 Scanner

A fast multi-threaded command-line Rust tool to traverse directories, generate structured file/hash logs, and report duplicate files.

## Features
* 🚀 **Multi-threaded:** Leverages thread-pool scheduling to scale across multiple CPU cores.
* 🔢 **Rotational Logs:** Automatically indexes filenames sequentially (e.g., `_1.txt`, `_2.txt`) if matching reports exist, preserving past scan histories.
* 🗂️ **Alphabetical Logs:** Outputs are ordered by directory paths and individual filenames.

## Installation 
Ensure you have the Rust toolchain installed. Clone this repository and compile the release binary:

```bash
cargo build --release
```

The compiled binary will be available at `./target/release/tree_sha1_scanner`.

## Usage

Run the scanner by passing your target directory as an argument. By default, it operates on a footprint of **2 parallel worker threads**.

```bash
# Basic run (Uses default 4 threads)
cargo run --release -- --directory "path/to/target_directory"

# Faster run overriding thread limits explicitly
# Probably only of value if scanned contents are on an SSD or large numbers of small files
cargo run --release -- --directory "path/to/target_directory" --threads 8

# Set logs output path
cargo run --release -- --directory "path/to/target_directory" --output my_output_dir_path
```

### CLI Arguments
* `-d, --directory <DIR>` — **Required.** The target directory tree path to index.
* `-o, --output <OUTPUT_DIR>  Optional path to save report output files (defaults to current directory)
* `-t, --threads <INT>`   — *Optional.* Number of parallel threads to orchestrate (Defaults to `4`).

## Generated Output Profiles
Processes file data into three text files in the output directory (execution directory default):

1. **`<Folder>_tree_output_#.txt`**
   A plain text directory hierarchy tree layout containing `filename : sha1` .
2. **`<Folder>_output_#.csv`**
   A tab-separated filename + sha1 listing.
3. **`<Folder>_DupesQ_#.txt`**
   Duplicate sha1 files showing exact copy target paths.

