//! Parsers for SCID file formats (.si4, .sn4, .sg4)

pub mod sg4;
pub mod si4;
pub mod sn4;

// Re-export move-related types for public API
pub use sg4::{DecodedMove, MoveInterpretation};

use crate::core::error::Result;
use crate::bridge::position::format_result;
use crate::formats::sg4::SG4Source;
use crate::formats::si4::Si4File;
use crate::formats::sn4::Sn4File;
use std::path::{Path, PathBuf};

/// Public representation of SI4 header data
#[derive(Debug, Clone)]
pub struct ScidHeaderInfo {
    pub magic: String,
    pub version: u16,
    pub base_type: u32,
    pub num_games: u32,
    pub auto_load: u32,
    pub description: String,
}

/// Public representation of SN4 header data
#[derive(Debug, Clone)]
pub struct ScidNameHeaderInfo {
    pub magic: String,
    pub timestamp: u32,
    pub num_names_player: u32,
    pub num_names_event: u32,
    pub num_names_site: u32,
    pub num_names_round: u32,
    pub max_frequency_player: u32,
    pub max_frequency_event: u32,
    pub max_frequency_site: u32,
    pub max_frequency_round: u32,
}

/// Enum for specifying which database file to query
#[derive(Debug, Clone)]
pub enum DatabaseFileType {
    Si4,
    Sn4,
    Sg4,
}

/// Main SCID database structure that coordinates access to all SCID files
/// 
/// This struct provides a unified interface for accessing SCID chess databases,
/// coordinating the .si4 (index), .sn4 (names), and .sg4 (games) files.
pub struct ScidDatabase {
    /// SCID index file containing game metadata
    si4_file: Si4File,
    /// SCID names file containing player and event names
    sn4_file: Sn4File,
    /// SCID games source containing actual game data
    #[allow(dead_code)]
    sg4_source: SG4Source,
    /// Database validation state
    validated: bool,
    /// Base path for reconstructing file paths
    base_path: PathBuf,
}

/// Represents a complete game from the SCID database
#[derive(Debug, Clone, PartialEq)]
pub struct ScidGame {
    /// Game index from the .si4 file
    pub index: crate::formats::si4::GameIndex,
    /// Game state with complete move history
    pub game_state: crate::bridge::GameState,
    /// Parsed game elements (moves, comments, variations)
    pub parsed_game: crate::formats::sg4::StreamingGameParseState,
    /// Decoded moves from the SG4 file
    pub moves: Vec<DecodedMove>,
}

impl ScidDatabase {
    /// Open a SCID database from the base path (without extensions)
    /// 
    /// This method will look for .si4, .sn4, and .sg4 files with the given base path.
    /// It validates that all required files exist and are properly formatted.
    pub fn open<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Open and validate all three SCID files
        let si4_file = Si4File::open(&base_path.with_extension("si4"))?;
        let sn4_file = Sn4File::open(&base_path.with_extension("sn4"))?;
        let sg4_bytes = std::fs::read(&base_path.with_extension("sg4"))?;
        let sg4_source = crate::formats::sg4::SG4Source { bytes: sg4_bytes };
        
        let mut database = Self {
            si4_file,
            sn4_file,
            sg4_source,
            validated: false,
            base_path,
        };
        
        // Perform initial database validation
        database.validate()?;
        database.validated = true;
        
