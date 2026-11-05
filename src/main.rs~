use clap::Parser;
use std::io;
use std::path::PathBuf;

mod app;
mod crypto;
mod reports;
mod utils;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The path to the target directory
    #[arg(short, long, value_name = "DIR")]
    pub directory: PathBuf,

    /// Optional path to save report output files (defaults to current directory)
    #[arg(short, long, value_name = "OUTPUT_DIR")]
    pub output: Option<PathBuf>,

    /// Number of threads to use
    #[arg(short, long, value_name = "INT", default_value = "2")]
    pub threads: usize,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    if args.directory.is_dir() {
        println!("Successfully passed target directory: {:?}", args.directory);
    } else {
        println!("Error: Target path is not a directory or does not exist: {:?}", args.directory);
        std::process::exit(1);
    }

    if let Some(ref out_dir) = args.output {
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir)?;
            println!("Created output directory: {:?}", out_dir);
        }
    }

    // Configure Rayon global thread limits explicitly
    rayon::ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();
    println!("Thread pool initialized with {} threads.", args.threads);

    // Target directory and optional output destination
    app::run(&args.directory, args.output.as_deref())
}

