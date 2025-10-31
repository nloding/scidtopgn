// SCID Parser Library
//
//! A high-performance library for parsing SCID chess database files and converting them to standard formats.
//! 
//! This library provides comprehensive functionality for working with SCID (Shane's Chess Information Database) files,
//! including database access, game parsing, position tracking, and PGN export capabilities.
//! 
//! # Features
//! 
//! - **Database Access**: Unified interface for accessing SCID databases (.si4, .sn4, .sg4 files)
//! - **Position Tracking**: Chess position management with shakmaty integration
//! - **PGN Export**: Standards-compliant PGN export with configurable options
//! - **Move Parsing**: Position-aware SCID move decoding with shakmaty validation
//! - **Game State Management**: Complete game state tracking with metadata support
//! 
//! # Quick Start
//! 
//! ```rust
//! use scidtopgn::api::{ScidDatabase, Result};
//! 
//! // Open a SCID database
//! let database = ScidDatabase::open("path/to/database")?;
//! 
//! // Get the number of games
//! let num_games = database.num_games();
//! 
//! // Iterate over all games
//! for game_result in database.games().take(10) {
//!     let game = game_result?;
//!     println!("Game {}: {} vs {}", 
//!              game.index.white_id, 
//!              game.index.black_id);
//! }
//! 
//! // Export a specific game to PGN
//! let pgn_content = database.export_game_to_pgn(0)?;
//! println!("{}", pgn_content);
//! ```
//! 
//! # Architecture
//! 
//! The library is organized into several key modules:
//! 
//! - **`api`**: Curated public API with re-exports of commonly used types
//! - **`bridge`**: Integration layer between SCID and shakmaty chess library
//! - **`formats`**: Parsers for SCID file formats (.si4, .sn4, .sg4)
//! - **`pgn`**: PGN export functionality with standards compliance
//! - **`position`**: Position-aware decoding and state management
//! - **`core`**: Core error types and utilities
//! 
//! # Error Handling
//! 
//! The library uses `Result<T, ScidError>` for error handling throughout. Common errors include:
//! 
//! - `ScidError::FileNotFound`: When SCID files are missing
//! - `ScidError::InvalidFormat`: When file format is corrupted
//! - `ScidError::GameIndexOutOfBounds`: When requesting a non-existent game
//! - `ScidError::ConversionError`: When move conversion fails
//! 
//! # Performance
//! 
//! The library is designed for high performance with:
//! - Memory-mapped file access for large databases
//! - Zero-copy parsing where possible
//! - Efficient iterators for game processing
//! - Minimal allocations during parsing

// Declare internal modules as private
mod bridge;
pub mod cli;
mod core;
mod formats;
mod pgn;
mod position;
mod variation;

/// Curated, minimal public API for downstream consumers.
///
/// This module provides a clean, well-documented interface to the library's core functionality.
/// Prefer these re-exports instead of reaching into internal modules.
pub mod api {
    // Re-export core error types
    pub use crate::core::error::{Result, ScidError};

    // Re-export primary database and index structures
    pub use crate::formats::si4::GameIndex;
    pub use crate::formats::sn4::NameRecord;
    pub use crate::formats::sn4::NameType;
    pub use crate::formats::{ScidDatabase, ScidGame, ScidHeaderInfo, ScidNameHeaderInfo, DatabaseFileType};
    
    // Re-export move types for public API
    pub use crate::formats::{DecodedMove, MoveInterpretation};

    // Re-export key chess-related types
    pub use crate::bridge::{
        ChessNotation, GameMetadata, GameState, PositionContext,
    };

    // Re-export the main PGN exporter
    pub use crate::pgn::{PgnExporter, ExportOptions};

    // Re-export position tracking types
    pub use crate::bridge::position_tracker::ScidPositionTracker;

    // Re-export PGN header management
    pub use crate::pgn::PgnHeader;

    // Re-export game state management
    pub use crate::position::state_manager::PositionStateManager;
}

// Re-export legacy types for backward compatibility
pub use crate::bridge::{GameState, GameMetadata, PositionContext, ChessNotation};
pub use crate::core::error::{ScidError, Result};
pub use crate::pgn::PgnExporter;
pub use crate::variation::{VariationTree, Variation, VariationMove, VariationGameElement};

// Re-export commonly used types that users might expect
pub use crate::cli::{Cli, Commands};
pub use crate::formats::si4::GameIndex;
pub use crate::formats::sn4::NameRecord;
pub use crate::formats::ScidDatabase;

// Re-export move types for direct access
pub use crate::formats::{DecodedMove, MoveInterpretation};

// Re-export position tracking for advanced users
pub use crate::bridge::position_tracker::ScidPositionTracker;

// Re-export PGN header management
pub use crate::pgn::PgnHeader;

// Re-export game state management
pub use crate::position::state_manager::PositionStateManager;