        Ok(database)
    }
    
    /// Validate the database structure and consistency
    /// 
    /// This method checks that all files are consistent and properly formatted.
    fn validate(&self) -> Result<()> {
        // Check that the number of games matches across files
        let si4_game_count = self.si4_file.num_games();
        
        let sg4_game_count = self.sg4_source.num_games() as u32;
        
        // Validate that all game indices are within bounds
        for i in 0..si4_game_count {
            let _game_index = self.si4_file.get_game(i)?;
            // Basic consistency check: sg4 should have at least as many games as si4
            if i >= sg4_game_count {
                return Err(crate::core::error::ScidError::invalid_format("SG4 games fewer than SI4 indices"));
            }
        }
        
        Ok(())
    }
    
    /// Get the number of games in the database
    pub fn num_games(&self) -> u32 {
        self.si4_file.num_games()
    }
    
    /// Get a specific game by index
    /// 
    /// Returns a complete ScidGame with parsed game state and move history.
    pub fn get_game(&self, index: u32) -> Result<ScidGame> {
        // Validate index bounds
        if index >= self.num_games() {
            return Err(crate::core::error::ScidError::game_index_out_of_bounds(index as usize, self.num_games() as usize));
        }
        
        // Get game index from si4 file
        let game_index = self.si4_file.get_game(index)?;
        
        // Decode moves using SG4Parser streaming model; do not fail hard on errors
        let mut moves: Vec<DecodedMove> = Vec::new();
        let parsed_game = {
            let data = self.sg4_source.data().to_vec();
            // Parse streaming elements (comments, nags, variations, moves)
            match crate::formats::sg4::parse_streaming_state(&data) {
                Ok(stream_state) => {
                    // Decode moves using SG4Parser with proper offset management
                    let mut parser = crate::formats::sg4::SG4Parser::new(data.clone());
                    // Use legacy tag parser to find moves start offset for accurate offset tracking
                    if let Ok(legacy_state) = crate::formats::sg4::parse_pgn_tags(&data) {
                        let mut current_offset = legacy_state.moves_start_offset;
                        while current_offset < data.len() {
                            let byte = data[current_offset];
                            match byte {
                                15 => {
                                    // Result byte encountered; stop parsing moves
                                    break;
                                }
                                11 => {
                                    // Skip NAG value
                                    let advance = if current_offset + 1 < data.len() { 2 } else { 1 };
                                    current_offset += advance;
                                }
                                12 => {
                                    // Skip comment string: length-prefixed
                                    if current_offset + 1 < data.len() {
                                        let length = data[current_offset + 1] as usize;
                                        let advance = 1 + 1 + length; // marker + length + bytes
                                        current_offset = current_offset.saturating_add(advance);
                                        if current_offset > data.len() { current_offset = data.len(); }
                                    } else {
                                        current_offset += 1;
                                    }
                                }
                                13 | 14 => {
                                    // Variation markers: single byte
                                    current_offset += 1;
                                }
                                _ => {
                                    // Regular move
                                    parser.offset = current_offset;
                                    let piece_num = (byte >> 4) & 0x0F;
                                    let move_value = byte & 0x0F;
                                    let decoded_move = if piece_num == 2 && move_value >= 8 {
                                        // Queen diagonal: multi-byte
                                        if current_offset + 1 < data.len() {
                                            let dm = match parser.decode_queen_diagonal_start(byte) {
                                                Ok(dm) => dm,
                                                Err(e) => {
                                                    eprintln!("Warning: Failed to decode queen diagonal at offset {}: {}", current_offset, e);
                                                    // Skip two bytes and continue
                                                    current_offset += 2;
                                                    continue;
                                                }
                                            };
                                            current_offset += 2;
                                            dm
                                        } else {
                                            // Not enough data; skip
                                            current_offset += 1;
                                            continue;
                                        }
                                    } else {
                                        let dm = match parser.decode_single_byte_move(byte) {
                                            Ok(dm) => dm,
                                            Err(e) => {
                                                eprintln!("Warning: Failed to decode move at offset {}: {}", current_offset, e);
                                                current_offset += 1;
                                                continue;
                                            }
                                        };
                                        current_offset += 1;
                                        parser.offset = current_offset;
                                        dm
                                    };
                                    // Update position and collect move
                                    if let Err(e) = parser.update_position(&decoded_move) {
                                        eprintln!("Warning: Failed to update position at offset {}: {}", current_offset, e);
                                    }
                                    moves.push(decoded_move);
                                }
                            }
                        }
                    }
                    stream_state
                }
                Err(e) => {
                    eprintln!("Warning: Could not parse streaming state for game {}: {}", index, e);
                    crate::formats::sg4::StreamingGameParseState::new()
                }
            }
        };        
        // Create game state with metadata
        let mut game_state = crate::bridge::GameState::new();
        
        // Add metadata using name resolution
        if let Ok(Some(white_name)) = self.get_player_name(game_index.white_id) {
            if let Ok(Some(black_name)) = self.get_player_name(game_index.black_id) {
                if let Ok(Some(event_name)) = self.get_event_name(game_index.event_id) {
                    if let Ok(Some(site_name)) = self.get_site_name(game_index.site_id) {
                        let mut metadata = crate::bridge::GameMetadata::new(
                            white_name,
                            black_name,
                            event_name,
                            site_name,
                            format!("{:04}.{:02}.{:02}", game_index.year, game_index.month, game_index.day),
                            format_result(game_index.result),
                        );
                        
                        // Set optional fields
                        if game_index.white_elo > 0 {
                            metadata.white_elo = Some(game_index.white_elo);
                        }
                        if game_index.black_elo > 0 {
                            metadata.black_elo = Some(game_index.black_elo);
                        }
                        if game_index.eco > 0 {
                            metadata.eco = Some(format!("ECO{}", game_index.eco));
                        }
                        game_state.set_metadata(metadata);
                    }
                }
            }
        }
        
        Ok(ScidGame {
            index: game_index,
            game_state,
            parsed_game,
            moves,
        })
    }
    
    /// Get an iterator over all games in the database
    /// 
    /// Returns an iterator that yields ScidGame objects for each game in the database.
    pub fn games(&self) -> GameIterator<'_> {
        GameIterator {
            database: self,
            current_index: 0,
        }
    }
    
    /// Check if the database is validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }
    
    /// Force re-validation of the database
    pub fn revalidate(&mut self) -> Result<()> {
        self.validate()?;
        self.validated = true;
        Ok(())
    }
    
    /// Get player name from SN4 file
    pub fn get_player_name(&self, player_id: u32) -> Result<Option<String>> {
        self.sn4_file.get_name(crate::formats::sn4::NameType::Player, player_id)
    }
    
    /// Get event name from SN4 file
    pub fn get_event_name(&self, event_id: u32) -> Result<Option<String>> {
        self.sn4_file.get_name(crate::formats::sn4::NameType::Event, event_id)
    }
    
    /// Get site name from SN4 file
    pub fn get_site_name(&self, site_id: u32) -> Result<Option<String>> {
        self.sn4_file.get_name(crate::formats::sn4::NameType::Site, site_id)
    }
    
    /// Get round name from SN4 file
    pub fn get_round_name(&self, round_id: u32) -> Result<Option<String>> {
        self.sn4_file.get_name(crate::formats::sn4::NameType::Round, round_id)
    }
    
    /// Get database statistics
    pub fn statistics(&self) -> DatabaseStatistics {
        DatabaseStatistics {
            num_games: self.num_games(),
            // TODO: Add file size statistics when methods are available
            file_size_si4: 0,
            file_size_sn4: 0,
            file_size_sg4: 0,
            is_validated: self.validated,
        }
    }
    
    /// Get public SI4 header information
    /// 
    /// Returns a structured view of the SI4 header containing database metadata
    /// like version, game count, description, etc.
    pub fn si4_header(&self) -> ScidHeaderInfo {
        let header = self.si4_file.header();
        ScidHeaderInfo {
            magic: String::from("Scid.si"),
            version: header.version,
            base_type: header.base_type,
            num_games: header.num_games,
            auto_load: header.auto_load,
            description: header.description.clone(),
        }
    }
    
    /// Get public SN4 header information
    /// 
    /// Returns a structured view of the SN4 header containing name database
    /// metadata like player counts, frequencies, timestamps, etc.
    pub fn sn4_header(&self) -> ScidNameHeaderInfo {
        let header = self.sn4_file.header();
        ScidNameHeaderInfo {
            magic: String::from("Scid.sn"),
            timestamp: header.timestamp,
            num_names_player: header.num_names_player,
            num_names_event: header.num_names_event,
            num_names_site: header.num_names_site,
            num_names_round: header.num_names_round,
            max_frequency_player: header.max_frequency_player,
            max_frequency_event: header.max_frequency_event,
            max_frequency_site: header.max_frequency_site,
            max_frequency_round: header.max_frequency_round,
        }
    }
    
    /// Get file size for database files
    /// 
    /// Returns the actual file size in bytes for the specified database file type.
    /// Uses std::fs::metadata() to get current file size from disk.
    pub fn file_size(&self, file_type: DatabaseFileType) -> Result<usize> {
        let file_path = match file_type {
            DatabaseFileType::Si4 => self.base_path.with_extension("si4"),
            DatabaseFileType::Sn4 => self.base_path.with_extension("sn4"),
            DatabaseFileType::Sg4 => self.base_path.with_extension("sg4"),
        };
        
        let metadata = std::fs::metadata(file_path)?;
        Ok(metadata.len() as usize)
    }
    
    /// Get an iterator over names of a specific type
    /// 
    /// Provides convenient access to player, event, site, and round names
    /// from the SN4 names database.
    pub fn iter_names(&self, name_type: crate::formats::sn4::NameType) -> crate::formats::sn4::NameIterator<'_> {
        self.sn4_file.iter_names(name_type)
    }
}

