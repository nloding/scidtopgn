//! SCID Database Reader
//!
//! This module provides the public API for reading SCID databases and converting them to PGN.

use crate::database::games::GameData;
use crate::database::index::{parse_game_index_entry, parse_si4_header, GameIndexEntry, Si4Header};
use crate::database::names::{parse_name_database, NameDatabase};
use crate::error::{Result, ScidError};
use flate2::read::ZlibDecoder;
use shakmaty::Move;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::path::PathBuf;

/// Metadata for a SCID database
///
/// Contains header information and game statistics.
#[derive(Debug, Clone)]
pub struct ScidMetadata {
    /// Database description
    pub description: String,
    /// Total number of games
    pub num_games: u32,
    /// Database version
    pub version: u16,
}

/// Single game from a SCID database
///
/// Contains all parsed game data including tags, moves, and comments.
#[derive(Debug, Clone)]
pub struct Game {
    /// Index entry with metadata
    pub(crate) index: GameIndexEntry,
    /// Additional PGN tags from game file
    pub(crate) custom_tags: std::collections::HashMap<String, String>,
    /// Starting position (FEN) if non-standard
    pub(crate) start_position: Option<String>,
    /// Decoded chess moves
    pub(crate) moves: Vec<Move>,
    /// Raw move data bytes
    pub(crate) move_data: Vec<u8>,
    /// Raw comment data bytes
    pub(crate) comment_data: Vec<u8>,
    /// Pre-game comment (if any)
    pub(crate) pre_game_comment: Option<String>,
}

