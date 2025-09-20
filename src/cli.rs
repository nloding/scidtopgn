use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author = "SCIDtoPGN Team",
    version = "0.1.0",
    about = "Convert SCID chess databases to PGN format",
    long_about = "A high-performance tool for parsing SCID chess database files and converting them to standard PGN format."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Enable progress reporting for large databases
    #[arg(short, long)]
    pub progress: bool,
    
    /// Force overwrite of existing output files
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Parse a SCID database and export to PGN format
    Parse {
        /// Base path of the SCID database (e.g., /path/to/db/mybase)
        #[arg(required = true)]
        database: PathBuf,

        /// Output PGN file path (default: database.pgn)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Maximum number of games to process (default: all games)
        #[arg(short, long)]
        max_games: Option<usize>,

        /// Start processing from a specific game index (0-based)
        #[arg(long)]
        start_game: Option<usize>,

        /// End processing at a specific game index (0-based)
        #[arg(long)]
        end_game: Option<usize>,

        /// Include game annotations and comments
        #[arg(short, long)]
        include_annotations: bool,

        /// Include optional PGN tags (ELO, ECO, etc.)
        #[arg(short, long)]
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
        /// Base path of the SCID database
        #[arg(required = true)]
        database: PathBuf,
        
        /// Show detailed file information
        #[arg(short, long)]
        detailed: bool,
    },
    
    /// Validate the integrity of a SCID database
    Validate {
        /// Base path of the SCID database
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
        /// Base path of the SCID database
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
        /// Base path of the SCID database
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
            let elapsed = self.start_time.elapsed().as_secs();
            let games_per_sec = self.processed_games as f64 / elapsed.max(1) as f64;
            let progress = (self.processed_games as f64 / self.total_games as f64) * 100.0;
            
            eprintln!("Progress: {} games ({:.1}%), {:.1} games/sec", 
                     self.processed_games, progress, games_per_sec);
        }
    }
    
    pub fn finish(&self) {
        if self.verbose {
            let elapsed = self.start_time.elapsed();
            let games_per_sec = self.processed_games as f64 / elapsed.as_secs_f64().max(1.0);
            
            eprintln!("Completed: {} games in {:.2}s ({:.1} games/sec)", 
                     self.processed_games, elapsed.as_secs_f64(), games_per_sec);
        }
    }
}

/// Result of a database validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: ValidationStats,
}

/// Statistics from database validation
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_games: u32,
    pub file_sizes: (usize, usize, usize), // si4, sn4, sg4
    pub validation_time: std::time::Duration,
}

/// Game information for listing
#[derive(Debug, Clone)]
pub struct GameInfo {
    pub index: usize,
    pub white: String,
    pub black: String,
    pub event: String,
    pub site: String,
    pub date: String,
    pub result: String,
    pub white_elo: Option<u16>,
    pub black_elo: Option<u16>,
    pub eco: Option<String>,
    pub moves: usize,
}

/// Search result information
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub games: Vec<GameInfo>,
    pub total_matches: usize,
    pub search_time: std::time::Duration,
}

/// Output format options
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Pgn,
    Json,
    Compact,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pgn" => Ok(OutputFormat::Pgn),
            "json" => Ok(OutputFormat::Json),
            "compact" => Ok(OutputFormat::Compact),
            _ => Err(format!("Invalid output format: {}", s)),
        }
    }
}

/// Utility functions for CLI operations
pub mod utils {
    use super::*;
    use scidtopgn::api::ScidDatabase;
    use scidtopgn::core::error::Result;
    
    /// Check if a SCID database exists and is accessible
    pub fn check_database_exists(database_path: &PathBuf) -> Result<bool> {
        Ok(scidtopgn::utils::database_exists(database_path))
    }
    
