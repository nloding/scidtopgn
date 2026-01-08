//! Game file (.sg4) parsing
//!
//! # CRITICAL: Multiple Games Per File
//!
//! **THE .SG4 FILE CONTAINS ALL GAMES IN THE DATABASE**
//!
//! This is the most common source of parsing errors. Let's be crystal clear:
//!
//! ## Common Misconception
//!
//! - **WRONG ASSUMPTION**: Each .sg4 file contains one game
//! - **REALITY**: One .sg4 file contains ALL games
//!
//! - **WRONG APPROACH**: Scan file for patterns to find game boundaries
//! - **CORRECT APPROACH**: Use index file offsets
//!
//! ## Why Pattern Scanning Fails
//!
//! You might think: "I'll scan for double-null (0x00 0x00) to separate games"
//!
//! **This fails because**:
//! - Comments are null-terminated, creating 0x00 bytes within games
//! - Tag values can contain 0x00
//! - Variations can have nested null terminators
//! - End-of-game marker (0x0F) comes AFTER game data, not between games
//!
//! ## The Correct Approach
//!
//! SCID uses the index file (.si4) to track game locations:
//!
//! ```rust,ignore
//! // From index file, we know:
//! let game_offset = 0;      // Game 1 starts at byte 0
//! let game_length = 1234;   // Game 1 is 1234 bytes long
//!
//! // Read exact slice
//! let game_bytes = &sg4_data[offset..offset+length];
//! ```
//!
//! ## Real-World Example
//!
//! From five.sg4 (5-game database):
//!
//! ```text
//! Game 1: offset=0,     length=168  → bytes [0..168]
//! Game 2: offset=168,   length=165  → bytes [168..333]
//! Game 3: offset=333,   length=183  → bytes [333..516]
//! Game 4: offset=516,   length=194  → bytes [516..710]
//! Game 5: offset=710,   length=166  → bytes [710..876]
//! ```
//!
//! No gaps, no separators, no magic patterns. Just sequential data.
//!
//! ## Implementation Rule
//!
//! **ALWAYS USE INDEX FILE OFFSETS**
//!
//! Never implement:
//! - Pattern scanning
//! - Delimiter searching
//! - "Find next game" logic
//!
//! Always implement:
//! - Read offset and length from index entry
//! - Read exact byte slice from .sg4
//! - Parse that slice independently
//!
//! See SCID_DATABASE_FORMAT.md lines 482-654 for complete explanation.

use crate::error::{Result, ScidError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;

/// Maximum tag name length for regular tags
const MAX_TAG_LEN: u8 = 240;

/// Common tag names (indices 0-9 correspond to tag IDs 241-250)
///
/// From SCID source: game.cpp, commonTags array
const COMMON_TAGS: [&str; 10] = [
    "WhiteTitle",   // 241
    "BlackTitle",   // 242
    "Annotator",    // 243
    "Opening",      // 244
    "Variation",    // 245
    "SubVariation", // 246
    "WhiteElo",     // 247
    "BlackElo",     // 248
    "WhiteUSCF",    // 249
    "BlackUSCF",    // 250
];

/// Special byte for EventDate tag
const EVENT_DATE_TAG: u8 = 255;

/// Game flags bit definitions
///
/// From SCID source: game.h
pub mod flags {
    /// Non-standard starting position (bit 0)
    ///
    /// If set, game includes FEN string for starting position
    pub const NON_STANDARD_START: u8 = 0x01;

    /// Game has pawn promotions (bit 1)
    pub const HAS_PROMOTIONS: u8 = 0x02;

    /// Game has under-promotions (bit 2)
    ///
    /// Promotions to pieces other than Queen
    pub const HAS_UNDER_PROMOS: u8 = 0x04;

    /// Game is marked for deletion (bit 3)
    pub const MARKED_DELETED: u8 = 0x08;

    /// White opening repertoire (bit 4)
    pub const WHITE_OPENINGS: u8 = 0x10;

    /// Black opening repertoire (bit 5)
    pub const BLACK_OPENINGS: u8 = 0x20;

    /// Check if flag is set
    pub fn is_set(flags: u8, flag: u8) -> bool {
        (flags & flag) != 0
    }
}

/// Game data extracted from .sg4 file
///
/// Contains parsed tags, flags, and raw move data.
/// Move data will be parsed in Phase 5.
#[derive(Debug, Clone)]
pub struct GameData {
    /// PGN tags (e.g., "WhiteTitle" → "GM", "Event" → "Tournament")
    pub tags: HashMap<String, String>,

    /// Game flags byte (promotions, custom start, etc.)
    pub flags: u8,

    /// Starting position FEN (if non-standard start)
    pub start_position: Option<String>,

    /// Raw move data bytes (to be parsed in Phase 5)
    pub move_data: Vec<u8>,
}

impl GameData {
    /// Create empty game data
    pub fn new() -> Self {
        GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            move_data: Vec::new(),
        }
    }
}