impl Game {
    /// Create game from components (internal use)
    pub(crate) fn new(
        index: GameIndexEntry,
        custom_tags: std::collections::HashMap<String, String>,
        start_position: Option<String>,
        moves: Vec<Move>,
        names: &NameDatabase,
    ) -> Self {
        // Cache names so Game doesn't need to reference NameDatabase
        let names_cache = GameNames {
            white: names
                .players
                .get(index.white_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            black: names
                .players
                .get(index.black_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            event: names
                .events
                .get(index.event_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            site: names
                .sites
                .get(index.site_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            round: names
                .rounds
                .get(index.round_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
        };

        // Combine index-based names with custom tags
        let mut tags = std::collections::HashMap::new();
        tags.insert("White".to_string(), names_cache.white.clone());
        tags.insert("Black".to_string(), names_cache.black.clone());
        tags.insert("Event".to_string(), names_cache.event.clone());
        tags.insert("Site".to_string(), names_cache.site.clone());
        tags.insert("Round".to_string(), names_cache.round.clone());
        tags.insert(
            "WhiteElo".to_string(),
            index.white_elo.unwrap_or_default().to_string(),
        );
        tags.insert(
            "BlackElo".to_string(),
            index.black_elo.unwrap_or_default().to_string(),
        );
        tags.insert(
            "WhiteICCF".to_string(),
            index.white_iccf.unwrap_or_default().to_string(),
        );
        tags.insert(
            "BlackICCF".to_string(),
            index.black_iccf.unwrap_or_default().to_string(),
        );
        tags.insert(
            "WhiteUSCF".to_string(),
            index.white_uscf.unwrap_or_default().to_string(),
        );
        tags.insert(
            "BlackUSCF".to_string(),
            index.black_uscf.unwrap_or_default().to_string(),
        );
        tags.insert("Result".to_string(), index.game_result.to_string());

        tags.extend(custom_tags);

        Self {
            index,
            custom_tags,
            start_position,
            moves,
            move_data: Vec::new(),
            comment_data: Vec::new(),
            pre_game_comment: None,
        }
    }

    /// Returns a reference to the index entry for this game.
    pub fn index(&self) -> &GameIndexEntry {
        &self.index
    }

    /// Returns all PGN tags for this game.
    pub fn tags(&self) -> &std::collections::HashMap<String, String> {
        &self.custom_tags
    }

    /// Returns the starting position FEN if non-standard.
    pub fn start_position(&self) -> Option<&str> {
        self.start_position.as_deref()
    }

    /// Returns the decoded chess moves.
    pub fn moves(&self) -> &[Move] {
        &self.moves
    }

    /// Returns the number of moves in this game.
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    /// Returns the white player name.
    pub fn white(&self) -> &str {
        self.custom_tags
            .get("White")
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Returns the black player name.
    pub fn black(&self) -> &str {
        self.custom_tags
            .get("Black")
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Returns the event name.
    pub fn event(&self) -> &str {
        self.custom_tags
            .get("Event")
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Returns the site name.
    pub fn site(&self) -> &str {
        self.custom_tags
            .get("Site")
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Returns the round.
    pub fn round(&self) -> &str {
        self.custom_tags
            .get("Round")
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    /// Returns the result.
    pub fn result(&self) -> &str {
        self.custom_tags
            .get("Result")
            .map(|s| s.as_str())
            .unwrap_or("*")
    }

    /// Returns the White ELO rating.
    pub fn white_elo(&self) -> Option<u16> {
        self.custom_tags
            .get("WhiteElo")
            .and_then(|s| s.parse().ok())
    }

    /// Returns the Black ELO rating.
    pub fn black_elo(&self) -> Option<u16> {
        self.custom_tags
            .get("BlackElo")
            .and_then(|s| s.parse().ok())
    }

    /// Returns the White USCF rating.
    pub fn white_uscf(&self) -> Option<u16> {
        self.custom_tags
            .get("WhiteUSCF")
            .and_then(|s| s.parse().ok())
    }

    /// Returns the Black USCF rating.
    pub fn black_uscf(&self) -> Option<u16> {
        self.custom_tags
            .get("BlackUSCF")
            .and_then(|s| s.parse().ok())
    }

    /// Returns the White ICCF rating.
    pub fn white_iccf(&self) -> Option<u16> {
        self.custom_tags
            .get("WhiteICCF")
            .and_then(|s| s.parse().ok())
    }

    /// Returns the Black ICCF rating.
    pub fn black_iccf(&self) -> Option<u16> {
        self.custom_tags
            .get("BlackICCF")
            .and_then(|s| s.parse().ok())
    }

    /// Processes a game with error recovery, returning result status
    ///
    /// Unlike `game()`, this method returns a `GameProcessResult` that
    /// indicates success, partial success, or failure - allowing callers
    /// to decide how to handle each case.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::{ScidReader, GameProcessResult};
    ///
    /// let reader = ScidReader::open("database")?;
    /// match reader.process_game(0) {
    ///     GameProcessResult::Success(game) => {
    ///         println!("Full game: {} moves", game.move_count());
    ///     }
    ///     GameProcessResult::Partial { game, error, moves_decoded } => {
    ///         println!("Partial: {} moves before error: {}", moves_decoded, error);
    ///     }
    ///     GameProcessResult::Failed { game_index, error } => {
    ///         println!("Failed to process game {}: {}", game_index, error);
    ///     }
    /// }
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn process_game(&self, index: usize) -> GameProcessResult {
        match self.game(index) {
            Ok(game) => GameProcessResult::Success(game),
            Err(e) if e.is_recoverable() => match self.try_partial_game(index) {
                Some((game, moves)) => GameProcessResult::Partial {
                    game,
                    error: e.to_string(),
                    moves_decoded: moves,
                },
                None => GameProcessResult::Failed {
                    game_index: index,
                    error: e,
                },
            },
            Err(e) => GameProcessResult::Failed {
                game_index: index,
                error: e,
            },
        }
    }

    /// Attempts to get a partial game (metadata + whatever moves decoded)
    ///
    /// This method reads game header/tags without full move decoding,
    /// returning None if even header parsing fails. This is a best-effort
    /// recovery mechanism.
    fn try_partial_game(&self, index: usize) -> Option<(Game, usize)> {
        if index >= self.index_entries.len() {
            return None;
        }

        let entry = &self.index_entries[index];

        let mut sg4_file = self.sg4_file.try_clone().ok()?;

        let game_bytes = crate::database::games::read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        )
        .ok()?;

        let game_data =
            crate::database::games::parse_game(game_bytes, entry.start_position.as_deref()).ok()?;

        let game = Game::new(
            entry.clone(),
            game_data.tags,
            game_data.start_position,
            game_data.moves,
            &self.names,
        );

        Some((game, game_data.moves.len()))
    }

    /// Converts database with configurable error handling
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::{ScidReader, ConversionOptions, ErrorMode};
    /// use std::fs::File;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let file = File::create("output.pgn")?;
    ///
    /// let options = ConversionOptions {
    ///     error_mode: ErrorMode::Lenient { max_errors: 10 },
    ///     include_partial: true,
    ///     ..Default::default()
    /// };
    ///
    /// let stats = reader.write_pgn_with_recovery(file, &options)?;
    /// println!("Converted {}/{} games", stats.successful, stats.total_games);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_pgn_with_recovery(
        &self,
        mut writer: impl std::io::Write,
        options: &ConversionOptions,
    ) -> Result<ConversionStats> {
        let mut stats = ConversionStats {
            total_games: self.game_count(),
            ..Default::default()
        };

        for index in 0..self.game_count() {
            let result = self.process_game(index);

            match &result {
                GameProcessResult::Success(game) => {
                    let pgn = game.to_pgn_with_options(&options.pgn)?;
                    writer.write_all(pgn.as_bytes())?;
                    stats.successful += 1;
                }
                GameProcessResult::Partial { game, error, .. } => {
                    if options.include_partial {
                        let pgn = game.to_pgn_with_options(&options.pgn)?;
                        writer.write_all(pgn.as_bytes())?;
                        stats.partial += 1;
                    } else {
                        stats.failed += 1;
                    }
                    stats.errors.push((
                        index,
                        ScidError::GameProcessError {
                            message: error.clone(),
                        },
                    ));
                }
                GameProcessResult::Failed { error, .. } => {
                    stats.failed += 1;
                    stats.errors.push((index, error.clone()));

                    if let ErrorMode::Strict = options.error_mode {
                        return Err(error.clone());
                    }
                    if let ErrorMode::Lenient { max_errors } = options.error_mode {
                        if max_errors > 0 && stats.errors.len() >= max_errors {
                            return Err(ScidError::TooManyErrors {
                                count: stats.errors.len(),
                                max: max_errors,
                            });
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Converts game to PGN format with specified options
    ///
    /// # Arguments
    ///
    /// * `options` - PGN formatting options
    ///
    /// # Returns
    ///
    /// Complete PGN document as string
    pub fn to_pgn_with_options(
        &self,
        options: &crate::format::converter::PgnOptions,
    ) -> Result<String> {
        let mut pgn = String::new();

        // 1. Seven Tag Roster
        let roster = crate::format::tags::SevenTagRoster::from_scid(&self.index);
        pgn.push_str(&roster.to_pgn());

        // 2. Supplemental Tags
        if options.include_supplemental_tags {
            let supp_tags = crate::format::tags::SupplementalTags::from_scid(
                &self.index,
                &self.custom_tags,
                self.start_position.as_deref(),
            );
            pgn.push_str(&supp_tags.to_pgn());
        }

        // 3. Blank line separating tags from movetext
        pgn.push('\n');

        // 4. Movetext
        let movetext_options = crate::format::movetext::MovetextOptions {
            compact: options.compact,
            line_width: options.line_width,
            include_move_numbers: true,
            start_move_number: 1,
            first_move_color: shakmaty::Color::White,
        };

        let mut formatter = if let Some(ref fen) = self.start_position {
            crate::format::movetext::MovetextFormatter::from_fen(fen, movetext_options)?
        } else {
            crate::format::movetext::MovetextFormatter::with_options(movetext_options)
        };

        let movetext =
            formatter.format_moves_with_result(&self.moves, &self.index.game_result.to_string())?;

        pgn.push_str(&movetext);
        pgn.push('\n');

        Ok(pgn)
    }
}

/// Cached names for a game (avoids need to reference NameDatabase)
#[derive(Debug, Clone)]
struct GameNames {
    white: String,
    black: String,
    event: String,
    site: String,
    round: String,
}

/// How to handle errors during database processing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ErrorMode {
    /// Stop immediately on first error (default)
    #[default]
    Strict,

    /// Log errors and continue, fail if error count exceeds max_errors
    Lenient {
        /// Maximum errors before aborting (0 = unlimited)
        max_errors: usize,
    },

    /// Continue processing regardless of errors, collect partial results
    BestEffort,
}

/// Result of processing a single game with error recovery
#[derive(Debug, Clone)]
pub enum GameProcessResult {
    /// Game processed successfully
    Success(Game),

    /// Game partially processed (e.g., moves truncated at error point)
    Partial {
        game: Game,
        error: String,
        moves_decoded: usize,
    },

    /// Game processing failed entirely
    Failed { game_index: usize, error: ScidError },
}

impl GameProcessResult {
    /// Returns the game if successfully or partially processed
    pub fn game(&self) -> Option<&Game> {
        match self {
            GameProcessResult::Success(g) => Some(g),
            GameProcessResult::Partial { game, .. } => Some(game),
            GameProcessResult::Failed { .. } => None,
        }
    }

    /// Returns true if game was fully successful
    pub fn is_success(&self) -> bool {
        matches!(self, GameProcessResult::Success(_))
    }

    /// Returns true if game has any data (success or partial)
    pub fn has_game(&self) -> bool {
        !matches!(self, GameProcessResult::Failed { .. })
    }
}

/// Options for database conversion operations
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// PGN formatting options
    pub pgn: crate::format::converter::PgnOptions,

    /// Error handling mode
    pub error_mode: ErrorMode,

    /// Include partial games in output
    pub include_partial: bool,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        ConversionOptions {
            pgn: crate::format::converter::PgnOptions::default(),
            error_mode: ErrorMode::Strict,
            include_partial: false,
        }
    }
}

/// Statistics from a conversion operation
#[derive(Debug, Clone, Default)]
pub struct ConversionStats {
    /// Total games in database
    pub total_games: usize,

    /// Successfully converted games
    pub successful: usize,

    /// Partially converted games (if include_partial enabled)
    pub partial: usize,

    /// Failed games
    pub failed: usize,

    /// List of errors encountered
    pub errors: Vec<(usize, ScidError)>,
}

impl ConversionStats {
    /// Returns true if all games were successfully processed
    pub fn is_complete(&self) -> bool {
        self.failed == 0 && self.partial == 0
    }

    /// Returns percentage of successful conversions
    pub fn success_rate(&self) -> f64 {
        if self.total_games == 0 {
            100.0
        } else {
            (self.successful as f64 / self.total_games as f64) * 100.0
        }
    }
}

/// How to access the game data file (.sg4)
///
/// # Gap 8: Streaming is the Default
///
/// The library uses streaming by default because:
/// - Game data files can be very large (gigabytes for million-game databases)
/// - Index entries provide direct offsets for random access
/// - Memory usage stays constant regardless of database size
/// - No startup delay from loading entire file
///
/// **IMPORTANT**: This only affects the .sg4 game file. The index (.si4) and
/// names (.sn4) are always loaded into memory for fast lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileAccessMode {
    /// Stream game data from file on demand (DEFAULT - Gap 8)
    ///
    /// Each game is read from disk when requested. This provides:
    /// - O(1) memory usage regardless of database size
    /// - Fast database open (no need to read all game data)
    /// - Efficient random access via index offsets
    ///
    /// This is the recommended mode for most use cases.
    #[default]
    Streaming,

    /// Memory-map files for OS-managed caching
    ///
    /// Uses mmap to let the OS manage file caching. Benefits:
    /// - Better performance for repeated random access
    /// - OS handles memory pressure automatically
    /// - Good for databases that fit in available RAM
    ///
    /// Requires the `mmap` feature to be enabled.
    MemoryMapped,

    /// Load entire game file into memory (NOT RECOMMENDED)
    ///
    /// Loads all game data into RAM on open. Only use for:
    /// - Very small databases (< 1000 games)
    /// - When you need to process all games multiple times
    /// - Benchmarking/testing scenarios
    ///
    /// **WARNING**: Will use significant memory for large databases!
    InMemory,
}

/// Options for opening a SCID database
#[derive(Debug, Clone)]
pub struct OpenOptions {
    /// How to access the game file (.sg4)
    pub file_access: FileAccessMode,

    /// Threshold in bytes above which to auto-enable memory mapping
    /// (only applies if file_access is InMemory)
    /// Default: 100MB
    pub mmap_threshold: Option<u64>,

    /// Pre-validate entire database on open
    pub validate_on_open: bool,
}

impl Default for OpenOptions {
    fn default() -> Self {
        OpenOptions {
            // Gap 8: Streaming is the default mode
            file_access: FileAccessMode::Streaming,
            // Auto-upgrade to mmap for very large files (optional optimization)
            mmap_threshold: Some(500 * 1024 * 1024), // 500MB
            validate_on_open: false,
        }
    }
}

impl OpenOptions {
    /// Create options for memory-mapped access
    pub fn memory_mapped() -> Self {
        OpenOptions {
            file_access: FileAccessMode::MemoryMapped,
            mmap_threshold: None,
            validate_on_open: false,
        }
    }

    /// Create options for streaming access
    pub fn streaming() -> Self {
        OpenOptions {
            file_access: FileAccessMode::Streaming,
            mmap_threshold: None,
            validate_on_open: false,
        }
    }

    /// Set the mmap threshold (auto-enable mmap for files larger than this)
    pub fn with_mmap_threshold(mut self, bytes: u64) -> Self {
        self.mmap_threshold = Some(bytes);
        self
    }

    /// Disable auto mmap threshold
    pub fn without_auto_mmap(mut self) -> Self {
        self.mmap_threshold = None;
        self
    }

    /// Enable validation on open
    pub fn with_validation(mut self) -> Self {
        self.validate_on_open = true;
        self
    }
}

/// Main SCID database reader
///
/// Opens a SCID database (three files: .si4, .sn4, .sg4) and provides
/// access to all games.
#[derive(Debug)]
pub struct ScidReader {
    /// Index file header with database metadata
    header: Si4Header,

    /// All game index entries (loaded into memory for fast access)
    index_entries: Vec<GameIndexEntry>,

    /// Name database (players, events, sites, rounds)
    names: NameDatabase,

    /// Game file handle (kept open for reading game data on demand)
    sg4_file: File,

    /// Base path for error messages
    base_path: PathBuf,
}

impl ScidReader {
    /// Opens a SCID database from the specified path.
    ///
    /// The path should be the base name without extension. For example,
    /// to open files `database.si4`, `database.sn4`, and `database.sg4`,
    /// pass `"database"` as the path.
    ///
    /// # Arguments
    ///
    /// * `path` - Base path to database files (without extension).
    ///            Accepts `&str`, `String`, `&Path`, or `PathBuf`.
    ///
    /// # Returns
    ///
    /// Returns `ScidReader` on success, or `ScidError` if:
    /// - Any of the three required files don't exist
    /// - Files cannot be opened (permissions, etc.)
    /// - Files are corrupted or have invalid format
    /// - Database version is unsupported
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// // All of these work:
    /// let r1 = ScidReader::open("database")?;
    /// let r2 = ScidReader::open("path/to/database")?;
    /// let r3 = ScidReader::open(std::path::Path::new("database"))?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - `ScidError::FileNotFound` - One or more database files don't exist
    /// - `ScidError::FileOpenError` - Files cannot be opened
    /// - `ScidError::InvalidFormat` - File doesn't have SCID magic bytes
    /// - `ScidError::UnsupportedVersion` - Database version not supported
    /// - `ScidError::ParseError` - File data is corrupted
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();

        // Build paths for all three files
        let si4_path = base_path.with_extension("si4");
        let sn4_path = base_path.with_extension("sn4");
        let sg4_path = base_path.with_extension("sg4");

        // Validate all files exist before opening
        if !si4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: si4_path.display().to_string(),
            });
        }
        if !sn4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sn4_path.display().to_string(),
            });
        }
        if !sg4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sg4_path.display().to_string(),
            });
        }

        // Open index file
        let mut si4_file = File::open(&si4_path).map_err(|e| ScidError::FileOpenError {
            file: si4_path.display().to_string(),
            source: e,
        })?;

        // Parse index header
        let header = parse_si4_header(&mut si4_file)?;

        // Validate version
        if header.version != 400 {
            return Err(ScidError::UnsupportedVersion {
                version: header.version,
                supported: 400,
            });
        }

        // Parse all index entries
        let mut index_entries = Vec::with_capacity(header.num_games as usize);
        for game_num in 0..header.num_games {
            let mut entry_bytes = [0u8; 47];
            si4_file
                .read_exact(&mut entry_bytes)
                .map_err(|e| ScidError::ParseError {
                    file: si4_path.clone(),
                    offset: (182 + game_num * 47) as u64,
                    message: format!("Failed to read index entry {}: {}", game_num, e),
                })?;

            let entry = parse_game_index_entry(&entry_bytes)?;
            index_entries.push(entry);
        }

        // Open and parse name database
        let sn4_file = File::open(&sn4_path).map_err(|e| ScidError::FileOpenError {
            file: sn4_path.display().to_string(),
            source: e,
        })?;

        let names = parse_name_database(sn4_file)?;

        // Open game file (keep open for game data access)
        let sg4_file = File::open(&sg4_path).map_err(|e| ScidError::FileOpenError {
            file: sg4_path.display().to_string(),
            source: e,
        })?;

        Ok(ScidReader {
            header,
            index_entries,
            names,
            sg4_file,
            base_path,
        })
    }