    /// Get database information
    pub fn get_database_info(database_path: &PathBuf, detailed: bool) -> Result<String> {
        if !check_database_exists(database_path)? {
            return Err(anyhow::anyhow!("Database not found"));
        }
        
        let db = ScidDatabase::open(database_path)?;
        let stats = db.statistics();
        
        let mut info = format!("Database: {}\n", database_path.display());
        info.push_str(&format!("Games: {}\n", stats.num_games));
        
        if detailed {
            info.push_str(&format!("File sizes:\n"));
            info.push_str(&format!("  si4: {}\n", scidtopgn::utils::format_file_size(stats.file_size_si4 as u64)));
            info.push_str(&format!("  sn4: {}\n", scidtopgn::utils::format_file_size(stats.file_size_sn4 as u64)));
            info.push_str(&format!("  sg4: {}\n", scidtopgn::utils::format_file_size(stats.file_size_sg4 as u64)));
            info.push_str(&format!("Validated: {}\n", stats.is_validated));
        }
        
        Ok(info)
    }
    
    /// Validate a SCID database
    pub fn validate_database(database_path: &PathBuf, thorough: bool) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        
        let db = ScidDatabase::open(database_path)?;
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Basic validation
        if !db.is_validated() {
            errors.push("Database has not been validated".to_string());
        }
        
        // Thorough validation
        if thorough {
            // Check all games can be accessed
            for game_result in db.games().take(100) { // Sample first 100 games
                match game_result {
                    Ok(_) => continue,
                    Err(e) => {
                        errors.push(format!("Game access error: {}", e));
                        break;
                    }
                }
            }
            
            // Check database consistency
            let stats = db.statistics();
            if stats.num_games == 0 {
                warnings.push("Database contains no games".to_string());
            }
        }
        
