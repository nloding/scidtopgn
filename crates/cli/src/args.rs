//! CLI Argument Parsing

use clap::Parser;

/// SCID to PGN Converter - Command Line Interface
///
/// Converts SCID chess database files (.si4, .sn4, .sg4) to PGN format.
#[derive(Parser)]
#[command(
    name = "scidtopgn",
    about = "Convert SCID chess databases to PGN format",
    long_about = "Convert SCID chess database files (.si4, .sn4, .sg4) to PGN format.\n\nThe tool reads SCID binary databases and outputs valid PGN files with proper tags, moves, and optional content like comments and variations."
)]
pub struct Args {
    /// Path to SCID database (without extension)
    ///
    /// The database consists of three files (.si4, .sn4, .sg4).
    /// Specify the base path without extension.
    ///
    /// Examples:
    ///   scidtopgn database
    ///   scidtopgn /path/to/database
    ///   scidtopgn ./my-database
    pub input: std::path::PathBuf,

    /// Output file (writes to stdout if not specified)
    ///
    /// Use '-' to explicitly write to stdout.
    ///
    /// Examples:
    ///   -o output.pgn
    ///   -o /path/to/output.pgn
    ///   -o -  (stdout)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<std::path::PathBuf>,

    /// Compact output (no line breaks in movetext)
    ///
    /// Reduces file size but makes PGN less readable.
    /// Useful for machine processing.
    #[arg(long)]
    pub compact: bool,

    /// Exclude comments from output
    ///
    /// Default is to include comments if present.
    #[arg(long)]
    pub no_comments: bool,

    /// Exclude variations from output
    ///
    /// Default is to include variations if present.
    #[arg(long)]
    pub no_variations: bool,

    /// Convert only specific games (1-based indexing)
    ///
    /// Format: START-END or START or -END or START-
    ///
    /// Examples:
    ///   --range 1-100     (first 100 games)
    ///   --range 50        (only game 50)
    ///   --range 100-      (from game 100 to end)
    ///   --range -50       (first 50 games)
    #[arg(short = 'r', long, value_name = "RANGE")]
    pub range: Option<String>,

    /// Filter games by player name (partial match)
    ///
    /// Matches White or Black player.
    /// Case-insensitive.
    ///
    /// Example:
    ///   --player Carlsen
    #[arg(long)]
    pub player: Option<String>,

    /// Filter games where both players >= rating
    ///
    /// Example:
    ///   --min-elo 2700
    #[arg(long)]
    pub min_elo: Option<u16>,

    /// Show progress and details
    ///
    /// Displays progress bar for databases > 1000 games.
    /// Shows game count, time elapsed, etc.
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Suppress all non-essential output
    ///
    /// Only errors are printed.
    /// Useful in scripts.
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Show database information and exit
    ///
    /// Displays:
    ///   - Database description
    ///   - Number of games
    ///   - Version
    ///   - File sizes
    ///   - Player/event/site counts
    #[arg(long)]
    pub info: bool,

    /// Error handling mode for game parsing failures
    ///
    /// Controls how the tool handles games that fail to parse:
    ///   - strict:      Stop on first error (default)
    ///   - lenient:     Skip failed games, continue processing
    ///   - best-effort: Output partial games when possible
    ///
    /// Examples:
    ///   --error-mode lenient
    ///   --error-mode best-effort
    #[arg(long)]
    pub error_mode: Option<String>,

    /// Maximum errors before stopping (lenient/best-effort mode)
    ///
    /// After this many errors, stop processing.
    /// Use 0 for unlimited. Default: 0
    ///
    /// Example:
    ///   --error-mode lenient --max-errors 100
    #[arg(long)]
    #[arg(value_name = "N")]
    #[arg(default_value = "0")]
    pub max_errors: usize,

    /// Include partially decoded games in output (best-effort mode)
    ///
    /// When a game fails mid-parse, output the moves decoded so far.
    /// Adds a comment noting where parsing failed.
    /// Only effective with --error-mode best-effort.
    #[arg(long)]
    pub include_partial: bool,

    /// Force memory-mapped file access for large databases
    ///
    /// Memory mapping can improve performance for large databases
    /// by letting the OS handle file caching efficiently.
    /// By default, files are read entirely into memory.
    #[arg(long)]
    pub mmap: bool,

    /// Auto-enable memory mapping above this file size
    ///
    /// Files larger than this threshold automatically use memory mapping.
    /// Specify size with suffix: 50M, 1G, 500K
    /// Default: 100M (100 megabytes)
    ///
    /// Examples:
    ///   --mmap-threshold 50M    Enable mmap for files > 50MB
    ///   --mmap-threshold 1G     Enable mmap for files > 1GB
    #[arg(long)]
    #[arg(value_name = "SIZE")]
    #[arg(default_value = "100M")]
    pub mmap_threshold: String,
}

