use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

/// SCID sg4 game file parser
/// Contains the actual moves, variations and comments of each game

pub struct GameFile {
    file: File,
}

impl GameFile {
    /// Load a SCID .sg4 game file
    pub fn load<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Ok(GameFile { file })
    }
    
    /// Get the raw game data for a specific offset and length
    pub fn game_data(&mut self, offset: u32, length: u16) -> io::Result<Vec<u8>> {
        self.file.seek(SeekFrom::Start(offset as u64))?;
        
        let mut buffer = vec![0u8; length as usize];
        self.file.read_exact(&mut buffer)?;
        
        Ok(buffer)
    }
}