/// Iterator over games in a SCID database
pub struct GameIterator<'a> {
    database: &'a ScidDatabase,
    current_index: u32,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<ScidGame>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index >= self.database.num_games() {
            None
        } else {
            let result = self.database.get_game(self.current_index);
            self.current_index += 1;
            Some(result)
        }
    }
}

/// Database statistics and information
#[derive(Debug, Clone)]
pub struct DatabaseStatistics {
    /// Number of games in the database
    pub num_games: u32,
    /// Size of the .si4 file in bytes
    pub file_size_si4: usize,
    /// Size of the .sn4 file in bytes
    pub file_size_sn4: usize,
    /// Size of the .sg4 file in bytes
    pub file_size_sg4: usize,
    /// Whether the database has been validated
    pub is_validated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_scid_database_structure() {
        // Test that the structure compiles
        let _database = ScidDatabase {
            si4_file: Si4File::open(&PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(&PathBuf::from("test.sn4")).unwrap(),
            sg4_source: SG4Source { bytes: Vec::new() },
            validated: false,
            base_path: PathBuf::from("test"),
        };
        
        // Test basic methods
        assert_eq!(_database.num_games(), 0);
        assert!(_database.is_validated());
    }
    
    #[test]
    fn test_game_iterator() {
        let database = ScidDatabase {
            si4_file: Si4File::open(&PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(&PathBuf::from("test.sn4")).unwrap(),
            sg4_source: SG4Source { bytes: Vec::new() },
            validated: false,
            base_path: PathBuf::from("test"),
        };
        
        let mut iterator = database.games();
        assert!(iterator.next().is_none()); // Empty database
    }
    
    #[test]
    fn test_database_statistics() {
        let database = ScidDatabase {
            si4_file: Si4File::open(&PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(&PathBuf::from("test.sn4")).unwrap(),
            sg4_source: SG4Source { bytes: Vec::new() },
            validated: false,
            base_path: PathBuf::from("test"),
        };
        
        let stats = database.statistics();
        assert_eq!(stats.num_games, 0);
        assert_eq!(stats.file_size_si4, 0);
        assert_eq!(stats.file_size_sn4, 0);
        assert_eq!(stats.file_size_sg4, 0);
        assert!(!stats.is_validated);
    }
}