impl Args {
    /// Validate arguments and return errors if invalid
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref range) = self.range {
            Self::parse_range(range)?;
        }

        if let Some(ref output) = self.output {
            if output.as_os_str() != "-" {
                if let Some(parent) = output.parent() {
                    if !parent.exists() {
                        return Err(format!(
                            "Output directory does not exist: {}",
                            parent.display()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse range string into (start, end) tuple
    ///
    /// Returns (Some(start), Some(end)) for inclusive range.
    /// Uses None to indicate "from beginning" or "to end".
    pub fn parse_range(range: &str) -> Result<(Option<usize>, Option<usize>), String> {
        if !range.contains('-') {
            let num = range
                .parse::<usize>()
                .map_err(|_| format!("Invalid game number: {}", range))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            return Ok((Some(num), Some(num)));
        }

        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format: {} (use START-END)", range));
        }

        let start = if parts[0].is_empty() {
            None
        } else {
            let num = parts[0]
                .parse::<usize>()
                .map_err(|_| format!("Invalid start number: {}", parts[0]))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            Some(num)
        };

        let end = if parts[1].is_empty() {
            None
        } else {
            let num = parts[1]
                .parse::<usize>()
                .map_err(|_| format!("Invalid end number: {}", parts[1]))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            Some(num)
        };

        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return Err(format!("Invalid range: start ({}) > end ({})", s, e));
            }
        }

        Ok((start, end))
    }

    /// Check if output is to stdout
    pub fn is_stdout(&self) -> bool {
        self.output.is_none()
            || self
                .output
                .as_ref()
                .map(|p| p.as_os_str() == "-")
                .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_args() {
        let args = Args::try_parse_from(&["scidtopgn", "database"]).unwrap();
        assert_eq!(args.input.to_str().unwrap(), "database");
        assert!(args.output.is_none());
    }

    #[test]
    fn test_output_file() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "-o", "output.pgn"]).unwrap();
        assert_eq!(args.output.unwrap().to_str().unwrap(), "output.pgn");
    }

    #[test]
    fn test_output_stdout() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "-o", "-"]).unwrap();
        assert!(args.is_stdout());
    }

    #[test]
    fn test_range_single() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "--range", "50"]).unwrap();
        assert_eq!(args.range.unwrap(), "50");
    }

    #[test]
    fn test_range_simple() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "--range", "10-20"]).unwrap();
        let (start, end) = args.parse_range("10-20").unwrap();
        assert_eq!(start, Some(10));
        assert_eq!(end, Some(20));
    }

    #[test]
    fn test_range_from_beginning() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "--range", "-50"]).unwrap();
        let (start, end) = args.parse_range("-50").unwrap();
        assert_eq!(start, None);
        assert_eq!(end, Some(50));
    }

    #[test]
    fn test_range_to_end() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "--range", "100-"]).unwrap();
        let (start, end) = args.parse_range("100-").unwrap();
        assert_eq!(start, Some(100));
        assert_eq!(end, None);
    }

    #[test]
    fn test_range_error_empty() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "--range", "--"]).unwrap();
        let result = args.parse_range("--");
        assert!(result.is_err());
    }

    #[test]
    fn test_range_error_zero() {
        let result = Args::parse_range("0");
        assert!(result.is_err());
    }

    #[test]
    fn test_range_error_invalid_format() {
        let result = Args::parse_range("10-20-30");
        assert!(result.is_err());
    }

    #[test]
    fn test_range_error_start_greater() {
        let result = Args::parse_range("20-10");
        assert!(result.is_err());
    }

    #[test]
    fn test_output_directory_validation_missing() {
        let args =
            Args::try_parse_from(&["scidtopgn", "database", "-o", "/nonexistent/output.pgn"])
                .unwrap();
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_output_directory_validation_existing() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "-o", "output.pgn"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_output_directory_validation_dash() {
        let args = Args::try_parse_from(&["scidtopgn", "database", "-o", "-"]).unwrap();
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_mmap_and_threshold() {
        let args =
            Args::try_parse_from(&["scidtopgn", "database", "--mmap", "--mmap-threshold", "50M"])
                .unwrap();
        assert!(args.mmap);
        assert_eq!(args.mmap_threshold, "50M");
    }

    #[test]
    fn test_filtering_options() {
        let args = Args::try_parse_from(&[
            "scidtopgn",
            "database",
            "--player",
            "Carlsen",
            "--min-elo",
            "2700",
            "--range",
            "1-100",
        ])
        .unwrap();
        assert_eq!(args.player.unwrap(), "Carlsen");
        assert_eq!(args.min_elo, Some(2700));
        assert_eq!(args.range.unwrap(), "1-100");
    }

    #[test]
    fn test_no_flags() {
        let args = Args::try_parse_from(&["scidtopgn", "database"]).unwrap();
        assert!(!args.compact);
        assert!(!args.no_comments);
        assert!(!args.no_variations);
        assert!(!args.verbose);
        assert!(!args.quiet);
    }

    #[test]
    fn test_error_mode_defaults() {
        let args = Args::try_parse_from(&["scidtopgn", "database"]).unwrap();
        assert_eq!(args.max_errors, 0);
        assert!(!args.include_partial);
    }

    #[test]
    fn test_invalid_game_number() {
        let result = Args::parse_range("0");
        assert!(result.is_err());
    }

    #[test]
    fn test_verbose_and_quiet_exclusive() {
        let result = Args::try_parse_from(&["scidtopgn", "database", "-v", "-q"]);
        assert!(result.is_err());
    }
}
