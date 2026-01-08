//! Index file (.si4) parsing
//!
//! The SCID index file contains all game metadata in a compact binary format:
//! - 182-byte header with database info
//! - 47-byte entries for each game
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
//! # Critical Implementation Details
//!
//! All multi-byte values use BIG-ENDIAN byte order.
//! See SCID_DATABASE_FORMAT.md lines 1013-1051 for verification.

use crate::error::{Result, ScidError};
use crate::types::{GameDate, GameResult};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Magic bytes identifying a valid SI4 file.
/// All SCID index files must start with these exact 8 bytes.
/// See SCID_DATABASE_FORMAT.md line 88.
pub const SI4_MAGIC: &[u8; 8] = b"Scid.si\0";

/// Size of SI4 header in bytes.
/// The header is always exactly 182 bytes.
/// See SCID_DATABASE_FORMAT.md line 85.
pub const SI4_HEADER_SIZE: usize = 182;

/// Size of each game index entry in bytes.
/// Each game has exactly one 47-byte entry.
/// See SCID_DATABASE_FORMAT.md line 120.
pub const GAME_ENTRY_SIZE: usize = 47;

/// Expected SCID version number.
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
/// See SCID_DATABASE_FORMAT.md lines 85-96.
#[derive(Debug, Clone, PartialEq)]
pub struct Si4Header {
    /// File format version (should be 400)
    pub version: u16,

    /// Database type flags
    pub base_type: u32,

    /// Total number of games in database (24-bit value, max 16,777,215 games)
    pub num_games: u32,

    /// Auto-load game number (usually 0)
    pub auto_load: u32,

    /// Database description text (UTF-8, max 108 bytes)
    pub description: String,

    /// Custom flag descriptions (6 flags × 9 bytes each)
    pub custom_flags: [String; 6],
}

impl Si4Header {
    /// Create a new header with default values (for testing)
    pub fn new() -> Self {
        Si4Header {
            version: SCID_VERSION,
            base_type: 0,
            num_games: 0,
            auto_load: 0,
            description: String::new(),
            custom_flags: Default::default(),
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
}

impl Default for Si4Header {
    fn default() -> Self {
        Self::new()
    }
}

/// Game index entry (47 bytes)
///
/// Contains all metadata for a single game. Stored immediately after the header.
///
/// # Critical Implementation Notes
///
/// - All multi-byte values use BIG-ENDIAN byte order
/// - Many fields use packed bit fields across multiple bytes
/// - Dates are at fixed offset 25-28 in each entry
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
/// See SCID_DATABASE_FORMAT.md lines 120-210.
#[derive(Debug, Clone, PartialEq)]
pub struct GameIndexEntry {
    /// Byte offset of game data in .sg4 file
    pub game_offset: u32,

    /// Length of game data in bytes (17-bit value, max 131,071)
    pub game_length: u32,

    /// White player name ID (20-bit value, index into name file)
    pub white_id: u32,

    /// Black player name ID (20-bit value, index into name file)
    pub black_id: u32,

    /// Event name ID (19-bit value)
    pub event_id: u32,

    /// Site name ID (19-bit value)
    pub site_id: u32,

    /// Round name ID (18-bit value)
    pub round_id: u32,

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

    /// White rating type (4-bit value: 0=None, 1=Elo, 2=FIDE, etc.)
    pub white_rating_type: u8,

    /// Black rating type (4-bit value)
    pub black_rating_type: u8,

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
            white_rating_type: 0,
            black_rating_type: 0,
            eco_code: 0,
            half_moves: 0,
            variation_count: 0,
            comment_count: 0,
            nag_count: 0,
            flags: 0,
            final_material_signature: 0,
        }
    }
}

impl Default for GameIndexEntry {
    fn default() -> Self {
        Self::new()
    }
}

// Name lookup methods - requires NameDatabase from names.rs
use super::names::NameDatabase;

impl GameIndexEntry {
    /// Get white player name from name database
    pub fn get_white_name<'a>(&self, names: &'a NameDatabase) -> Option<&'a str> {
        names.get_player(self.white_id)
    }

    /// Get black player name from name database
    pub fn get_black_name<'a>(&self, names: &'a NameDatabase) -> Option<&'a str> {
        names.get_player(self.black_id)
    }

    /// Get both player names as tuple (white, black)
    pub fn get_player_names<'a>(
        &self,
        names: &'a NameDatabase,
    ) -> (Option<&'a str>, Option<&'a str>) {
        (self.get_white_name(names), self.get_black_name(names))
    }

    /// Get event name from name database
    pub fn get_event_name<'a>(&self, names: &'a NameDatabase) -> Option<&'a str> {
        names.get_event(self.event_id)
    }

    /// Get site name from name database
    pub fn get_site_name<'a>(&self, names: &'a NameDatabase) -> Option<&'a str> {
        names.get_site(self.site_id)
    }

    /// Get round name from name database
    pub fn get_round_name<'a>(&self, names: &'a NameDatabase) -> Option<&'a str> {
        names.get_round(self.round_id)
    }
}

