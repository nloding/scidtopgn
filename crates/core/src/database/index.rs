//! Index file (.si4) parsing
//!
//! The index file contains a 182-byte header followed by 47-byte entries for each game.
//! All metadata is stored here for fast searching without loading game data.
//!
//! # File Structure
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ SI4 Header (182 bytes)                                      │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Game 1 Entry (47 bytes)                                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Game 2 Entry (47 bytes)                                     │
//! ├─────────────────────────────────────────────────────────────┤
//! │ ...                                                         │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # References
//!
//! See SCID_DATABASE_FORMAT.md:
//! - Lines 65-119: Header specification
//! - Lines 120-210: Game entry specification
//! - Lines 1013-1051: Endianness (BIG-ENDIAN required!)

use crate::error::{Result, ScidError};
use crate::types::{GameDate, GameResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

/// Magic bytes identifying a valid SI4 file
///
/// All SCID index files must start with these exact 8 bytes.
/// See SCID_DATABASE_FORMAT.md line 88.
pub const SI4_MAGIC: &[u8; 8] = b"Scid.si\0";

/// Size of SI4 header in bytes
///
/// The header is always exactly 182 bytes.
/// See SCID_DATABASE_FORMAT.md line 85.
pub const SI4_HEADER_SIZE: usize = 182;

/// Size of each game index entry in bytes
///
/// Each game has exactly one 47-byte entry.
/// See SCID_DATABASE_FORMAT.md line 120.
pub const GAME_ENTRY_SIZE: usize = 47;

/// Expected SCID version number
///
/// Standard SCID databases use version 400.
/// See SCID_DATABASE_FORMAT.md line 90.
pub const SCID_VERSION: u16 = 400;

/// SI4 file header (182 bytes)
///
/// Contains database-level metadata. All multi-byte values are BIG-ENDIAN.
///
/// # Field Offsets
///
/// | Offset | Size | Field          |
/// |--------|------|----------------|
/// | 0-7    | 8    | magic          |
/// | 8-9    | 2    | version        |
/// | 10-13  | 4    | base_type      |
/// | 14-16  | 3    | num_games      |
/// | 17-19  | 3    | auto_load      |
/// | 20-127 | 108  | description    |
/// | 128-181| 54   | custom_flags   |
///
/// # Complete SI4 File Structure
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────┐
/// │ HEADER (182 bytes)                                          │
/// │   Bytes 0-7:    Magic "Scid.si\0"                          │
/// │   Bytes 8-9:    Version (big-endian u16, typically 400)    │
/// │   Bytes 10-13:  Base type (big-endian u32)                 │
/// │   Bytes 14-16:  Number of games (24-bit big-endian)        │
/// │   Bytes 17-19:  Auto-load game number (24-bit big-endian)  │
/// │   Bytes 20-127: Description (UTF-8, null-padded)           │
/// │   Bytes 128-181: Custom flag names (6 × 9 bytes each)      │
/// ├─────────────────────────────────────────────────────────────┤
/// │ INDEX ENTRIES (47 bytes × num_games)                        │
/// │   Each entry contains game metadata (see GameIndexEntry)    │
/// │   Games are stored in insertion order (game 0, 1, 2, ...)   │
/// ├─────────────────────────────────────────────────────────────┤
/// │ SORTING INDEX (4 bytes × num_games) - OPTIONAL              │
/// │   Array of 32-bit game numbers in sorted order              │
/// │   Allows iteration in a specific sort order without         │
/// │   re-sorting the index entries                              │
/// │                                                             │
/// │   Example: If sorting_index = [3, 1, 0, 2], then:          │
/// │   - First in sorted order: game 3                           │
/// │   - Second in sorted order: game 1                          │
/// │   - etc.                                                    │
/// │                                                             │
/// │   This section may be absent in older databases             │
/// └─────────────────────────────────────────────────────────────┘
/// ```
///
/// See SCID_DATABASE_FORMAT.md lines 85-96 for complete specification.
#[derive(Debug, Clone, PartialEq)]
pub struct Si4Header {
    /// File format version (should be 400)
    pub version: u16,

    /// Database type flags
    pub base_type: u32,

    /// Total number of games in database
    ///
    /// This is a 24-bit value (max 16,777,215 games).
    pub num_games: u32,

    /// Auto-load game number
    ///
    /// Game to automatically load when opening database (usually 0).
    pub auto_load: u32,

    /// Database description text
    ///
    /// UTF-8 string, null-terminated, max 108 bytes.
    pub description: String,

    /// Custom flag names (6 user-defined flags)
    ///
    /// SCID allows users to define 6 custom flags with names up to 8 characters.
    /// These correspond to game_flags::CUSTOM_FLAG_1 through CUSTOM_FLAG_6.
    /// Empty strings indicate unnamed/unused flags.
    pub custom_flag_names: [String; 6],
}

/// Size of each custom flag name field in the header
const CUSTOM_FLAG_NAME_SIZE: usize = 9; // 8 chars + null terminator

impl Si4Header {
    /// Create a new header with default values (for testing)
    pub fn new() -> Self {
        Si4Header {
            version: SCID_VERSION,
            base_type: 0,
            num_games: 0,
            auto_load: 0,
            description: String::new(),
            custom_flag_names: Default::default(),
        }
    }

    /// Builder: set number of games
    pub fn with_num_games(mut self, num_games: u32) -> Self {
        self.num_games = num_games;
        self
    }

    /// Builder: set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Get the name for a custom flag (1-6)
    ///
    /// Returns None if flag_num is out of range.
    /// Returns empty string if flag is not named.
    pub fn get_custom_flag_name(&self, flag_num: u8) -> Option<&str> {
        if flag_num >= 1 && flag_num <= 6 {
            Some(&self.custom_flag_names[(flag_num - 1) as usize])
        } else {
            None
        }
    }
}

impl Default for Si4Header {
    fn default() -> Self {
        Self::new()
    }
}

/// Game flags module
///
/// SCID uses a 16-bit flags field to store various game attributes.
/// The flags are stored in bytes 7-8 of each game index entry.
pub mod game_flags {
    /// Game has a non-standard starting position (custom FEN)
    pub const START_FLAG: u16 = 1 << 0;

    /// Game contains pawn promotions
    pub const PROMO_FLAG: u16 = 1 << 1;

    /// Game contains underpromotions (promotion to non-queen)
    pub const UNDER_PROMO_FLAG: u16 = 1 << 2;

    /// Game is marked for deletion
    pub const DELETE_FLAG: u16 = 1 << 3;

    /// White is missing opening theory (marked by user)
    pub const WHITE_OP_FLAG: u16 = 1 << 4;

    /// Black is missing opening theory (marked by user)
    pub const BLACK_OP_FLAG: u16 = 1 << 5;

    /// Game contains middle-game annotation
    pub const MIDDLEGAME_FLAG: u16 = 1 << 6;

    /// Game contains endgame annotation
    pub const ENDGAME_FLAG: u16 = 1 << 7;

    /// Game contains novelty move
    pub const NOVELTY_FLAG: u16 = 1 << 8;

    /// Game is pawn structure study
    pub const PAWN_FLAG: u16 = 1 << 9;

    /// Custom flag 1 (user-defined, name from header)
    pub const CUSTOM_FLAG_1: u16 = 1 << 10;

    /// Custom flag 2 (user-defined, name from header)
    pub const CUSTOM_FLAG_2: u16 = 1 << 11;

    /// Custom flag 3 (user-defined, name from header)
    pub const CUSTOM_FLAG_3: u16 = 1 << 12;

    /// Custom flag 4 (user-defined, name from header)
    pub const CUSTOM_FLAG_4: u16 = 1 << 13;

    /// Custom flag 5 (user-defined, name from header)
    pub const CUSTOM_FLAG_5: u16 = 1 << 14;

    /// Custom flag 6 (user-defined, name from header)
    pub const CUSTOM_FLAG_6: u16 = 1 << 15;

    /// All custom flag bits (flags 1-6)
    pub const ALL_CUSTOM_FLAGS: u16 = CUSTOM_FLAG_1
        | CUSTOM_FLAG_2
        | CUSTOM_FLAG_3
        | CUSTOM_FLAG_4
        | CUSTOM_FLAG_5
        | CUSTOM_FLAG_6;

    /// Get the bit mask for a custom flag (1-6)
    pub const fn custom_flag_mask(flag_num: u8) -> Option<u16> {
        match flag_num {
            1 => Some(CUSTOM_FLAG_1),
            2 => Some(CUSTOM_FLAG_2),
            3 => Some(CUSTOM_FLAG_3),
            4 => Some(CUSTOM_FLAG_4),
            5 => Some(CUSTOM_FLAG_5),
            6 => Some(CUSTOM_FLAG_6),
            _ => None,
        }
    }
}

/// Extended game flags (stored in separate field, typically byte 6)
///
/// These flags are stored separately from the main 16-bit flags field.
/// The FLAG_PACKED flag is CRITICAL - it indicates the game data is
/// compressed with zlib and MUST be decompressed before parsing moves.
///
/// See SCID_DATABASE_FORMAT.md and scidvspc codec_scid4.cpp for details.
pub mod extended_game_flags {
    /// Game data is zlib compressed (CRITICAL!)
    ///
    /// When this flag is set, the game data in .sg4 MUST be decompressed
    /// using zlib before parsing. Most real-world SCID databases use
    /// compression, so this is essential for proper parsing.
    ///
    /// # Implementation
    ///
    /// ```text
    /// use flate2::read::ZlibDecoder;
    ///
    /// if entry.is_packed() {
    ///     let decoder = ZlibDecoder::new(&compressed_data[..]);
    ///     // Read decompressed data...
    /// }
    /// ```
    ///
    /// See Phase 4 for complete decompression implementation.
    pub const FLAG_PACKED: u8 = 0x80; // Bit 7 of length_high byte
}

/// Rating type enumeration (Gap 15)
///
/// SCID stores ratings with a 4-bit type indicator in the high bits.
/// This allows distinguishing between different rating systems.
///
/// # Encoding
///
/// Rating values are stored as 16-bit values:
/// - Bits 15-12: Rating type (0-15)
/// - Bits 11-0: Rating value (0-4095)
///
/// # Example
///
/// ```
/// # use scidtopgn_core::database::RatingType;
/// let raw_rating: u16 = 0x1944; // Type 1 (Elo), value 2372
/// let rating_type = RatingType::from_u8((raw_rating >> 12) as u8);
/// let rating_value = raw_rating & 0x0FFF;
/// assert_eq!(rating_type, RatingType::Elo);
/// assert_eq!(rating_value, 2372);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RatingType {
    /// No rating type specified (value 0)
    #[default]
    None = 0,

    /// Standard Elo rating (most common)
    Elo = 1,

    /// Rapid chess rating
    Rapid = 2,

    /// ICCF (International Correspondence Chess Federation) rating
    Iccf = 3,

    /// USCF (United States Chess Federation) rating
    Uscf = 4,

    /// DWZ (Deutsche Wertungszahl) - German rating system
    Dwz = 5,

    /// ECF (English Chess Federation) rating
    Ecf = 6,

    /// Unknown or unsupported rating type
    Unknown = 255,
}