/// Version information for the library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Build information including git commit hash if available
pub const BUILD_INFO: &str = "development build";

/// Library name for display purposes
pub const LIBRARY_NAME: &str = "scidtopgn";

/// Library description
pub const LIBRARY_DESCRIPTION: &str = "SCID Chess Database Parser and Converter";

/// Minimum supported SCID database version
pub const MIN_SCID_VERSION: u32 = 4;

/// Maximum supported SCID database version
pub const MAX_SCID_VERSION: u32 = 4;

/// Default maximum number of games to process in batch operations
pub const DEFAULT_MAX_GAMES: usize = 1000;

/// Default buffer size for file operations
pub const DEFAULT_BUFFER_SIZE: usize = 8192;

/// Library initialization and configuration
pub mod config {
    use super::*;
    
    /// Global configuration for the library
    #[derive(Debug, Clone)]
    pub struct Config {
        /// Maximum number of games to process in batch operations
        pub max_games: usize,
        /// Buffer size for file operations
        pub buffer_size: usize,
        /// Whether to enable verbose logging
        pub verbose: bool,
        /// Whether to validate game moves during parsing
        pub validate_moves: bool,
    }
    
    impl Default for Config {
        fn default() -> Self {
            Self {
                max_games: DEFAULT_MAX_GAMES,
                buffer_size: DEFAULT_BUFFER_SIZE,
                verbose: false,
                validate_moves: true,
            }
        }
    }
    
    impl Config {
        /// Create a new configuration with default settings
        pub fn new() -> Self {
            Self::default()
        }
        
        /// Create a configuration with custom settings
        pub fn with_settings(max_games: usize, buffer_size: usize, verbose: bool, validate_moves: bool) -> Self {
            Self {
                max_games,
                buffer_size,
                verbose,
                validate_moves,
            }
        }
        
        /// Get the global configuration
        pub fn global() -> &'static Config {
            static GLOBAL_CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
            GLOBAL_CONFIG.get_or_init(|| Config::default())
        }
        
        /// Set the global configuration
        pub fn set_global(config: Config) {
            static GLOBAL_CONFIG: std::sync::OnceLock<Config> = std::sync::OnceLock::new();
            let _ = GLOBAL_CONFIG.set(config);
        }
    }
}

/// Utility functions for common operations
pub mod utils {
    use super::*;
    
    /// Check if a file exists and is readable
    pub fn file_exists<P: AsRef<std::path::Path>>(path: P) -> bool {
        path.as_ref().exists() && path.as_ref().is_file()
    }
    
    /// Check if a SCID database exists (all three files)
    pub fn database_exists<P: AsRef<std::path::Path>>(base_path: P) -> bool {
        let base_path = base_path.as_ref();
        file_exists(base_path.with_extension("si4")) &&
        file_exists(base_path.with_extension("sn4")) &&
        file_exists(base_path.with_extension("sg4"))
    }
    
    /// Get the size of a file in bytes
    pub fn file_size<P: AsRef<std::path::Path>>(path: P) -> Result<u64> {
        Ok(std::fs::metadata(path)?.len())
    }
    
    /// Format a file size in human-readable format
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[(&str, u64)] = &[("B", 1), ("KB", 1024), ("MB", 1024 * 1024), ("GB", 1024 * 1024 * 1024)];
        
        for (unit, divisor) in UNITS.iter().rev() {
            if size >= *divisor {
                return format!("{:.1} {}", size as f64 / *divisor as f64, unit);
            }
        }
        
        format!("{} B", size)
    }
}

/// Performance monitoring and benchmarking utilities
pub mod perf {
    use super::*;
    use std::time::Instant;
    
    /// Simple performance timer for measuring operation duration
    pub struct Timer {
        start: Instant,
        name: String,
    }
    
    impl Timer {
        /// Create a new timer with the given operation name
        pub fn new(name: &str) -> Self {
            Self {
                start: Instant::now(),
                name: name.to_string(),
            }
        }
        
        /// Get the elapsed time in milliseconds
        pub fn elapsed_ms(&self) -> u64 {
            self.start.elapsed().as_millis() as u64
        }
        
        /// Get the elapsed time as a formatted string
        pub fn elapsed_string(&self) -> String {
            format!("{}: {}ms", self.name, self.elapsed_ms())
        }
    }
    
    /// Memory usage statistics
    #[derive(Debug, Clone)]
    pub struct MemoryStats {
        pub current_memory: usize,
        pub peak_memory: usize,
        pub allocations: usize,
    }
    
    impl MemoryStats {
        /// Create new memory statistics
        pub fn new() -> Self {
            Self {
                current_memory: 0,
                peak_memory: 0,
                allocations: 0,
            }
        }
        
        /// Update memory statistics
        pub fn update(&mut self, current: usize) {
            self.current_memory = current;
            self.peak_memory = self.peak_memory.max(current);
            self.allocations += 1;
        }
        
