use clap::Parser;
use std::path::PathBuf;
use std::process;

mod scid;
mod pgn;

use scid::ScidDatabase;
use pgn::PgnExporter;

/// SCID to PGN Converter - MAJOR FIXES IMPLEMENTED (July 2025)
/// 
/// ## Critical Issues Resolved:
/// 
/// ### 1. Date Parsing Bug
/// **Problem**: Dates showed as "52298.152.207" instead of readable dates
/// **Fix**: Correct SCID bit-field extraction in index.rs
/// **Result**: Now shows proper dates like "1791.12.24"
/// 
/// ### 2. Name Extraction Bug  
/// **Problem**: Names extracted partially - "Michael" became "ichael"
/// **Fix**: Proper SCID .sn4 front-coded string parsing in names.rs
/// **Result**: Complete names extracted correctly
/// 
/// ### 3. Development Speed Optimization
/// **Addition**: --max-games=10 default for faster testing during development
/// **Purpose**: Avoid processing 1.8M games during debugging
/// 
/// ## Current Status:
/// ✅ Date parsing working
/// ✅ Name extraction working  
/// ✅ Basic PGN structure output
/// ❌ Game moves parsing (next major task)
/// 
/// ## Usage Examples:
/// ```bash
/// # Convert first 10 games (development mode)
/// ./scidtopgn database_name
/// 
/// # Convert all games
/// ./scidtopgn --max-games=0 database_name
/// 
/// # Specify output file
/// ./scidtopgn -o output.pgn database_name
/// ```
#[derive(Parser)]
#[command(name = "scidtopgn")]
#[command(about = "Convert SCID databases to PGN format")]
#[command(version = "0.1.0")]
struct Args {
    /// Path to the SCID database (without extension - will look for .si4, .sg4, .sn4)
    #[arg(value_name = "DATABASE")]
    database: PathBuf,
    
    /// Output PGN file (if not specified, uses database name with .pgn extension)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    
    /// Force overwrite existing output file
    #[arg(short, long)]
    force: bool,
    
    /// Include variations in PGN output
    #[arg(short, long)]
    variations: bool,
    
    /// Include comments in PGN output  
    #[arg(short, long)]
    comments: bool,
    
    /// Maximum number of games to export (0 = all games)
    #[arg(long, default_value = "10")]
    max_games: usize,
}

fn main() {
    let args = Args::parse();
    
    // Determine output file path
    let output_path = match args.output {
        Some(path) => path,
        None => {
            let mut path = args.database.clone();
            path.set_extension("pgn");
            path
        }
    };
    
    // Check if output file exists and we're not forcing overwrite
    if output_path.exists() && !args.force {
        eprintln!("Error: Output file '{}' already exists. Use --force to overwrite.", 
                 output_path.display());
        process::exit(1);
    }
    
    println!("Converting SCID database '{}' to PGN format...", args.database.display());
    
    // Load SCID database
    let mut database = match ScidDatabase::load(&args.database) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Error loading SCID database: {}", e);
            process::exit(1);
        }
    };
    
    println!("Loaded database with {} games", database.num_games());
    
    // Create PGN exporter
    let mut exporter = PgnExporter::new()
        .with_variations(args.variations)
        .with_comments(args.comments);
    
    if args.max_games > 0 {
        exporter = exporter.with_max_games(args.max_games);
    }
    
    // Export to PGN
    match exporter.export(&mut database, &output_path) {
        Ok(exported_count) => {
            println!("Successfully exported {} games to '{}'", 
                    exported_count, output_path.display());
        }
        Err(e) => {
            eprintln!("Error exporting to PGN: {}", e);
            process::exit(1);
        }
    }
}