impl RatingType {
    /// Convert from raw u8 value to RatingType
    ///
    /// # Arguments
    ///
    /// * `value` - The 4-bit rating type value (0-15)
    ///
    /// # Returns
    ///
    /// The corresponding RatingType enum variant
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => RatingType::None,
            1 => RatingType::Elo,
            2 => RatingType::Rapid,
            3 => RatingType::Iccf,
            4 => RatingType::Uscf,
            5 => RatingType::Dwz,
            6 => RatingType::Ecf,
            _ => RatingType::Unknown,
        }
    }

    /// Convert to a human-readable string for PGN output
    ///
    /// Returns None for RatingType::None (no rating type to display)
    pub fn to_pgn_suffix(&self) -> Option<&'static str> {
        match self {
            RatingType::None => None,
            RatingType::Elo => Some("Elo"),
            RatingType::Rapid => Some("Rapid"),
            RatingType::Iccf => Some("ICCF"),
            RatingType::Uscf => Some("USCF"),
            RatingType::Dwz => Some("DWZ"),
            RatingType::Ecf => Some("ECF"),
            RatingType::Unknown => Some("Rating"),
        }
    }

    /// Check if this is a valid/known rating type
    pub fn is_known(&self) -> bool {
        !matches!(self, RatingType::Unknown)
    }
}

/// Parse a raw rating value into type and value components
///
/// # Arguments
///
/// * `raw` - The raw 16-bit rating value from the index entry
///
/// # Returns
///
/// Tuple of (rating_type, rating_value)
///
/// # Example
///
/// ```
/// # use scidtopgn_core::database::{parse_rating, RatingType};
/// let (rating_type, value) = parse_rating(0x1944);
/// assert_eq!(rating_type, RatingType::Elo);
/// assert_eq!(value, 2372);
/// ```
pub fn parse_rating(raw: u16) -> (RatingType, u16) {
    let rating_type = RatingType::from_u8((raw >> 12) as u8);
    let rating_value = raw & 0x0FFF;
    (rating_type, rating_value)
}

/// Material signature encoding (Gap 16)
///
/// A compact 24-bit encoding of the material (pieces) in the final position.
/// Used for material-based position searches and database integrity checks.
///
/// # Bit Layout (24 bits used of 32-bit field)
///
/// ```text
/// Bits 22-23 (2): White Queens  (0-3)
/// Bits 20-21 (2): White Rooks   (0-3)
/// Bits 18-19 (2): White Bishops (0-3)
/// Bits 16-17 (2): White Knights (0-3)
/// Bits 12-15 (4): White Pawns   (0-15)
/// Bits 10-11 (2): Black Queens  (0-3)
/// Bits  8-9  (2): Black Rooks   (0-3)
/// Bits  6-7  (2): Black Bishops (0-3)
/// Bits  4-5  (2): Black Knights (0-3)
/// Bits  0-3  (4): Black Pawns   (0-15)
/// ```
///
/// # Notes
///
/// - Kings are not included (always 1 per side in valid positions)
/// - Pawns use 4 bits (0-15) to handle promotion edge cases
/// - Pieces use 2 bits (0-3) to handle promotion edge cases
/// - Standard starting position: `0x00FF8888` (Q:1, R:2, B:2, N:2, P:8 each side)
///
/// # Example
///
/// ```
/// # use scidtopgn_core::database::MaterialSignature;
/// let sig = MaterialSignature::from_raw(0x006A86A8);
/// assert_eq!(sig.white_pawns(), 8);
/// assert_eq!(sig.white_queens(), 1);
/// assert_eq!(sig.to_string(), "QRRBBNNPPPPPPPP:qrrbbnnpppppppp");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MaterialSignature {
    raw: u32,
}

// Bit shift positions for each piece type
mod matsig_shifts {
    pub const BP: u32 = 0; // Black Pawns:   bits 0-3
    pub const BN: u32 = 4; // Black Knights: bits 4-5
    pub const BB: u32 = 6; // Black Bishops: bits 6-7
    pub const BR: u32 = 8; // Black Rooks:   bits 8-9
    pub const BQ: u32 = 10; // Black Queens:  bits 10-11
    pub const WP: u32 = 12; // White Pawns:   bits 12-15
    pub const WN: u32 = 16; // White Knights: bits 16-17
    pub const WB: u32 = 18; // White Bishops: bits 18-19
    pub const WR: u32 = 20; // White Rooks:   bits 20-21
    pub const WQ: u32 = 22; // White Queens:  bits 22-23
}

// Bit masks for each piece type
mod matsig_masks {
    pub const BP: u32 = 0x0000_000F; // 4 bits for pawns
    pub const BN: u32 = 0x0000_0030; // 2 bits
    pub const BB: u32 = 0x0000_00C0; // 2 bits
    pub const BR: u32 = 0x0000_0300; // 2 bits
    pub const BQ: u32 = 0x0000_0C00; // 2 bits
    pub const WP: u32 = 0x0000_F000; // 4 bits for pawns
    pub const WN: u32 = 0x0003_0000; // 2 bits
    pub const WB: u32 = 0x000C_0000; // 2 bits
    pub const WR: u32 = 0x0030_0000; // 2 bits
    pub const WQ: u32 = 0x00C0_0000; // 2 bits
}

/// Standard starting position material signature
///
/// Each side has: 1 Queen, 2 Rooks, 2 Bishops, 2 Knights, 8 Pawns
pub const MATSIG_STANDARD_START: u32 = 0x006A_86A8;

/// Empty board material signature
pub const MATSIG_EMPTY: u32 = 0;

impl MaterialSignature {
    /// Create from raw 32-bit value (as stored in index entry)
    pub fn from_raw(raw: u32) -> Self {
        // Only lower 24 bits are used
        Self {
            raw: raw & 0x00FF_FFFF,
        }
    }

    /// Get the raw 32-bit value
    pub fn raw(&self) -> u32 {
        self.raw
    }

    /// Check if this is the standard starting position material
    pub fn is_standard_start(&self) -> bool {
        self.raw == MATSIG_STANDARD_START
    }

    /// Check if the board is empty (no pieces)
    pub fn is_empty(&self) -> bool {
        self.raw == MATSIG_EMPTY
    }

    // ========== White piece counts ==========

    /// Get white queen count (0-3)
    #[inline]
    pub fn white_queens(&self) -> u8 {
        ((self.raw & matsig_masks::WQ) >> matsig_shifts::WQ) as u8
    }

    /// Get white rook count (0-3)
    #[inline]
    pub fn white_rooks(&self) -> u8 {
        ((self.raw & matsig_masks::WR) >> matsig_shifts::WR) as u8
    }

    /// Get white bishop count (0-3)
    #[inline]
    pub fn white_bishops(&self) -> u8 {
        ((self.raw & matsig_masks::WB) >> matsig_shifts::WB) as u8
    }

    /// Get white knight count (0-3)
    #[inline]
    pub fn white_knights(&self) -> u8 {
        ((self.raw & matsig_masks::WN) >> matsig_shifts::WN) as u8
    }