        /// Format memory usage in human-readable format
        pub fn format(&self) -> String {
            format!(
                "Current: {}, Peak: {}, Allocations: {}",
                utils::format_file_size(self.current_memory as u64),
                utils::format_file_size(self.peak_memory as u64),
                self.allocations
            )
        }
    }
}

/// Version compatibility checking
pub mod version {
    use super::*;
    
    /// Check if the given SCID version is supported
    pub fn is_version_supported(version: u32) -> bool {
        version >= MIN_SCID_VERSION && version <= MAX_SCID_VERSION
    }
    
    /// Get the version compatibility message
    pub fn compatibility_message(version: u32) -> &'static str {
        if is_version_supported(version) {
            "Version is supported"
        } else if version < MIN_SCID_VERSION {
            "Version is too old (minimum supported is 4.0)"
        } else {
            "Version is too new (maximum supported is 4.0)"
        }
    }
}

/// Result types for common operations
pub mod result {
    use super::*;
    
    /// Result type for PGN export operations
    pub type PgnResult = Result<String>;
    
    /// Result type for validation operations
    pub type ValidationResult = Result<bool>;
}

/// Common traits for extending library functionality
pub mod traits {
    use super::*;
    
    /// Trait for objects that can be exported to PGN format
    pub trait ToPgn {
        /// Export the object to PGN format
        fn to_pgn(&self) -> Result<String>;
        
        /// Export the object to PGN format with custom options
        fn to_pgn_with_options(&self, options: &crate::pgn::ExportOptions) -> Result<String>;
    }
    
    /// Trait for objects that can be validated
    pub trait Validatable {
        /// Validate the object and return true if valid
        fn validate(&self) -> Result<bool>;
    }
    
    /// Trait for objects that have associated metadata
    pub trait HasMetadata {
        /// Get the metadata associated with the object
        fn metadata(&self) -> Option<&GameMetadata>;
    }
}

/// Prelude module for common imports
pub mod prelude {
    // Re-export most commonly used types
    pub use crate::api::*;
    pub use crate::core::error::*;
    pub use crate::config::Config;
    pub use crate::result::*;
    pub use crate::traits::*;
    pub use crate::utils::*;
}

/// Initialize the library with default configuration
/// 
/// This function should be called once at program startup to set up
/// the global configuration and perform any necessary initialization.
pub fn init() -> Result<()> {
    // Set up global configuration
    config::Config::set_global(config::Config::default());
    
    // Initialize logging if needed
    if config::Config::global().verbose {
        // TODO: Set up logging when logging infrastructure is added
    }
    
    Ok(())
}

/// Initialize the library with custom configuration
pub fn init_with_config(config: config::Config) -> Result<()> {
    config::Config::set_global(config);
    Ok(())
}

/// Get library information as a formatted string
pub fn library_info() -> String {
    format!(
        "{} v{} - {}\nBuild: {}\nSCID Version Support: {}-{}",
        LIBRARY_NAME,
        VERSION,
        LIBRARY_DESCRIPTION,
        BUILD_INFO,
        MIN_SCID_VERSION,
        MAX_SCID_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_library_info() {
        let info = library_info();
        assert!(info.contains(LIBRARY_NAME));
        assert!(info.contains(VERSION));
    }
    
    #[test]
    fn test_version_compatibility() {
        assert!(version::is_version_supported(4));
        assert!(!version::is_version_supported(3));
        assert!(!version::is_version_supported(5));
    }
    
    #[test]
    fn test_config() {
        let config = config::Config::new();
        assert_eq!(config.max_games, DEFAULT_MAX_GAMES);
        assert_eq!(config.buffer_size, DEFAULT_BUFFER_SIZE);
        assert!(!config.verbose);
        assert!(config.validate_moves);
    }
    
    #[test]
    fn test_utils() {
        assert!(utils::file_exists("Cargo.toml"));
        assert!(!utils::file_exists("nonexistent.txt"));
        
        let size = utils::file_size("Cargo.toml").unwrap();
        assert!(size > 0);
        
        let formatted = utils::format_file_size(size);
        assert!(formatted.contains("B"));
    }
    
    #[test]
    fn test_perf_timer() {
        let timer = perf::Timer::new("test");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 10);
        
        let formatted = timer.elapsed_string();
        assert!(formatted.contains("test"));
        assert!(formatted.contains("ms"));
    }
    
    #[test]
    fn test_memory_stats() {
        let mut stats = perf::MemoryStats::new();
        stats.update(1024);
        stats.update(2048);
        
        assert_eq!(stats.current_memory, 2048);
        assert_eq!(stats.peak_memory, 2048);
        assert_eq!(stats.allocations, 2);
        
        let formatted = stats.format();
        assert!(formatted.contains("Current:"));
        assert!(formatted.contains("Peak:"));
        assert!(formatted.contains("Allocations:"));
    }
}
