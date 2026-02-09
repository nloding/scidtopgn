//! Game file (.sg4) parsing
//!
//! The game file stores the actual chess moves and annotations for all games.
//! Game data is stored compressed using zlib for space efficiency.
//!
//! # File Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ SG4 Header (4 bytes)                                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Game 1 Data (variable length, compressed)                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Game 2 Data (variable length, compressed)                  │
//! ├─────────────────────────────────────────────────────────────┤
//! │ ...                                                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Compression
//!
//! Most SCID databases use zlib compression. The `is_packed` flag in the
//! game index entry indicates whether a game's data is compressed.
//!
//! # See Also
//!
//! - `SCID_DATABASE_FORMAT.md` for complete file format specification
//! - `index` module for game metadata that references these games

use crate::error::{Result, ScidError};
use flate2::read::ZlibDecoder;
use shakmaty::Move;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// Magic bytes identifying a valid SG4 file
///
/// All SCID game files must start with these exact 4 bytes.
/// See SCID_DATABASE_FORMAT.md for details.
pub const SG4_MAGIC: &[u8; 4] = b"Scid";

/// Size of SG4 header in bytes
pub const SG4_HEADER_SIZE: usize = 4;

/// Game data structure containing parsed game information
///
/// This struct contains all information about a single game extracted from
/// the .sg4 file.
#[derive(Debug, Clone)]
pub struct GameData {
    /// Custom tags from the game file
    pub tags: HashMap<String, String>,

    /// Game flags
    pub flags: u32,

    /// Starting position FEN string (if non-standard start)
    pub start_position: Option<String>,

    /// List of moves in the game
    pub moves: Vec<Move>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: Vec::new(),
        }
    }
}

/// Read raw game data from .sg4 file
///
/// # Arguments
///
/// * `file` - Reference to the opened .sg4 file
/// * `offset` - Byte offset where the game data starts
/// * `length` - Length of the game data in bytes
///
/// # Returns
///
/// Raw bytes of the game data
///
/// # Errors
///
/// - `ScidError::Io` - File read error
/// - `ScidError::InvalidFormat` - Invalid file structure
pub fn read_game_data(file: &mut File, offset: u32, length: u32) -> Result<Vec<u8>> {
    // Seek to the game data offset
    file.seek(SeekFrom::Start(offset as u64))
        .map_err(ScidError::Io)?;

    // Allocate buffer for game data
    let mut buffer = vec![0u8; length as usize];

    // Read the game data
    file.read_exact(&mut buffer).map_err(ScidError::Io)?;

    Ok(buffer)
}

/// Decompress game data using zlib
///
/// # Arguments
///
/// * `compressed_data` - Compressed game data bytes
///
/// # Returns
///
/// Decompressed game data bytes
///
/// # Errors
///
/// - `ScidError::DecompressionError` - Decompression failed
pub fn decompress_game_data(compressed_data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed_data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| ScidError::DecompressionError(e.to_string()))?;

    Ok(decompressed)
}

/// Read and decompress game data from .sg4 file
///
/// This is a convenience function that combines `read_game_data` and
/// `decompress_game_data`. It's the most commonly used function for
/// reading game data from SCID databases.
///
/// # Arguments
///
/// * `file` - Reference to the opened .sg4 file
/// * `offset` - Byte offset where the game data starts
/// * `length` - Length of the game data in bytes
/// * `packed` - Whether the game data is zlib compressed
///
/// # Returns
///
/// Decompressed game data bytes (or raw bytes if not packed)
///
/// # Errors
///
/// - `ScidError::Io` - File read error
/// - `ScidError::DecompressionError` - Decompression failed
/// - `ScidError::InvalidFormat` - Invalid file structure
///
/// # Example
///
/// ```no_run
/// # use scidtopgn_core::database::games::read_and_decompress_game;
/// # use std::fs::File;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut sg4_file = File::open("database.sg4")?;
///
/// // Get offset and length from index entry
/// let offset = index_entry.game_offset;
/// let length = index_entry.game_length;
/// let packed = index_entry.is_packed();
///
/// // Read and decompress game data
/// let game_bytes = read_and_decompress_game(&mut sg4_file, offset, length, packed)?;
/// # Ok(())
/// # }
/// ```
pub fn read_and_decompress_game(
    file: &mut File,
    offset: u32,
    length: u32,
    packed: bool,
) -> Result<Vec<u8>> {
    let raw_data = read_game_data(file, offset, length)?;

    if packed {
        decompress_game_data(&raw_data)
    } else {
        Ok(raw_data)
    }
}

/// Parse game data from decompressed bytes
///
/// # Arguments
///
/// * `data` - Decompressed game data bytes
///
/// # Returns
///
/// Parsed GameData structure
///
/// # Errors
///
/// - `ScidError::ParseError` - Failed to parse game data
///
/// # Note
///
/// This is a placeholder implementation. The actual parsing of SCID game data
/// format is complex and requires implementing the move decoding algorithms
/// described in SCID_DATABASE_FORMAT.md.
pub fn parse_game(data: &[u8]) -> Result<GameData> {
    // Placeholder implementation - returns empty game data
    // Full implementation would parse:
    // 1. Game flags
    // 2. Tags (if any)
    // 3. Starting position (if custom)
    // 4. Moves using SCID move encoding

    let game_data = GameData {
        tags: HashMap::new(),
        flags: 0,
        start_position: None,
        moves: Vec::new(),
    };

    Ok(game_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data_default() {
        let game_data = GameData::default();

        assert!(game_data.tags.is_empty());
        assert_eq!(game_data.flags, 0);
        assert!(game_data.start_position.is_none());
        assert!(game_data.moves.is_empty());
    }

    #[test]
    fn test_game_data_clone() {
        let mut game_data = GameData::default();
        game_data
            .tags
            .insert("Test".to_string(), "Value".to_string());

        let cloned = game_data.clone();
        assert_eq!(cloned.tags.get("Test"), Some(&"Value".to_string()));
    }
}