    /// Get white pawn count (0-15)
    #[inline]
    pub fn white_pawns(&self) -> u8 {
        ((self.raw & matsig_masks::WP) >> matsig_shifts::WP) as u8
    }

    // ========== Black piece counts ==========

    /// Get black queen count (0-3)
    #[inline]
    pub fn black_queens(&self) -> u8 {
        ((self.raw & matsig_masks::BQ) >> matsig_shifts::BQ) as u8
    }

    /// Get black rook count (0-3)
    #[inline]
    pub fn black_rooks(&self) -> u8 {
        ((self.raw & matsig_masks::BR) >> matsig_shifts::BR) as u8
    }

    /// Get black bishop count (0-3)
    #[inline]
    pub fn black_bishops(&self) -> u8 {
        ((self.raw & matsig_masks::BB) >> matsig_shifts::BB) as u8
    }

    /// Get black knight count (0-3)
    #[inline]
    pub fn black_knights(&self) -> u8 {
        ((self.raw & matsig_masks::BN) >> matsig_shifts::BN) as u8
    }

    /// Get black pawn count (0-15)
    #[inline]
    pub fn black_pawns(&self) -> u8 {
        ((self.raw & matsig_masks::BP) >> matsig_shifts::BP) as u8
    }

    // ========== Aggregate helpers ==========

    /// Check if position has any queens
    pub fn has_queens(&self) -> bool {
        (self.raw & (matsig_masks::WQ | matsig_masks::BQ)) != 0
    }

    /// Check if position has any rooks
    pub fn has_rooks(&self) -> bool {
        (self.raw & (matsig_masks::WR | matsig_masks::BR)) != 0
    }

    /// Check if position has any bishops
    pub fn has_bishops(&self) -> bool {
        (self.raw & (matsig_masks::WB | matsig_masks::BB)) != 0
    }

    /// Check if position has any knights
    pub fn has_knights(&self) -> bool {
        (self.raw & (matsig_masks::WN | matsig_masks::BN)) != 0
    }

    /// Check if position has any pawns
    pub fn has_pawns(&self) -> bool {
        (self.raw & (matsig_masks::WP | matsig_masks::BP)) != 0
    }

    /// Get total white material (excluding king)
    pub fn white_total(&self) -> u8 {
        self.white_queens()
            + self.white_rooks()
            + self.white_bishops()
            + self.white_knights()
            + self.white_pawns()
    }

    /// Get total black material (excluding king)
    pub fn black_total(&self) -> u8 {
        self.black_queens()
            + self.black_rooks()
            + self.black_bishops()
            + self.black_knights()
            + self.black_pawns()
    }

    /// Convert to SCID-style string representation
    ///
    /// Format: "QRRBBNNPPPPPPPP:qrrbbnnpppppppp" (uppercase=white, lowercase=black)
    /// Each piece letter is repeated by its count.
    pub fn to_material_string(&self) -> String {
        let mut s = String::with_capacity(32);

        // White pieces
        for _ in 0..self.white_queens() {
            s.push('Q');
        }
        for _ in 0..self.white_rooks() {
            s.push('R');
        }
        for _ in 0..self.white_bishops() {
            s.push('B');
        }
        for _ in 0..self.white_knights() {
            s.push('N');
        }
        for _ in 0..self.white_pawns() {
            s.push('P');
        }

        s.push(':');

        // Black pieces
        for _ in 0..self.black_queens() {
            s.push('q');
        }
        for _ in 0..self.black_rooks() {
            s.push('r');
        }
        for _ in 0..self.black_bishops() {
            s.push('b');
        }
        for _ in 0..self.black_knights() {
            s.push('n');
        }
        for _ in 0..self.black_pawns() {
            s.push('p');
        }

        s
    }
}

impl std::fmt::Display for MaterialSignature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_material_string())
    }
}

/// Game index entry (47 bytes)
///
/// Contains all metadata for a single game. Stored immediately after the header,
/// one entry per game.
///
/// # Critical Implementation Notes
///
/// ⚠️ **BIG-ENDIAN BYTE ORDER** - All multi-byte values use big-endian encoding!
///
/// ⚠️ **PACKED BIT FIELDS** - Many fields span partial bytes:
/// - Player IDs are 20 bits each (split across multiple bytes)
/// - Event/Site/Round IDs use complex bit packing
/// - Dates use 20+12 bit encoding
/// - ELO ratings are 12 bits with 4-bit type
///
/// ⚠️ **FIXED OFFSETS** - Some fields are at critical fixed positions:
/// - Dates: ALWAYS at offset 25-28
/// - Player IDs: Offset 9-13 (split high/low)
/// - Game offset/length: 0-6
///
/// # Field Layout
///
/// | Offset | Size | Field              | Format      |
/// |--------|------|--------------------|-------------|
/// | 0-3    | 4    | game_offset        | BE u32      |
/// | 4-5    | 2    | length_low         | BE u16      |
/// | 6      | 1    | length_high        | packed      |
/// | 7-8    | 2    | game_flags         | BE u16      |
/// | 9      | 1    | white_black_high   | packed      |
/// | 10-11  | 2    | white_id_low       | BE u16      |
/// | 12-13  | 2    | black_id_low       | BE u16      |
/// | 14     | 1    | event_site_rnd_high| packed      |
/// | 15-16  | 2    | event_id_low       | BE u16      |
/// | 17-18  | 2    | site_id_low        | BE u16      |
/// | 19-20  | 2    | round_id_low       | BE u16      |
/// | 21-22  | 2    | var_counts         | BE u16      |
/// | 23-24  | 2    | eco_code           | BE u16      |
/// | 25-28  | 4    | dates              | BE u32      |
/// | 29-30  | 2    | white_elo          | BE u16      |
/// | 31-32  | 2    | black_elo          | BE u16      |
/// | 33-36  | 4    | final_mat_sig      | BE u32      |
/// | 37     | 1    | num_half_moves     | u8          |
/// | 38-46  | 9    | home_pawn_data     | packed      |
///
/// See SCID_DATABASE_FORMAT.md lines 120-210 for complete specification.
#[derive(Debug, Clone, PartialEq)]
pub struct GameIndexEntry {
    // Game location in .sg4 file
    /// Byte offset of game data in .sg4 file
    pub game_offset: u32,

    /// Length of game data in bytes (17-bit value, max 131,071)
    pub game_length: u32,

    // Player information
    /// White player name ID (20-bit value, index into name file)
    pub white_id: u32,

    /// Black player name ID (20-bit value, index into name file)
    pub black_id: u32,

    // Event information
    /// Event name ID (19-bit value)
    pub event_id: u32,

    /// Site name ID (19-bit value)
    pub site_id: u32,

    /// Round name ID (18-bit value)
    pub round_id: u32,

    // Game metadata
    /// Game date (when game was played)
    pub game_date: GameDate,

    /// Event date (when event started, if different from game date)
    pub event_date: Option<GameDate>,

    /// Game result
    pub result: GameResult,

    /// White player ELO rating (12-bit value, 0-4095)
    pub white_elo: u16,

    /// Black player ELO rating (12-bit value, 0-4095)
    pub black_elo: u16,

    /// White rating type (4-bit value: 0=None, 1=Elo, 2=Rapid, etc.)
    /// Use `white_rating_type()` method for the enum version.
    pub white_rating_type_raw: u8,

    /// Black rating type (4-bit value)
    /// Use `black_rating_type()` method for the enum version.
    pub black_rating_type_raw: u8,

    /// ECO opening code
    pub eco_code: u16,

    /// Number of half-moves in game (10-bit value, max 1023)
    pub half_moves: u16,

    /// Number of variations in game (4-bit value)
    pub variation_count: u8,

    /// Number of comments (4-bit value)
    pub comment_count: u8,

    /// Number of NAGs (annotations) (4-bit value)
    pub nag_count: u8,

    /// Game flags (promotions, custom start, etc.)
    pub flags: u16,

    /// Material signature of final position
    pub final_material_signature: u32,

    /// Game data is zlib compressed (from length_high byte bit 7)
    ///
    /// **CRITICAL**: When true, game data MUST be decompressed before parsing!
    /// Most real SCID databases use compression.
    pub packed: bool,
}

impl GameIndexEntry {
    /// Create a new entry with default values (for testing)
    pub fn new() -> Self {
        GameIndexEntry {
            game_offset: 0,
            game_length: 0,
            white_id: 0,
            black_id: 0,
            event_id: 0,
            site_id: 0,
            round_id: 0,
            game_date: GameDate::default(),
            event_date: None,
            result: GameResult::Unknown,
            white_elo: 0,
            black_elo: 0,
            white_rating_type_raw: 0,
            black_rating_type_raw: 0,
            eco_code: 0,
            half_moves: 0,
            variation_count: 0,
            comment_count: 0,
            nag_count: 0,
            flags: 0,
            final_material_signature: 0,
            packed: false,
        }
    }

    // ========== Flag Helper Methods ==========

    /// Check if a flag is set
    #[inline]
    pub fn has_flag(&self, flag: u16) -> bool {
        (self.flags & flag) != 0
    }