/// Parse SI4 header from file.
///
/// Reads and validates the 182-byte header from a .si4 file.
/// All multi-byte values are read as BIG-ENDIAN.
///
/// # Errors
///
/// - `ScidError::InvalidFormat` - Wrong magic bytes or invalid version
/// - `ScidError::Io` - File read error
pub fn parse_si4_header(file: &mut File) -> Result<Si4Header> {
    let mut header_bytes = [0u8; SI4_HEADER_SIZE];
    file.read_exact(&mut header_bytes).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            ScidError::InvalidFormat(format!(
                "File too short: expected {} bytes for header",
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
    let version = u16::from_be_bytes([header_bytes[8], header_bytes[9]]);

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
    let num_games = u32::from_be_bytes([0, header_bytes[14], header_bytes[15], header_bytes[16]]);

    // Parse auto-load (offset 17-19, BIG-ENDIAN 24-bit value)
    let auto_load = u32::from_be_bytes([0, header_bytes[17], header_bytes[18], header_bytes[19]]);

    // Parse description (offset 20-127, UTF-8 string)
    let description = String::from_utf8_lossy(&header_bytes[20..128])
        .chars()
        .filter(|&c| c >= ' ')
        .collect::<String>()
        .trim()
        .to_string();

    // Parse custom flags (offset 128-181, six 9-byte strings)
    let mut custom_flags: [String; 6] = Default::default();
    for (i, flag) in custom_flags.iter_mut().enumerate() {
        let start = 128 + (i * 9);
        let end = start + 9;
        *flag = String::from_utf8_lossy(&header_bytes[start..end])
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
        custom_flags,
    })
}

/// Parse the dates field (32-bit value at offset 25-28).
///
/// The dates field contains TWO dates packed together:
/// - Upper 12 bits: Event date (relative encoding)
/// - Lower 20 bits: Game date (absolute encoding)
///
/// # Game Date (20 bits, absolute)
///
/// - Bits 19-9: Year (0-2047)
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// # Event Date (12 bits, relative)
///
/// - Bits 11-9: Year offset (0-7) relative to game year
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// See SCID_DATABASE_FORMAT.md lines 211-313.
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
        None
    } else {
        let event_day = (event_data & 0x1F) as u8;
        let event_month = ((event_data >> 5) & 0x0F) as u8;
        let year_offset = ((event_data >> 9) & 0x7) as i16;

        if year_offset == 0 {
            None
        } else {
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

/// Parse a single game index entry from 47 bytes.
///
/// All multi-byte values use BIG-ENDIAN byte order.
///
/// # Errors
///
/// Returns error if bytes slice is not exactly 47 bytes.
pub fn parse_game_index_entry(bytes: &[u8]) -> Result<GameIndexEntry> {
    if bytes.len() != GAME_ENTRY_SIZE {
        return Err(ScidError::Validation(format!(
            "Invalid entry size: expected {} bytes, got {}",
            GAME_ENTRY_SIZE,
            bytes.len()
        )));
    }

    // Parse game offset (bytes 0-3, BIG-ENDIAN u32)
    let game_offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);

    // Parse game length (17-bit value split across bytes 4-6)
    // See SCID_DATABASE_FORMAT.md lines 148-157
    let length_low = u16::from_be_bytes([bytes[4], bytes[5]]) as u32;
    let length_high = bytes[6];
    let game_length = length_low | (((length_high & 0x80) as u32) << 9);

    // Parse game flags (bytes 7-8, BIG-ENDIAN u16)
    let flags = u16::from_be_bytes([bytes[7], bytes[8]]);

    // Parse Player IDs (20 bits each)
    // See SCID_DATABASE_FORMAT.md lines 159-172
    let white_black_high = bytes[9];
    let white_id = ((white_black_high & 0xF0) as u32) << 12
        | u16::from_be_bytes([bytes[10], bytes[11]]) as u32;
    let black_id = ((white_black_high & 0x0F) as u32) << 16
        | u16::from_be_bytes([bytes[12], bytes[13]]) as u32;

    // Parse Event/Site/Round IDs
    let event_site_rnd_high = bytes[14];
    let event_id = ((event_site_rnd_high & 0xE0) as u32) << 11
        | u16::from_be_bytes([bytes[15], bytes[16]]) as u32;
    let site_id = ((event_site_rnd_high & 0x1C) as u32) << 14
        | u16::from_be_bytes([bytes[17], bytes[18]]) as u32;
    let round_id = ((event_site_rnd_high & 0x03) as u32) << 16
        | u16::from_be_bytes([bytes[19], bytes[20]]) as u32;

    // Parse variation counts and result (bytes 21-22, BIG-ENDIAN u16)
    // See SCID_DATABASE_FORMAT.md lines 174-187
    let var_counts = u16::from_be_bytes([bytes[21], bytes[22]]);
    let result_code = (var_counts >> 12) as u8;
    let result = GameResult::from_scid(result_code);
    let nag_count = ((var_counts >> 8) & 0x0F) as u8;
    let comment_count = ((var_counts >> 4) & 0x0F) as u8;
    let variation_count = (var_counts & 0x0F) as u8;

    // Parse ECO code (bytes 23-24, BIG-ENDIAN u16)
    let eco_code = u16::from_be_bytes([bytes[23], bytes[24]]);

    // Parse dates field (bytes 25-28, BIG-ENDIAN u32)
    // CRITICAL: This is at FIXED OFFSET 25-28!
    let dates_field = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);
    let (game_date, event_date) = parse_dates_field(dates_field);

    // Parse ELO ratings (bytes 29-32)
    // See SCID_DATABASE_FORMAT.md lines 189-200
    let white_elo_raw = u16::from_be_bytes([bytes[29], bytes[30]]);
    let white_elo = white_elo_raw & 0x0FFF;
    let white_rating_type = (white_elo_raw >> 12) as u8;

    let black_elo_raw = u16::from_be_bytes([bytes[31], bytes[32]]);
    let black_elo = black_elo_raw & 0x0FFF;
    let black_rating_type = (black_elo_raw >> 12) as u8;

    // Parse final material signature (bytes 33-36, BIG-ENDIAN u32)
    let final_material_signature = u32::from_be_bytes([bytes[33], bytes[34], bytes[35], bytes[36]]);

    // Parse half-move count (10-bit value split across bytes 37-38)
    let half_moves_low = bytes[37] as u16;
    let half_moves_high = ((bytes[38] >> 6) & 0x03) as u16;
    let half_moves = half_moves_low | (half_moves_high << 8);

    Ok(GameIndexEntry {
        game_offset,
        game_length,
        white_id,
        black_id,
        event_id,
        site_id,
        round_id,
        game_date,
        event_date,
        result,
        white_elo,
        black_elo,
        white_rating_type,
        black_rating_type,
        eco_code,
        half_moves,
        variation_count,
        comment_count,
        nag_count,
        flags,
        final_material_signature,
    })
}

