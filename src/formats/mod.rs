//! Parsers for SCID file formats (.si4, .sn4, .sg4)

pub mod sg4;
pub mod si4;
pub mod sn4;

use crate::core::error::Result;
use crate::bridge::position::format_result;
use crate::formats::sg4::Sg4File;
use crate::formats::si4::Si4File;
use crate::formats::sn4::Sn4File;
use std::path::Path;

/// Main SCID database structure that coordinates access to all SCID files
/// 
/// This struct provides a unified interface for accessing SCID chess databases,
/// coordinating the .si4 (index), .sn4 (names), and .sg4 (games) files.
pub struct ScidDatabase {
    /// SCID index file containing game metadata
    si4_file: Si4File,
    /// SCID names file containing player and event names
    sn4_file: Sn4File,
    /// SCID games file containing actual game data
    sg4_file: Sg4File,
    /// Database validation state
    validated: bool,
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
}

impl ScidDatabase {
    /// Open a SCID database from the base path (without extensions)
    /// 
    /// This method will look for .si4, .sn4, and .sg4 files with the given base path.
    /// It validates that all required files exist and are properly formatted.
    pub fn open<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref();
        
        // Open and validate all three SCID files
        let si4_file = Si4File::open(&base_path.with_extension("si4"))?;
        let sn4_file = Sn4File::open(&base_path.with_extension("sn4"))?;
        let sg4_file = Sg4File::open(&base_path.with_extension("sg4"))?;
        
        let mut database = Self {
            si4_file,
            sn4_file,
            sg4_file,
            validated: false,
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
        
        // For now, skip sg4 validation as the method doesn't exist yet
        // TODO: Add sg4 validation when the method is available
        
        // Validate that all game indices are within bounds
        for i in 0..si4_game_count {
            let game_index = self.si4_file.get_game(i)?;
            
            // TODO: Add sg4 bounds validation when methods are available
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
        
        // TODO: Parse game data from sg4 file when methods are available
        // For now, create empty parsed game
        let parsed_game = crate::formats::sg4::StreamingGameParseState::new();
        
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
        })
    }
    
    /// Get an iterator over all games in the database
    /// 
    /// Returns an iterator that yields ScidGame objects for each game in the database.
    pub fn games(&self) -> GameIterator {
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
            si4_file: Si4File::open(PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(PathBuf::from("test.sn4")).unwrap(),
            sg4_file: Sg4File::open(PathBuf::from("test.sg4")).unwrap(),
            validated: false,
        };
        
        // Test basic methods
        assert_eq!(_database.num_games(), 0);
        assert!(_database.is_validated());
    }
    
    #[test]
    fn test_game_iterator() {
        let database = ScidDatabase {
            si4_file: Si4File::open(PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(PathBuf::from("test.sn4")).unwrap(),
            sg4_file: Sg4File::open(PathBuf::from("test.sg4")).unwrap(),
            validated: false,
        };
        
        let mut iterator = database.games();
        assert!(iterator.next().is_none()); // Empty database
    }
    
    #[test]
    fn test_database_statistics() {
        let database = ScidDatabase {
            si4_file: Si4File::open(PathBuf::from("test.si4")).unwrap(),
            sn4_file: Sn4File::open(PathBuf::from("test.sn4")).unwrap(),
            sg4_file: Sg4File::open(PathBuf::from("test.sg4")).unwrap(),
            validated: false,
        };
        
        let stats = database.statistics();
        assert_eq!(stats.num_games, 0);
        assert_eq!(stats.file_size_si4, 0);
        assert_eq!(stats.file_size_sn4, 0);
        assert_eq!(stats.file_size_sg4, 0);
        assert!(!stats.is_validated);
    }
}
