use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod table_display;

#[derive(Parser)]
#[command(
    author = "SCIDtoPGN Team",
    version = "0.1.0",
    about = "Convert SCID chess databases to PGN format",
    long_about = "A high-performance tool for parsing SCID chess database files and converting them to standard PGN format."
)]
pub struct Cli {
    /// Increase verbosity level
    #[arg(short, long)]
    pub verbose: bool,

    /// Enable progress reporting
    #[arg(short, long)]
    pub progress: bool,
    
    /// Force overwrite of existing output files
    #[arg(short, long)]
    pub force: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Parse a SCID database and export to PGN format
    Parse {
        /// Base path of SCID database (e.g., /path/to/db/mybase)
        #[arg(required = true)]
        database: PathBuf,

        /// Output PGN file path (default: database.pgn)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum number of games to process
        #[arg(short, long)]
        max_games: Option<usize>,

        /// Starting game index (0-based)
        #[arg(short, long, default_value = "0")]
        start_game: usize,

        /// Ending game index (exclusive)
        #[arg(short, long)]
        end_game: Option<usize>,

        /// Include move annotations and comments
        #[arg(long, default_value = "true")]
        include_annotations: bool,

        /// Include optional PGN tags (ELO, ECO, etc.)
        #[arg(long, default_value = "true")]
        include_optional_tags: bool,

        /// Validate moves during parsing
        #[arg(short, long)]
        validate_moves: bool,

        /// Output format (pgn, json, compact)
        #[arg(short, long, default_value = "pgn")]
        format: String,
    },
    
    /// Show metadata and information about a SCID database
    Info {
        /// Base path of SCID database
        #[arg(required = true)]
        database: PathBuf,
        
        /// Show detailed file information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Validate integrity of a SCID database
    Validate {
        /// Base path of SCID database
        #[arg(required = true)]
        database: PathBuf,
        
        /// Perform thorough validation (slower but more comprehensive)
        #[arg(short, long)]
        thorough: bool,
        
        /// Show validation results in JSON format
        #[arg(short, long)]
        json: bool,
    },
    
    /// List games in a SCID database
    List {
        /// Base path of SCID database
        #[arg(required = true)]
        database: PathBuf,
        
        /// Maximum number of games to list (default: 10)
        #[arg(short, long, default_value = "10")]
        max_games: usize,
        
        /// Show detailed game information
        #[arg(short, long)]
        detailed: bool,
        
        /// Format output as JSON
        #[arg(short, long)]
        json: bool,
    },
    
    /// Search for games matching criteria
    Search {
        /// Base path of SCID database
        #[arg(required = true)]
        database: PathBuf,
        
        /// Search by player name
        #[arg(short, long)]
        player: Option<String>,
        
        /// Search by event name
        #[arg(short, long)]
        event: Option<String>,
        
        /// Search by ECO code
        #[arg(short, long)]
        eco: Option<String>,
        
        /// Search by result (1-0, 0-1, 1/2-1/2, *)
        #[arg(short, long)]
        result: Option<String>,
        
        /// Minimum ELO rating
        #[arg(long)]
        min_elo: Option<u16>,
        
        /// Maximum ELO rating
        #[arg(long)]
        max_elo: Option<u16>,
        
        /// Year range (e.g., 2020-2023)
        #[arg(long)]
        year_range: Option<String>,
        
        /// Maximum number of results (default: 50)
        #[arg(short, long, default_value = "50")]
        max_results: usize,
        
        /// Development mode: limit processing for testing
        #[arg(long, default_value = "false")]
        dev_mode: bool,
        
        /// Development: maximum games to process (for testing)
        #[arg(long, default_value = "100")]
        dev_max_games: usize,
    },

    /// Development and testing utilities
    #[cfg(debug_assertions)]
    Dev {
        /// Run performance benchmarks
        #[arg(long)]
        benchmark: bool,
        
        /// Run memory usage analysis
        #[arg(long)]
        memory: bool,
        
        /// Test with specific database file
        #[arg(long)]
        test_db: Option<PathBuf>,
        
        /// Enable detailed debug output
        #[arg(long)]
        debug: bool,
    },
}

/// Progress reporting structure
#[derive(Debug, Clone)]
pub struct ProgressReporter {
    total_games: usize,
    processed_games: usize,
    start_time: std::time::Instant,
    verbose: bool,
}

impl ProgressReporter {
    pub fn new(total_games: usize, verbose: bool) -> Self {
        Self {
            total_games,
            processed_games: 0,
            start_time: std::time::Instant::now(),
            verbose,
        }
    }
    
    pub fn increment(&mut self) {
        self.processed_games += 1;
        
        if self.verbose && self.processed_games % 100 == 0 {
            let elapsed = self.start_time.elapsed();
            let games_per_sec = self.processed_games as f64 / elapsed.as_secs_f64();
            let progress = (self.processed_games as f64 / self.total_games as f64) * 100.0;
            
            eprintln!(
                "Progress: {}/{} games ({:.1}%) - {:.1} games/sec",
                self.processed_games, self.total_games, progress, games_per_sec
            );
        }
    }
    
    pub fn finish(&self) {
        if self.verbose {
            let elapsed = self.start_time.elapsed();
            let games_per_sec = self.processed_games as f64 / elapsed.as_secs_f64();
            
            eprintln!(
                "Completed: {} games in {:.2}s ({:.1} games/sec)",
                self.processed_games, elapsed.as_secs_f64(), games_per_sec
            );
        }
    }
}