    /// Check if game is marked for deletion
    #[inline]
    pub fn is_deleted(&self) -> bool {
        self.has_flag(game_flags::DELETE_FLAG)
    }

    /// Check if game has a custom starting position (non-standard FEN)
    #[inline]
    pub fn has_custom_start(&self) -> bool {
        self.has_flag(game_flags::START_FLAG)
    }

    /// Check if game contains pawn promotions
    #[inline]
    pub fn has_promotions(&self) -> bool {
        self.has_flag(game_flags::PROMO_FLAG)
    }

    /// Check if game contains underpromotions (non-queen promotions)
    #[inline]
    pub fn has_underpromotions(&self) -> bool {
        self.has_flag(game_flags::UNDER_PROMO_FLAG)
    }

    /// Check if a custom flag (1-6) is set
    pub fn has_custom_flag(&self, flag_num: u8) -> bool {
        match game_flags::custom_flag_mask(flag_num) {
            Some(mask) => self.has_flag(mask),
            None => false,
        }
    }

    // ========== Compression Detection ==========

    /// Check if game data is zlib compressed
    ///
    /// **CRITICAL**: When this returns true, the game data in .sg4 MUST be
    /// decompressed using zlib before parsing moves. Most real-world SCID
    /// databases use compression.
    ///
    /// The packed flag is stored in bit 7 of the `length_high` byte (byte 6
    /// of the index entry), which is the same byte that contains bit 16 of
    /// the game length.
    ///
    /// # Example
    ///
    /// ```rust
    /// if entry.is_packed() {
    ///     let decompressed = decompress_game_data(&raw_game_bytes)?;
    ///     parse_moves(&decompressed)?;
    /// } else {
    ///     parse_moves(&raw_game_bytes)?;
    /// }
    /// ```
    #[inline]
    pub fn is_packed(&self) -> bool {
        self.packed
    }

    // ========== Rating Type Helpers (Gap 15) ==========

    /// Get white player's rating type as enum
    ///
    /// # Example
    ///
    /// ```rust
    /// let entry = parse_game_index_entry(&bytes)?;
    /// match entry.white_rating_type() {
    ///     RatingType::Elo => println!("White Elo: {}", entry.white_elo),
    ///     RatingType::Uscf => println!("White USCF: {}", entry.white_elo),
    ///     RatingType::None => println!("White rating not specified"),
    ///     _ => println!("White rating ({}): {}", entry.white_rating_type().to_pgn_suffix().unwrap_or(""), entry.white_elo),
    /// }
    /// ```
    pub fn white_rating_type(&self) -> RatingType {
        RatingType::from_u8(self.white_rating_type_raw)
    }

    /// Get black player's rating type as enum
    pub fn black_rating_type(&self) -> RatingType {
        RatingType::from_u8(self.black_rating_type_raw)
    }

    /// Get white rating with type information
    ///
    /// Returns tuple of (rating_type, rating_value)
    pub fn white_rating(&self) -> (RatingType, u16) {
        (self.white_rating_type(), self.white_elo)
    }

    /// Get black rating with type information
    ///
    /// Returns tuple of (rating_type, rating_value)
    pub fn black_rating(&self) -> (RatingType, u16) {
        (self.black_rating_type(), self.black_elo)
    }

    /// Check if white has a valid rating
    pub fn has_white_rating(&self) -> bool {
        self.white_elo > 0
    }

    /// Check if black has a valid rating
    pub fn has_black_rating(&self) -> bool {
        self.black_elo > 0
    }

    // ========== Material Signature Helper (Gap 16) ==========

    /// Get the final position's material signature as a structured type
    ///
    /// The material signature encodes piece counts for both sides and can be
    /// used for material-based searches or database integrity verification.
    ///
    /// # Example
    ///
    /// ```rust
    /// let entry = parse_game_index_entry(&bytes)?;
    /// let mat = entry.material_signature();
    ///
    /// if mat.is_standard_start() {
    ///     println!("Game ended with starting material (no captures)");
    /// }
    ///
    /// println!("Final material: {}", mat); // e.g., "QRRBBNNPPPPPPPP:qrrbbnnpppppppp"
    /// ```
    pub fn material_signature(&self) -> MaterialSignature {
        MaterialSignature::from_raw(self.final_material_signature)
    }

    // ========== ECO Code Helper ==========

    /// Get ECO code as string (e.g., "A00", "B12", "E99")
    ///
    /// ECO codes are encoded as: `letter_index * 100 + number`
    /// - Letter index: 0=A, 1=B, 2=C, 3=D, 4=E
    /// - Number: 00-99
    ///
    /// Returns None if eco_code is 0 (no ECO assigned)
    pub fn eco_to_string(&self) -> Option<String> {
        eco_to_string(self.eco_code)
    }
}

/// Convert an ECO code value to its string representation
///
/// ECO codes are encoded as: `letter_index * 100 + number`
/// - Letter index: 0=A, 1=B, 2=C, 3=D, 4=E
/// - Number: 00-99
///
/// # Examples
///
/// ```
/// assert_eq!(eco_to_string(0), None);
/// assert_eq!(eco_to_string(1), Some("A01".to_string()));
/// assert_eq!(eco_to_string(100), Some("B00".to_string()));
/// assert_eq!(eco_to_string(199), Some("B99".to_string()));
/// assert_eq!(eco_to_string(499), Some("E99".to_string()));
/// ```
pub fn eco_to_string(eco_code: u16) -> Option<String> {
    if eco_code == 0 {
        return None;
    }

    let letter_index = (eco_code / 100) as u8;
    let number = (eco_code % 100) as u8;

    // ECO letters are A-E (indices 0-4)
    if letter_index > 4 {
        return None; // Invalid ECO code
    }

    let letter = (b'A' + letter_index) as char;
    Some(format!("{}{:02}", letter, number))
}

impl Default for GameIndexEntry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse the dates field (32-bit value at offset 25-28)
///
/// This is the most critical field to get right!
///
/// # Format
///
/// The dates field is a 32-bit big-endian value containing TWO dates:
/// - Upper 12 bits: Event date (relative encoding)
/// - Lower 20 bits: Game date (absolute encoding)
///
/// ```text
/// 31  28 27  24 23  20 19  16 15  12 11   8 7    4 3    0
/// ├────┼─────┼─────┼─────┼─────┼─────┼─────┼─────┤
/// │ Event Date (12 bits)        │ Game Date (20 bits)      │
/// └──────────────────────────────┴───────────────────────────┘
/// ```
///
/// # Game Date (20 bits, absolute)
///
/// - Bits 19-9: Year (0-2047)
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// Formula: `((year << 9) | (month << 5) | day)`
///
/// # Event Date (12 bits, relative)
///
/// - Bits 11-9: Year offset (0-7) relative to game year
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// ## Year Offset Encoding (CRITICAL)
///
/// The event year is stored as a 3-bit offset relative to the game year.
/// This allows events up to 3 years before or after the game.
///
/// **Encoding formula**: `stored_offset = (event_year - game_year + 4) & 0x7`
/// **Decoding formula**: `event_year = game_year + stored_offset - 4`
///
/// ### Year Offset Examples
///
/// | Event Year | Game Year | Difference | Stored Offset | Decoded Year |
/// |------------|-----------|------------|---------------|--------------|
/// | 2020       | 2023      | -3         | 1 (=-3+4)     | 2023+1-4=2020|
/// | 2021       | 2023      | -2         | 2 (=-2+4)     | 2023+2-4=2021|
/// | 2022       | 2023      | -1         | 3 (=-1+4)     | 2023+3-4=2022|
/// | 2023       | 2023      | 0          | 4 (=0+4)      | 2023+4-4=2023|
/// | 2024       | 2023      | +1         | 5 (=1+4)      | 2023+5-4=2024|
/// | 2025       | 2023      | +2         | 6 (=2+4)      | 2023+6-4=2025|
/// | 2026       | 2023      | +3         | 7 (=3+4)      | 2023+7-4=2026|
///
/// ### Special Values
///
/// - **Event date = 0**: No event date stored (entire 12-bit field is zero)
/// - **Year offset = 0**: Invalid/no event date (reserved value)
///
/// The offset of 4 is the "center point" meaning same year as game.
/// Valid offsets are 1-7, allowing events from 3 years before to 3 years after.
///
/// See SCID_DATABASE_FORMAT.md lines 211-313 for complete specification.
fn parse_dates_field(dates_field: u32) -> (GameDate, Option<GameDate>) {
    // Extract game date (lower 20 bits)
    let game_date_raw = dates_field & 0x000F_FFFF;

    let game_day = (game_date_raw & 0x1F) as u8;
    let game_month = ((game_date_raw >> 5) & 0x0F) as u8;
    let game_year = ((game_date_raw >> 9) & 0x7FF) as u16;

    let game_date = GameDate {
        year: game_year,
        month: game_month,
        day: game_day,
    };

    // Extract event date (upper 12 bits)
    let event_data = (dates_field >> 20) & 0xFFF;

    let event_date = if event_data == 0 {
        // No event date set
        None
    } else {
        let event_day = (event_data & 0x1F) as u8;
        let event_month = ((event_data >> 5) & 0x0F) as u8;
        let year_offset = ((event_data >> 9) & 0x7) as i16;

        if year_offset == 0 {
            // Invalid event date
            None
        } else {
            // Calculate actual event year using relative offset
            // stored_offset = (event_year - game_year + 4) & 0x7
            // Therefore: event_year = game_year + stored_offset - 4
            let event_year = (game_year as i16 + year_offset - 4) as u16;

            Some(GameDate {
                year: event_year,
                month: event_month,
                day: event_day,
            })
        }
    };

    (game_date, event_date)
}