/// Parse entire index file and return header + all entries.
pub fn parse_si4_file(path: impl AsRef<Path>) -> Result<(Si4Header, Vec<GameIndexEntry>)> {
    let mut file = File::open(path.as_ref())?;

    let header = parse_si4_header(&mut file)?;

    let mut entries = Vec::with_capacity(header.num_games as usize);

    for game_num in 0..header.num_games {
        let mut entry_bytes = [0u8; GAME_ENTRY_SIZE];
        file.read_exact(&mut entry_bytes)
            .map_err(|e| ScidError::ParseError {
                file: path.as_ref().to_path_buf(),
                offset: SI4_HEADER_SIZE as u64 + (game_num * GAME_ENTRY_SIZE as u32) as u64,
                message: format!("Failed to read game {} entry: {}", game_num + 1, e),
            })?;

        let entry = parse_game_index_entry(&entry_bytes)?;
        entries.push(entry);
    }

    Ok((header, entries))
}

/// Iterator over game index entries.
///
/// Allows lazy loading of entries without reading entire file into memory.
pub struct IndexEntryIter {
    file: File,
    current_game: u32,
    total_games: u32,
}

impl IndexEntryIter {
    /// Create iterator from opened .si4 file.
    /// File should already have header parsed; this seeks to first entry.
    pub fn new(mut file: File, total_games: u32) -> Result<Self> {
        file.seek(SeekFrom::Start(SI4_HEADER_SIZE as u64))?;

        Ok(IndexEntryIter {
            file,
            current_game: 0,
            total_games,
        })
    }
}

impl Iterator for IndexEntryIter {
    type Item = Result<GameIndexEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_game >= self.total_games {
            return None;
        }