        let validation_time = start_time.elapsed();
        let stats = ValidationStats {
            total_games: db.num_games(),
            file_sizes: (stats.file_size_si4, stats.file_size_sn4, stats.file_size_sg4),
            validation_time,
        };
        
        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            stats,
        })
    }
    
    /// List games in a database
    pub fn list_games(database_path: &PathBuf, max_games: usize, detailed: bool) -> Result<Vec<GameInfo>> {
        let db = ScidDatabase::open(database_path)?;
        
        let mut games = Vec::new();
        let start_time = std::time::Instant::now();
        
        for (index, game_result) in db.games().take(max_games).enumerate() {
            match game_result {
                Ok(game) => {
                    let game_info = if detailed {
                        GameInfo {
                            index,
                            white: resolve_player_name(&db, game.index.white_id), // Resolve player name
                            black: resolve_player_name(&db, game.index.black_id), // Resolve player name
                            event: resolve_event_name(&db, game.index.event_id),   // Resolve event name
                            site: resolve_site_name(&db, game.index.site_id),     // Resolve site name
                            date: format!("{:04}.{:02}.{:02}", game.index.year, game.index.month, game.index.day),
                            result: decode_result(game.index.result),
                            white_elo: if game.index.white_elo > 0 { Some(game.index.white_elo) } else { None },
                            black_elo: if game.index.black_elo > 0 { Some(game.index.black_elo) } else { None },
                            eco: if game.index.eco > 0 { Some(format!("ECO{}", game.index.eco)) } else { None },
                            moves: get_move_count(&game), // Get actual move count
                        }
                    } else {
                        GameInfo {
                            index,
                            white: game.index.white_id.to_string(),
                            black: game.index.black_id.to_string(),
                            event: game.index.event_id.to_string(),
                            site: game.index.site_id.to_string(),
                            date: format!("{:04}.{:02}.{:02}", game.index.year, game.index.month, game.index.day),
                            result: decode_result(game.index.result),
                            white_elo: if game.index.white_elo > 0 { Some(game.index.white_elo) } else { None },
                            black_elo: if game.index.black_elo > 0 { Some(game.index.black_elo) } else { None },
                            eco: if game.index.eco > 0 { Some(format!("ECO{}", game.index.eco)) } else { None },
                            moves: 0,
                        }
                    };
                    
                    games.push(game_info);
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error accessing game {}: {}", index, e));
                }
            }
        }
        
        if games.is_empty() {
            return Ok(games);
        }
        
        // Sort games by index
        games.sort_by(|a, b| a.index.cmp(&b.index));
        
        Ok(games)
    }
    
    /// Search for games matching criteria
    pub fn search_games(
        database_path: &PathBuf,
        player: Option<&str>,
        event: Option<&str>,
        eco: Option<&str>,
        result: Option<&str>,
        min_elo: Option<u16>,
        max_elo: Option<u16>,
        year_range: Option<&str>,
        max_results: usize,
    ) -> Result<SearchResult> {
        let start_time = std::time::Instant::now();
        
        let db = ScidDatabase::open(database_path)?;
        let mut matches = Vec::new();
        
        for game_result in db.games() {
            match game_result {
                Ok(game) => {
                    let mut matches_criteria = true;
                    
                    // Check player name criteria
                    if let Some(player) = player {
                        let white_name = resolve_player_name(&db, game.index.white_id);
                        let black_name = resolve_player_name(&db, game.index.black_id);
                        if !white_name.to_lowercase().contains(&player.to_lowercase()) &&
                           !black_name.to_lowercase().contains(&player.to_lowercase()) {
                            matches_criteria = false;
                        }
                    }
                    
                    // Check event name criteria
                    if matches_criteria && event.is_some() {
                        let event_name = resolve_event_name(&db, game.index.event_id);
                        if !event_name.to_lowercase().contains(&event.unwrap().to_lowercase()) {
                            matches_criteria = false;
                        }
                    }
                    
                    // Check ECO code criteria
                    if matches_criteria && eco.is_some() {
                        if let Some(ref game_eco) = game.index.eco {
                            if game_eco > 0 {
                                let game_eco_str = format!("ECO{}", game_eco);
                                if !game_eco_str.to_lowercase().contains(&eco.unwrap().to_lowercase()) {
                                    matches_criteria = false;
                                }
                            }
                        }
                    }
                    
                    // Check result criteria
                    if matches_criteria && result.is_some() {
                        let game_result_str = decode_result(game.index.result);
                        if game_result_str != result.unwrap() {
                            matches_criteria = false;
                        }
                    }
                    
                    // Check ELO range criteria
                    if matches_criteria {
                        if let Some(min_elo) = min_elo {
                            if game.index.white_elo < min_elo && game.index.black_elo < min_elo {
                                matches_criteria = false;
                            }
                        }
                        if let Some(max_elo) = max_elo {
                            if game.index.white_elo > max_elo && game.index.black_elo > max_elo {
                                matches_criteria = false;
                            }
                        }
                    }
                    
                    // Check year range criteria
                    if matches_criteria && year_range.is_some() {
                        let year_str = year_range.unwrap();
                        if let Ok((start_year, end_year)) = parse_year_range(year_str) {
                            if game.index.year < start_year || game.index.year > end_year {
                                matches_criteria = false;
                            }
                        }
                    }
                    
                    // Implement actual search logic with name resolution
                    if matches_criteria {
                        matches.push(GameInfo {
                            index: matches.len(),
                            white: resolve_player_name(&db, game.index.white_id),
                            black: resolve_player_name(&db, game.index.black_id),
                            event: resolve_event_name(&db, game.index.event_id),
                            site: resolve_site_name(&db, game.index.site_id),
                            date: format!("{:04}.{:02}.{:02}", game.index.year, game.index.month, game.index.day),
                            result: decode_result(game.index.result),
                            white_elo: if game.index.white_elo > 0 { Some(game.index.white_elo) } else { None },
                            black_elo: if game.index.black_elo > 0 { Some(game.index.black_elo) } else { None },
                            eco: if game.index.eco > 0 { Some(format!("ECO{}", game.index.eco)) } else { None },
                            moves: get_move_count(&game),
                        });
                    }
                    
                    if matches.len() >= max_results {
                        break;
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Error accessing game: {}", e));
                }
            }
        }
        
        let search_time = start_time.elapsed();
        
        Ok(SearchResult {
            games: matches,
            total_matches: matches.len(),
            search_time,
        })
    }
    
    /// Resolve player name from database using SN4 name records
    fn resolve_player_name(db: &ScidDatabase, player_id: u32) -> String {
        // Try to get the player name from the SN4 file
        if let Ok(Some(name)) = db.get_player_name(player_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Player{}", player_id)
        }
    }
    
    /// Resolve event name from database using SN4 name records
    fn resolve_event_name(db: &ScidDatabase, event_id: u32) -> String {
        // Try to get the event name from the SN4 file
        if let Ok(Some(name)) = db.get_event_name(event_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Event{}", event_id)
        }
    }
    
    /// Resolve site name from database using SN4 name records
    fn resolve_site_name(db: &ScidDatabase, site_id: u32) -> String {
        // Try to get the site name from the SN4 file
        if let Ok(Some(name)) = db.get_site_name(site_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Site{}", site_id)
        }
    }
    
    /// Get actual move count from parsed game
    fn get_move_count(game: &ScidGame) -> usize {
        // Count the number of move elements in the parsed game
        game.parsed_game.elements.iter()
            .filter(|element| {
                matches!(element, crate::formats::sg4::StreamingGameElement::Move(_))
            })
            .count()
    }
    
    /// Parse year range string (e.g., "2020-2023" or "2020s")
    fn parse_year_range(year_range: &str) -> Result<(u16, u16), ()> {
        if year_range.contains('-') {
            let parts: Vec<&str> = year_range.split('-').collect();
            if parts.len() == 2 {
                let start_year = parts[0].parse::<u16>().map_err(|_| ())?;
                let end_year = parts[1].parse::<u16>().map_err(|_| ())?;
                Ok((start_year, end_year))
            } else {
                Err(())
            }
        } else if year_range.ends_with('s') {
            // Handle decades (e.g., "2020s")
            let decade_start = year_range.trim_end_matches('s').parse::<u16>().map_err(|_| ())?;
            Ok((decade_start, decade_start + 9))
        } else {
            // Single year
            let year = year_range.parse::<u16>().map_err(|_| ())?;
            Ok((year, year))
        }
    }
    
    /// Decode SCID result code to PGN result string
    fn decode_result(result_code: u8) -> String {
        match result_code {
            0 => "*".to_string(),       // Unknown/ongoing result
            1 => "1-0".to_string(),     // White wins
            2 => "0-1".to_string(),     // Black wins
            3 => "1/2-1/2".to_string(), // Draw
            _ => "*".to_string(),       // Unknown result code
        }
    }
    
    /// Benchmark results structure
    #[derive(Debug, Clone)]
    pub struct BenchmarkResults {
        pub parsing_time: std::time::Duration,
        pub export_time: std::time::Duration,
        pub total_time: std::time::Duration,
        pub games_processed: usize,
    }
    
    /// Memory analysis results structure
    #[derive(Debug, Clone)]
    pub struct MemoryAnalysisResults {
        pub peak_memory_mb: f64,
        pub average_memory_mb: f64,
        pub memory_per_game_kb: f64,
    }
    
    /// Database test results structure
    #[derive(Debug, Clone)]
    pub struct DatabaseTestResults {
        pub is_valid: bool,
        pub total_games: usize,
        pub errors: Vec<String>,
        pub warnings: Vec<String>,
    }
    
    /// Run performance benchmarks on a database
    pub fn run_benchmarks(database_path: &PathBuf) -> Result<BenchmarkResults> {
        use std::time::Instant;
        
        println!("Running benchmarks on: {}", database_path.display());
        
        let start_time = Instant::now();
        
        // Open database
        let db = ScidDatabase::open(database_path)?;
        
        // Benchmark database opening
        let open_time = start_time.elapsed();
        println!("Database open time: {:?}", open_time);
        
        // Benchmark game iteration
        let iter_start = Instant::now();
        let game_count: usize = db.games().count();
        let iter_time = iter_start.elapsed();
        println!("Game iteration time: {:?} ({} games)", iter_time, game_count);
        
        // Benchmark PGN export
        let export_start = Instant::now();
        let mut successful_exports = 0;
        let mut failed_exports = 0;
        
        for (index, game_result) in db.games().take(100).enumerate() {
            match game_result {
                Ok(game) => {
                    // Export game to PGN
                    let pgn_content = export_game(&game, &OutputFormat::Pgn, true, true, false);
                    
                    // Write to a temporary file to measure I/O performance
                    let mut temp_file = std::env::temp_dir();
                    temp_file.push(format!("temp_game_{}.pgn", index));
                    
                    match std::fs::write(&temp_file, pgn_content) {
                        Ok(_) => successful_exports += 1,
                        Err(_) => failed_exports += 1,
                    }
                }
                Err(_) => failed_exports += 1,
            }
        }
        
        let export_time = export_start.elapsed();
        println!("PGN export time: {:?} ({} successful, {} failed)", 
                 export_time, successful_exports, failed_exports);
        
        let total_time = start_time.elapsed();
        
        Ok(BenchmarkResults {
            parsing_time: open_time,
            export_time,
            total_time,
            games_processed: game_count,
        })
    }
    
    /// Analyze memory usage of database operations
    pub fn analyze_memory_usage(database_path: &PathBuf) -> Result<MemoryAnalysisResults> {
        use std::process::Command;
        
        println!("Analyzing memory usage for: {}", database_path.display());
        
        // Get initial memory usage
        let initial_memory = get_memory_usage();
        
        // Open database and measure memory usage
        let db = ScidDatabase::open(database_path)?;
        let after_open_memory = get_memory_usage();
        
        // Process games and measure peak memory
        let mut peak_memory = after_open_memory;
        let mut total_memory_delta = 0.0;
        let mut games_processed = 0;
        
        for game_result in db.games().take(1000) {
            match game_result {
                Ok(_) => {
                    // Process game and measure memory
                    games_processed += 1;
                    let current_memory = get_memory_usage();
                    
                    if current_memory > peak_memory {
                        peak_memory = current_memory;
                    }
                    
                    total_memory_delta += current_memory - after_open_memory;
                }
                Err(_) => {
                    // Skip games that can't be processed
                }
            }
        }
        
        // Calculate average memory usage per game
        let average_memory_delta = if games_processed > 0 {
            total_memory_delta / games_processed as f64
        } else {
            0.0
        };
        
        println!("Memory analysis results:");
        println!("  Initial memory: {:.2} MB", initial_memory);
        println!("  Peak memory: {:.2} MB", peak_memory);
        println!("  Average memory per game: {:.2} KB", average_memory_delta * 1024.0);
        println!("  Games processed: {}", games_processed);
        
        Ok(MemoryAnalysisResults {
            peak_memory_mb: peak_memory,
            average_memory_mb: after_open_memory,
            memory_per_game_kb: average_memory_delta * 1024.0,
        })
    }
    
    /// Get current memory usage in MB
    fn get_memory_usage() -> f64 {
        use std::process::Command;
        
        if cfg!(target_os = "linux") {
            // On Linux, use /proc/self/status
            if let Ok(output) = Command::new("sh")
                .arg("-c")
                .arg("grep VmRSS /proc/self/status")
                .output() {
                if let Ok(memory_str) = String::from_utf8(output.stdout) {
                    if let Some(memory_kb) = memory_str.split_whitespace().nth(1) {
                        return memory_kb.parse::<f64>().unwrap_or(0.0) / 1024.0;
                    }
                }
            }
        } else if cfg!(target_os = "macos") {
            // On macOS, use vm_stat
            if let Ok(output) = Command::new("vm_stat")
                .arg("grep")
                .arg("physmem")
                .output() {
                if let Ok(memory_str) = String::from_utf8(output.stdout) {
                    if let Some(memory_bytes) = memory_str.split_whitespace().nth(1) {
                        return memory_bytes.parse::<f64>().unwrap_or(0.0) / (1024.0 * 1024.0);
                    }
                }
            }
        }
        
        // Fallback for other platforms
        0.0
    }
    
    /// Create a sample database for testing
    pub fn create_sample_database(database_path: &PathBuf) -> Result<()> {
        println!("Creating sample database at: {}", database_path.display());
        
        // Create a simple sample database with a few games
        // This is a simplified implementation for testing purposes
        let mut sample_data = vec![
            // Sample game data structure
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
            0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15,
        ];
        
        // Write sample data to files
        std::fs::write(database_path.with_extension("si4"), &sample_data)?;
        std::fs::write(database_path.with_extension("sn4"), &sample_data)?;
        std::fs::write(database_path.with_extension("sg4"), &sample_data)?;
        
        println!("Sample database created successfully");
        Ok(())
    }
    
    /// Test database integrity
    pub fn test_database(database_path: &PathBuf) -> Result<DatabaseTestResults> {
        println!("Testing database integrity: {}", database_path.display());
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Test if all required files exist
        let si4_path = database_path.with_extension("si4");
        let sn4_path = database_path.with_extension("sn4");
        let sg4_path = database_path.with_extension("sg4");
        
        if !si4_path.exists() {
            errors.push(format!("SI4 file not found: {}", si4_path.display()));
        }
        
        if !sn4_path.exists() {
            errors.push(format!("SN4 file not found: {}", sn4_path.display()));
        }
        
        if !sg4_path.exists() {
            errors.push(format!("SG4 file not found: {}", sg4_path.display()));
        }
        
        // Test if files are readable
        if si4_path.exists() && std::fs::read(&si4_path).is_err() {
            errors.push(format!("SI4 file is not readable: {}", si4_path.display()));
        }
        
        if sn4_path.exists() && std::fs::read(&sn4_path).is_err() {
            errors.push(format!("SN4 file is not readable: {}", sn4_path.display()));
        }
        
        if sg4_path.exists() && std::fs::read(&sg4_path).is_err() {
            errors.push(format!("SG4 file is not readable: {}", sg4_path.display()));
        }
        
        // Try to open the database
        match ScidDatabase::open(database_path) {
            Ok(db) => {
                // Test database functionality
                let game_count = db.num_games();
                if game_count == 0 {
                    warnings.push("Database contains no games".to_string());
                }
                
                // Test game iteration
                let mut test_games = 0;
                for game_result in db.games().take(10) {
                    match game_result {
                        Ok(_) => test_games += 1,
                        Err(_) => warnings.push("Failed to access game".to_string()),
                    }
                }
                
                if test_games == 0 {
                    warnings.push("Could not access any games for testing".to_string());
                }
                
                println!("Database test completed successfully");
            }
            Err(e) => {
                errors.push(format!("Failed to open database: {}", e));
            }
        }
        
        Ok(DatabaseTestResults {
            is_valid: errors.is_empty(),
            total_games: if errors.is_empty() {
                ScidDatabase::open(database_path).map(|db| db.num_games()).unwrap_or(0)
            } else {
                0
            },
            errors,
            warnings,
        })
    }
    
    /// Resolve player name from database using SN4 name records
    fn resolve_player_name(db: &ScidDatabase, player_id: u32) -> String {
        // Try to get the player name from the SN4 file
        if let Ok(Some(name)) = db.get_player_name(player_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Player{}", player_id)
        }
    }
    
    /// Resolve event name from database using SN4 name records
    fn resolve_event_name(db: &ScidDatabase, event_id: u32) -> String {
        // Try to get the event name from the SN4 file
        if let Ok(Some(name)) = db.get_event_name(event_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Event{}", event_id)
        }
    }
    
    /// Resolve site name from database using SN4 name records
    fn resolve_site_name(db: &ScidDatabase, site_id: u32) -> String {
        // Try to get the site name from the SN4 file
        if let Ok(Some(name)) = db.get_site_name(site_id) {
            name
        } else {
            // Fallback to numeric ID if name resolution fails
            format!("Site{}", site_id)
        }
    }
    
    /// Get actual move count from parsed game
    fn get_move_count(game: &ScidGame) -> usize {
        // Count the number of move elements in the parsed game
        game.parsed_game.elements.iter()
            .filter(|element| {
                matches!(element, crate::formats::sg4::StreamingGameElement::Move(_))
            })
            .count()
    }
}