/// Parse a single game index entry from 47 bytes
///
/// # Arguments
///
/// * `bytes` - Exactly 47 bytes containing the entry data
///
/// # Returns
///
/// Parsed entry or error if data is invalid
///
/// # SCID Format Critical Details
///
/// **BIG-ENDIAN**: All multi-byte values use big-endian byte order!
///
/// **17-bit Game Length**: Game length uses special encoding:
/// - Bits 0-15: `length_low` (bytes 4-5)
/// - Bit 16: High bit from byte 6 (bit 7)
/// - Formula: `length = length_low | ((byte[6] & 0x80) << 9)`
///
/// See SCID_DATABASE_FORMAT.md lines 148-157 for length encoding.
pub fn parse_game_index_entry(bytes: &[u8]) -> Result<GameIndexEntry> {
    // Validate input length
    if bytes.len() != GAME_ENTRY_SIZE {
        return Err(ScidError::ParseError {
            file: PathBuf::from("si4"),
            offset: 0,
            message: format!(
                "Invalid entry size: expected {} bytes, got {}",
                GAME_ENTRY_SIZE,
                bytes.len()
            ),
        });
    }

    // Parse game offset (bytes 0-3, BIG-ENDIAN u32)
    let game_offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

    // Parse game length (17-bit value split across bytes 4-6)
    // See SCID_DATABASE_FORMAT.md lines 148-157
    //
    // Byte 6 format:
    //   Bit 7: FLAG_PACKED (game data is zlib compressed)
    //   Bit 6-0: Other flags/unused
    //
    // Length encoding:
    //   Bits 0-15: length_low (bytes 4-5)
    //   Bit 16: ACTUALLY bit 0 of byte 6, NOT bit 7!
    //
    // IMPORTANT: Bit 7 of byte 6 is the PACKED flag, not part of length!
    let length_low = u16::from_be_bytes([bytes[4], bytes[5]]) as u32;
    let length_high = bytes[6];

    // Extract packed flag (bit 7) - CRITICAL for decompression!
    let packed = (length_high & 0x80) != 0;

    // Game length uses bit 0 of byte 6, not bit 7
    // (Bit 7 is the packed flag, which we already extracted)
    let game_length = length_low | (((length_high & 0x01) as u32) << 16);

    // Parse game flags (bytes 7-8, BIG-ENDIAN u16)
    let flags = u16::from_be_bytes([bytes[7], bytes[8]]);

    // Parse Player IDs (20 bits each)
    // See SCID_DATABASE_FORMAT.md lines 159-172
    //
    // Byte 9 contains high bits for both white and black:
    //   Bits 7-4: White ID high 4 bits
    //   Bits 3-0: Black ID high 4 bits
    // Bytes 10-11: White ID low 16 bits (big-endian)
    // Bytes 12-13: Black ID low 16 bits (big-endian)
    //
    // Formula: ID = (high_bits << 16) | low_bits

    let white_black_high = bytes[9];

    let white_id = ((white_black_high & 0xF0) as u32) << 12 // High 4 bits shifted to position 16-19
                 | u16::from_be_bytes([bytes[10], bytes[11]]) as u32; // Low 16 bits

    let black_id = ((white_black_high & 0x0F) as u32) << 16 // High 4 bits shifted to position 16-19
                 | u16::from_be_bytes([bytes[12], bytes[13]]) as u32; // Low 16 bits

    // Parse Event/Site/Round IDs (19/19/18 bits)
    // See SCID_DATABASE_FORMAT.md lines 159-172
    //
    // Byte 14 contains high bits:
    //   Bits 7-5: Event ID high 3 bits
    //   Bits 4-2: Site ID high 3 bits
    //   Bits 1-0: Round ID high 2 bits

    let event_site_rnd_high = bytes[14];

    let event_id = ((event_site_rnd_high & 0xE0) as u32) << 11 // High 3 bits to position 16-18
                 | u16::from_be_bytes([bytes[15], bytes[16]]) as u32;

    let site_id = ((event_site_rnd_high & 0x1C) as u32) << 14  // High 3 bits to position 16-18
                | u16::from_be_bytes([bytes[17], bytes[18]]) as u32;

    let round_id = ((event_site_rnd_high & 0x03) as u32) << 16 // High 2 bits to position 16-17
                 | u16::from_be_bytes([bytes[19], bytes[20]]) as u32;

    // Parse dates field (bytes 25-28, BIG-ENDIAN u32)
    // CRITICAL: This is at FIXED OFFSET 25-28!
    // See SCID_DATABASE_FORMAT.md lines 211-313
    let dates_field = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);

    let (game_date, event_date) = parse_dates_field(dates_field);

    // Parse variation counts and result (bytes 21-22, BIG-ENDIAN u16)
    // See SCID_DATABASE_FORMAT.md lines 174-187
    //
    // Bits 15-12: Result (0=*, 1=1-0, 2=0-1, 3=1/2-1/2)
    // Bits 11-8: NAG count
    // Bits 7-4: Comment count
    // Bits 3-0: Variation count

    let var_counts = u16::from_be_bytes([bytes[21], bytes[22]]);

    let result_code = (var_counts >> 12) as u8;
    let result = GameResult::from_scid(result_code);

    let nag_count = ((var_counts >> 8) & 0x0F) as u8;
    let comment_count = ((var_counts >> 4) & 0x0F) as u8;
    let variation_count = (var_counts & 0x0F) as u8;

    // Parse ECO code (bytes 23-24, BIG-ENDIAN u16)
    let eco_code = u16::from_be_bytes([bytes[23], bytes[24]]);

    // Parse ELO ratings (bytes 29-30 and 31-32)
    // See SCID_DATABASE_FORMAT.md lines 189-200
    //
    // Each rating is 16 bits:
    //   Bits 15-12: Rating type (0=None, 1=Elo, 2=FIDE, etc.)
    //   Bits 11-0: Rating value (0-4095)

    let white_elo_raw = u16::from_be_bytes([bytes[29], bytes[30]]);
    let white_elo = white_elo_raw & 0x0FFF;
    let white_rating_type = (white_elo_raw >> 12) as u8;

    let black_elo_raw = u16::from_be_bytes([bytes[31], bytes[32]]);
    let black_elo = black_elo_raw & 0x0FFF;
    let black_rating_type = (black_elo_raw >> 12) as u8;

    // Parse final material signature (bytes 33-36, BIG-ENDIAN u32)
    let final_material_signature = u32::from_be_bytes([bytes[33], bytes[34], bytes[35], bytes[36]]);

    // Parse half-move count (10-bit value split across bytes 37-38)
    // See SCID_DATABASE_FORMAT.md lines 201-209
    //
    // Low 8 bits: byte 37
    // High 2 bits: bits 7-6 of byte 38

    let half_moves_low = bytes[37] as u16;
    let half_moves_high = ((bytes[38] >> 6) & 0x03) as u16;
    let half_moves = half_moves_low | (half_moves_high << 8);

    let mut entry = GameIndexEntry::new();
    entry.game_offset = game_offset;
    entry.game_length = game_length;
    entry.flags = flags;
    entry.packed = packed; // CRITICAL: Indicates zlib compression
    entry.white_id = white_id;
    entry.black_id = black_id;
    entry.event_id = event_id;
    entry.site_id = site_id;
    entry.round_id = round_id;
    entry.game_date = game_date;
    entry.event_date = event_date;
    entry.result = result;
    entry.nag_count = nag_count;
    entry.comment_count = comment_count;
    entry.variation_count = variation_count;
    entry.eco_code = eco_code;
    entry.white_elo = white_elo;
    entry.black_elo = black_elo;
    entry.white_rating_type_raw = white_rating_type;
    entry.black_rating_type_raw = black_rating_type;
    entry.final_material_signature = final_material_signature;
    entry.half_moves = half_moves;

    Ok(entry)
}