impl Default for GameData {
    fn default() -> Self {
        Self::new()
    }
}

/// Read game data from .sg4 file using index entry
///
/// This is the ONLY correct way to read games from .sg4 files.
///
/// # Arguments
///
/// * `file` - Opened .sg4 file
/// * `offset` - Byte offset from index entry
/// * `length` - Byte length from index entry
///
/// # Returns
///
/// Exact byte slice containing this game's data
///
/// # How It Works
///
/// ```text
/// .sg4 file layout:
/// ┌────────────┬────────────┬────────────┬────────────┐
/// │ Game 1     │ Game 2     │ Game 3     │ Game 4     │
/// │ (168 bytes)│ (165 bytes)│ (183 bytes)│ (194 bytes)│
/// └────────────┴────────────┴────────────┴────────────┘
///  ↑            ↑            ↑            ↑
///  offset=0     offset=168   offset=333   offset=516
/// ```
///
/// We seek to `offset` and read exactly `length` bytes.
///
/// # Why Not Scan?
///
/// Scanning for patterns FAILS because:
/// - Comments contain null bytes within games
/// - No reliable separator between games
/// - Bytes like 0x0F (END_GAME) are not separators
///
/// See SCID_DATABASE_FORMAT.md lines 535-598 for proof.
pub fn read_game_data(file: &mut File, offset: u32, length: u32) -> Result<Vec<u8>> {
    // Validate length
    if length == 0 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: offset as u64,
            message: "Game length is zero".to_string(),
        });
    }

    // Maximum game size is 131,071 bytes (17-bit value from index)
    if length > 131071 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: offset as u64,
            message: format!("Game length {} exceeds maximum 131071", length),
        });
    }

    // Seek to offset
    file.seek(SeekFrom::Start(offset as u64))?;

    // Read exact length
    let mut game_bytes = vec![0u8; length as usize];
    file.read_exact(&mut game_bytes)
        .map_err(|e| ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: offset as u64,
            message: format!("Failed to read {} bytes: {}", length, e),
        })?;

    Ok(game_bytes)
}

/// Parse PGN tags from game data
///
/// Reads tags until null terminator (0x00), then returns tags and position.
///
/// # Arguments
///
/// * `bytes` - Game data bytes (from read_game_data)
///
/// # Returns
///
/// Tuple of:
/// - HashMap of tags (name → value)
/// - Position in bytes where tags end (next byte is flags)
///
/// # Tag Types
///
/// 1. **Common tags** (byte > 240): `[tag_id][value_length][value_bytes]`
/// 2. **Regular tags** (byte ≤ 240): `[name_length][name_bytes][value_length][value_bytes]`
/// 3. **Special EventDate** (byte = 255): `[255][3_date_bytes]`
///
/// See SCID_DATABASE_FORMAT.md lines 690-724 for encoding details.
pub fn parse_game_tags(bytes: &[u8]) -> Result<(HashMap<String, String>, usize)> {
    let mut tags = HashMap::new();
    let mut pos = 0;

    loop {
        // Check bounds
        if pos >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: pos as u64,
                message: "Unexpected end of data while parsing tags".to_string(),
            });
        }

        let byte = bytes[pos];
        pos += 1;

        // Check for end of tags
        if byte == 0 {
            // Null terminator - tags section complete
            break;
        }

        // Parse tag based on type
        if byte == EVENT_DATE_TAG {
            // Special EventDate encoding (255 + 3 bytes)
            // Skip for now - we get date from index file
            if pos + 3 > bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: "Truncated EventDate tag".to_string(),
                });
            }
            pos += 3;
        } else if byte > MAX_TAG_LEN {
            // Common tag (241-250)
            let tag_index = (byte - MAX_TAG_LEN - 1) as usize;

            if tag_index >= COMMON_TAGS.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Unknown common tag ID: {}", byte),
                });
            }

            let tag_name = COMMON_TAGS[tag_index];

            // Read value length
            if pos >= bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: "Missing value length for common tag".to_string(),
                });
            }

            let value_len = bytes[pos] as usize;
            pos += 1;

            // Read value
            if pos + value_len > bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Truncated value for tag {}", tag_name),
                });
            }

            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(tag_name.to_string(), value);
        } else {
            // Regular tag (byte ≤ 240)
            let name_len = byte as usize;

            // Read tag name
            if pos + name_len > bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: "Truncated tag name".to_string(),
                });
            }

            let name = String::from_utf8_lossy(&bytes[pos..pos + name_len]).to_string();
            pos += name_len;

            // Read value length
            if pos >= bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Missing value length for tag {}", name),
                });
            }

            let value_len = bytes[pos] as usize;
            pos += 1;

            // Read value
            if pos + value_len > bytes.len() {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Truncated value for tag {}", name),
                });
            }

            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(name, value);
        }
    }

    Ok((tags, pos))
}