        let mut entry_bytes = [0u8; GAME_ENTRY_SIZE];
        match self.file.read_exact(&mut entry_bytes) {
            Ok(_) => {
                self.current_game += 1;
                Some(parse_game_index_entry(&entry_bytes))
            }
            Err(e) => Some(Err(ScidError::Io(e))),
        }
    }
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
    fn test_game_index_entry_default() {
        let entry = GameIndexEntry::default();
        assert_eq!(entry.game_offset, 0);
        assert_eq!(entry.game_length, 0);
        assert_eq!(entry.result, GameResult::Unknown);
    }

    #[test]
    fn test_parse_game_offset_and_length() {
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

        // Set length_low = 0xFFFF (65535)
        bytes[4..6].copy_from_slice(&0xFFFFu16.to_be_bytes());

        // Set high bit in byte 6
        bytes[6] = 0x80;

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Expected: 65535 | (1 << 16) = 65535 + 65536 = 131071
        assert_eq!(entry.game_length, 131071);
    }

    #[test]
    fn test_invalid_entry_size() {
        let bytes = [0u8; 40]; // Wrong size
        let result = parse_game_index_entry(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_player_ids() {
        let mut bytes = [0u8; 47];

        // White ID: 0x12345 (74565)
        // High 4 bits: 0x1, Low 16 bits: 0x2345
        bytes[9] = 0x10; // High nibble = 1
        bytes[10..12].copy_from_slice(&0x2345u16.to_be_bytes());

        // Black ID: 0xABCDE (703710)
        // High 4 bits: 0xA, Low 16 bits: 0xBCDE
        bytes[9] |= 0x0A; // Low nibble = A
        bytes[12..14].copy_from_slice(&0xBCDEu16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.white_id, 0x12345);
        assert_eq!(entry.black_id, 0xABCDE);
    }

    #[test]
    fn test_parse_game_date_2022_12_19() {
        // Date: 2022.12.19
        let year = 2022u32;
        let month = 12u32;
        let day = 19u32;
        let encoded = (year << 9) | (month << 5) | day;

        let (game_date, _) = parse_dates_field(encoded);

        assert_eq!(game_date.year, 2022);
        assert_eq!(game_date.month, 12);
        assert_eq!(game_date.day, 19);
        assert_eq!(game_date.to_pgn_string(), "2022.12.19");
    }

    #[test]
    fn test_parse_dates_no_event_date() {
        let dates_field = (2022 << 9) | (12 << 5) | 19;

        let (game_date, event_date) = parse_dates_field(dates_field);

        assert_eq!(game_date.to_pgn_string(), "2022.12.19");
        assert!(event_date.is_none());
    }

    #[test]
    fn test_parse_dates_with_event_date() {
        // Game date: 2022.06.15
        let game_year = 2022u32;
        let game_month = 6u32;
        let game_day = 15u32;
        let game_encoded = (game_year << 9) | (game_month << 5) | game_day;

        // Event date: 2022.08.10 (same year, so offset = 4)
        let event_month = 8u32;
        let event_day = 10u32;
        let year_offset = 4u32; // (2022 - 2022 + 4) = 4
        let event_encoded = (year_offset << 9) | (event_month << 5) | event_day;

        let dates_field = (event_encoded << 20) | game_encoded;

        let (game_date, event_date) = parse_dates_field(dates_field);

        assert_eq!(game_date.to_pgn_string(), "2022.06.15");
        assert!(event_date.is_some());
        assert_eq!(event_date.unwrap().to_pgn_string(), "2022.08.10");
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

        // Black ELO: 2500, Type: 2 (FIDE)
        // Encoded: (2 << 12) | 2500 = 0x29C4
        bytes[31..33].copy_from_slice(&0x29C4u16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        assert_eq!(entry.white_elo, 2372);
        assert_eq!(entry.white_rating_type, 1);
        assert_eq!(entry.black_elo, 2500);
        assert_eq!(entry.black_rating_type, 2);
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
    fn test_parse_event_site_round_ids() {
        let mut bytes = [0u8; 47];

        // Event ID: 19-bit max value (0x7FFFF = 524287)
        // High 3 bits: 0x7, Low 16 bits: 0xFFFF
        bytes[14] = 0xE0; // Bits 7-5 = 111
        bytes[15..17].copy_from_slice(&0xFFFFu16.to_be_bytes());

        let entry = parse_game_index_entry(&bytes).unwrap();

        // Validate event ID
        assert_eq!(entry.event_id, 0x7FFFF); // Max 19-bit value
    }
}