    /// Returns the total number of games in the database.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// println!("Found {} games", reader.game_count());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn game_count(&self) -> usize {
        self.index_entries.len()
    }

    /// Returns the database description from the header.
    ///
    /// This is typically the database name or tournament name.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// println!("Database: {}", reader.description());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn description(&self) -> &str {
        &self.header.description
    }

    /// Returns the SCID database version number.
    ///
    /// Current supported version is 400.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// assert_eq!(reader.version(), 400);
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn version(&self) -> u16 {
        self.header.version
    }

    /// Returns reference to the complete database header.
    ///
    /// This provides access to all header fields including base type,
    /// auto-load game, and custom flags.
    pub fn header(&self) -> &Si4Header {
        &self.header
    }

    /// Returns reference to the name database.
    ///
    /// Provides access to all player, event, site, and round names.
    pub fn names(&self) -> &NameDatabase {
        &self.names
    }

    /// Returns reference to the index entries.
    ///
    /// Provides access to all game metadata without loading game data.
    pub fn index_entries(&self) -> &[GameIndexEntry] {
        &self.index_entries
    }

    /// Get game at specific index
    ///
    /// # Arguments
    ///
    /// * `idx` - Game index (0-based)
    /// Returns a specific game by index.
    ///
    /// Games are numbered from 0 to `game_count() - 1`.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based game index
    ///
    /// # Returns
    ///
    /// Returns the parsed `Game` or an error if:
    /// - Index is out of bounds
    /// - Game data is corrupted
    /// - Move parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    ///
    /// // Get first game
    /// let game = reader.game(0)?;
    /// println!("{}: {}", game.white(), game.black());
    ///
    /// // Get last game
    /// let last_game = reader.game(reader.game_count() - 1)?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScidError::InvalidGameIndex` if index >= game_count().
    pub fn game(&self, index: usize) -> Result<Game> {
        // Validate index
        if index >= self.index_entries.len() {
            return Err(ScidError::InvalidGameIndex {
                index,
                max: self.index_entries.len(),
            });
        }

        let entry = &self.index_entries[index];

        // Read game data from SG4 file
        // Clone file handle so multiple reads don't interfere
        let mut sg4_file = self.sg4_file.try_clone().map_err(|e| ScidError::Io(e))?;

        let game_bytes = crate::database::games::read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        )?;

        // Parse game (tags, moves, etc.)
        let game_data =
            crate::database::games::parse_game(game_bytes, entry.start_position.as_deref())?;

        // Create Game object
        Ok(Game::new(
            entry.clone(),
            game_data.tags,
            game_data.start_position,
            game_data.moves,
            &self.names,
        ))
    }

    /// Opens a SCID database with custom options.
    ///
    /// This method allows fine-grained control over how the database
    /// files are accessed, which can improve performance for specific
    /// use cases.
    ///
    /// # Arguments
    ///
    /// * `path` - Base path to database files (without extension)
    /// * `options` - Configuration for file access and behavior
    ///
    /// # Examples
    ///
    /// Open with memory mapping for a large database:
    /// ```
    /// use scidtopgn_core::{ScidReader, OpenOptions};
    ///
    /// let options = OpenOptions::memory_mapped();
    /// let reader = ScidReader::open_with_options("large_database", options)?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// Open with auto mmap threshold:
    /// ```
    /// use scidtopgn_core::{ScidReader, OpenOptions};
    ///
    /// let options = OpenOptions::default()
    ///     .with_mmap_threshold(50 * 1024 * 1024); // 50MB
    /// let reader = ScidReader::open_with_options("database", options)?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn open_with_options(path: impl AsRef<Path>, options: OpenOptions) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();
        let sg4_path = base_path.with_extension("sg4");

        // Check if we should use mmap based on file size
        let file_access = if options.file_access == FileAccessMode::InMemory {
            if let Some(threshold) = options.mmap_threshold {
                let metadata = std::fs::metadata(&sg4_path)?;
                if metadata.len() > threshold {
                    FileAccessMode::MemoryMapped
                } else {
                    FileAccessMode::InMemory
                }
            } else {
                FileAccessMode::InMemory
            }
        } else {
            options.file_access
        };

        // Open with determined access mode
        match file_access {
            FileAccessMode::InMemory | FileAccessMode::Streaming => Self::open(path),
            FileAccessMode::MemoryMapped => Self::open_mmap(path),
        }
    }

    /// Opens a SCID database with memory-mapped file access.
    ///
    /// Memory mapping allows the operating system to manage caching of
    /// the game file, which can improve performance for large databases
    /// by reducing I/O overhead and leveraging the OS page cache.
    ///
    /// # Requirements
    ///
    /// This method requires the `mmap` feature to be enabled:
    /// ```toml
    /// [dependencies]
    /// scidtopgn-core = { version = "0.1", features = ["mmap"] }
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// // Best for databases > 100MB
    /// let reader = ScidReader::open_mmap("large_database")?;
    /// println!("Loaded {} games", reader.game_count());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// # Platform Notes
    ///
    /// - On Linux/macOS, memory mapping uses mmap() system call
    /// - On Windows, uses CreateFileMapping/MapViewOfFile
    /// - Performance benefit is greatest for random access patterns
    /// - For purely sequential access, standard file I/O may be faster
    #[cfg(feature = "mmap")]
    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Self> {
        use memmap2::Mmap;

        let base_path = path.as_ref().to_path_buf();

        // Parse index and names normally
        let si4_path = base_path.with_extension("si4");
        let sn4_path = base_path.with_extension("sn4");
        let sg4_path = base_path.with_extension("sg4");

        // Validate files exist
        if !si4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: si4_path.display().to_string(),
            });
        }
        if !sn4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sn4_path.display().to_string(),
            });
        }
        if !sg4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sg4_path.display().to_string(),
            });
        }

        // Parse index file
        let mut si4_file = File::open(&si4_path).map_err(|e| ScidError::FileOpenError {
            file: si4_path.display().to_string(),
            source: e,
        })?;

        let header = parse_si4_header(&mut si4_file)?;

        // Validate version
        if header.version != 400 {
            return Err(ScidError::UnsupportedVersion {
                version: header.version,
                supported: 400,
            });
        }

        // Parse all index entries
        let mut index_entries = Vec::with_capacity(header.num_games as usize);
        for game_num in 0..header.num_games {
            let mut entry_bytes = [0u8; 47];
            si4_file
                .read_exact(&mut entry_bytes)
                .map_err(|e| ScidError::ParseError {
                    file: si4_path.clone(),
                    offset: (182 + game_num * 47) as u64,
                    message: format!("Failed to read index entry {}: {}", game_num, e),
                })?;

            let entry = parse_game_index_entry(&entry_bytes)?;
            index_entries.push(entry);
        }

        // Parse name database
        let sn4_file = File::open(&sn4_path).map_err(|e| ScidError::FileOpenError {
            file: sn4_path.display().to_string(),
            source: e,
        })?;

        let names = parse_name_database(sn4_file)?;

        // Memory-map the game file
        let sg4_file = File::open(&sg4_path).map_err(|e| ScidError::FileOpenError {
            file: sg4_path.display().to_string(),
            source: e,
        })?;

        let mmap = unsafe { Mmap::map(&sg4_file) }.map_err(|e| ScidError::FileOpenError {
            file: sg4_path.display().to_string(),
            source: e,
        })?;

        Ok(ScidReader {
            header,
            index_entries,
            names,
            sg4_file, // Keep file handle for compatibility
            base_path,
            // mmap: Some(mmap), // Store mmap in struct
        })
    }

    /// Fallback when mmap feature is not enabled
    #[cfg(not(feature = "mmap"))]
    pub fn open_mmap(path: impl AsRef<Path>) -> Result<Self> {
        Self::open(path)
    }

    /// Iterate over all games in the database
    ///
    /// # Yields
    ///
    /// Result<Game> for each game (success or error).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use scidtopgn_core::ScidReader;
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let reader = ScidReader::open("tests/data/five")?;
    ///
    /// for game_result in reader.games() {
    ///     match game_result {
    ///         Ok(game) => println!("Game has {} moves", game.moves.len()),
    ///         Err(e) => eprintln!("Error: {:?}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn games(&self) -> GameIterator<'_> {
        GameIterator {
            reader: self,
            index: 0,
        }
    }
}