/// Parse complete game structure: tags, flags, optional FEN
///
/// # Arguments
///
/// * `bytes` - Complete game data from read_game_data()
///
/// # Returns
///
/// Complete GameData with tags, flags, and move data
///
/// # Structure
///
/// ```text
/// bytes layout:
/// ┌──────────────┬──────┬──────────┬────────────┐
/// │ Tags         │Flags │ FEN?     │ Move Data  │
/// │ [var len]    │[1]   │ [var]    │ [var len]  │
/// └──────────────┴──────┴──────────┴────────────┘
/// ```
pub fn parse_game_structure(bytes: &[u8]) -> Result<GameData> {
    // Step 1: Parse tags
    let (tags, mut pos) = parse_game_tags(bytes)?;

    // Step 2: Read flags byte
    if pos >= bytes.len() {
        return Err(ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: pos as u64,
            message: "Missing flags byte".to_string(),
        });
    }

    let game_flags = bytes[pos];
    pos += 1;

    // Step 3: Check for non-standard start position
    let start_position = if flags::is_set(game_flags, flags::NON_STANDARD_START) {
        // Read null-terminated FEN string
        let fen_start = pos;
        let mut fen_end = pos;

        // Find null terminator
        while fen_end < bytes.len() && bytes[fen_end] != 0 {
            fen_end += 1;
        }

        if fen_end >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: pos as u64,
                message: "Unterminated FEN string".to_string(),
            });
        }

        let fen = String::from_utf8_lossy(&bytes[fen_start..fen_end]).to_string();
        pos = fen_end + 1; // Skip past null terminator

        Some(fen)
    } else {
        None
    };

    // Step 4: Remaining bytes are move data
    let move_data = bytes[pos..].to_vec();

    Ok(GameData {
        tags,
        flags: game_flags,
        start_position,
        move_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data_default() {
        let data = GameData::default();
        assert_eq!(data.flags, 0);
        assert!(data.tags.is_empty());
        assert!(data.start_position.is_none());
        assert!(data.move_data.is_empty());
    }

    #[test]
    fn test_read_game_data_validation() {
        // Test maximum length validation
        let max_length = 131071u32;
        assert!(max_length <= 131071);

        let invalid_length = 200000u32;
        assert!(invalid_length > 131071);
    }

    #[test]
    fn test_parse_common_tag() {
        // WhiteTitle (241) = "GM"
        let bytes = vec![
            241, // WhiteTitle tag ID
            2,   // Value length
            b'G', b'M', // Value
            0,    // End of tags
        ];

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        assert_eq!(tags.get("WhiteTitle"), Some(&"GM".to_string()));
        assert_eq!(pos, 5); // Position after null terminator
    }

    #[test]
    fn test_parse_regular_tag() {
        // Custom tag "MyTag" = "value"
        let bytes = vec![
            5, // Name length
            b'M', b'y', b'T', b'a', b'g', // Name
            5,    // Value length
            b'v', b'a', b'l', b'u', b'e', // Value
            0,    // End of tags
        ];

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        assert_eq!(tags.get("MyTag"), Some(&"value".to_string()));
        assert_eq!(pos, 13);
    }

    #[test]
    fn test_parse_multiple_tags() {
        let bytes = vec![
            // WhiteTitle = "GM"
            241, 2, b'G', b'M', // BlackTitle = "IM"
            242, 2, b'I', b'M', // Custom tag
            4, b'T', b'e', b's', b't', // "Test"
            2, b'O', b'K', // "OK"
            // End
            0,
        ];

        let (tags, _) = parse_game_tags(&bytes).unwrap();

        assert_eq!(tags.len(), 3);
        assert_eq!(tags.get("WhiteTitle"), Some(&"GM".to_string()));
        assert_eq!(tags.get("BlackTitle"), Some(&"IM".to_string()));
        assert_eq!(tags.get("Test"), Some(&"OK".to_string()));
    }

    #[test]
    fn test_parse_empty_tags() {
        let bytes = vec![0]; // Just null terminator

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        assert!(tags.is_empty());
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_parse_event_date_skip() {
        let bytes = vec![
            255, // EventDate special tag
            0x12, 0x34, 0x56, // 3-byte date (skipped)
            0,    // End of tags
        ];

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        // EventDate is skipped, not added to tags
        assert!(tags.is_empty());
        assert_eq!(pos, 5);
    }

    #[test]
    fn test_parse_tags_error_truncated() {
        let bytes = vec![
            241, // WhiteTitle
            10,  // Claims 10 bytes but...
            b'G', b'M', // Only 2 bytes!
                  // Missing data
        ];

        let result = parse_game_tags(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_tags_error_no_terminator() {
        let bytes = vec![
            241, 2, b'G', b'M',
            // Missing null terminator
        ];

        let result = parse_game_tags(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_flags_standard_start() {
        let bytes = vec![
            0,    // No tags, just null terminator
            0x02, // Flags: HAS_PROMOTIONS
            // Move data follows...
            0x10, 0x20, 0x30,
        ];

        let game_data = parse_game_structure(&bytes).unwrap();

        assert_eq!(game_data.flags, 0x02);
        assert!(flags::is_set(game_data.flags, flags::HAS_PROMOTIONS));
        assert!(!flags::is_set(game_data.flags, flags::NON_STANDARD_START));
        assert!(game_data.start_position.is_none());
        assert_eq!(game_data.move_data, vec![0x10, 0x20, 0x30]);
    }

    #[test]
    fn test_parse_flags_custom_start() {
        let bytes = vec![
            0,    // No tags
            0x01, // Flags: NON_STANDARD_START
            // FEN string (null-terminated)
            b'r', b'n', b'b', b'q', 0, // Move data
            0x10, 0x20,
        ];

        let game_data = parse_game_structure(&bytes).unwrap();

        assert_eq!(game_data.flags, 0x01);
        assert!(flags::is_set(game_data.flags, flags::NON_STANDARD_START));
        assert!(game_data.start_position.is_some());
        assert_eq!(game_data.start_position.unwrap(), "rnbq");
        assert_eq!(game_data.move_data, vec![0x10, 0x20]);
    }

    #[test]
    fn test_flags_helpers() {
        let game_flags: u8 = 0x03; // Bits 0 and 1 set

        assert!(flags::is_set(game_flags, flags::NON_STANDARD_START));
        assert!(flags::is_set(game_flags, flags::HAS_PROMOTIONS));
        assert!(!flags::is_set(game_flags, flags::HAS_UNDER_PROMOS));
    }

    #[test]
    fn test_parse_complete_game_structure() {
        let bytes = vec![
            // Tags
            241, 2, b'G', b'M', // WhiteTitle = "GM"
            0,    // End tags
            // Flags
            0x00, // Standard start
            // Move data
            0xAA, 0xBB, 0xCC, 0x0F, // Some move bytes + END_GAME marker
        ];

        let game_data = parse_game_structure(&bytes).unwrap();

        assert_eq!(game_data.tags.len(), 1);
        assert_eq!(game_data.tags.get("WhiteTitle"), Some(&"GM".to_string()));
        assert_eq!(game_data.flags, 0x00);
        assert!(game_data.start_position.is_none());
        assert_eq!(game_data.move_data, vec![0xAA, 0xBB, 0xCC, 0x0F]);
    }
}
