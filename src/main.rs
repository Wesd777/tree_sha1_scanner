use clap::Parser;
use is_terminal::IsTerminal;
use std::io::{self, Write};
use std::path::PathBuf;

mod app;
mod crypto;
mod reports;
mod utils;

#[derive(Parser, Debug)]
#[command(
    name = "tree_sha1_scanner",
    version,				// ← uses env!("CARGO_PKG_VERSION")
    author = "Wesd777 (https://github.com)",
    about = "A fast media duplicate scanner."
)]
pub struct Args {
    /// Path to the target directory
    #[arg(short, long, value_name = "DIR")]
    pub directory: Option<PathBuf>,

    /// Optional path to save report output files (defaults to a 'scans' folder inside the target directory)
    #[arg(short, long, value_name = "OUTPUT_DIR")]
    pub output: Option<PathBuf>,

    /// Run in single-threaded minimum performance mode to save system resources
    #[arg(long)]
    pub min: bool,

    /// Custom file extensions to scan (comma-separated or repeated; e.g. --types mp4,mkv or --types .MKV --types .avi)
    #[arg(short = 't', long = "types", value_name = "EXTENSIONS", value_delimiter = ',')]
    pub types: Vec<String>,
}

fn main() -> io::Result<()> {
    let standard_input_attached = io::stdin().is_terminal();
    let no_arguments_passed = std::env::args().len() == 1;
    let is_graphical_double_click = no_arguments_passed && standard_input_attached;

    let mut args = if is_graphical_double_click {
        println!("=== Tree SHA-1 Scanner ===");
        print!("Enter the target directory path to scan: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let trimmed = input.trim();

        if trimmed.is_empty() {
            eprintln!("Error: No directory path provided.");
            print!("\nPress Enter to exit...");
            io::stdout().flush()?;
            let mut tmp = String::new();
            let _ = io::stdin().read_line(&mut tmp);
            std::process::exit(1);
        }

        Args {
            directory: Some(PathBuf::from(trimmed)),
            output: None,
            min: false,
            types: Vec::new(),
        }
    } else if no_arguments_passed && !standard_input_attached {
        eprintln!("Error: This application must be run inside a terminal environment.");
        eprintln!("Usage example: ./tree_sha1_scanner --directory /path/to/scan");
        std::process::exit(1);
    } else {
        Args::parse()
    };

    let target_directory = match args.directory.take() {
        Some(path) => path,
        None => {
            eprintln!(
                "Error: The following required arguments were not provided:\n  --directory <DIR>"
            );
            std::process::exit(1);
        }
    };

    if !target_directory.is_dir() {
        eprintln!(
            "Error: Target path is not a directory or does not exist: {:?}",
            target_directory
        );
        if is_graphical_double_click {
            print!("\nPress Enter to exit...");
            io::stdout().flush()?;
            let mut tmp = String::new();
            let _ = io::stdin().read_line(&mut tmp);
        }
        std::process::exit(1);
    }

    let resolved_output_dir = match args.output {
        Some(out_path) => out_path,
        None => target_directory.join("dupe_scans"),
    };

    if !resolved_output_dir.exists() {
        std::fs::create_dir_all(&resolved_output_dir)?;
    }

    // Default to total cores - 2, fallback to 1 core if --min is specified
    let thread_count = if args.min {
        1
    } else {
        std::thread::available_parallelism()
            .map(|n| (n.get() as isize - 2).max(1) as usize)
            .unwrap_or(1)
    };

    // Propagate the error instead of panicking; if a global pool already exists
    // (should not happen in a single-shot CLI), fall back to the default pool.
    if let Err(e) = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()
    {
        eprintln!(
            "Warning: could not configure Rayon thread pool ({}); using default.",
            e
        );
    }

    let result = app::run(&target_directory, Some(&resolved_output_dir), &args.types);

    if is_graphical_double_click {
        print!("Press Enter to close window...");
        io::stdout().flush()?;
        let mut tmp = String::new();
        let _ = io::stdin().read_line(&mut tmp);
    }

    result
}