/// Parse SI4 header from file
///
/// Reads and validates the 182-byte header from a .si4 file.
///
/// # Arguments
///
/// * `file` - Mutable reference to opened .si4 file
///
/// # Returns
///
/// Parsed header or error if file is invalid.
///
/// # Errors
///
/// - `ScidError::InvalidFormat` - Wrong magic bytes or invalid structure
/// - `ScidError::Io` - File read error
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// # use scidtopgn_core::database::index::parse_si4_header;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("database.si4")?;
/// let header = parse_si4_header(&mut file)?;
/// println!("Database has {} games", header.num_games);
/// # Ok(())
/// # }
/// ```
///
/// # SCID Format Details
///
/// All multi-byte values are BIG-ENDIAN (see SCID_DATABASE_FORMAT.md line 1017).
///
/// Byte layout:
/// - 0-7: Magic "Scid.si\0"
/// - 8-9: Version (u16 BE) - should be 400
/// - 10-13: Base type (u32 BE)
/// - 14-16: Game count (u24 BE) - note: 24-bit value!
/// - 17-19: Auto-load (u24 BE)
/// - 20-127: Description (UTF-8 string)
/// - 128-181: Custom flags (6 × 9 bytes)
pub fn parse_si4_header(file: &mut File) -> Result<Si4Header> {
    // Read exactly 182 bytes
    let mut header_bytes = [0u8; SI4_HEADER_SIZE];
    file.read_exact(&mut header_bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            ScidError::InvalidFormat(format!(
                "File too short: expected {} bytes for header, got less",
                SI4_HEADER_SIZE
            ))
        } else {
            ScidError::Io(e)
        }
    })?;

    // Validate magic bytes (offset 0-7)
    if &header_bytes[0..8] != SI4_MAGIC {
        return Err(ScidError::InvalidFormat(format!(
            "Invalid magic bytes: expected {:?}, got {:?}",
            SI4_MAGIC,
            &header_bytes[0..8]
        )));
    }

    // Parse version (offset 8-9, BIG-ENDIAN u16)
    // CRITICAL: Must use from_be_bytes, not from_le_bytes!
    let version = u16::from_be_bytes([header_bytes[8], header_bytes[9]]);

    // Validate version
    if version != SCID_VERSION {
        return Err(ScidError::InvalidFormat(format!(
            "Unsupported SCID version: expected {}, got {}",
            SCID_VERSION, version
        )));
    }

    // Parse base type (offset 10-13, BIG-ENDIAN u32)
    let base_type = u32::from_be_bytes([
        header_bytes[10],
        header_bytes[11],
        header_bytes[12],
        header_bytes[13],
    ]);

    // Parse game count (offset 14-16, BIG-ENDIAN 24-bit value)
    // 24-bit values are stored as 3 bytes, construct u32 with high byte = 0
    let num_games = u32::from_be_bytes([0, header_bytes[14], header_bytes[15], header_bytes[16]]);

    // Parse auto-load (offset 17-19, BIG-ENDIAN 24-bit value)
    let auto_load = u32::from_be_bytes([0, header_bytes[17], header_bytes[18], header_bytes[19]]);

    // Parse description (offset 20-127, UTF-8 string)
    // Remove null terminators and control characters
    let description = String::from_utf8_lossy(&header_bytes[20..128])
        .chars()
        .filter(|&c| c >= ' ') // Remove control characters
        .collect::<String>()
        .trim()
        .to_string();

    // Parse custom flag names (offset 128-181, six 9-byte strings)
    // Each flag name is 9 bytes (8 chars + null terminator)
    let mut custom_flag_names: [String; 6] = Default::default();
    for i in 0..6 {
        let start = 128 + (i * CUSTOM_FLAG_NAME_SIZE);
        let end = start + CUSTOM_FLAG_NAME_SIZE;
        custom_flag_names[i] = String::from_utf8_lossy(&header_bytes[start..end])
            .chars()
            .filter(|&c| c >= ' ')
            .collect::<String>()
            .trim()
            .to_string();
    }

    Ok(Si4Header {
        version,
        base_type,
        num_games,
        auto_load,
        description,
        custom_flag_names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_si4_header_builder() {
        let header = Si4Header::new()
            .with_num_games(100)
            .with_description("My Database");

        assert_eq!(header.num_games, 100);
        assert_eq!(header.description, "My Database");
        assert_eq!(header.version, SCID_VERSION);
    }

    #[test]
    fn test_si4_header_default() {
        let header = Si4Header::default();
        assert_eq!(header.version, SCID_VERSION);
        assert_eq!(header.num_games, 0);
    }

    #[test]
    fn test_rating_type_from_u8() {
        assert_eq!(RatingType::from_u8(0), RatingType::None);
        assert_eq!(RatingType::from_u8(1), RatingType::Elo);
        assert_eq!(RatingType::from_u8(2), RatingType::Rapid);
        assert_eq!(RatingType::from_u8(3), RatingType::Iccf);
        assert_eq!(RatingType::from_u8(4), RatingType::Uscf);
        assert_eq!(RatingType::from_u8(5), RatingType::Dwz);
        assert_eq!(RatingType::from_u8(6), RatingType::Ecf);
        assert_eq!(RatingType::from_u8(255), RatingType::Unknown);
    }

    #[test]
    fn test_rating_type_pgn_suffix() {
        assert_eq!(RatingType::None.to_pgn_suffix(), None);
        assert_eq!(RatingType::Elo.to_pgn_suffix(), Some("Elo"));
        assert_eq!(RatingType::Rapid.to_pgn_suffix(), Some("Rapid"));
        assert_eq!(RatingType::Iccf.to_pgn_suffix(), Some("ICCF"));
        assert_eq!(RatingType::Uscf.to_pgn_suffix(), Some("USCF"));
        assert_eq!(RatingType::Dwz.to_pgn_suffix(), Some("DWZ"));
        assert_eq!(RatingType::Ecf.to_pgn_suffix(), Some("ECF"));
        assert_eq!(RatingType::Unknown.to_pgn_suffix(), Some("Rating"));
    }

    #[test]
    fn test_parse_rating() {
        // Type 1 (Elo), value 2372
        let (rating_type, value) = parse_rating(0x1944);
        assert_eq!(rating_type, RatingType::Elo);
        assert_eq!(value, 2372);

        // Type 0 (None), value 0
        let (rating_type, value) = parse_rating(0);
        assert_eq!(rating_type, RatingType::None);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_material_signature_standard_start() {
        let sig = MaterialSignature::from_raw(MATSIG_STANDARD_START);
        assert!(sig.is_standard_start());

        assert_eq!(sig.white_queens(), 1);
        assert_eq!(sig.white_rooks(), 2);
        assert_eq!(sig.white_bishops(), 2);
        assert_eq!(sig.white_knights(), 2);
        assert_eq!(sig.white_pawns(), 8);

        assert_eq!(sig.black_queens(), 1);
        assert_eq!(sig.black_rooks(), 2);
        assert_eq!(sig.black_bishops(), 2);
        assert_eq!(sig.black_knights(), 2);
        assert_eq!(sig.black_pawns(), 8);
    }

    #[test]
    fn test_material_signature_empty() {
        let sig = MaterialSignature::from_raw(MATSIG_EMPTY);
        assert!(sig.is_empty());
        assert_eq!(sig.white_total(), 0);
        assert_eq!(sig.black_total(), 0);
    }

    #[test]
    fn test_material_signature_has_methods() {
        let sig = MaterialSignature::from_raw(MATSIG_STANDARD_START);
        assert!(sig.has_queens());
        assert!(sig.has_rooks());
        assert!(sig.has_bishops());
        assert!(sig.has_knights());
        assert!(sig.has_pawns());

        let empty = MaterialSignature::from_raw(MATSIG_EMPTY);
        assert!(!empty.has_queens());
        assert!(!empty.has_rooks());
        assert!(!empty.has_bishops());
        assert!(!empty.has_knights());
        assert!(!empty.has_pawns());
    }

    #[test]
    fn test_material_signature_to_string() {
        let sig = MaterialSignature::from_raw(MATSIG_STANDARD_START);
        let s = sig.to_material_string();
        assert_eq!(s, "QRRBBNNPPPPPPPP:qrrbbnnpppppppp");
    }

    #[test]
    fn test_game_flags() {
        use game_flags::*;

        // Test individual flags
        assert_eq!(START_FLAG, 1 << 0);
        assert_eq!(PROMO_FLAG, 1 << 1);
        assert_eq!(UNDER_PROMO_FLAG, 1 << 2);
        assert_eq!(DELETE_FLAG, 1 << 3);
        assert_eq!(WHITE_OP_FLAG, 1 << 4);
        assert_eq!(BLACK_OP_FLAG, 1 << 5);
        assert_eq!(MIDDLEGAME_FLAG, 1 << 6);
        assert_eq!(ENDGAME_FLAG, 1 << 7);
        assert_eq!(NOVELTY_FLAG, 1 << 8);
        assert_eq!(PAWN_FLAG, 1 << 9);
        assert_eq!(CUSTOM_FLAG_1, 1 << 10);
        assert_eq!(CUSTOM_FLAG_2, 1 << 11);
        assert_eq!(CUSTOM_FLAG_3, 1 << 12);
        assert_eq!(CUSTOM_FLAG_4, 1 << 13);
        assert_eq!(CUSTOM_FLAG_5, 1 << 14);
        assert_eq!(CUSTOM_FLAG_6, 1 << 15);

        // Test custom flag masks
        assert_eq!(custom_flag_mask(1), Some(CUSTOM_FLAG_1));
        assert_eq!(custom_flag_mask(3), Some(CUSTOM_FLAG_3));
        assert_eq!(custom_flag_mask(6), Some(CUSTOM_FLAG_6));
        assert_eq!(custom_flag_mask(0), None);
        assert_eq!(custom_flag_mask(7), None);
    }

    #[test]
    fn test_custom_flag_name() {
        let mut header = Si4Header::new();
        header.custom_flag_names[0] = "Annotated".to_string();
        header.custom_flag_names[3] = "Tactics".to_string();

        assert_eq!(header.get_custom_flag_name(1), Some("Annotated"));
        assert_eq!(header.get_custom_flag_name(4), Some("Tactics"));
        assert_eq!(header.get_custom_flag_name(2), Some(""));
        assert_eq!(header.get_custom_flag_name(0), None);
        assert_eq!(header.get_custom_flag_name(7), None);
    }

    #[test]
    fn test_parse_game_offset_and_length() {
        // Create test entry with known values
        let mut bytes = [0u8; 47];

        // Game offset: 0x00001000 (4096)
        bytes[0..4].copy_from_slice(&0x00001000u32.to_be_bytes());

        // Length low: 500 (0x01F4)
        bytes[4..6].copy_from_slice(&500u16.to_be_bytes());

        // Length high: 0x00 (no high bit set)
        bytes[6] = 0x00;

        let entry = parse_game_index_entry(&bytes).unwrap();
        assert_eq!(entry.game_offset, 4096);
        assert_eq!(entry.game_length, 500);
    }

    #[test]
    fn test_parse_17bit_game_length() {
        let mut bytes = [0u8; 47];

        // Test 17-bit length encoding
        // Set length_low = 0xFFFF (65535)
        bytes[4..6].copy_from_slice(&0xFFFFu16.to_be_bytes());

        // Set high bit of length (bit 0 of byte 6, NOT bit 7)
        // Bit 7 is the packed flag, not part of length
        bytes[6] = 0x01;

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Expected: 65535 | (1 << 16) = 65535 + 65536 = 131071
        assert_eq!(entry.game_length, 131071);
        assert!(!entry.packed); // Packed flag (bit 7) is not set
    }

    #[test]
    fn test_invalid_entry_size() {
        let bytes = [0u8; 40]; // Wrong size
        let result = parse_game_index_entry(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_game_flags() {
        let mut bytes = [0u8; 47];

        // Set some flags: DELETE_FLAG (bit 3) = 0x08
        bytes[7..9].copy_from_slice(&0x0008u16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();
        assert_eq!(entry.flags, 0x0008);
        assert!(entry.is_deleted());
    }

    #[test]
    fn test_parse_packed_flag() {
        let mut bytes = [0u8; 47];

        // Set packed flag (bit 7 of byte 6)
        bytes[6] = 0x80;

        let entry = parse_game_index_entry(&bytes).unwrap();
        assert!(entry.packed);
        assert!(entry.is_packed());
    }

    #[test]
    fn test_parse_not_packed() {
        let bytes = [0u8; 47];

        // Clear packed flag
        let entry = parse_game_index_entry(&bytes).unwrap();
        assert!(!entry.packed);
        assert!(!entry.is_packed());
    }

    #[test]
    fn test_parse_player_ids() {
        let mut bytes = [0u8; 47];

        // White ID: 0x12345 (74565)
        // High 4 bits: 0x1, Low 16 bits: 0x2345
        bytes[9] = 0x10; // High nibble = 1
        bytes[10..12].copy_from_slice(&0x2345u16.to_be_bytes());

        // Black ID: 0xABCDE (704734)
        // High 4 bits: 0xA, Low 16 bits: 0xBCDE
        bytes[9] |= 0x0A; // Low nibble = A
        bytes[12..14].copy_from_slice(&0xBCDEu16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.white_id, 0x12345);
        assert_eq!(entry.black_id, 0xABCDE);
    }

    #[test]
    fn test_parse_event_site_round_ids() {
        let mut bytes = [0u8; 47];

        // Event ID: 19-bit max value (0x7FFFF = 524287)
        // High 3 bits: 0x7, Low 16 bits: 0xFFFF
        bytes[14] = 0xE0; // Bits 7-5 = 111
        bytes[15..17].copy_from_slice(&0xFFFFu16.to_be_bytes());

        // Site ID: some value
        bytes[14] |= 0x1C; // Bits 4-2 = 111
        bytes[17..19].copy_from_slice(&0x1234u16.to_be_bytes());

        // Round ID: 18-bit value
        bytes[14] |= 0x03; // Bits 1-0 = 11
        bytes[19..21].copy_from_slice(&0x5678u16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Validate event ID
        assert_eq!(entry.event_id, 0x7FFFF); // Max 19-bit value
    }

    #[test]
    fn test_parse_game_date_2022_12_19() {
        // Test exact date from SCID spec (lines 1556-1567)
        // Date: 2022.12.19

        // Calculate encoded value
        let year = 2022u32;
        let month = 12u32;
        let day = 19u32;
        let encoded = (year << 9) | (month << 5) | day;

        // Parse it back
        let (game_date, _) = parse_dates_field(encoded);

        assert_eq!(game_date.year, 2022);
        assert_eq!(game_date.month, 12);
        assert_eq!(game_date.day, 19);
        assert_eq!(game_date.to_pgn_string(), "2022.12.19");
    }

    #[test]
    fn test_parse_dates_with_event_date() {
        // Game date: 2022.06.15
        let game_year = 2022u32;
        let game_month = 6u32;
        let game_day = 15u32;
        let game_encoded = (game_year << 9) | (game_month << 5) | game_day;

        // Event date: 2022.08.10 (same year, so offset = 0 + 4 = 4)
        let event_year = 2022u32;
        let event_month = 8u32;
        let event_day = 10u32;
        let year_offset = ((event_year - game_year) + 4) & 0x7;
        let event_encoded = (year_offset << 9) | (event_month << 5) | event_day;

        // Combine into 32-bit field
        let dates_field = (event_encoded << 20) | game_encoded;

        let (game_date, event_date) = parse_dates_field(dates_field);

        assert_eq!(game_date.to_pgn_string(), "2022.06.15");
        assert!(event_date.is_some());
        assert_eq!(event_date.unwrap().to_pgn_string(), "2022.08.10");
    }

    #[test]
    fn test_parse_dates_no_event_date() {
        // Game date only, no event date
        let dates_field = (2022 << 9) | (12 << 5) | 19;

        let (game_date, event_date) = parse_dates_field(dates_field);

        assert_eq!(game_date.to_pgn_string(), "2022.12.19");
        assert!(event_date.is_none());
    }

    #[test]
    fn test_parse_dates_field_from_bytes() {
        let mut bytes = [0u8; 47];

        // Encode date 2022.12.19 at offset 25-28
        let dates_value = (2022u32 << 9) | (12u32 << 5) | 19u32;
        bytes[25..29].copy_from_slice(&dates_value.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.game_date.year, 2022);
        assert_eq!(entry.game_date.month, 12);
        assert_eq!(entry.game_date.day, 19);
    }

    #[test]
    fn test_parse_result_and_counts() {
        let mut bytes = [0u8; 47];

        // var_counts field (bytes 21-22):
        // Result: 3 (Draw)
        // NAG: 2
        // Comments: 1
        // Variations: 0
        let var_counts = (3u16 << 12) | (2u16 << 8) | (1u16 << 4) | 0u16;
        bytes[21..23].copy_from_slice(&var_counts.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.result, GameResult::Draw);
        assert_eq!(entry.nag_count, 2);
        assert_eq!(entry.comment_count, 1);
        assert_eq!(entry.variation_count, 0);
    }

    #[test]
    fn test_parse_elo_ratings() {
        let mut bytes = [0u8; 47];

        // White ELO: 2372, Type: 1 (standard Elo)
        // Encoded: (1 << 12) | 2372 = 0x1944
        bytes[29..31].copy_from_slice(&0x1944u16.to_be_bytes());

        // Black ELO: 2500, Type: 2 (Rapid)
        // Encoded: (2 << 12) | 2500 = 0x29C4
        bytes[31..33].copy_from_slice(&0x29C4u16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.white_elo, 2372);
        assert_eq!(entry.white_rating_type_raw, 1);
        assert_eq!(entry.black_elo, 2500);
        assert_eq!(entry.black_rating_type_raw, 2);
    }

    #[test]
    fn test_rating_type_enum() {
        assert_eq!(RatingType::from_u8(0), RatingType::None);
        assert_eq!(RatingType::from_u8(1), RatingType::Elo);
        assert_eq!(RatingType::from_u8(2), RatingType::Rapid);
        assert_eq!(RatingType::from_u8(3), RatingType::Iccf);
        assert_eq!(RatingType::from_u8(4), RatingType::Uscf);
        assert_eq!(RatingType::from_u8(5), RatingType::Dwz);
        assert_eq!(RatingType::from_u8(6), RatingType::Ecf);
        assert_eq!(RatingType::from_u8(7), RatingType::Unknown);
        assert_eq!(RatingType::from_u8(255), RatingType::Unknown);
    }

    #[test]
    fn test_parse_rating_function() {
        let (rating_type, value) = parse_rating(0x1944); // Type 1 (Elo), value 2372
        assert_eq!(rating_type, RatingType::Elo);
        assert_eq!(value, 2372);

        let (rating_type, value) = parse_rating(0x29C4); // Type 2 (Rapid), value 2500
        assert_eq!(rating_type, RatingType::Rapid);
        assert_eq!(value, 2500);

        let (rating_type, value) = parse_rating(0x4800); // Type 4 (USCF), value 2048
        assert_eq!(rating_type, RatingType::Uscf);
        assert_eq!(value, 2048);

        let (rating_type, value) = parse_rating(0x0000); // Type 0 (None), value 0
        assert_eq!(rating_type, RatingType::None);
        assert_eq!(value, 0);
    }

    #[test]
    fn test_entry_rating_type_helpers() {
        let mut bytes = [0u8; 47];

        // White: Elo 2372, Black: USCF 1800
        bytes[29..31].copy_from_slice(&0x1944u16.to_be_bytes()); // Type 1, value 2372
        bytes[31..33].copy_from_slice(&0x4708u16.to_be_bytes()); // Type 4, value 1800

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Test helper methods
        assert_eq!(entry.white_rating_type(), RatingType::Elo);
        assert_eq!(entry.black_rating_type(), RatingType::Uscf);

        // Test combined rating method
        let (w_type, w_value) = entry.white_rating();
        assert_eq!(w_type, RatingType::Elo);
        assert_eq!(w_value, 2372);

        let (b_type, b_value) = entry.black_rating();
        assert_eq!(b_type, RatingType::Uscf);
        assert_eq!(b_value, 1800);

        // Test has_rating methods
        assert!(entry.has_white_rating());
        assert!(entry.has_black_rating());
    }

    #[test]
    fn test_entry_no_rating() {
        let bytes = [0u8; 47]; // All zeros
        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.white_rating_type(), RatingType::None);
        assert_eq!(entry.black_rating_type(), RatingType::None);
        assert!(!entry.has_white_rating());
        assert!(!entry.has_black_rating());
    }

    #[test]
    fn test_material_signature_endgame() {
        // King + Rook vs King endgame (KR:k)
        // White: 1 rook, 0 everything else
        // Black: 0 everything
        let raw = 0x00_10_00_00u32; // Only WR bit set (bits 20-21 = 1)
        let sig = MaterialSignature::from_raw(raw);

        assert_eq!(sig.white_rooks(), 1);
        assert_eq!(sig.white_queens(), 0);
        assert_eq!(sig.white_pawns(), 0);
        assert_eq!(sig.black_rooks(), 0);

        assert!(sig.has_rooks());
        assert!(!sig.has_queens());
        assert!(!sig.has_pawns());
    }

    #[test]
    fn test_material_signature_bit_layout() {
        // Test individual bit positions by setting one piece type at a time

        // Black pawn (bits 0-3)
        let sig = MaterialSignature::from_raw(0x00000005); // 5 black pawns
        assert_eq!(sig.black_pawns(), 5);
        assert_eq!(sig.black_knights(), 0);

        // Black knight (bits 4-5)
        let sig = MaterialSignature::from_raw(0x00000020); // 2 black knights
        assert_eq!(sig.black_knights(), 2);
        assert_eq!(sig.black_pawns(), 0);

        // White queen (bits 22-23)
        let sig = MaterialSignature::from_raw(0x00C00000); // 3 white queens
        assert_eq!(sig.white_queens(), 3);
        assert_eq!(sig.white_rooks(), 0);
    }

    #[test]
    fn test_entry_material_signature_helper() {
        let mut bytes = [0u8; 47];

        // Set material signature at bytes 33-36
        let mat_sig = MATSIG_STANDARD_START;
        bytes[33..37].copy_from_slice(&mat_sig.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();
        let sig = entry.material_signature();

        assert!(sig.is_standard_start());
        assert_eq!(sig.white_pawns(), 8);
        assert_eq!(sig.black_pawns(), 8);
    }

    #[test]
    fn test_parse_half_moves_10bit() {
        let mut bytes = [0u8; 47];

        // Test 10-bit half-move count
        // Low 8 bits: 255 (byte 37)
        bytes[37] = 255;

        // High 2 bits: 3 (bits 7-6 of byte 38)
        bytes[38] = 0xC0; // Binary: 11000000

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Expected: 255 | (3 << 8) = 255 + 768 = 1023
        assert_eq!(entry.half_moves, 1023);
    }

    #[test]
    fn test_entry_game_flags() {
        let mut bytes = [0u8; 47];

        // Set flags: DELETE_FLAG (bit 3) and PROMO_FLAG (bit 1)
        let flags = game_flags::DELETE_FLAG | game_flags::PROMO_FLAG;
        bytes[7..9].copy_from_slice(&flags.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert!(entry.is_deleted());
        assert!(entry.has_promotions());
        assert!(!entry.has_custom_start());
        assert!(!entry.has_underpromotions());
    }

    #[test]
    fn test_custom_flags() {
        let mut bytes = [0u8; 47];

        // Set custom flag 3 (bit 12)
        let flags = game_flags::CUSTOM_FLAG_3;
        bytes[7..9].copy_from_slice(&flags.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert!(!entry.has_custom_flag(1));
        assert!(!entry.has_custom_flag(2));
        assert!(entry.has_custom_flag(3));
        assert!(!entry.has_custom_flag(4));
        assert!(!entry.has_custom_flag(7)); // Out of range
    }

    #[test]
    fn test_packed_flag_detection() {
        let mut bytes = [0u8; 47];

        // Set FLAG_PACKED (bit 7 of byte 6)
        bytes[6] = 0x80;

        let entry = parse_game_index_entry(&bytes).unwrap();
        assert!(entry.is_packed(), "Should detect packed/compressed game");

        // Test without packed flag
        let mut bytes_uncompressed = [0u8; 47];
        bytes_uncompressed[6] = 0x00;

        let entry_uncompressed = parse_game_index_entry(&bytes_uncompressed).unwrap();
        assert!(
            !entry_uncompressed.is_packed(),
            "Should not be marked as packed"
        );
    }

    #[test]
    fn test_packed_flag_with_length() {
        // Verify packed flag doesn't interfere with game length parsing
        let mut bytes = [0u8; 47];

        // Set length_low = 1000 (bytes 4-5)
        bytes[4..6].copy_from_slice(&1000u16.to_be_bytes());

        // Set byte 6 with BOTH packed flag (bit 7) AND length bit (bit 0)
        // This tests that we correctly separate the two uses of byte 6
        bytes[6] = 0x81; // Packed=1, length_high_bit=1

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert!(entry.is_packed(), "Should be marked as packed");
        // Length should be 1000 + (1 << 16) = 66536
        assert_eq!(
            entry.game_length,
            1000 + 65536,
            "Length should include high bit"
        );
    }

    #[test]
    fn test_eco_to_string() {
        // Test basic ECO codes
        assert_eq!(eco_to_string(0), None); // No ECO
        assert_eq!(eco_to_string(1), Some("A01".to_string())); // A01
        assert_eq!(eco_to_string(99), Some("A99".to_string())); // A99
        assert_eq!(eco_to_string(100), Some("B00".to_string())); // B00
        assert_eq!(eco_to_string(199), Some("B99".to_string())); // B99
        assert_eq!(eco_to_string(200), Some("C00".to_string())); // C00
        assert_eq!(eco_to_string(300), Some("D00".to_string())); // D00
        assert_eq!(eco_to_string(400), Some("E00".to_string())); // E00
        assert_eq!(eco_to_string(499), Some("E99".to_string())); // E99

        // Invalid (beyond E99)
        assert_eq!(eco_to_string(500), None);
    }

    #[test]
    fn test_entry_eco_to_string() {
        let mut entry = GameIndexEntry::new();

        entry.eco_code = 0;
        assert_eq!(entry.eco_to_string(), None);

        entry.eco_code = 245; // C45 (Scotch Game)
        assert_eq!(entry.eco_to_string(), Some("C45".to_string()));
    }

    #[test]
    fn test_event_date_year_offset() {
        // Test event date year offset encoding/decoding
        // Game year: 2023, Event year: 2020 (3 years before)

        let game_year = 2023u32;
        let game_month = 6u32;
        let game_day = 15u32;
        let game_encoded = (game_year << 9) | (game_month << 5) | game_day;

        // Event year 2020 is 3 years before, so offset = -3 + 4 = 1
        let event_year_offset = 1u32;
        let event_month = 1u32;
        let event_day = 1u32;
        let event_encoded = (event_year_offset << 9) | (event_month << 5) | event_day;

        let dates_field = (event_encoded << 20) | game_encoded;

        let (game_date, event_date) = parse_dates_field(dates_field);

        assert_eq!(game_date.year, 2023);
        assert!(event_date.is_some());
        let event = event_date.unwrap();
        assert_eq!(event.year, 2020); // 2023 + 1 - 4 = 2020
        assert_eq!(event.month, 1);
        assert_eq!(event.day, 1);
    }
}
