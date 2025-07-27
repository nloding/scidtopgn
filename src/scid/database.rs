use std::path::{Path, PathBuf};
use std::io;

use super::{index::IndexFile, names::NameDatabase, games::GameFile};
use super::{ScidHeader, GameIndex};

/// Main SCID database structure - INTEGRATION OF ALL MAJOR FIXES
/// 
/// ## Integration Notes (July 2025)
/// This structure combines all the major fixes implemented:
/// 1. **Fixed Date Parsing**: IndexFile now correctly parses dates (1791.12.24 vs 52298.152.207)
/// 2. **Fixed Name Extraction**: NameDatabase correctly extracts full names (Michael vs ichael)  
/// 3. **Proper Error Handling**: Converts between error types for seamless integration
/// 
/// ## SCID Database File Structure
/// - **base_name.si4**: Index file with game metadata, dates, player/event IDs
/// - **base_name.sn4**: Name database with player, event, site, round names
/// - **base_name.sg4**: Game file with actual chess moves and annotations
/// 
/// ## Error Type Integration Challenge
/// NameDatabase::parse_names() returns `Result<NameDatabase, Box<dyn std::error::Error>>`
/// but load() needs to return `io::Result<Self>`. Fixed with .map_err() conversion.
/// 
/// Contains all three SCID files integrated into a single interface
pub struct ScidDatabase {
    index: IndexFile,
    names: NameDatabase,
    games: GameFile,
    base_path: PathBuf,
}

impl ScidDatabase {
    /// Load a SCID database from the base path (without extension)
    /// Will look for .si4, .sn4, and .sg4 files
    pub fn load<P: AsRef<Path>>(base_path: P) -> io::Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        // Construct file paths
        let mut si4_path = base_path.clone();
        si4_path.set_extension("si4");
        
        let mut sn4_path = base_path.clone();
        sn4_path.set_extension("sn4");
        
        let mut sg4_path = base_path.clone();
        sg4_path.set_extension("sg4");
        
        // Check that all files exist
        if !si4_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Index file not found: {}", si4_path.display())
            ));
        }
        
        if !sn4_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Name file not found: {}", sn4_path.display())
            ));
        }
        
        if !sg4_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Game file not found: {}", sg4_path.display())
            ));
        }
        
        // Load the files
        let index = IndexFile::load(si4_path)?;
        let names = NameDatabase::parse_names(sn4_path.to_str().unwrap())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
        let games = GameFile::load(sg4_path)?;
        
        Ok(ScidDatabase {
            index,
            names,
            games,
            base_path,
        })
    }
    
    /// Get the database header
    pub fn header(&self) -> &ScidHeader {
        self.index.header()
    }
    
    /// Get the number of games in the database
    pub fn num_games(&self) -> usize {
        self.index.num_games()
    }
    
    /// Get a game index by ID
    pub fn game_index(&self, game_id: usize) -> Option<&GameIndex> {
        self.index.game_index(game_id)
    }
    
    /// Get all game indices
    pub fn game_indices(&self) -> &[GameIndex] {
        self.index.game_indices()
    }
    
    /// Get a player name by ID
    pub fn player_name(&self, player_id: u32) -> Option<&str> {
        self.names.player_name(player_id)
    }
    
    /// Get an event name by ID
    pub fn event_name(&self, event_id: u32) -> Option<&str> {
        self.names.event_name(event_id)
    }
    
    /// Get a site name by ID
    pub fn site_name(&self, site_id: u32) -> Option<&str> {
        self.names.site_name(site_id)
    }
    
    /// Get a round name by ID
    pub fn round_name(&self, round_id: u16) -> Option<&str> {
        self.names.round_name(round_id)
    }
    
    /// Get the raw game data for a game
    pub fn game_data(&mut self, game_index: &GameIndex) -> io::Result<Vec<u8>> {
        self.games.game_data(game_index.offset, game_index.length)
    }
    
    /// Get the base path of the database
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}