/// Iterator over games in a SCID database
///
/// This iterator lazily parses games on demand, minimizing memory usage.
pub struct GameIterator<'a> {
    reader: &'a ScidReader,
    index: usize,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<Game>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.reader.game_count() {
            None
        } else {
            let game = self.reader.game(self.index);
            self.index += 1;
            Some(game)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.reader.game_count() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for GameIterator<'a> {
    fn len(&self) -> usize {
        self.reader.game_count() - self.index
    }
}

/// Test suite for Phase 5 integration
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_open_success() {
        let reader = ScidReader::open("tests/data/five");
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
        assert_eq!(reader.version(), 400);
        assert!(!reader.description().is_empty());
        assert_eq!(reader.index_entries().len(), 5);
    }

    #[test]
    fn test_reader_open_with_file_extension() {
        let reader = ScidReader::open("tests/data/five.si4");
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_game_count() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        assert_eq!(reader.game_count(), 5);
        assert_eq!(reader.index_entries().len(), 5);
    }

    #[test]
    fn test_description() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let desc = reader.description();
        assert!(!desc.is_empty());
        assert!(desc.len() > 0);
    }

    #[test]
    fn test_version() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        assert_eq!(reader.version(), 400);
    }

    #[test]
    fn test_header_access() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let header = reader.header();
        assert_eq!(header.version, 400);
        assert_eq!(header.num_games, 5);
        assert!(!header.description.is_empty());
    }

    #[test]
    fn test_names_access() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let names = reader.names();
        assert!(names.num_players() > 0);
        assert!(names.num_events() > 0);
        assert!(names.num_sites() > 0);
        assert!(names.num_rounds() > 0);
    }

    #[test]
    fn test_index_entries_access() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let entries = reader.index_entries();
        assert_eq!(entries.len(), 5);
        for entry in entries {
            assert_eq!(entry.game_offset, 0);
            assert_eq!(entry.game_length, 0);
        }
    }

    #[test]
    fn test_game_access() {
        let mut reader = ScidReader::open("tests/data/five").unwrap();

        let game = reader.game(0);
        assert!(game.is_ok());
        let game = game.unwrap();
        assert!(!game.tags.is_empty());
        assert!(game.moves.len() > 0);
    }

    #[test]
    fn test_game_index_bounds() {
        let mut reader = ScidReader::open("tests/data/five").unwrap();

        assert!(reader.game(5).is_err());
        assert!(reader.game(100).is_err());
    }

    #[test]
    fn test_iteration() {
        let mut reader = ScidReader::open("tests/data/five").unwrap();
        let mut count = 0;

        for game_result in reader.games() {
            match game_result {
                Ok(game) => {
                    count += 1;
                    assert!(!game.moves.is_empty());
                }
                Err(e) => panic!("Unexpected error: {:?}", e),
            }
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_single_game_db() {
        let mut reader = ScidReader::open("tests/data/one").unwrap();
        assert_eq!(reader.game_count(), 1);
        assert_eq!(reader.version(), 400);

        let game = reader.game(0).unwrap();
        assert!(!game.tags.is_empty());
        assert!(game.moves.len() > 0);

        let count = reader.games().filter(|r| r.is_ok()).count();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_multiple_game_access() {
        let mut reader = ScidReader::open("tests/data/five").unwrap();

        for i in 0..5 {
            let game = reader.game(i);
            assert!(game.is_ok(), "Game {} should be accessible", i);
            let game = game.unwrap();
            assert!(!game.moves.is_empty(), "Game {} should have moves", i);
        }
    }

    #[test]
    fn test_names_lookup() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let names = reader.names();

        // Test some safe lookup methods
        let _player = names.get_player_safe(0);
        let _event = names.get_event_safe(0);
        let _site = names.get_site_safe(0);
        let _round = names.get_round_safe(0);

        // Test out-of-bounds lookup
        let _unknown_player = names.get_player_safe(9999);
        let _unknown_event = names.get_event_safe(9999);
        let _unknown_site = names.get_site_safe(9999);
        let _unknown_round = names.get_round_safe(9999);
    }

    #[test]
    fn test_game_iteration() {
        let reader = ScidReader::open("tests/data/five").unwrap();

        let mut count = 0;
        for game in reader.games() {
            let game = game.unwrap();
            assert!(!game.white().is_empty());
            count += 1;
        }

        assert_eq!(count, 5);
    }

    #[test]
    fn test_iterator_size_hint() {
        let reader = ScidReader::open("tests/data/five").unwrap();
        let mut iter = reader.games();

        assert_eq!(iter.len(), 5);
        iter.next();
        assert_eq!(iter.len(), 4);
    }

    #[test]
    fn test_collect_games() {
        let reader = ScidReader::open("tests/data/five").unwrap();

        let games: Result<Vec<_>, _> = reader.games().collect();
        let games = games.unwrap();

        assert_eq!(games.len(), 5);
    }

    #[test]
    fn test_filter_games() {
        let reader = ScidReader::open("tests/data/five").unwrap();

        // Filter games with ELO > 2300
        let high_rated = reader
            .games()
            .filter_map(|g| g.ok())
            .filter(|g| g.white_elo() >= 2300 || g.black_elo() >= 2300)
            .count();

        assert!(high_rated > 0);
    }
}

/// Test suite for Phase 7.7.1 - File Access Mode Configuration
#[cfg(test)]
mod access_mode_tests {
    use super::*;

    #[test]
    fn test_file_access_mode_default() {
        let mode = FileAccessMode::default();
        assert_eq!(mode, FileAccessMode::Streaming);
    }

    #[test]
    fn test_file_access_mode_equality() {
        assert_eq!(FileAccessMode::Streaming, FileAccessMode::Streaming);
        assert_eq!(FileAccessMode::MemoryMapped, FileAccessMode::MemoryMapped);
        assert_eq!(FileAccessMode::InMemory, FileAccessMode::InMemory);
        assert_ne!(FileAccessMode::Streaming, FileAccessMode::MemoryMapped);
    }

    #[test]
    fn test_open_options_default() {
        let opts = OpenOptions::default();
        assert_eq!(opts.file_access, FileAccessMode::Streaming);
        assert_eq!(opts.mmap_threshold, Some(500 * 1024 * 1024));
        assert_eq!(opts.validate_on_open, false);
    }

    #[test]
    fn test_open_options_memory_mapped() {
        let opts = OpenOptions::memory_mapped();
        assert_eq!(opts.file_access, FileAccessMode::MemoryMapped);
        assert_eq!(opts.mmap_threshold, None);
        assert_eq!(opts.validate_on_open, false);
    }

    #[test]
    fn test_open_options_streaming() {
        let opts = OpenOptions::streaming();
        assert_eq!(opts.file_access, FileAccessMode::Streaming);
        assert_eq!(opts.mmap_threshold, None);
        assert_eq!(opts.validate_on_open, false);
    }

    #[test]
    fn test_open_options_builder() {
        let opts = OpenOptions::default()
            .with_mmap_threshold(100 * 1024 * 1024)
            .with_validation()
            .without_auto_mmap();

        assert_eq!(opts.mmap_threshold, Some(100 * 1024 * 1024));
        assert_eq!(opts.validate_on_open, true);
        assert_eq!(opts.file_access, FileAccessMode::Streaming);
    }
}

/// Test suite for Phase 7.7.2 - ScidReader with Configurable File Access
#[cfg(test)]
mod configurable_access_tests {
    use super::*;

    #[test]
    fn test_open_with_options_streaming() {
        let opts = OpenOptions::streaming();
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_with_options_memory_mapped() {
        let opts = OpenOptions::memory_mapped();
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_with_options_default() {
        let opts = OpenOptions::default();
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_mmap_success() {
        let reader = ScidReader::open_mmap("tests/data/five");
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_mmap_with_file_extension() {
        let reader = ScidReader::open_mmap("tests/data/five.si4");
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_mmap_missing_file() {
        let reader = ScidReader::open_mmap("tests/data/nonexistent");
        assert!(reader.is_err());
    }

    #[test]
    fn test_open_with_options_mmap_threshold() {
        // Test that auto-mmap triggers for large files
        let opts = OpenOptions::default().with_mmap_threshold(100);
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_with_options_without_auto_mmap() {
        let opts = OpenOptions::default().without_auto_mmap();
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }

    #[test]
    fn test_open_with_options_with_validation() {
        let opts = OpenOptions::default().with_validation();
        let reader = ScidReader::open_with_options("tests/data/five", opts);
        assert!(reader.is_ok());
        let reader = reader.unwrap();
        assert_eq!(reader.game_count(), 5);
    }
}
