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
    version = "0.1.0",
    author = "Wesd777 (https://github.com)",
    about = "A fast media duplicate scanner."
)]
pub struct Args {
    /// Path to the target directory
    #[arg(short, long, value_name = "DIR")]
    pub directory: Option<PathBuf>, // Option configuration for interactive fallback

    /// Optional path to save report output files (defaults to a 'scans' folder inside the target directory)
    #[arg(short, long, value_name = "OUTPUT_DIR")]
    pub output: Option<PathBuf>,

    /// Run in single-threaded minimum performance mode to save system resources
    #[arg(long)]
    pub min: bool,
}

fn main() -> io::Result<()> {
    // Detect if the program was double-clicked launched
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
            output: None, // Will default to target_directory/scans below
            min: false,
        }
    } else if no_arguments_passed && !standard_input_attached {
        eprintln!("Error: This application must be run inside a terminal environment.");
        eprintln!("Usage example: ./tree_sha1_scanner --directory /path/to/scan");
        std::process::exit(1);
    } else {
        Args::parse()
    };

    // Extract target path safely
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

    // If provided via --output, use it. Otherwise, default to target_directory/dupe_scans globally.
    let resolved_output_dir = match args.output {
        Some(out_path) => out_path,
        None => target_directory.join("dupe_scans"),
    };

    // Guarantee directory existence safely
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

    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()
        .unwrap();

    let result = app::run(&target_directory, Some(&resolved_output_dir));

    // Keep window open at the end if we used interactive terminal prompting
    if is_graphical_double_click {
        print!("Press Enter to close window...");
        io::stdout().flush()?;
        let mut tmp = String::new();
        let _ = io::stdin().read_line(&mut tmp);
    }

    result
}
