# SCID to PGN Converter - Implementation Plan

## Overview

This plan outlines the step-by-step implementation of a SCID database parser and PGN converter from scratch, following the proposed project structure and using the SCID database format specification as the technical reference.

## Guiding Principles

1. **Build incrementally** - Start simple, add complexity gradually
2. **Test continuously** - Validate each component before moving forward
3. **Follow the spec** - SCID_DATABASE_FORMAT.md is the source of truth
4. **Library-first** - Core functionality before CLI wrapper
5. **Leverage shakmaty** - Use battle-tested chess library for position/move logic
6. **Real data validation** - Test with actual SCID databases early and often

## Key Technologies

- **Rust** - Systems programming language for performance and safety
- **Shakmaty** - Comprehensive chess library handling position, moves, and SAN notation
- **Thiserror** - Ergonomic error handling
- **Clap** - Command-line argument parsing
- **Byteorder** - Big-endian binary parsing
- **Memmap2** - Cross-platform memory mapping for large files (optional)

By using shakmaty, we avoid reimplementing chess rules, move validation, and algebraic notation generation. This reduces implementation time by ~25% and eliminates an entire class of potential bugs.

---

## Test Data

### Location

All test data is located in the `tests/data/` folder.

### Available Datasets

Two reference datasets are provided for validation:

| Dataset | Files | Description |
|---------|-------|-------------|
| **one** | `one.pgn`, `one.si4`, `one.sg4`, `one.sn4` | Single game for basic parsing validation |
| **five** | `five.pgn`, `five.si4`, `five.sg4`, `five.sn4` | Five games for comprehensive testing |

### Relationship Between Files

Each PGN file is the **source of truth** for its corresponding SCID database:

- The SCID databases were created by importing the PGN files into SCID
- The PGN and SCID files contain **identical chess content** in different formats
- This enables **round-trip validation**: parse SCID → generate PGN → compare with original PGN

```
┌─────────────┐     Import      ┌─────────────────────────┐
│  one.pgn    │  ─────────────► │  one.si4 + sg4 + sn4    │
│  (source)   │                 │  (SCID database)        │
└─────────────┘                 └─────────────────────────┘
       ▲                                    │
       │                                    │ Parse
       │         Compare                    ▼
       └──────────────────────────  Generated PGN output
```

### Validation Strategy

1. **Unit tests**: Parse individual components (headers, index entries, names) and verify against known values
2. **Integration tests**: Parse complete SCID database and compare generated PGN against source PGN file
3. **Regression tests**: Ensure changes don't break previously working functionality

### Using Test Data in Each Phase

| Phase | Test Data Usage |
|-------|-----------------|
| Phase 2 (Index File Parser) | Parse `one.si4` and `five.si4` headers and index entries |
| Phase 3 (Name File Parser) | Parse `one.sn4` and `five.sn4`, verify player/event names |
| Phase 4 (Game File Structure) | Read game data from `one.sg4` and `five.sg4` |
| Phase 5 (Move Parsing) | Decode moves and verify against expected PGN movetext |
| Phase 6 (PGN Output) | Generate PGN, compare against `one.pgn` and `five.pgn` |
| Phase 9 (Testing & Validation) | Full round-trip validation with both datasets |

### Adding New Test Data

When adding new test datasets:

1. Create the PGN file with the desired games
2. Import into SCID to generate the `.si4`, `.sg4`, `.sn4` files
3. Place all files in `tests/data/` with matching base names
4. Document any special characteristics (variations, comments, non-standard positions, etc.)

---

## Phase 1: Foundation (Week 1)

### 1.1 Project Structure Setup

**Goal**: Create the workspace skeleton with all directories and basic configuration.

**Tasks**:
- Create workspace root `Cargo.toml` with resolver and workspace members
- Create `crates/core/` directory structure with all modules
- Create `crates/cli/` directory structure
- Set up workspace-level dependencies (thiserror, shakmaty, byteorder, etc.)
- Initialize git repository with .gitignore
- Create README.md with project description

**Deliverable**: Empty but compilable workspace with `cargo build` succeeding.

---

### 1.2 Core Types and Error Handling

**Goal**: Define fundamental types and error handling before parsing logic.

**Files to create**:
- `crates/core/src/error.rs` - Centralized error types
- `crates/core/src/types.rs` - Shared chess types

**Implementation**:

```rust
// error.rs - Use thiserror for clean error definitions
#[derive(Debug, Error)]
pub enum ScidError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid SCID file: {0}")]
    InvalidFormat(String),

    #[error("Parse error in {file} at offset {offset}: {message}")]
    ParseError {
        file: PathBuf,
        offset: u64,
        message: String,
    },

    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),
}

pub type Result<T> = std::result::Result<T, ScidError>;
```

```rust
// types.rs - Shared types (chess types come from shakmaty)
pub use shakmaty::{Color, Role, Square, Piece, Move as ChessMove};

#[derive(Debug, Clone)]
pub struct GameDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl GameDate {
    /// Convert to PGN date string format
    ///
    /// PGN standard requires unknown components to use "??" or "????":
    /// - Unknown year: "????.MM.DD"
    /// - Unknown month: "YYYY.??.DD"
    /// - Unknown day: "YYYY.MM.??"
    /// - Fully unknown: "????.??.??"
    ///
    /// SCID uses 0 to indicate unknown date components.
    ///
    /// # Examples
    /// ```
    /// GameDate { year: 1997, month: 5, day: 11 }.to_pgn_string() // "1997.05.11"
    /// GameDate { year: 1997, month: 5, day: 0 }.to_pgn_string()  // "1997.05.??"
    /// GameDate { year: 1997, month: 0, day: 0 }.to_pgn_string()  // "1997.??.??"
    /// GameDate { year: 0, month: 0, day: 0 }.to_pgn_string()     // "????.??.??"
    /// ```
    pub fn to_pgn_string(&self) -> String {
        let year_str = if self.year == 0 {
            "????".to_string()
        } else {
            format!("{:04}", self.year)
        };

        let month_str = if self.month == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.month)
        };

        let day_str = if self.day == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.day)
        };

        format!("{}.{}.{}", year_str, month_str, day_str)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    Unknown,
}

impl GameResult {
    pub fn to_string(&self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unknown => "*",
        }
    }
}
```

**Testing**:
- Unit tests for error construction and display
- Unit tests for basic type operations

**Deliverable**: Compiled types and error handling ready for use.

---

## Phase 2: Index File Parser (Week 1-2)

### 2.1 SI4 Header Parsing

**Goal**: Read and validate .si4 file headers.

**File**: `crates/core/src/database/reader.rs`

**Implementation**:

```rust
const SI4_MAGIC: &[u8; 8] = b"Scid.si\0";
const SI4_HEADER_SIZE: usize = 182;

pub struct Si4Header {
    pub version: u16,
    pub base_type: u32,
    pub num_games: u32,
    pub auto_load: u32,
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

pub fn parse_si4_header(file: &mut File) -> Result<Si4Header> {
    let mut header_bytes = [0u8; SI4_HEADER_SIZE];
    file.read_exact(&mut header_bytes)?;

    // Validate magic
    if &header_bytes[0..8] != SI4_MAGIC {
        return Err(ScidError::InvalidFormat("Invalid SI4 magic".into()));
    }

    // Parse version (BIG-ENDIAN!)
    let version = u16::from_be_bytes([header_bytes[8], header_bytes[9]]);

    // Parse game count (24-bit big-endian)
    let num_games = u32::from_be_bytes([0, header_bytes[14], header_bytes[15], header_bytes[16]]);

    // Extract description (UTF-8, null-terminated)
    let description = String::from_utf8_lossy(&header_bytes[20..128])
        .trim_end_matches('\0')
        .to_string();

    // Parse custom flag names (bytes 128-182, 6 names × 9 bytes each)
    // Each name is up to 8 characters plus null terminator, null-padded
    let mut custom_flag_names: [String; 6] = Default::default();
    for i in 0..6 {
        let start = 128 + (i * CUSTOM_FLAG_NAME_SIZE);
        let end = start + CUSTOM_FLAG_NAME_SIZE;
        let name = String::from_utf8_lossy(&header_bytes[start..end])
            .trim_end_matches('\0')
            .to_string();
        custom_flag_names[i] = name;
    }

    Ok(Si4Header {
        version,
        base_type: u32::from_be_bytes([header_bytes[10], header_bytes[11], header_bytes[12], header_bytes[13]]),
        num_games,
        auto_load: u32::from_be_bytes([0, header_bytes[17], header_bytes[18], header_bytes[19]]),
        description,
        custom_flag_names,
    })
}

impl Si4Header {
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
```

**Testing**:
- Test with real SCID database (e.g., `five.si4`)
- Verify version == 400
- Verify game count matches expected
- Test error handling with corrupted files

**Deliverable**: Validated header parsing with tests passing.

#### SI4 Complete File Structure

The SI4 file contains three sections:

```
┌─────────────────────────────────────────────────────────────┐
│ HEADER (182 bytes)                                          │
│   Bytes 0-7:    Magic "Scid.si\0"                          │
│   Bytes 8-9:    Version (big-endian u16, typically 400)    │
│   Bytes 10-13:  Base type (big-endian u32)                 │
│   Bytes 14-16:  Number of games (24-bit big-endian)        │
│   Bytes 17-19:  Auto-load game number (24-bit big-endian)  │
│   Bytes 20-127: Description (UTF-8, null-padded)           │
│   Bytes 128-182: Custom flag names (6 × 9 bytes each)      │
├─────────────────────────────────────────────────────────────┤
│ INDEX ENTRIES (47 bytes × num_games)                        │
│   Each entry contains game metadata (see Phase 2.2)         │
│   Games are stored in insertion order (game 0, 1, 2, ...)   │
├─────────────────────────────────────────────────────────────┤
│ SORTING INDEX (4 bytes × num_games) - OPTIONAL              │
│   Array of 32-bit game numbers in sorted order              │
│   Allows iteration in a specific sort order without         │
│   re-sorting the index entries                              │
│                                                             │
│   Example: If sorting_index = [3, 1, 0, 2], then:          │
│   - First in sorted order: game 3                           │
│   - Second in sorted order: game 1                          │
│   - Third in sorted order: game 0                           │
│   - Fourth in sorted order: game 2                          │
│                                                             │
│   Sort criteria depends on SCID settings (date, names, etc) │
│   This section may be absent in older databases             │
└─────────────────────────────────────────────────────────────┘
```

**Sorting Index Parsing** (optional):

```rust
/// Parse the optional sorting index from SI4 file
///
/// The sorting index appears after all index entries and contains
/// game numbers in a pre-computed sort order.
///
/// Returns None if:
/// - File doesn't have enough bytes for sorting index
/// - Database was created without sorting
pub fn parse_sorting_index(
    si4_data: &[u8],
    num_games: u32,
) -> Option<Vec<u32>> {
    let header_size = SI4_HEADER_SIZE;
    let entries_size = num_games as usize * 47;
    let sorting_start = header_size + entries_size;
    let sorting_size = num_games as usize * 4;

    // Check if file has sorting index
    if si4_data.len() < sorting_start + sorting_size {
        return None; // No sorting index present
    }

    let sorting_data = &si4_data[sorting_start..sorting_start + sorting_size];

    let mut sorting_index = Vec::with_capacity(num_games as usize);
    for chunk in sorting_data.chunks_exact(4) {
        let game_num = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        sorting_index.push(game_num);
    }

    Some(sorting_index)
}

/// Iterate games in sorted order (if sorting index available)
impl ScidDatabase {
    pub fn iter_sorted(&self) -> Box<dyn Iterator<Item = (usize, &GameIndexEntry)> + '_> {
        if let Some(ref sorting) = self.sorting_index {
            // Iterate in pre-sorted order
            Box::new(sorting.iter().filter_map(|&game_num| {
                let idx = game_num as usize;
                self.index_entries.get(idx).map(|entry| (idx, entry))
            }))
        } else {
            // Fall back to insertion order
            Box::new(self.iter_games())
        }
    }
}
```

**Note**: The sorting index is a convenience feature. For our converter tool, we typically iterate in insertion order (sequential file access is faster). The sorting index is mainly useful for UI applications that display games in a specific order.

---

### 2.2 Game Index Entry Parsing

**Goal**: Parse 47-byte game index entries with all metadata fields.

**File**: `crates/core/src/database/index.rs`

**Critical Implementation Details**:
- All multi-byte values are **BIG-ENDIAN**
- Dates are at **fixed offset 25-28**
- IDs use packed bit fields (see spec for exact bit positions)
- Game result encoded in variation counts field

**Implementation**:

```rust
#[derive(Debug, Clone)]
pub struct GameIndexEntry {
    pub game_offset: u32,
    pub game_length: u32,
    pub white_id: u32,
    pub black_id: u32,
    pub event_id: u32,
    pub site_id: u32,
    pub round_id: u32,
    pub game_date: GameDate,
    pub event_date: Option<GameDate>,
    pub result: GameResult,
    pub white_elo: u16,
    pub black_elo: u16,
    pub eco_code: u16,
    pub half_moves: u16,
    pub flags: u16,
}

/// Game flag bit constants
///
/// The flags field is a 16-bit value where each bit has a specific meaning.
/// These flags are stored in the SI4 index file for quick filtering without
/// needing to read the full game data.
pub mod game_flags {
    /// Bit 0: Non-standard starting position
    /// When set, the game data contains a FEN string after the flags byte.
    /// Used for Chess960 games and games starting from specific positions.
    pub const START_FLAG: u16 = 0x0001;

    /// Bit 1: Game contains pawn promotions
    /// Useful for searching games with promotion tactics.
    pub const PROMO_FLAG: u16 = 0x0002;

    /// Bit 2: Game contains underpromotions (to R, B, or N instead of Q)
    /// Subset of PROMO_FLAG - underpromotions are relatively rare and
    /// often indicate tactical themes.
    pub const UNDER_PROMO: u16 = 0x0004;

    /// Bit 3: Game is marked as deleted
    /// Deleted games are typically skipped during iteration.
    /// They remain in the database until compaction.
    pub const DELETE_FLAG: u16 = 0x0008;

    /// Bits 4-9: Custom user flags (6 flags)
    /// Users can assign custom meanings to these flags in SCID.
    /// Flag names are stored in the SI4 header (bytes 128-182).
    pub const CUSTOM_FLAG_1: u16 = 0x0010;
    pub const CUSTOM_FLAG_2: u16 = 0x0020;
    pub const CUSTOM_FLAG_3: u16 = 0x0040;
    pub const CUSTOM_FLAG_4: u16 = 0x0080;
    pub const CUSTOM_FLAG_5: u16 = 0x0100;
    pub const CUSTOM_FLAG_6: u16 = 0x0200;

    // Bits 10-15: Reserved for future use / additional metadata
}

impl GameIndexEntry {
    /// Check if game has a non-standard starting position (FEN required)
    pub fn has_custom_start(&self) -> bool {
        self.flags & game_flags::START_FLAG != 0
    }

    /// Check if game contains any pawn promotions
    pub fn has_promotions(&self) -> bool {
        self.flags & game_flags::PROMO_FLAG != 0
    }

    /// Check if game contains underpromotions (to R, B, or N)
    pub fn has_underpromotions(&self) -> bool {
        self.flags & game_flags::UNDER_PROMO != 0
    }

    /// Check if game is marked as deleted
    ///
    /// Deleted games should typically be skipped during iteration.
    /// Use `ScidReader::iter_games()` which filters deleted games by default,
    /// or `ScidReader::iter_all_games()` to include deleted games.
    pub fn is_deleted(&self) -> bool {
        self.flags & game_flags::DELETE_FLAG != 0
    }

    /// Check a specific custom flag (1-6)
    ///
    /// Returns None if flag_num is not in range 1-6.
    pub fn has_custom_flag(&self, flag_num: u8) -> Option<bool> {
        let flag_bit = match flag_num {
            1 => game_flags::CUSTOM_FLAG_1,
            2 => game_flags::CUSTOM_FLAG_2,
            3 => game_flags::CUSTOM_FLAG_3,
            4 => game_flags::CUSTOM_FLAG_4,
            5 => game_flags::CUSTOM_FLAG_5,
            6 => game_flags::CUSTOM_FLAG_6,
            _ => return None,
        };
        Some(self.flags & flag_bit != 0)
    }
}

/// Convert SCID eco_code to PGN ECO string
///
/// ECO (Encyclopedia of Chess Openings) codes classify chess openings.
/// Format: Letter (A-E) + two-digit number (00-99)
/// Total: 500 possible codes (A00-E99)
///
/// SCID stores ECO as: (letter_index * 100) + number
/// Where letter_index: A=0, B=1, C=2, D=3, E=4
///
/// # Examples
/// ```
/// eco_to_string(0)   // -> None (unknown/unclassified)
/// eco_to_string(1)   // -> Some("A01")
/// eco_to_string(100) // -> Some("B00")
/// eco_to_string(112) // -> Some("B12") - Caro-Kann Defense
/// eco_to_string(499) // -> Some("E99") - King's Indian
/// eco_to_string(500) // -> None (invalid - beyond E99)
/// ```
pub fn eco_to_string(eco: u16) -> Option<String> {
    if eco == 0 {
        return None; // Unknown/unclassified opening
    }

    let letter_index = (eco / 100) as u8;
    let number = eco % 100;

    // Valid ECO codes are A00-E99 (letter_index 0-4)
    if letter_index > 4 {
        return None; // Invalid ECO code
    }

    let letter = (b'A' + letter_index) as char;
    Some(format!("{}{:02}", letter, number))
}

pub fn parse_game_index_entry(bytes: &[u8; 47]) -> Result<GameIndexEntry> {
    // Game offset and length (with 17-bit length extraction)
    let game_offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let length_low = u16::from_be_bytes([bytes[4], bytes[5]]);
    let length_high = bytes[6];
    let game_length = length_low as u32 | (((length_high & 0x80) as u32) << 9);

    // Player IDs (packed 20-bit values)
    let white_id = ((bytes[9] & 0xF0) as u32) << 12
                 | u16::from_be_bytes([bytes[10], bytes[11]]) as u32;
    let black_id = ((bytes[9] & 0x0F) as u32) << 16
                 | u16::from_be_bytes([bytes[12], bytes[13]]) as u32;

    // Event/Site/Round IDs (packed)
    let event_id = ((bytes[14] & 0xE0) as u32) << 11
                 | u16::from_be_bytes([bytes[15], bytes[16]]) as u32;
    let site_id = ((bytes[14] & 0x1C) as u32) << 14
                | u16::from_be_bytes([bytes[17], bytes[18]]) as u32;
    let round_id = ((bytes[14] & 0x03) as u32) << 16
                 | u16::from_be_bytes([bytes[19], bytes[20]]) as u32;

    // Dates field (CRITICAL: offset 25-28, big-endian)
    let dates_field = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);
    let (game_date, event_date) = parse_dates_field(dates_field);

    // Result from variation counts
    let var_counts = u16::from_be_bytes([bytes[21], bytes[22]]);
    let result = match var_counts >> 12 {
        1 => GameResult::WhiteWins,
        2 => GameResult::BlackWins,
        3 => GameResult::Draw,
        _ => GameResult::Unknown,
    };

    // ELO ratings (12-bit values)
    let white_elo = u16::from_be_bytes([bytes[29], bytes[30]]) & 0x0FFF;
    let black_elo = u16::from_be_bytes([bytes[31], bytes[32]]) & 0x0FFF;

    // Other fields
    let eco_code = u16::from_be_bytes([bytes[23], bytes[24]]);
    let flags = u16::from_be_bytes([bytes[7], bytes[8]]);

    // Half-move count (10-bit split field)
    let half_moves_low = bytes[37] as u16;
    let half_moves_high = ((bytes[38] >> 6) & 0x03) as u16;
    let half_moves = half_moves_low | (half_moves_high << 8);

    Ok(GameIndexEntry {
        game_offset, game_length, white_id, black_id,
        event_id, site_id, round_id, game_date, event_date,
        result, white_elo, black_elo, eco_code, half_moves, flags,
    })
}

/// Parse the 32-bit dates field into game date and optional event date
///
/// # Date Field Layout (32 bits)
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │ Bits 31-20 (12 bits): Event Date (relative encoding)           │
/// │   Bits 31-29 (3 bits): Year offset (0-7, subtract 4 for delta) │
/// │   Bits 28-25 (4 bits): Month (1-12, 0 = unknown)               │
/// │   Bits 24-20 (5 bits): Day (1-31, 0 = unknown)                 │
/// ├─────────────────────────────────────────────────────────────────┤
/// │ Bits 19-0 (20 bits): Game Date (absolute encoding)             │
/// │   Bits 19-9 (11 bits): Year (0-2047)                           │
/// │   Bits 8-5 (4 bits): Month (1-12, 0 = unknown)                 │
/// │   Bits 4-0 (5 bits): Day (1-31, 0 = unknown)                   │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
///
/// # Event Date Year Offset Formula
///
/// The event date year is stored as a 3-bit offset (0-7) relative to the
/// game date year. The formula to calculate the actual event year is:
///
/// ```text
/// event_year = game_year + year_offset - 4
/// ```
///
/// This allows event dates to range from -3 to +3 years relative to the game:
///
/// | year_offset | Calculation  | Delta | Meaning                    |
/// |-------------|--------------|-------|----------------------------|
/// | 0           | N/A          | N/A   | No event date / unknown    |
/// | 1           | game + 1 - 4 | -3    | Event 3 years before game  |
/// | 2           | game + 2 - 4 | -2    | Event 2 years before game  |
/// | 3           | game + 3 - 4 | -1    | Event 1 year before game   |
/// | 4           | game + 4 - 4 | 0     | Event same year as game    |
/// | 5           | game + 5 - 4 | +1    | Event 1 year after game    |
/// | 6           | game + 6 - 4 | +2    | Event 2 years after game   |
/// | 7           | game + 7 - 4 | +3    | Event 3 years after game   |
///
/// **Why -4?** The subtraction of 4 centers the 3-bit range (1-7) around zero,
/// giving a symmetric range of -3 to +3 years. Value 0 is reserved to indicate
/// "no event date" rather than a delta of -4.
///
/// **Why relative encoding?** Events (tournaments) almost always occur in the
/// same year as their games. Using relative encoding saves bits compared to
/// storing another full 11-bit year. The ±3 year range handles edge cases like:
/// - Multi-year tournaments spanning New Year
/// - Data entry errors
/// - Games from ongoing events entered later
///
fn parse_dates_field(dates_field: u32) -> (GameDate, Option<GameDate>) {
    // Game date (lower 20 bits, absolute encoding)
    let game_date_raw = dates_field & 0x000FFFFF;
    let game_date = GameDate {
        day: (game_date_raw & 0x1F) as u8,
        month: ((game_date_raw >> 5) & 0x0F) as u8,
        year: ((game_date_raw >> 9) & 0x7FF) as u16,
    };

    // Event date (upper 12 bits, relative encoding)
    let event_data = (dates_field >> 20) & 0xFFF;
    let event_date = if event_data == 0 {
        None // No event date stored
    } else {
        let day = (event_data & 0x1F) as u8;
        let month = ((event_data >> 5) & 0x0F) as u8;
        let year_offset = ((event_data >> 9) & 0x7) as i16;

        if year_offset == 0 {
            None // year_offset of 0 also means no event date
        } else {
            // Apply the year offset formula: event_year = game_year + offset - 4
            // This gives a range of -3 to +3 years relative to game date
            let event_year = (game_date.year as i16 + year_offset - 4) as u16;
            Some(GameDate { day, month, year: event_year })
        }
    };

    (game_date, event_date)
}
```

**Testing**:
- Parse all entries from `five.si4`
- Verify dates match expected (e.g., 2022.12.19)
- Verify IDs are reasonable
- Verify ELO ratings are in valid range
- Test packed field extraction accuracy

**Deliverable**: Complete index entry parsing with comprehensive tests.

---

## Phase 3: Name File Parser (Week 2)

### 3.1 SN4 Header Parsing

**Goal**: Parse name file header to determine name counts and frequencies.

**File**: `crates/core/src/database/names.rs`

**Implementation**:

```rust
const SN4_MAGIC: &[u8; 8] = b"Scid.sn\0";

pub struct Sn4Header {
    pub timestamp: u32,
    pub num_players: u32,
    pub num_events: u32,
    pub num_sites: u32,
    pub num_rounds: u32,
    pub max_freq_players: u32,
    pub max_freq_events: u32,
    pub max_freq_sites: u32,
    pub max_freq_rounds: u32,
}

pub fn parse_sn4_header(file: &mut File) -> Result<Sn4Header> {
    let mut header_bytes = [0u8; 36];
    file.read_exact(&mut header_bytes)?;

    if &header_bytes[0..8] != SN4_MAGIC {
        return Err(ScidError::InvalidFormat("Invalid SN4 magic".into()));
    }

    // Parse 24-bit counts (big-endian)
    let num_players = u32::from_be_bytes([0, header_bytes[12], header_bytes[13], header_bytes[14]]);
    let num_events = u32::from_be_bytes([0, header_bytes[15], header_bytes[16], header_bytes[17]]);
    let num_sites = u32::from_be_bytes([0, header_bytes[18], header_bytes[19], header_bytes[20]]);
    let num_rounds = u32::from_be_bytes([0, header_bytes[21], header_bytes[22], header_bytes[23]]);

    // Parse max frequencies
    let max_freq_players = u32::from_be_bytes([0, header_bytes[24], header_bytes[25], header_bytes[26]]);
    let max_freq_events = u32::from_be_bytes([0, header_bytes[27], header_bytes[28], header_bytes[29]]);
    let max_freq_sites = u32::from_be_bytes([0, header_bytes[30], header_bytes[31], header_bytes[32]]);
    let max_freq_rounds = u32::from_be_bytes([0, header_bytes[33], header_bytes[34], header_bytes[35]]);

    Ok(Sn4Header {
        timestamp: u32::from_be_bytes([header_bytes[8], header_bytes[9], header_bytes[10], header_bytes[11]]),
        num_players, num_events, num_sites, num_rounds,
        max_freq_players, max_freq_events, max_freq_sites, max_freq_rounds,
    })
}
```

**Testing**:
- Verify header parsing with real .sn4 file
- Validate counts are reasonable
- Test error handling

**Deliverable**: SN4 header parsing working.

---

### 3.2 Front-Coding Decompression

**Goal**: Implement front-coding algorithm to decompress name strings.

**File**: `crates/core/src/database/names.rs`

**Implementation** (most complex parsing so far):

```rust
pub struct NameDatabase {
    pub players: Vec<String>,
    pub events: Vec<String>,
    pub sites: Vec<String>,
    pub rounds: Vec<String>,
}

fn read_variable_int(reader: &mut BufReader<File>, max_value: u32) -> Result<(u32, usize)> {
    if max_value < 256 {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok((buf[0] as u32, 1))
    } else if max_value < 65536 {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok((u16::from_be_bytes(buf) as u32, 2))
    } else {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        Ok((u32::from_be_bytes([0, buf[0], buf[1], buf[2]]), 3))
    }
}

fn read_names_section(
    reader: &mut BufReader<File>,
    count: u32,
    max_frequency: u32,
) -> Result<Vec<String>> {
    let mut names = Vec::with_capacity(count as usize);
    let mut previous_name = String::new();

    for i in 0..count {
        // Read variable-length ID and frequency
        let (_name_id, _) = read_variable_int(reader, count)?;
        let (_frequency, _) = read_variable_int(reader, max_frequency)?;

        // Read total length
        let mut len_buf = [0u8; 1];
        reader.read_exact(&mut len_buf)?;
        let total_length = len_buf[0] as usize;

        // Read prefix length (not for first name)
        let prefix_length = if i == 0 {
            0
        } else {
            let mut prefix_buf = [0u8; 1];
            reader.read_exact(&mut prefix_buf)?;
            prefix_buf[0] as usize
        };

        // Read suffix
        let suffix_length = total_length - prefix_length;
        let mut suffix_bytes = vec![0u8; suffix_length];
        reader.read_exact(&mut suffix_bytes)?;

        // Reconstruct name using front-coding
        previous_name.truncate(prefix_length);
        previous_name.push_str(&String::from_utf8_lossy(&suffix_bytes));

        // Clean and store
        let clean_name = previous_name
            .chars()
            .filter(|&c| c >= ' ')
            .collect::<String>()
            .trim()
            .to_string();

        names.push(clean_name);
    }

    Ok(names)
}

pub fn parse_name_database(file: File) -> Result<NameDatabase> {
    let mut reader = BufReader::new(file);
    let header = parse_sn4_header_from_reader(&mut reader)?;

    // Read four sections in order
    let players = read_names_section(&mut reader, header.num_players, header.max_freq_players)?;
    let events = read_names_section(&mut reader, header.num_events, header.max_freq_events)?;
    let sites = read_names_section(&mut reader, header.num_sites, header.max_freq_sites)?;
    let rounds = read_names_section(&mut reader, header.num_rounds, header.max_freq_rounds)?;

    Ok(NameDatabase { players, events, sites, rounds })
}
```

**Testing**:
- Parse complete name database
- Verify player names match expected (e.g., "Hossain, Enam")
- Test front-coding reconstruction accuracy
- Test with various name lengths and special characters

**Deliverable**: Complete name database parsing.

---

### 3.2.1 Name ID Lookup and Edge Cases

**Goal**: Properly handle Name ID lookups including ID 0, empty names, and out-of-bounds IDs.

**Background**:

SCID Name IDs are 0-indexed into the name arrays. Key facts from the Bible:
- Names are stored sequentially starting at index 0
- "Empty names are allowed and stored as empty strings"
- IDs can be 0 to N-1 where N is the count for that name type

**Edge Cases**:

| ID Value | Meaning | PGN Output |
|----------|---------|------------|
| Valid ID pointing to non-empty name | Normal case | The name string |
| Valid ID pointing to empty string ("") | Unknown/unspecified | "?" |
| ID out of bounds (>= count) | Data corruption or error | "?" with warning |

**Implementation**:

```rust
/// NameDatabase with safe lookup methods
pub struct NameDatabase {
    pub players: Vec<String>,
    pub events: Vec<String>,
    pub sites: Vec<String>,
    pub rounds: Vec<String>,
}

impl NameDatabase {
    /// Get player name by ID, returning "?" for unknown/invalid
    ///
    /// # Name ID Semantics
    /// - ID is a 0-based index into the players array
    /// - Empty strings ("") represent unknown players → returns "?"
    /// - Out-of-bounds IDs → returns "?" (data may be corrupted)
    ///
    /// # PGN Standard
    /// PGN spec requires "?" for unknown values in the Seven Tag Roster
    pub fn get_player(&self, id: u32) -> &str {
        self.players
            .get(id as usize)
            .map(|s| if s.is_empty() { "?" } else { s.as_str() })
            .unwrap_or("?")
    }

    /// Get event name by ID
    pub fn get_event(&self, id: u32) -> &str {
        self.events
            .get(id as usize)
            .map(|s| if s.is_empty() { "?" } else { s.as_str() })
            .unwrap_or("?")
    }

    /// Get site name by ID
    pub fn get_site(&self, id: u32) -> &str {
        self.sites
            .get(id as usize)
            .map(|s| if s.is_empty() { "?" } else { s.as_str() })
            .unwrap_or("?")
    }

    /// Get round by ID
    ///
    /// Note: Round "?" is valid PGN for unknown round
    pub fn get_round(&self, id: u32) -> &str {
        self.rounds
            .get(id as usize)
            .map(|s| if s.is_empty() { "?" } else { s.as_str() })
            .unwrap_or("?")
    }

    /// Get player name with detailed error information
    /// Use when you need to distinguish between empty name and out-of-bounds
    pub fn get_player_detailed(&self, id: u32) -> NameLookupResult {
        match self.players.get(id as usize) {
            Some(name) if name.is_empty() => NameLookupResult::Empty,
            Some(name) => NameLookupResult::Found(name.as_str()),
            None => NameLookupResult::OutOfBounds(id),
        }
    }
}

/// Result of a name lookup operation
#[derive(Debug, Clone, PartialEq)]
pub enum NameLookupResult<'a> {
    /// Name found and non-empty
    Found(&'a str),
    /// ID valid but name is empty string (unknown)
    Empty,
    /// ID is out of bounds (possible data corruption)
    OutOfBounds(u32),
}

impl<'a> NameLookupResult<'a> {
    /// Convert to PGN-safe string
    pub fn to_pgn(&self) -> &str {
        match self {
            NameLookupResult::Found(name) => name,
            NameLookupResult::Empty => "?",
            NameLookupResult::OutOfBounds(_) => "?",
        }
    }

    /// Check if lookup indicates a potential data issue
    pub fn is_error(&self) -> bool {
        matches!(self, NameLookupResult::OutOfBounds(_))
    }
}
```

**Important Notes**:

1. **ID 0 is NOT special**: It's simply the first name in the array. If that name is "", it means unknown.

2. **Empty vs Missing**:
   - Empty string at valid index = intentionally unknown/unspecified
   - Out of bounds = data error, should log warning

3. **PGN Standard**: The Seven Tag Roster requires "?" for unknown values:
   - `[White "?"]` - Unknown white player
   - `[Event "?"]` - Unknown event
   - etc.

4. **Logging**: Consider logging out-of-bounds lookups as warnings for database validation.

---

### 3.2.2 Round String Formats

**Goal**: Document and handle various round string formats in SCID databases.

**Background**:

Rounds in SCID are stored as plain strings in the name database, just like players, events, and sites. They use front-coding compression and are sorted alphabetically. There is NO special encoding - the string is stored exactly as entered.

**Common Round Formats**:

| Format | Example | Meaning |
|--------|---------|---------|
| Simple number | `"1"`, `"2"`, `"15"` | Round number only |
| With sub-round | `"1.1"`, `"1.2"` | Round.Game format |
| Complex | `"1.2.3"` | Round.Board.Game (team events) |
| Unknown | `"?"` | Unknown round |
| Hyphen | `"-"` | No round / not applicable |
| Descriptive | `"Final"`, `"Semifinal"` | Named rounds |
| Empty string | `""` | Unknown (stored as empty) |

**PGN Standard Round Tag**:

From the PGN specification:
> The Round tag value gives the playing round ordinal of the game within the event.
> Some organizers employ unusual round designations such as "1.1", which usually
> means "Round 1, Game 1" in a multi-game match.

**Implementation**:

Rounds are already handled by `get_round()` from Phase 3.2.1. Additional considerations:

```rust
impl NameDatabase {
    /// Get round by ID with PGN-safe output
    ///
    /// Round strings are stored as-is in SCID. Common formats:
    /// - Simple: "1", "2", "15"
    /// - Sub-round: "1.1", "1.2" (Round.Game)
    /// - Complex: "1.2.3" (Round.Board.Game)
    /// - Special: "?", "-", "Final"
    /// - Empty: "" → returns "?"
    pub fn get_round(&self, id: u32) -> &str {
        self.rounds
            .get(id as usize)
            .map(|s| if s.is_empty() { "?" } else { s.as_str() })
            .unwrap_or("?")
    }

    /// Get round with hyphen normalization
    ///
    /// Some databases use "-" to mean unknown/no round.
    /// This method converts "-" to "?" for PGN consistency.
    pub fn get_round_normalized(&self, id: u32) -> &str {
        let round = self.get_round(id);
        if round == "-" { "?" } else { round }
    }
}
```

**PGN Output Escaping**:

Round strings may contain characters that need escaping in PGN:

```rust
/// Escape a string for PGN tag value output
///
/// PGN tag values are enclosed in double quotes.
/// - Backslash (\) must be escaped as \\
/// - Double quote (") must be escaped as \"
pub fn escape_pgn_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            _ => vec![c],
        })
        .collect()
}

// Usage in PGN output:
pgn.push_str(&format!("[Round \"{}\"]\n", escape_pgn_string(names.get_round(round_id))));
```

**Special Cases**:

1. **Hyphen ("-")**: Some databases use "-" for unknown rounds. Consider normalizing to "?" for consistency, or preserve as-is depending on user preference.

2. **Alphabetic rounds**: Tournament software may use "Final", "Semifinal", "QF1" (Quarterfinal 1), etc. These should be preserved as-is.

3. **Leading zeros**: Some databases store "01", "02" instead of "1", "2". Preserve as-is to maintain compatibility.

4. **Very long rounds**: PGN viewers may have display issues with very long round strings. No truncation is recommended - preserve original data.

**Testing**:
- Test simple round numbers: "1", "2", "15"
- Test sub-rounds: "1.1", "2.3"
- Test complex rounds: "1.2.3"
- Test special values: "?", "-", ""
- Test descriptive rounds: "Final", "Semifinal"
- Test PGN escaping with quotes/backslashes

**Deliverable**: Proper round string handling with format documentation.

---

**Testing** (for Phase 3.2.1):
- Test ID 0 lookup when first name is non-empty
- Test ID 0 lookup when first name is empty ("")
- Test out-of-bounds ID returns "?"
- Test normal lookups work correctly

**Deliverable**: Safe name lookup methods with proper edge case handling.

---

## Phase 4: Game File Structure (Week 3)

### 4.1 Game Boundary Detection & Decompression

**Goal**: Locate individual games within .sg4 file using index offsets, and decompress if needed.

**File**: `crates/core/src/database/games.rs`

**CRITICAL**: Most real SCID databases use zlib compression! Check `entry.is_packed()` and decompress before parsing.

**Dependencies**: `flate2 = "1.0"` crate for zlib decompression

**Implementation**:

```rust
use flate2::read::ZlibDecoder;
use std::io::Read;

pub struct GameData {
    pub tags: HashMap<String, String>,
    pub flags: u8,
    pub start_position: Option<String>, // FEN if non-standard (when START_FLAG set)
    pub move_data: Vec<u8>, // Raw move bytes (decoded by ScidMoveDecoder)
}

pub fn read_game_data(
    sg4_file: &mut File,
    offset: u32,
    length: u32,
) -> Result<Vec<u8>> {
    sg4_file.seek(SeekFrom::Start(offset as u64))?;

    let mut game_bytes = vec![0u8; length as usize];
    sg4_file.read_exact(&mut game_bytes)?;

    Ok(game_bytes)
}

/// Decompress zlib-compressed game data
pub fn decompress_game_data(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    Ok(decompressed)
}

/// Read and optionally decompress game data (recommended high-level function)
pub fn read_and_decompress_game(
    file: &mut File,
    entry: &GameIndexEntry,
) -> Result<Vec<u8>> {
    let raw_data = read_game_data(file, entry.game_offset, entry.game_length)?;
    if entry.is_packed() {
        decompress_game_data(&raw_data)
    } else {
        Ok(raw_data)
    }
}
```

**Testing**:
- Read game slices from `five.sg4` using index offsets
- Verify boundary detection
- Ensure no overlap or gaps between games

**Deliverable**: Game data extraction working.

---

### 4.2 Tag and Flags Parsing

**Goal**: Parse PGN tags and game flags from game data.

**File**: `crates/core/src/database/games.rs`

**Implementation**:

```rust
const MAX_TAG_LEN: u8 = 240;
const COMMON_TAGS: [&str; 10] = [
    "WhiteTitle", "BlackTitle", "Annotator", "Opening",
    "Variation", "SubVariation", "WhiteElo", "BlackElo",
    "WhiteUSCF", "BlackUSCF",
];

pub fn parse_game_tags(bytes: &[u8]) -> Result<(HashMap<String, String>, u8, usize)> {
    let mut tags = HashMap::new();
    let mut pos = 0;

    loop {
        if pos >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: pos as u64,
                message: "Unexpected end of tags".into(),
            });
        }

        let byte = bytes[pos];
        pos += 1;

        if byte == 0 {
            // Null terminator - tags end
            break;
        }

        if byte == 255 {
            // Special 3-byte EventDate encoding
            pos += 3;
        } else if byte > MAX_TAG_LEN {
            // Common tag
            let tag_index = (byte - MAX_TAG_LEN - 1) as usize;
            let tag_name = COMMON_TAGS.get(tag_index)
                .ok_or_else(|| ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Unknown common tag: {}", byte),
                })?;

            let value_len = bytes[pos] as usize;
            pos += 1;
            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(tag_name.to_string(), value);
        } else {
            // Regular tag
            let name_len = byte as usize;
            let name = String::from_utf8_lossy(&bytes[pos..pos + name_len]).to_string();
            pos += name_len;

            let value_len = bytes[pos] as usize;
            pos += 1;
            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(name, value);
        }
    }

    // Read flags byte
    let flags = bytes[pos];
    pos += 1;

    Ok((tags, flags, pos))
}

/// Parse complete game structure including optional FEN for non-standard starts
///
/// Game data layout:
/// 1. PGN tags (parsed by parse_game_tags)
/// 2. Flags byte
/// 3. Optional FEN string (if flags & START_FLAG, null-terminated)
/// 4. Move data (remaining bytes)
///
/// IMPORTANT: When START_FLAG (bit 0) is set, the FEN string appears
/// AFTER the flags byte and BEFORE the move data. This is used for:
/// - Chess960 games with shuffled starting positions
/// - Games starting from specific positions (studies, puzzles)
/// - Continuation games
pub fn parse_game_structure(bytes: &[u8]) -> Result<GameData> {
    let (tags, flags, mut pos) = parse_game_tags(bytes)?;

    // Check for non-standard start position (Chess960, custom FEN, etc.)
    let start_position = if flags & game_flags::START_FLAG as u8 != 0 {
        // Read null-terminated FEN string
        let fen_start = pos;
        while pos < bytes.len() && bytes[pos] != 0 {
            pos += 1;
        }

        if pos >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::new(),
                offset: fen_start as u64,
                message: "FEN string not null-terminated".to_string(),
            });
        }

        let fen = String::from_utf8_lossy(&bytes[fen_start..pos]).to_string();
        pos += 1; // Skip null terminator

        // Validate FEN has required components (at minimum: piece placement)
        if fen.is_empty() || !fen.contains('/') {
            return Err(ScidError::ParseError {
                file: PathBuf::new(),
                offset: fen_start as u64,
                message: format!("Invalid FEN string: {}", fen),
            });
        }

        Some(fen)
    } else {
        None
    };

    // Remaining bytes are move data
    let move_data = bytes[pos..].to_vec();

    Ok(GameData {
        tags,
        flags,
        start_position,
        move_data,
    })
}
```

**Testing**:
- Parse tags from real game data
- Verify common vs. regular tag handling
- Test null terminator detection
- Test FEN parsing when START_FLAG is set
- Verify Chess960 positions parse correctly

**Deliverable**: Tag and game structure parsing working correctly.

---

### 4.3 Game Data Structure & Comment Encoding

**Goal**: Understand the complete game data layout, especially the two-part comment encoding.

**File**: `crates/core/src/database/games.rs`

#### Special Byte Constants

**CRITICAL**: Define these constants for move data parsing:

```rust
/// Special marker bytes in SCID move data
/// From SCID source game.h lines 88-94
pub mod special_bytes {
    /// End of game marker (alternative)
    pub const END_GAME_ALT: u8 = 0x00;

    /// NAG (Numeric Annotation Glyph) marker
    /// Followed by 1 byte containing the NAG value (1-255)
    pub const ENCODE_NAG: u8 = 0x0B;

    /// Comment marker - indicates a comment exists at this position
    /// The actual comment text is stored at the END of game data
    pub const ENCODE_COMMENT: u8 = 0x0C;

    /// Start variation marker - begins a new variation branch
    pub const ENCODE_START_MARKER: u8 = 0x0D;

    /// End variation marker - returns to parent variation
    pub const ENCODE_END_MARKER: u8 = 0x0E;

    /// End of game marker (primary)
    pub const ENCODE_END_GAME: u8 = 0x0F;
}
```

**CRITICAL: Game Data Structure**:

```
┌─────────────────────────────────────────────────────────────┐
│ 1. PGN Tags (null-terminated)                               │
│ 2. Flags byte                                               │
│ 3. Optional FEN (if flags & 0x01, null-terminated string)   │
│ 4. Move data with MARKERS ONLY:                             │
│    - 0x00 = End of game (alternative marker)                │
│    - 0x0B = NAG marker (followed by 1 NAG byte)             │
│    - 0x0C = Comment marker (NO text here!)                  │
│    - 0x0D = Start variation marker                          │
│    - 0x0E = End variation marker                            │
│    - 0x0F = End of game marker (primary)                    │
│    - All other bytes (0x10-0xFF) are move encodings         │
│ 5. Comments section (null-terminated strings in tree order) │
└─────────────────────────────────────────────────────────────┘
```

**Two-Part Comment Encoding**:

SCID uses a two-part system for comments:

1. **In move data**: Only a single marker byte `0x0C` appears
2. **At end of game data**: Actual comment text as null-terminated strings

#### Comment Association Algorithm

**CRITICAL**: Comments are stored in **pre-order DFS traversal order** of the move tree.

```rust
/// Tracks comment markers during move parsing
#[derive(Debug)]
pub struct MoveNode {
    pub chess_move: Option<Move>,  // None for root
    pub has_comment: bool,         // True if 0x0C marker found
    pub comment: Option<String>,   // Filled in second pass
    pub nags: Vec<u8>,             // NAG values
    pub variations: Vec<MoveTree>, // Child variations
}

pub type MoveTree = Vec<MoveNode>;

/// State saved when entering a variation
///
/// When a variation starts, we must save:
/// 1. The tree pointer (to track where to add moves)
/// 2. The chess position (to restore after variation ends)
struct VariationState {
    tree_ptr: *mut MoveTree,
    position: ScidPosition,
}

/// Result of parsing move data
struct ParseMoveDataResult {
    tree: MoveTree,
    has_pre_game_comment: bool,  // True if comment marker appeared before first move
    comment_section_start: usize,
}

/// Phase 1: Parse move data, tracking comment markers and position
///
/// CRITICAL: Position must be cloned when entering variations and
/// restored when exiting. Without this, moves after variations will
/// be decoded relative to the wrong position.
fn parse_move_data(bytes: &[u8], start_position: &ScidPosition) -> Result<ParseMoveDataResult> {
    use special_bytes::*;

    let mut tree = MoveTree::new();
    let mut position = start_position.clone();
    let mut variation_stack: Vec<VariationState> = vec![VariationState {
        tree_ptr: &mut tree as *mut _,
        position: position.clone(),
    }];
    let mut pos = 0;
    let mut has_pre_game_comment = false;

    while pos < bytes.len() {
        let byte = bytes[pos];
        pos += 1;

        match byte {
            END_GAME_ALT | ENCODE_END_GAME => {
                // End of move data - return position for comment reading
                return Ok(ParseMoveDataResult {
                    tree,
                    has_pre_game_comment,
                    comment_section_start: pos,
                });
            }
            ENCODE_COMMENT => {
                // Mark that a comment exists at the current position
                // Do NOT read any text here - comments are at END of game data
                if let Some(current_state) = variation_stack.last() {
                    if let Some(last_node) = unsafe { (*current_state.tree_ptr).last_mut() } {
                        // Comment on a move
                        last_node.has_comment = true;
                    } else {
                        // No moves yet - this is a PRE-GAME comment
                        // (comment on the starting position before any moves)
                        has_pre_game_comment = true;
                    }
                }
            }
            ENCODE_NAG => {
                // Read NAG value byte
                if pos < bytes.len() {
                    let nag_value = bytes[pos];
                    pos += 1;
                    if let Some(current_state) = variation_stack.last() {
                        if let Some(last_node) = unsafe { (*current_state.tree_ptr).last_mut() } {
                            last_node.nags.push(nag_value);
                        }
                    }
                }
            }
            ENCODE_START_MARKER => {
                // Start new variation - save position and push onto stack
                //
                // CRITICAL: Clone the current position BEFORE entering the variation.
                // The variation's moves will modify position, but when we exit,
                // we need to restore to continue the main line.
                if let Some(current_state) = variation_stack.last() {
                    if let Some(last_node) = unsafe { (*current_state.tree_ptr).last_mut() } {
                        last_node.variations.push(MoveTree::new());
                        let new_var = last_node.variations.last_mut().unwrap();
                        variation_stack.push(VariationState {
                            tree_ptr: new_var as *mut _,
                            position: position.clone(),  // Save position before variation
                        });
                    }
                }
            }
            ENCODE_END_MARKER => {
                // End variation - restore position and pop from stack
                //
                // CRITICAL: Restore position to what it was before the variation.
                // This ensures moves after the variation are decoded correctly.
                if variation_stack.len() > 1 {
                    if let Some(state) = variation_stack.pop() {
                        position = state.position;  // Restore position after variation
                    }
                }
            }
            _ => {
                // Regular move byte - decode and add to current tree
                // (Move decoding handled by ScidMoveDecoder)
            }
        }
    }

    Ok(ParseMoveDataResult {
        tree,
        has_pre_game_comment,
        comment_section_start: pos,
    })
}

/// Character encoding for comment text
///
/// Older SCID databases (pre-2010) often used Latin-1 (ISO-8859-1) encoding,
/// while modern databases use UTF-8. This enum allows handling both.
#[derive(Debug, Clone, Copy, Default)]
pub enum CommentEncoding {
    /// Assume UTF-8 encoding (modern databases)
    /// Invalid UTF-8 sequences are replaced with U+FFFD
    #[default]
    Utf8,

    /// Assume Latin-1 (ISO-8859-1) encoding (older databases)
    /// Every byte maps directly to a Unicode code point
    Latin1,

    /// Auto-detect: try UTF-8 first, fall back to Latin-1 if invalid
    /// This is the safest option for unknown databases
    Auto,
}

/// Decode bytes to string using the specified encoding
fn decode_comment(bytes: &[u8], encoding: CommentEncoding) -> String {
    match encoding {
        CommentEncoding::Utf8 => {
            String::from_utf8_lossy(bytes).to_string()
        }
        CommentEncoding::Latin1 => {
            // Latin-1: each byte maps directly to Unicode code point 0-255
            bytes.iter().map(|&b| b as char).collect()
        }
        CommentEncoding::Auto => {
            // Try UTF-8 first
            match std::str::from_utf8(bytes) {
                Ok(s) => s.to_string(),
                Err(_) => {
                    // Fall back to Latin-1
                    bytes.iter().map(|&b| b as char).collect()
                }
            }
        }
    }
}

/// Phase 2: Read comments and associate with nodes in DFS order
///
/// IMPORTANT: If has_pre_game_comment is true, the FIRST comment in the
/// comment section belongs to the starting position, not the first move.
fn read_comments(
    bytes: &[u8],
    tree: &mut MoveTree,
    has_pre_game_comment: bool,
    encoding: CommentEncoding,
) -> Result<Option<String>> {
    let mut comment_pos = 0;
    let mut pre_game_comment = None;

    // Read pre-game comment FIRST if present
    // This comment appears before any moves in the game
    if has_pre_game_comment {
        let start = comment_pos;
        while comment_pos < bytes.len() && bytes[comment_pos] != 0 {
            comment_pos += 1;
        }
        pre_game_comment = Some(decode_comment(&bytes[start..comment_pos], encoding));
        comment_pos += 1; // Skip null terminator
    }

    // Pre-order DFS traversal - same order SCID writes comments
    // Note: We need to pass encoding through, so we use a closure that captures it
    fn visit_node(
        node: &mut MoveNode,
        bytes: &[u8],
        pos: &mut usize,
        encoding: CommentEncoding,
    ) -> Result<()> {
        // Process this node first (pre-order)
        if node.has_comment {
            // Read null-terminated string
            let start = *pos;
            while *pos < bytes.len() && bytes[*pos] != 0 {
                *pos += 1;
            }
            node.comment = Some(decode_comment(&bytes[start..*pos], encoding));
            *pos += 1; // Skip null terminator
        }

        // Then visit all variations (children)
        for variation in &mut node.variations {
            for child_node in variation {
                visit_node(child_node, bytes, pos, encoding)?;
            }
        }
        Ok(())
    }

    for node in tree {
        visit_node(node, bytes, &mut comment_pos, encoding)?;
    }

    Ok(pre_game_comment)
}
```

**Why Two-Part?**: This design separates the variable-length comment text from the compact move encoding, making the move data section more efficient to parse. The move data has predictable byte boundaries, while comments are variable-length strings.

#### NAG (Numeric Annotation Glyph) Handling

NAGs are standard chess annotation symbols encoded as single bytes (1-255).

**Common NAG Values**:
| NAG | Symbol | Meaning |
|-----|--------|---------|
| 1 | ! | Good move |
| 2 | ? | Mistake |
| 3 | !! | Brilliant move |
| 4 | ?? | Blunder |
| 5 | !? | Interesting move |
| 6 | ?! | Dubious move |
| 10 | = | Equal position |
| 13 | ∞ | Unclear position |
| 14 | += | Slight advantage White |
| 15 | =+ | Slight advantage Black |
| 16 | ± | Clear advantage White |
| 17 | ∓ | Clear advantage Black |
| 18 | +- | Winning for White |
| 19 | -+ | Winning for Black |

**PGN Output for NAGs**:
```rust
fn format_nags(nags: &[u8]) -> String {
    let mut result = String::new();
    for &nag in nags {
        // Standard NAGs 1-6 can use symbols
        match nag {
            1 => result.push('!'),
            2 => result.push('?'),
            3 => result.push_str("!!"),
            4 => result.push_str("??"),
            5 => result.push_str("!?"),
            6 => result.push_str("?!"),
            // All others use $N notation
            _ => result.push_str(&format!("${}", nag)),
        }
    }
    result
}
```

#### Variation Data Structures

**File**: `crates/core/src/parser/variation.rs`

```rust
/// Complete game representation with variations
#[derive(Debug, Clone)]
pub struct GameTree {
    /// Starting position (standard or FEN)
    pub start_fen: Option<String>,

    /// Comment before the first move (attached to starting position)
    /// SCID allows a comment marker before any moves are encoded.
    /// In PGN, this appears as: { Pre-game comment } 1. e4 ...
    pub pre_game_comment: Option<String>,

    /// Root of the move tree (main line + variations)
    pub root: MoveNode,
}

/// Single node in move tree
#[derive(Debug, Clone)]
pub struct MoveNode {
    /// The chess move (None for root node before first move)
    pub chess_move: Option<Move>,

    /// Comment attached to this move
    pub comment: Option<String>,

    /// NAG annotations
    pub nags: Vec<u8>,

    /// Continuation (next move in this line)
    pub continuation: Option<Box<MoveNode>>,

    /// Alternative variations starting from this position
    pub variations: Vec<MoveNode>,
}

impl MoveNode {
    /// Create root node
    pub fn root() -> Self {
        MoveNode {
            chess_move: None,
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }
    }

    /// Add a move as continuation
    pub fn add_move(&mut self, chess_move: Move) -> &mut MoveNode {
        self.continuation = Some(Box::new(MoveNode {
            chess_move: Some(chess_move),
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }));
        self.continuation.as_mut().unwrap()
    }

    /// Add a variation
    pub fn add_variation(&mut self, first_move: Move) -> &mut MoveNode {
        self.variations.push(MoveNode {
            chess_move: Some(first_move),
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        });
        self.variations.last_mut().unwrap()
    }
}
```

**Deliverable**: Complete game data parsing with proper comment, NAG, and variation handling.

---

## Phase 5: Chess Position Tracking with Shakmaty (Week 3-4)

### 5.1 Position Wrapper with SCID Piece Tracking

**Goal**: Wrap shakmaty's Chess position with SCID piece number tracking.

**File**: `crates/core/src/parser/position.rs`

**Implementation**:

```rust
use shakmaty::{Chess, Position, Square, Move, Role, Color};
use std::collections::HashMap;

/// Wrapper around shakmaty::Chess that tracks SCID piece numbers
///
/// Implements Clone to support variation parsing - position must be
/// saved before entering a variation and restored after exiting.
#[derive(Clone)]
pub struct ScidPosition {
    // Shakmaty handles all chess logic
    chess: Chess,

    // SCID piece numbers (0-15) to current squares - one per color
    // Updated as moves are made
    // CRITICAL: Each color has its own independent 0-15 piece list
    white_pieces: HashMap<u8, Square>,
    black_pieces: HashMap<u8, Square>,

    // Track piece count per color (for capture swap algorithm)
    white_count: u8,
    black_count: u8,
}

impl ScidPosition {
    pub fn new() -> Self {
        let chess = Chess::default(); // Standard starting position
        let mut white_pieces = HashMap::new();
        let mut black_pieces = HashMap::new();

        // Initialize SCID piece number mappings for starting position
        // Per SCID source Position::StdStart():
        // King=0, QR=1 (A1), QN=2 (B1), QB=3 (C1), Q=4 (D1), KB=5 (F1), KN=6 (G1), KR=7 (H1)
        // Pawns=8-15 (a-pawn to h-pawn)
        // Each color has separate 0-15 list; piece numbers are per-color
        Self::init_piece_mappings(&mut white_pieces, &mut black_pieces, &chess);

        ScidPosition {
            chess,
            white_pieces,
            black_pieces,
            white_count: 16,  // Standard start: 16 pieces each
            black_count: 16,
        }
    }

    /// Initialize piece mappings for BOTH colors
    /// CRITICAL: Each color has its own 0-15 piece list
    fn init_piece_mappings(
        white_pieces: &mut HashMap<u8, Square>,
        black_pieces: &mut HashMap<u8, Square>,
        _chess: &Chess
    ) {
        // SCID piece numbering is by STARTING FILE, not piece type!
        // Order: King(0), QR(1), QN(2), QB(3), Q(4), KB(5), KN(6), KR(7), Pawns(8-15)
        // From SCID source Position::StdStart() lines 77808-77858

        // White pieces (List[WHITE]):
        white_pieces.insert(0, Square::E1);  // King (ALWAYS piece 0)
        white_pieces.insert(1, Square::A1);  // Queen's Rook (QR) - a-file
        white_pieces.insert(2, Square::B1);  // Queen's Knight (QN) - b-file
        white_pieces.insert(3, Square::C1);  // Queen's Bishop (QB) - c-file
        white_pieces.insert(4, Square::D1);  // Queen - d-file
        white_pieces.insert(5, Square::F1);  // King's Bishop (KB) - f-file
        white_pieces.insert(6, Square::G1);  // King's Knight (KN) - g-file
        white_pieces.insert(7, Square::H1);  // King's Rook (KR) - h-file
        // White pawns 8-15 (a2-h2, files 0-7)
        for file in 0..8u8 {
            white_pieces.insert(8 + file, Square::new(8 + file)); // A2=8, B2=9, ..., H2=15
        }

        // Black pieces (List[BLACK]) - same numbering scheme:
        black_pieces.insert(0, Square::E8);  // King (ALWAYS piece 0)
        black_pieces.insert(1, Square::A8);  // Queen's Rook (QR) - a-file
        black_pieces.insert(2, Square::B8);  // Queen's Knight (QN) - b-file
        black_pieces.insert(3, Square::C8);  // Queen's Bishop (QB) - c-file
        black_pieces.insert(4, Square::D8);  // Queen - d-file
        black_pieces.insert(5, Square::F8);  // King's Bishop (KB) - f-file
        black_pieces.insert(6, Square::G8);  // King's Knight (KN) - g-file
        black_pieces.insert(7, Square::H8);  // King's Rook (KR) - h-file
        // Black pawns 8-15 (a7-h7, files 0-7)
        for file in 0..8u8 {
            black_pieces.insert(8 + file, Square::new(48 + file)); // A7=48, B7=49, ..., H7=55
        }
    }

    /// Get square for piece number (for the side to move)
    pub fn get_piece_square(&self, piece_num: u8) -> Option<Square> {
        match self.chess.turn() {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    /// Get square for piece number of specific color
    pub fn get_piece_square_for_color(&self, piece_num: u8, color: Color) -> Option<Square> {
        match color {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    pub fn board(&self) -> &shakmaty::Board {
        self.chess.board()
    }

    pub fn turn(&self) -> Color {
        self.chess.turn()
    }

    pub fn make_move(&mut self, chess_move: &Move) -> Result<()> {
        // Validate move is legal using shakmaty
        if !self.chess.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        // Apply move using shakmaty (handles all chess rules)
        self.chess = self.chess.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        // Update piece location tracking
        self.update_piece_locations(chess_move);

        Ok(())
    }

    /// Update piece locations after a move
    /// CRITICAL: Implements SCID Capture Swap Algorithm
    fn update_piece_locations(&mut self, chess_move: &Move) {
        // SCID Capture Swap Algorithm (from Position::DoSimpleMove lines 78903-78912):
        //   if (sm->capturedPiece != EMPTY) {
        //       sm->capturedNum = ListPos[sm->capturedSquare];
        //       Count[enemy]--;
        //       ListPos[List[enemy][Count[enemy]]] = sm->capturedNum;
        //       List[enemy][sm->capturedNum] = List[enemy][Count[enemy]];
        //   }

        let moving_color = self.chess.turn();

        // Handle captures FIRST (before updating moving piece)
        if chess_move.is_capture() {
            // Determine capture square (different for en passant!)
            let capture_square = if chess_move.is_en_passant() {
                // En passant: captured pawn is NOT on the target square
                // It's on the same file as target, but same rank as moving pawn
                let target = chess_move.to();
                let ep_rank = match moving_color {
                    Color::White => shakmaty::Rank::Fifth,   // Capturing on rank 6, pawn was on rank 5
                    Color::Black => shakmaty::Rank::Fourth,  // Capturing on rank 3, pawn was on rank 4
                };
                Square::from_coords(target.file(), ep_rank)
            } else {
                chess_move.to()
            };

            // Get enemy pieces
            let (enemy_pieces, enemy_count) = match moving_color {
                Color::White => (&mut self.black_pieces, &mut self.black_count),
                Color::Black => (&mut self.white_pieces, &mut self.white_count),
            };

            // Find the captured piece's slot number
            let captured_slot = enemy_pieces.iter()
                .find(|(_, &sq)| sq == capture_square)
                .map(|(&num, _)| num);

            if let Some(captured_num) = captured_slot {
                // SCID SWAP ALGORITHM:
                // 1. Decrement enemy piece count
                *enemy_count -= 1;

                // 2. Get the last piece slot (index = new count)
                let last_slot = *enemy_count;

                // 3. If captured piece wasn't the last, swap
                if captured_num != last_slot {
                    // Move last piece to captured slot
                    if let Some(&last_square) = enemy_pieces.get(&last_slot) {
                        enemy_pieces.insert(captured_num, last_square);
                    }
                }

                // 4. Remove the last slot
                enemy_pieces.remove(&last_slot);
            }
        }

        // Get own pieces for updating moving piece location
        let own_pieces = match moving_color {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces,
        };

        // Update moving piece location
        match chess_move {
            Move::Normal { from, to, .. } => {
                // Find which piece number is at from square
                let piece_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
                // Note: Promotion doesn't change piece NUMBER, only type
            }
            Move::Castle { king, rook } => {
                // King moves to standard castling square
                let king_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::G, king.rank())  // Kingside
                } else {
                    Square::from_coords(shakmaty::File::C, king.rank())  // Queenside
                };

                // Rook moves to standard castling square
                let rook_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::F, king.rank())  // Kingside
                } else {
                    Square::from_coords(shakmaty::File::D, king.rank())  // Queenside
                };

                // Update king (always piece 0)
                own_pieces.insert(0, king_to);

                // Find and update rook
                let rook_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *rook)
                    .map(|(&num, _)| num);

                if let Some(num) = rook_num {
                    own_pieces.insert(num, rook_to);
                }
            }
            Move::EnPassant { from, to } => {
                let piece_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
            }
            Move::Put { .. } => {
                // Crazyhouse - not supported in standard SCID
            }
        }
    }

    /// Initialize from FEN position
    /// CRITICAL: King is ALWAYS piece #0 - swap if needed
    pub fn from_fen(fen: &str) -> Result<Self> {
        let chess = fen.parse::<Chess>()
            .map_err(|e| ScidError::InvalidFormat(format!("Invalid FEN: {}", e)))?;

        let mut white_pieces = HashMap::new();
        let mut black_pieces = HashMap::new();

        // Initialize piece mappings for non-standard position
        let (white_count, black_count) = Self::init_fen_piece_mappings(
            &mut white_pieces,
            &mut black_pieces,
            &chess
        );

        Ok(ScidPosition {
            chess,
            white_pieces,
            black_pieces,
            white_count,
            black_count,
        })
    }

    /// Initialize piece mappings from FEN position
    /// CRITICAL: King is ALWAYS piece #0 - assigned first before any other pieces
    /// From SCID source position.cpp AddPiece()
    fn init_fen_piece_mappings(
        white_pieces: &mut HashMap<u8, Square>,
        black_pieces: &mut HashMap<u8, Square>,
        chess: &Chess,
    ) -> (u8, u8) {
        // STEP 1: Find and assign Kings FIRST (always slot 0)
        for square in chess.board().by_role(Role::King) {
            if let Some(piece) = chess.board().piece_at(square) {
                if piece.color == Color::White {
                    white_pieces.insert(0, square);
                } else {
                    black_pieces.insert(0, square);
                }
            }
        }

        // STEP 2: Scan board in FEN order (rank 8 to 1, file a to h)
        // and assign remaining pieces starting at slot 1
        let mut white_num = 1u8;
        let mut black_num = 1u8;

        // Scan from rank 8 (index 7) down to rank 1 (index 0)
        for rank in (0..8u32).rev() {
            for file in 0..8u32 {
                let square = Square::from_coords(
                    shakmaty::File::new(file),
                    shakmaty::Rank::new(rank)
                );

                if let Some(piece) = chess.board().piece_at(square) {
                    // Skip Kings - already assigned to slot 0
                    if piece.role == Role::King {
                        continue;
                    }

                    if piece.color == Color::White {
                        white_pieces.insert(white_num, square);
                        white_num += 1;
                    } else {
                        black_pieces.insert(black_num, square);
                        black_num += 1;
                    }
                }
            }
        }

        // Return piece counts (white_num and black_num are now 1 + actual count)
        (white_num, black_num)
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        // Use shakmaty's legal move generation
        let mut moves = Vec::new();
        self.chess.legal_moves(&mut moves);
        moves
    }

    pub fn find_move(&self, from: Square, to: Square, promotion: Option<Role>) -> Option<Move> {
        // Find legal move matching from/to squares
        // Used to convert SCID decoded moves to shakmaty Moves
        let legals = self.legal_moves();
        legals.into_iter().find(|m| {
            m.from() == Some(from) && m.to() == to &&
            m.promotion() == promotion
        })
    }
}
```

**Testing**:
- Test starting position setup
- Test piece tracking updates correctly after moves
- Test legal move validation via shakmaty
- Test special moves (castling, en passant, promotions) handled by shakmaty

**Deliverable**: Position wrapper with SCID piece tracking leveraging shakmaty.

---

### 5.1.1 Chess960 (Fischer Random Chess) Support

**Goal**: Support Chess960 variant games through FEN-based position detection.

**Background**:

Chess960 (also known as Fischer Random Chess or FRC) differs from standard chess:
- 960 possible starting positions with pieces shuffled on back rank
- King always between the two rooks
- Bishops on opposite colors
- Castling rules modified: King and Rook end on standard squares (c1/g1, d1/f1) but start from non-standard positions

**How SCID Handles Chess960**:

1. **Starting Position**: Chess960 games use the `NonStandardStart` flag (bit 0 in game flags) with a FEN specifying the shuffled position
2. **Move Encoding**: Same binary encoding as standard chess
3. **Castling**: Move values 9 (O-O-O) and 10 (O-O) still indicate queenside/kingside castle

**Detection Strategy**:

Chess960 is detected by examining the FEN castling rights. Standard chess uses `KQkq` while Chess960 uses file letters like `HAha` (indicating which file each rook is on):

```rust
/// Detect if a FEN represents a Chess960 position
fn is_chess960_fen(fen: &str) -> bool {
    // Chess960 FENs use file letters (A-H/a-h) for castling rights
    // instead of standard KQkq notation
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 3 {
        return false;
    }

    let castling = parts[2];
    if castling == "-" {
        // No castling rights - could be either variant
        // Check if King is on e-file
        return !is_king_on_e_file(parts[0]);
    }

    // Chess960 uses file letters A-H for castling (not K/Q)
    // Example: "HAha" means rooks on a-file and h-file for both colors
    castling.chars().any(|c| matches!(c, 'A'..='H' | 'a'..='h'))
}

/// Check if Kings are on standard e-file starting position
fn is_king_on_e_file(board_fen: &str) -> bool {
    // In standard chess, Kings start on e-file
    // If King is NOT on e-file, likely Chess960
    // This is a heuristic - FEN parsing is more reliable
    true // Simplified - full implementation checks board FEN
}
```

**Shakmaty Integration**:

Shakmaty provides separate types for Chess960:

```rust
use shakmaty::{Chess, CastlingMode};
use shakmaty::fen::Fen;

/// Position type that handles both standard and Chess960
pub enum ChessPosition {
    Standard(Chess),
    Chess960(Chess),  // Same type, different castling mode
}

impl ScidPosition {
    /// Create from FEN, auto-detecting Chess960
    pub fn from_fen_auto(fen: &str) -> Result<Self> {
        let is_960 = is_chess960_fen(fen);

        // Parse with appropriate castling mode
        let parsed: Fen = fen.parse()
            .map_err(|e| ScidError::InvalidFormat(format!("Invalid FEN: {}", e)))?;

        let castling_mode = if is_960 {
            CastlingMode::Chess960
        } else {
            CastlingMode::Standard
        };

        let chess: Chess = parsed.into_position(castling_mode)
            .map_err(|e| ScidError::InvalidFormat(format!("Invalid position: {}", e)))?;

        // Rest of initialization same as from_fen()
        let mut white_pieces = HashMap::new();
        let mut black_pieces = HashMap::new();

        let (white_count, black_count) = Self::init_fen_piece_mappings(
            &mut white_pieces,
            &mut black_pieces,
            &chess
        );

        Ok(ScidPosition {
            chess,
            white_pieces,
            black_pieces,
            white_count,
            black_count,
        })
    }
}
```

**Castling in Chess960**:

SCID's castling encoding works for Chess960 because:
- Move value 9 = "castle queenside" (King ends on c-file, Rook on d-file)
- Move value 10 = "castle kingside" (King ends on g-file, Rook on f-file)
- The DESTINATION squares are always standard, only START positions vary

```rust
fn decode_king_move_chess960(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
    match move_value {
        // ... standard moves 1-8 ...
        9 => {
            // Queenside castle - King ALWAYS ends on c-file
            let to = if self.position.turn() == Color::White {
                Square::C1
            } else {
                Square::C8
            };
            Ok((to, None))
        }
        10 => {
            // Kingside castle - King ALWAYS ends on g-file
            let to = if self.position.turn() == Color::White {
                Square::G1
            } else {
                Square::G8
            };
            Ok((to, None))
        }
        // ...
    }
}
```

**Important**: The current implementation's hardcoded castling squares (C1/G1, C8/G8) are CORRECT for both standard chess AND Chess960 because the destination is always the same.

**PGN Output for Chess960**:

When exporting Chess960 games to PGN:
- Include `[Variant "Chess960"]` tag
- Include `[SetUp "1"]` and `[FEN "..."]` tags
- Castling notation: Some tools use O-O/O-O-O, others use King-captures-Rook notation

```rust
/// Format PGN headers for Chess960 game
fn format_chess960_headers(fen: &str) -> String {
    format!(
        "[Variant \"Chess960\"]\n[SetUp \"1\"]\n[FEN \"{}\"]\n",
        fen
    )
}
```

**Testing**:
- Test Chess960 FEN detection
- Test castling decoding with non-standard King positions
- Test piece numbering with shuffled back rank
- Test PGN output with Variant tag

**Limitations**:
- No automatic Chess960 position number calculation (would require reverse-engineering starting position)
- Assumes shakmaty correctly validates Chess960 castling legality

**Deliverable**: Chess960 support through FEN detection and proper shakmaty castling mode.

---

### 5.2 Move Decoder (SCID to Shakmaty)

**Goal**: Decode SCID binary move encoding to shakmaty Move types.

**File**: `crates/core/src/parser/decoder.rs`

**Implementation** (decodes SCID format, outputs shakmaty Moves):

```rust
use shakmaty::{Square, Move, Role};

pub struct ScidMoveDecoder {
    position: ScidPosition,
}

impl ScidMoveDecoder {
    pub fn new() -> Self {
        ScidMoveDecoder {
            position: ScidPosition::new(),
        }
    }

    pub fn decode_move(&mut self, move_byte: u8, stream: &mut ByteStream) -> Result<Move> {
        let piece_num = (move_byte >> 4) & 0x0F;
        let move_value = move_byte & 0x0F;

        // Get piece location from SCID tracking
        let from_square = self.position.get_piece_square(piece_num)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid piece number: {}", piece_num),
            })?;

        // Get piece at that square from shakmaty
        let piece = self.position.board().piece_at(from_square)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("No piece at square: {:?}", from_square),
            })?;

        // Decode SCID move value to target square
        let (to_square, promotion) = match piece.role {
            Role::King => self.decode_king_move(from_square, move_value)?,
            Role::Queen => self.decode_queen_move(from_square, move_value, stream)?,
            Role::Rook => self.decode_rook_move(from_square, move_value)?,
            Role::Bishop => self.decode_bishop_move(from_square, move_value)?,
            Role::Knight => self.decode_knight_move(from_square, move_value)?,
            Role::Pawn => self.decode_pawn_move(from_square, move_value)?,
        };

        // Use shakmaty to find the legal move matching our from/to/promotion
        let chess_move = self.position.find_move(from_square, to_square, promotion)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("No legal move from {:?} to {:?}", from_square, to_square),
            })?;

        // Apply move using shakmaty (validates and updates position)
        self.position.make_move(&chess_move)?;

        Ok(chess_move)
    }

    fn decode_king_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        // From SCID decodeKing() - sqdiff array and castling values:
        // static const int sqdiff[] = { 0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2 };
        // Values 9 and 10 are castling (NOT 10 and 11!)
        // Value 0 is NULL MOVE (valid, King stays in place)
        let to = match move_value {
            0 => {
                // Null move - King stays on same square (used in analysis)
                from
            }
            1..=8 => {
                // Adjacent square move using sqdiff lookup
                let sqdiff: [i8; 9] = [-9, -8, -7, -1, 1, 7, 8, 9];
                let offset = sqdiff[(move_value - 1) as usize];
                self.offset_square(from, offset)?
            }
            9 => {
                // Queenside castle (O-O-O) - sqdiff = -2
                if self.position.turn() == Color::White {
                    Square::C1
                } else {
                    Square::C8
                }
            }
            10 => {
                // Kingside castle (O-O) - sqdiff = +2
                if self.position.turn() == Color::White {
                    Square::G1
                } else {
                    Square::G8
                }
            }
            _ => return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid king move value: {} (valid: 0-10)", move_value),
            }),
        };
        Ok((to, None))
    }

    fn decode_pawn_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        // From SCID decodePawn() - uses lookup tables:
        // static const int toSquareDiff[16] = {
        //     7,8,9, 7,8,9, 7,8,9, 7,8,9, 7,8,9, 16
        // };
        // static const pieceT promoPieceFromVal[16] = {
        //     EMPTY,EMPTY,EMPTY,  // 0-2: no promotion
        //     QUEEN,QUEEN,QUEEN,  // 3-5: queen promotion
        //     ROOK,ROOK,ROOK,     // 6-8: rook promotion
        //     BISHOP,BISHOP,BISHOP, // 9-11: bishop promotion
        //     KNIGHT,KNIGHT,KNIGHT, // 12-14: knight promotion
        //     EMPTY               // 15: double push, no promotion
        // };
        //
        // CRITICAL: Direction depends on color!
        // if (toMove == WHITE) { sm->to = sm->from + toSquareDiff[val]; }
        // else                 { sm->to = sm->from - toSquareDiff[val]; }

        const TO_SQUARE_DIFF: [i8; 16] = [
            7, 8, 9,   // 0-2: capture-left, forward, capture-right
            7, 8, 9,   // 3-5: same with Queen promotion
            7, 8, 9,   // 6-8: same with Rook promotion
            7, 8, 9,   // 9-11: same with Bishop promotion
            7, 8, 9,   // 12-14: same with Knight promotion
            16         // 15: double push
        ];

        let diff = TO_SQUARE_DIFF[move_value as usize];
        let to = if self.position.turn() == Color::White {
            self.offset_square(from, diff)?   // White adds (moves up)
        } else {
            self.offset_square(from, -diff)?  // Black subtracts (moves down)
        };

        // Determine promotion piece from move value
        let promotion = match move_value {
            0..=2 | 15 => None,
            3..=5 => Some(Role::Queen),
            6..=8 => Some(Role::Rook),
            9..=11 => Some(Role::Bishop),
            12..=14 => Some(Role::Knight),
            _ => return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid pawn move value: {}", move_value),
            }),
        };

        Ok((to, promotion))
    }

    fn decode_queen_move(&self, from: Square, move_value: u8, stream: &mut ByteStream)
        -> Result<(Square, Option<Role>)> {
        // CRITICAL: Queen diagonal moves are 2 bytes!
        let from_file = from.file() as u8;
        let from_rank = from.rank() as u8;

        let to = if move_value >= 8 {
            // Vertical move (1 byte)
            let target_rank = move_value - 8;
            Square::from_coords(shakmaty::File::new(from_file), shakmaty::Rank::new(target_rank))
        } else if move_value != from_file {
            // Horizontal move (1 byte)
            Square::from_coords(shakmaty::File::new(move_value), shakmaty::Rank::new(from_rank))
        } else {
            // Diagonal move (2 bytes) - read second byte from stream!
            let second_byte = stream.get_byte()?;
            if second_byte < 64 || second_byte > 127 {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: 0,
                    message: format!("Invalid queen diagonal byte: {}", second_byte),
                });
            }
            Square::new(second_byte - 64)
        };

        Ok((to, None))
    }

    fn decode_knight_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        // From SCID decodeKnight():
        // static const int sqdiff[] = { 0, -17, -15, -10, -6, 6, 10, 15, 17 };
        // if (val < 1 || val > 8) { return ERROR_Decode; }
        // CRITICAL: Values 0 and 9-15 are INVALID for knights!
        if move_value < 1 || move_value > 8 {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid knight move value: {} (valid: 1-8)", move_value),
            });
        }
        const SQDIFF: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];
        let offset = SQDIFF[move_value as usize];
        Ok((self.offset_square(from, offset)?, None))
    }

    fn decode_rook_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        // From SCID decodeRook():
        // if (val >= 8) { // Vertical move - to different rank
        //     sm->to = square_Make(square_Fyle(sm->from), val - 8);
        // } else {        // Horizontal move - to different file
        //     sm->to = square_Make(val, square_Rank(sm->from));
        // }
        let from_file = from.file() as u8;
        let from_rank = from.rank() as u8;

        let to = if move_value >= 8 {
            // Vertical move: keep file, change rank
            let target_rank = move_value - 8;
            Square::from_coords(shakmaty::File::new(from_file), shakmaty::Rank::new(target_rank))
        } else {
            // Horizontal move: keep rank, change file
            Square::from_coords(shakmaty::File::new(move_value), shakmaty::Rank::new(from_rank))
        };
        Ok((to, None))
    }

    fn decode_bishop_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        // From SCID decodeBishop():
        // byte fyle = (val & 7);
        // int fylediff = (int)fyle - (int)square_Fyle(sm->from);
        // if (val >= 8) {
        //     sm->to = sm->from - 7 * fylediff;  // up-left/down-right diagonal
        // } else {
        //     sm->to = sm->from + 9 * fylediff;  // up-right/down-left diagonal
        // }
        let target_file = (move_value & 7) as i8;
        let from_file = from.file() as i8;
        let file_diff = target_file - from_file;

        let offset = if move_value >= 8 {
            -7 * file_diff  // Up-left/down-right diagonal
        } else {
            9 * file_diff   // Up-right/down-left diagonal
        };
        Ok((self.offset_square(from, offset)?, None))
    }

    // All decode functions return (Square, Option<Role>) for target and optional promotion

    fn offset_square(&self, square: Square, offset: i8) -> Result<Square> {
        let idx = square.index() as i8 + offset;
        if idx < 0 || idx > 63 {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Square offset out of bounds: {}", idx),
            });
        }
        Ok(Square::new(idx as u8))
    }
}
```

**Critical Notes**:
- SCID move bytes decode to from/to squares + promotion
- Shakmaty validates legality and constructs proper Move type
- Queen diagonal moves require ByteStream for 2-byte reads
- Shakmaty handles all chess rules (castling rights, en passant, etc.)

**Testing**:
- Test each piece type move decoding
- Test 2-byte Queen diagonal moves
- Test shakmaty validates moves correctly
- Test position updates via shakmaty
- Validate against known game sequences

**Deliverable**: Complete SCID decoder outputting shakmaty Moves.

---

## Phase 6: PGN Output with Shakmaty (Week 4)

### 6.1 Algebraic Notation Generation (Using Shakmaty)

**Goal**: Use shakmaty's built-in SAN generation.

**File**: `crates/core/src/format/pgn.rs`

**Implementation** (much simpler with shakmaty!):

```rust
use shakmaty::{Chess, Position, Move, san::San};

/// Helper to track position and generate SAN notation
pub struct SanGenerator {
    position: Chess,
}

impl SanGenerator {
    pub fn new() -> Self {
        SanGenerator {
            position: Chess::default(),
        }
    }

    pub fn from_position(position: Chess) -> Self {
        SanGenerator { position }
    }

    /// Generate SAN for a move and update position
    pub fn move_to_san(&mut self, chess_move: &Move) -> Result<String> {
        // Shakmaty generates SAN notation automatically!
        let san = San::from_move(&self.position, chess_move);

        // Apply move to update position for next move
        self.position = self.position.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("pgn"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        Ok(san.to_string())
    }

    /// Generate SAN for a sequence of moves
    pub fn moves_to_san(&mut self, moves: &[Move]) -> Result<Vec<String>> {
        moves.iter()
            .map(|m| self.move_to_san(m))
            .collect()
    }
}
```

**Key Benefits**:
- Shakmaty handles all SAN rules (disambiguation, check/checkmate symbols, etc.)
- No need to implement complex disambiguation logic
- Guaranteed correct SAN output
- Handles edge cases automatically

**Testing**:
- Test SAN generation for various move types
- Verify disambiguation in complex positions
- Test check/checkmate notation
- Validate output with PGN parsers

**Deliverable**: SAN generation using shakmaty.

---

### 6.2 PGN Formatter

**Goal**: Generate complete PGN output from parsed game data.

**File**: `crates/core/src/format/converter.rs`

**Implementation**:

```rust
#[derive(Debug, Clone)]
pub struct PgnOptions {
    pub include_comments: bool,
    pub include_variations: bool,
    pub compact: bool,
}

/// Escape a string for PGN tag value output
///
/// PGN tag values are enclosed in double quotes.
/// Characters that need escaping:
/// - Backslash (\) → \\
/// - Double quote (") → \"
pub fn escape_pgn_string(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '\\' => vec!['\\', '\\'],
            '"' => vec!['\\', '"'],
            _ => vec![c],
        })
        .collect()
}

pub struct PgnFormatter;

impl PgnFormatter {
    pub fn format_game(
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        moves: &[ChessMove],
        options: &PgnOptions,
    ) -> String {
        let mut pgn = String::new();

        // Write seven tag roster
        // Using safe lookup methods from NameDatabase (see Phase 3.2.1)
        // These handle:
        // - Empty strings ("") → "?" (unknown value)
        // - Out-of-bounds IDs → "?" (with potential warning)
        // - Normal names → the actual string
        //
        // All string values are escaped for PGN safety (see Phase 3.2.2)
        pgn.push_str(&format!("[Event \"{}\"]\n", escape_pgn_string(names.get_event(index_entry.event_id))));
        pgn.push_str(&format!("[Site \"{}\"]\n", escape_pgn_string(names.get_site(index_entry.site_id))));
        pgn.push_str(&format!("[Date \"{}\"]\n", index_entry.game_date.to_pgn_string()));
        pgn.push_str(&format!("[Round \"{}\"]\n", escape_pgn_string(names.get_round(index_entry.round_id))));
        pgn.push_str(&format!("[White \"{}\"]\n", escape_pgn_string(names.get_player(index_entry.white_id))));
        pgn.push_str(&format!("[Black \"{}\"]\n", escape_pgn_string(names.get_player(index_entry.black_id))));
        pgn.push_str(&format!("[Result \"{}\"]\n", index_entry.result.to_string()));

        // Optional tags
        if index_entry.white_elo > 0 {
            pgn.push_str(&format!("[WhiteElo \"{}\"]\n", index_entry.white_elo));
        }
        if index_entry.black_elo > 0 {
            pgn.push_str(&format!("[BlackElo \"{}\"]\n", index_entry.black_elo));
        }

        // ECO code (opening classification)
        // Convert from SCID's numeric format to standard ECO string (e.g., "B12")
        if let Some(eco_str) = eco_to_string(index_entry.eco_code) {
            pgn.push_str(&format!("[ECO \"{}\"]\n", eco_str));
        }

        // Add custom tags from game data
        // Keys should be alphanumeric, values need escaping
        for (key, value) in &game_data.tags {
            pgn.push_str(&format!("[{} \"{}\"]\n", key, escape_pgn_string(value)));
        }

        pgn.push('\n');

        // Write moves using shakmaty
        let mut san_gen = SanGenerator::new();
        let mut move_num = 1;

        for (i, chess_move) in moves.iter().enumerate() {
            if i % 2 == 0 {
                pgn.push_str(&format!("{}. ", move_num));
            }

            // Shakmaty generates perfect SAN notation
            let san = san_gen.move_to_san(chess_move)?;
            pgn.push_str(&san);
            pgn.push(' ');

            if i % 2 == 1 {
                move_num += 1;
                if !options.compact && (i + 1) % 10 == 0 {
                    pgn.push('\n');
                }
            }
        }

        // Result
        pgn.push_str(&format!("{}\n\n", index_entry.result.to_string()));

        pgn
    }
}
```

**Testing**:
- Generate PGN for complete games
- Verify format matches PGN specification
- Test with various options
- Validate with PGN parsers/validators

**Deliverable**: Complete PGN generation.

---

## Phase 7: Public API Design (Week 5)

### 7.1 ScidReader Implementation

**Goal**: Create the main library entry point.

**File**: `crates/core/src/lib.rs`

**Implementation**:

```rust
pub struct ScidReader {
    si4_header: Si4Header,
    index_entries: Vec<GameIndexEntry>,
    names: NameDatabase,
    sg4_file: File,
}

impl ScidReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let base_path = path.as_ref();

        // Validate and open all three files
        let mut si4_file = File::open(base_path.with_extension("si4"))?;
        let sn4_file = File::open(base_path.with_extension("sn4"))?;
        let sg4_file = File::open(base_path.with_extension("sg4"))?;

        // Parse index header
        let si4_header = parse_si4_header(&mut si4_file)?;

        // Parse all index entries
        let mut index_entries = Vec::with_capacity(si4_header.num_games as usize);
        for _ in 0..si4_header.num_games {
            let mut entry_bytes = [0u8; 47];
            si4_file.read_exact(&mut entry_bytes)?;
            index_entries.push(parse_game_index_entry(&entry_bytes)?);
        }

        // Parse name database
        let names = parse_name_database(sn4_file)?;

        Ok(ScidReader {
            si4_header,
            index_entries,
            names,
            sg4_file,
        })
    }

    pub fn games(&self) -> impl Iterator<Item = Result<Game>> + '_ {
        (0..self.index_entries.len()).map(move |i| self.get_game(i))
    }

    pub fn get_game(&self, index: usize) -> Result<Game> {
        let entry = self.index_entries.get(index)
            .ok_or(ScidError::InvalidGameIndex(index))?;

        // Read game data from SG4
        let game_bytes = read_game_data(
            &mut self.sg4_file.try_clone()?,
            entry.game_offset,
            entry.game_length,
        )?;

        // Parse game
        let (tags, flags, move_start) = parse_game_tags(&game_bytes)?;

        // Decode moves
        let mut decoder = MoveDecoder::new();
        let moves = decoder.decode_all_moves(&game_bytes[move_start..])?;

        Ok(Game {
            index: *entry,
            tags,
            moves,
        })
    }

    pub fn to_pgn(&self, options: PgnOptions) -> Result<String> {
        let mut pgn = String::new();

        for game in self.games() {
            let game = game?;
            let game_pgn = PgnFormatter::format_game(
                &game.index,
                &self.names,
                &game.tags,
                &game.moves,
                &options,
            );
            pgn.push_str(&game_pgn);
        }

        Ok(pgn)
    }

    pub fn write_pgn(&self, mut writer: impl std::io::Write, options: PgnOptions) -> Result<()> {
        for game in self.games() {
            let game = game?;
            let game_pgn = PgnFormatter::format_game(
                &game.index,
                &self.names,
                &game.tags,
                &game.moves,
                &options,
            );
            writer.write_all(game_pgn.as_bytes())?;
        }
        Ok(())
    }

    pub fn metadata(&self) -> &Si4Header {
        &self.si4_header
    }
}
```

**Testing**:
- Test opening various SCID databases
- Test game iteration
- Test PGN generation
- Integration tests with real databases

**Deliverable**: Complete public API.

---

### 7.2 Prelude Module

**Goal**: Create convenient re-exports for library users.

**File**: `crates/core/src/prelude.rs`

**Implementation**:

```rust
pub use crate::{ScidReader, ScidError, Result};
pub use crate::database::{Game, GameIndexEntry};
pub use crate::format::{PgnOptions, PgnFormatter};
pub use crate::types::GameResult;

// Re-export common shakmaty types
pub use shakmaty::{Color, Role, Square, Move, Chess, Position};
```

**Deliverable**: Ergonomic imports for library users.

---

### 7.3 Error Recovery Strategy

**Goal**: Define comprehensive error handling for graceful recovery from corrupted or malformed data.

**Background**:

Real-world SCID databases may contain:
- Corrupted game data (disk errors, incomplete writes)
- Invalid move encodings (software bugs, version mismatches)
- Malformed tags or comments (encoding issues)
- Truncated files (interrupted operations)

A robust converter should handle these gracefully rather than failing completely.

**Error Classification**:

| Error Type | Severity | Recovery Strategy |
|------------|----------|-------------------|
| File not found / IO error | Fatal | Abort with clear message |
| Invalid magic number | Fatal | Abort - not a SCID file |
| Header parse failure | Fatal | Abort - can't determine structure |
| Index entry corruption | Recoverable | Skip game, log warning |
| Name lookup failure | Recoverable | Use "?" placeholder |
| Move decode failure | Recoverable | Output partial game or skip |
| Tag parse failure | Recoverable | Skip tag, continue with moves |
| Comment encoding error | Recoverable | Skip comment, continue |

**Implementation**:

```rust
// error.rs - Extended error types

#[derive(Debug, Error)]
pub enum ScidError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid SCID file: {0}")]
    InvalidFormat(String),

    #[error("Parse error in {file} at offset {offset}: {message}")]
    ParseError {
        file: PathBuf,
        offset: u64,
        message: String,
    },

    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),

    // NEW: Recoverable errors for error recovery
    #[error("Game {game_num} decode error: {message}")]
    GameDecodeError {
        game_num: usize,
        message: String,
    },

    #[error("Move decode error at move {move_num}: {message}")]
    MoveDecodeError {
        move_num: usize,
        message: String,
    },
}

impl ScidError {
    /// Check if error is recoverable (can continue processing other games)
    pub fn is_recoverable(&self) -> bool {
        matches!(self,
            ScidError::GameDecodeError { .. } |
            ScidError::MoveDecodeError { .. } |
            ScidError::ParseError { .. }
        )
    }

    /// Check if error is fatal (must stop processing)
    pub fn is_fatal(&self) -> bool {
        !self.is_recoverable()
    }
}

/// Result of processing a game - success, partial, or failure
#[derive(Debug)]
pub enum GameProcessResult {
    /// Game processed successfully
    Success(Game),

    /// Game partially processed (some moves decoded)
    Partial {
        game: Game,
        error: ScidError,
        moves_decoded: usize,
    },

    /// Game could not be processed
    Failed {
        game_num: usize,
        error: ScidError,
    },
}
```

**Conversion Options**:

```rust
/// Options controlling error recovery behavior
#[derive(Debug, Clone)]
pub struct ConversionOptions {
    /// How to handle game decode errors
    pub error_mode: ErrorMode,

    /// Maximum errors before aborting (0 = unlimited)
    pub max_errors: usize,

    /// Include partial games in output
    pub include_partial: bool,

    /// Log detailed error information
    pub verbose_errors: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorMode {
    /// Stop on first error
    Strict,

    /// Skip failed games, continue processing
    Lenient,

    /// Try to output partial games when possible
    BestEffort,
}

impl Default for ConversionOptions {
    fn default() -> Self {
        ConversionOptions {
            error_mode: ErrorMode::Lenient,
            max_errors: 0,  // Unlimited
            include_partial: false,
            verbose_errors: true,
        }
    }
}
```

**Conversion Statistics**:

```rust
/// Statistics from a conversion run
#[derive(Debug, Default)]
pub struct ConversionStats {
    /// Total games in database
    pub total_games: usize,

    /// Games successfully converted
    pub successful: usize,

    /// Games with partial output
    pub partial: usize,

    /// Games that failed completely
    pub failed: usize,

    /// List of failed game numbers with error messages
    pub errors: Vec<(usize, String)>,

    /// Processing time
    pub duration: std::time::Duration,
}

impl ConversionStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_games == 0 {
            100.0
        } else {
            (self.successful as f64 / self.total_games as f64) * 100.0
        }
    }

    pub fn summary(&self) -> String {
        format!(
            "Processed {}/{} games ({:.1}% success), {} partial, {} failed",
            self.successful,
            self.total_games,
            self.success_rate(),
            self.partial,
            self.failed
        )
    }
}
```

**Database Reader with Error Recovery**:

```rust
impl ScidDatabase {
    /// Convert to PGN with error recovery
    pub fn to_pgn_with_recovery(
        &self,
        pgn_options: &PgnOptions,
        conv_options: &ConversionOptions,
    ) -> (String, ConversionStats) {
        let start = std::time::Instant::now();
        let mut pgn = String::new();
        let mut stats = ConversionStats {
            total_games: self.si4_header.num_games as usize,
            ..Default::default()
        };

        for (game_num, game_result) in self.games().enumerate() {
            match game_result {
                Ok(game) => {
                    // Successful decode
                    let game_pgn = PgnFormatter::format_game(
                        &game.index,
                        &self.names,
                        &game.tags,
                        &game.moves,
                        pgn_options,
                    );
                    pgn.push_str(&game_pgn);
                    stats.successful += 1;
                }
                Err(e) if e.is_recoverable() => {
                    // Recoverable error
                    stats.failed += 1;
                    stats.errors.push((game_num, e.to_string()));

                    if conv_options.verbose_errors {
                        eprintln!("Warning: Game {} failed: {}", game_num + 1, e);
                    }

                    // Check error limit
                    if conv_options.max_errors > 0
                        && stats.failed >= conv_options.max_errors
                    {
                        eprintln!("Error limit reached, stopping conversion");
                        break;
                    }

                    // In strict mode, abort on any error
                    if conv_options.error_mode == ErrorMode::Strict {
                        break;
                    }
                }
                Err(e) => {
                    // Fatal error - must stop
                    eprintln!("Fatal error: {}", e);
                    stats.errors.push((game_num, e.to_string()));
                    break;
                }
            }
        }

        stats.duration = start.elapsed();
        (pgn, stats)
    }
}
```

**CLI Integration**:

Add CLI flags for error handling (see Phase 8.1):

```rust
#[derive(Parser, Debug)]
pub struct Args {
    // ... existing args ...

    /// Error handling mode: strict, lenient, best-effort
    #[arg(long, default_value = "lenient")]
    pub error_mode: String,

    /// Maximum errors before stopping (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_errors: usize,

    /// Include partial games in output
    #[arg(long)]
    pub include_partial: bool,

    /// Suppress error warnings
    #[arg(long)]
    pub quiet: bool,
}
```

**Move-Level Error Recovery**:

For partial game output when moves fail mid-game:

```rust
impl ScidMoveDecoder {
    /// Decode moves with recovery, returning partial results on error
    pub fn decode_moves_with_recovery(
        &mut self,
        move_data: &[u8],
    ) -> (Vec<Move>, Option<ScidError>) {
        let mut moves = Vec::new();
        let mut stream = ByteStream::new(move_data);
        let mut error = None;

        while stream.has_more() {
            let byte = match stream.get_byte() {
                Ok(b) => b,
                Err(e) => {
                    error = Some(ScidError::MoveDecodeError {
                        move_num: moves.len() + 1,
                        message: format!("Stream read error: {}", e),
                    });
                    break;
                }
            };

            // Handle special markers
            match byte {
                0x00 | 0x0F => break,  // End of game
                0x0B => { stream.get_byte().ok(); continue; }  // NAG
                0x0C => { skip_to_null(&mut stream); continue; }  // Comment
                0x0D | 0x0E => continue,  // Variation markers
                _ => {}
            }

            // Decode move
            match self.decode_move(byte, &mut stream) {
                Ok(chess_move) => moves.push(chess_move),
                Err(e) => {
                    error = Some(ScidError::MoveDecodeError {
                        move_num: moves.len() + 1,
                        message: e.to_string(),
                    });

                    // Try to continue or break based on error type
                    // Some errors (like unknown piece) are unrecoverable
                    break;
                }
            }
        }

        (moves, error)
    }
}
```

**Testing**:
- Test with intentionally corrupted game data
- Test error limit functionality
- Test partial game output
- Test statistics accuracy
- Test all error modes (strict, lenient, best-effort)

**Deliverable**: Comprehensive error recovery system with configurable behavior.

---

### 7.4 Memory Mapping for Large Databases

**Goal**: Provide optional memory-mapped file access for efficient processing of large databases.

**Background**:

SCID databases can be very large:
- Index file: 182 bytes + (47 bytes × games) → 470MB for 10M games
- Game file: Variable, can exceed several gigabytes
- Name file: Typically smaller, but can grow with unique names

Memory mapping provides:
- Zero-copy access to file data
- OS-managed page caching
- Reduced memory footprint for sequential access
- Efficient random access patterns

**Dependencies**:

Add to `Cargo.toml`:
```toml
[dependencies]
memmap2 = "0.9"  # Cross-platform memory mapping
```

**Implementation**:

```rust
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;

/// Streaming reader for game data (.sg4) files
///
/// DESIGN DECISION: Streaming is the DEFAULT and ONLY mode for reading game data.
///
/// Rationale:
/// 1. Memory efficient - only loads one game at a time, regardless of database size
/// 2. Simple API - no mode selection needed
/// 3. Sequential access pattern - converters process games one by one anyway
/// 4. Works for ALL database sizes - from tiny to multi-gigabyte
///
/// The SI4 (index) and SN4 (names) files are loaded into memory because:
/// - They're relatively small (index: 47 bytes/game, names: variable but bounded)
/// - Random access is needed for lookups
/// - Total size is typically <10MB even for large databases
pub struct GameFileReader {
    reader: BufReader<File>,
    file_size: u64,
}

impl GameFileReader {
    /// Open game file for streaming reads
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let file_size = file.metadata()?.len();
        let reader = BufReader::with_capacity(64 * 1024, file); // 64KB buffer

        Ok(GameFileReader { reader, file_size })
    }

    /// Read game data at specified offset and length
    ///
    /// This is the core method - seeks to the game's location and reads
    /// exactly the bytes needed for that game.
    pub fn read_game(&mut self, offset: u32, length: u32) -> Result<Vec<u8>> {
        // Validate bounds
        let end = offset as u64 + length as u64;
        if end > self.file_size {
            return Err(ScidError::ParseError {
                file: PathBuf::new(),
                offset: offset as u64,
                message: format!(
                    "Game data extends past end of file (offset {} + length {} > file size {})",
                    offset, length, self.file_size
                ),
            });
        }

        // Seek to game position
        self.reader.seek(SeekFrom::Start(offset as u64))?;

        // Read exactly the game's bytes
        let mut buffer = vec![0u8; length as usize];
        self.reader.read_exact(&mut buffer)?;

        Ok(buffer)
    }

    /// Get the file size
    pub fn file_size(&self) -> u64 {
        self.file_size
    }
}
```

**Updated ScidDatabase**:

```rust
/// SCID database reader using streaming for game data
///
/// Architecture:
/// - SI4 (index): Loaded into memory - small, needed for iteration
/// - SN4 (names): Loaded into memory - small, needed for lookups
/// - SG4 (games): Streamed on demand - can be gigabytes, read one game at a time
pub struct ScidDatabase {
    /// Index file header
    si4_header: Si4Header,

    /// Index entries (loaded for iteration and game lookup)
    index_entries: Vec<GameIndexEntry>,

    /// Name database (loaded for player/event/site lookups)
    names: NameDatabase,

    /// Game file reader (streaming)
    game_reader: GameFileReader,
}

impl ScidDatabase {
    /// Open a SCID database
    ///
    /// Opens the three database files (.si4, .sn4, .sg4) and prepares
    /// for streaming game access. Index and name data are loaded into
    /// memory; game data is read on demand.
    pub fn open(base_path: &str) -> Result<Self> {
        let si4_path = format!("{}.si4", base_path);
        let sn4_path = format!("{}.sn4", base_path);
        let sg4_path = format!("{}.sg4", base_path);

        // Load index file into memory (small, needed for iteration)
        let si4_data = std::fs::read(&si4_path)?;
        let (si4_header, index_entries) = parse_si4_data(&si4_data)?;

        // Load name file into memory (small, needed for lookups)
        let sn4_data = std::fs::read(&sn4_path)?;
        let names = parse_sn4_data(&sn4_data)?;

        // Open game file for streaming (can be gigabytes)
        let game_reader = GameFileReader::open(Path::new(&sg4_path))?;

        Ok(ScidDatabase {
            si4_header,
            index_entries,
            names,
            game_reader,
        })
    }

    /// Get number of games in database
    pub fn game_count(&self) -> usize {
        self.index_entries.len()
    }

    /// Get index entry for a specific game
    pub fn get_index(&self, game_num: usize) -> Option<&GameIndexEntry> {
        self.index_entries.get(game_num)
    }

    /// Read raw game data for a specific game
    ///
    /// Returns the raw bytes which can then be parsed with parse_game_structure()
    pub fn read_game_data(&mut self, game_num: usize) -> Result<Vec<u8>> {
        let entry = self.index_entries.get(game_num)
            .ok_or_else(|| ScidError::InvalidFormat(
                format!("Game number {} out of range (max {})", game_num, self.index_entries.len())
            ))?;

        self.game_reader.read_game(entry.game_offset, entry.game_length)
    }

    /// Read and parse a complete game
    pub fn read_game(&mut self, game_num: usize) -> Result<GameData> {
        let raw_data = self.read_game_data(game_num)?;
        parse_game_structure(&raw_data)
    }

    /// Get name database for lookups
    pub fn names(&self) -> &NameDatabase {
        &self.names
    }

    /// Iterate over non-deleted games
    ///
    /// Returns an iterator of (game_number, &GameIndexEntry) for games
    /// that are not marked as deleted.
    pub fn iter_games(&self) -> impl Iterator<Item = (usize, &GameIndexEntry)> {
        self.index_entries.iter()
            .enumerate()
            .filter(|(_, entry)| !entry.is_deleted())
    }

    /// Iterate over ALL games including deleted
    pub fn iter_all_games(&self) -> impl Iterator<Item = (usize, &GameIndexEntry)> {
        self.index_entries.iter().enumerate()
    }
}
```

**Why Streaming is the Right Default**:

| Aspect | Streaming | In-Memory | Memory-Mapped |
|--------|-----------|-----------|---------------|
| Memory Usage | O(1) per game | O(file size) | O(page cache) |
| Works for any size | ✅ Yes | ❌ Limited by RAM | ⚠️ Limited by address space |
| Sequential perf | Excellent | Excellent | Good |
| Implementation | Simple | Simple | Platform-specific |
| Dependencies | None | None | memmap2 crate |

**Key Design Points**:

1. **Index/Names in Memory**: These are small (typically <10MB total) and require random access for lookups. Loading them fully is efficient.

2. **Games Streamed**: Game data (.sg4) can be gigabytes. Reading one game at a time uses minimal memory regardless of database size.

3. **No Configuration Needed**: Users don't need to choose modes or set thresholds. It just works.

4. **BufReader Optimization**: The 64KB buffer amortizes syscall overhead for sequential access patterns.

**Testing**:
- Test with tiny databases (5 games) - verify correct behavior
- Test with medium databases (10,000 games) - verify performance
- Test with large databases (1M+ games, multi-GB) - verify memory stays bounded
- Measure memory usage during conversion
- Benchmark throughput (games/second)

**Deliverable**: Streaming-based game file access that works for any database size.

---

## Phase 8: CLI Tool (Week 5-6)

### 8.1 CLI Argument Parsing

**Goal**: Create user-friendly command-line interface.

**File**: `crates/cli/src/args.rs`

**Implementation**:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "scidtopgn")]
#[command(about = "Convert SCID chess databases to PGN format")]
pub struct Args {
    /// Path to SCID database (without extension)
    #[arg(value_name = "DATABASE")]
    pub input: String,

    /// Output file (stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Include comments in output
    #[arg(long, default_value = "true")]
    pub comments: bool,

    /// Include variations in output
    #[arg(long, default_value = "true")]
    pub variations: bool,

    /// Compact output (no line breaks)
    #[arg(long)]
    pub compact: bool,

    /// Error handling mode: strict, lenient, best-effort (see Phase 7.3)
    #[arg(long, default_value = "lenient", value_parser = ["strict", "lenient", "best-effort"])]
    pub error_mode: String,

    /// Maximum errors before stopping (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_errors: usize,

    /// Include partial games in output
    #[arg(long)]
    pub include_partial: bool,

    /// Suppress warning messages
    #[arg(short, long)]
    pub quiet: bool,

    /// Use memory mapping for large files (see Phase 7.4)
    #[arg(long)]
    pub mmap: bool,

    /// Memory mapping threshold in MB (default: 100)
    #[arg(long, default_value = "100")]
    pub mmap_threshold: u64,

    /// Convert only specific game range (e.g., "1-100")
    #[arg(short, long, value_name = "RANGE")]
    pub range: Option<String>,
}
```

**Testing**:
- Test argument parsing
- Test help output
- Test invalid arguments

**Deliverable**: CLI argument parsing.

---

### 8.2 Main CLI Logic

**Goal**: Wire everything together in the CLI tool.

**File**: `crates/cli/src/main.rs`

**Implementation**:

```rust
use clap::Parser;
use scidtopgn_core::prelude::*;
use std::fs::File;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Open SCID database
    eprintln!("Opening database: {}", args.input);
    let reader = ScidReader::open(&args.input)?;

    eprintln!("Database: {}", reader.metadata().description);
    eprintln!("Games: {}", reader.metadata().num_games);

    // Configure PGN options
    let options = PgnOptions {
        include_comments: args.comments,
        include_variations: args.variations,
        compact: args.compact,
    };

    // Write output
    match args.output {
        Some(path) => {
            eprintln!("Writing to file: {}", path);
            let file = File::create(path)?;
            reader.write_pgn(file, options)?;
        }
        None => {
            let stdout = io::stdout();
            let handle = stdout.lock();
            reader.write_pgn(handle, options)?;
        }
    }

    eprintln!("Conversion complete!");
    Ok(())
}
```

**Testing**:
- End-to-end CLI tests
- Test with various arguments
- Test error handling and user messages

**Deliverable**: Working CLI tool.

---

## Phase 9: Testing & Validation (Week 6)

### 9.1 Unit Tests

**Coverage**:
- All parsing functions
- All type conversions
- All edge cases

**Deliverable**: High unit test coverage.

---

### 9.2 Integration Tests

**File**: `crates/core/tests/integration.rs`

**Tests**:
- Complete database parsing
- PGN generation validation
- Performance benchmarks

**Deliverable**: Comprehensive integration tests.

---

### 9.3 Real-World Validation

**Goal**: Test with actual large databases.

**Tests**:
- Parse 100,000+ game database
- Verify PGN output with chess tools
- Performance profiling
- Memory usage analysis

**Deliverable**: Production-ready library.

---

## Phase 10: Documentation & Polish (Week 7)

### 10.1 API Documentation

**Tasks**:
- Document all public APIs with rustdoc
- Create examples in `examples/` directory
- Write comprehensive README
- Add usage examples

**Deliverable**: Complete documentation.

---

### 10.2 Performance Optimization

**Tasks**:
- Profile parsing performance
- Optimize hot paths
- Add benchmarks
- Memory optimization

**Deliverable**: Optimized library.

---

## Success Criteria

### Minimum Viable Product (MVP)
- ✅ Parse .si4, .sn4, .sg4 files correctly
- ✅ Extract game metadata (players, dates, results)
- ✅ Decode chess moves to algebraic notation
- ✅ Generate valid PGN output
- ✅ Working CLI tool

### Full Feature Set
- ✅ Support all SCID move encodings
- ✅ Handle variations and comments
- ✅ Support annotations (NAGs)
- ✅ Memory-efficient streaming for large databases
- ✅ Comprehensive error handling
- ✅ Full test coverage

### Production Ready
- ✅ Performance: Parse 1M+ games efficiently
- ✅ Reliability: Handle corrupted data gracefully
- ✅ Usability: Clear error messages and documentation
- ✅ Compatibility: Works with all SCID database versions

---

## Risk Mitigation

### Known Challenges

1. **Queen Diagonal Moves (2-byte encoding)**
   - **Risk**: Complex streaming parser required
   - **Mitigation**: Implement ByteStream early, test thoroughly

2. **Position Tracking Complexity**
   - **Risk**: Bugs in board state tracking
   - **Mitigation**: Extensive unit tests, validate with known games

3. **Performance with Large Databases**
   - **Risk**: Memory/speed issues with millions of games
   - **Mitigation**: Profile early, use streaming APIs, memory mapping

4. **SCID Format Variations**
   - **Risk**: Edge cases not in spec
   - **Mitigation**: Test with diverse databases, graceful error handling

---

## Timeline Summary (Updated with Shakmaty)

- **Week 1**: Foundation + Index Parser
- **Week 2**: Name Parser + Game Structure
- **Week 3**: SCID Position Wrapper + Move Decoder (simplified with shakmaty)
- **Week 4**: PGN Output (using shakmaty SAN) + Public API
- **Week 5**: CLI Tool + Integration Testing
- **Week 6**: Real-World Validation + Documentation + Polish

**Total**: ~5-6 weeks for production-ready implementation

**Time Savings from Shakmaty**:
- ~1-2 weeks saved on chess position logic
- ~1 week saved on SAN generation and disambiguation
- Fewer bugs to fix (battle-tested library)
- More time for SCID-specific parsing and optimization

---

## Next Steps

1. Set up workspace structure
2. Implement core types and errors
3. Begin SI4 header parsing with real test data
4. Iterate through phases systematically
5. Test continuously with real SCID databases

This plan provides a clear roadmap from empty workspace to production-ready SCID parser with comprehensive testing and documentation.

---

## Revision History

### January 2026 - Gap Filling Update (v2.1)

Additional gaps filled to make implementation plan complete:

#### Phases Affected by v2.1

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **4.3** | Special Bytes | ADDED: Complete `special_bytes` module with all marker constants |
| **4.3** | Comment Algorithm | ADDED: Complete comment tree traversal algorithm (pre-order DFS) |
| **4.3** | NAG Handling | ADDED: NAG code table and PGN formatting |
| **4.3** | Variation Data | ADDED: Complete `MoveNode` and `GameTree` structures |
| **5.1** | ScidPosition | FIXED: Now has separate white_pieces/black_pieces HashMaps |
| **5.1** | ScidPosition | FIXED: Added white_count/black_count for capture swap |
| **5.1** | init_piece_mappings | FIXED: Now initializes BOTH colors, not just white |
| **5.1** | update_piece_locations | ADDED: Complete capture swap algorithm with en passant handling |
| **5.1** | init_fen_piece_mappings | ADDED: Complete FEN initialization with King-always-0 |
| **5.1** | get_piece_square | FIXED: Now uses side-to-move to select correct piece map |

#### Detail Phase Documents Needing Updates

The following phase documents need to be updated to match these changes:

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_4_GAME_FILE_STRUCTURE.md` | Medium | Add `special_bytes` module, comment algorithm, NAG handling, variation structures |
| `PHASE_5_MOVE_PARSING.md` | **HIGH** | Fix Bishop decoder (uses wrong algorithm!), update ScidPosition struct |

**CRITICAL**: PHASE_5_MOVE_PARSING.md has INCORRECT Bishop decoder - uses fylediff formula `((val/4)+1) * (val&1 ? -1 : 1)` but correct algorithm is `fyle = val & 7; offset = (val >= 8) ? -7*fylediff : 9*fylediff`

---

### January 2026 - Chess960 Support (v2.2)

Added Chess960 (Fischer Random Chess) support to address Gap 10 from IMPLEMENTATION_GAPS.md.

#### Phases Affected by v2.2

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **5.1.1** | Chess960 Support | NEW: Complete section for Chess960/FRC variant handling |
| **5.1.1** | FEN Detection | ADDED: `is_chess960_fen()` function to detect Chess960 from castling rights |
| **5.1.1** | Shakmaty Integration | ADDED: `from_fen_auto()` using `CastlingMode::Chess960` |
| **5.1.1** | Castling | DOCUMENTED: Why hardcoded destination squares work for both variants |
| **5.1.1** | PGN Output | ADDED: Chess960-specific PGN headers (Variant, SetUp, FEN) |

#### Detail Phase Documents Needing Updates

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_5_MOVE_PARSING.md` | Medium | Add Chess960 support section (5.1.1) |

#### Gap Resolution

- **Gap 10** (Chess960/FRC Support): RESOLVED - Added comprehensive Chess960 support through FEN detection and shakmaty's CastlingMode

---

### January 2026 - Name ID Handling (v2.3)

Added proper Name ID lookup handling to address Gap 11 from IMPLEMENTATION_GAPS.md.

#### Phases Affected by v2.3

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **3.2.1** | Name ID Lookup | NEW: Complete section for safe name ID lookups |
| **3.2.1** | NameDatabase | ADDED: `get_player()`, `get_event()`, `get_site()`, `get_round()` safe lookup methods |
| **3.2.1** | NameLookupResult | ADDED: Enum for detailed lookup results (Found/Empty/OutOfBounds) |
| **3.2.1** | Edge Cases | DOCUMENTED: ID 0 behavior, empty names, out-of-bounds handling |
| **6.2** | PGN Formatter | UPDATED: Now uses safe lookup methods instead of direct array access |

#### Detail Phase Documents Needing Updates

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_3_NAME_FILE_PARSER.md` | Medium | Add Name ID lookup section (3.2.1) |
| `PHASE_6_PGN_OUTPUT.md` | Low | Update to use safe lookup methods |

#### Gap Resolution

- **Gap 11** (Name ID 0 Handling): RESOLVED - Added safe lookup methods with proper empty/out-of-bounds handling

---

### January 2026 - Round String Handling (v2.4)

Added comprehensive round string format documentation to address Gap 12 from IMPLEMENTATION_GAPS.md.

#### Phases Affected by v2.4

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **3.2.2** | Round String Formats | NEW: Complete documentation of round format variations |
| **3.2.2** | get_round_normalized | ADDED: Optional hyphen-to-? normalization method |
| **6.2** | escape_pgn_string | ADDED: PGN string escaping for quotes and backslashes |
| **6.2** | PGN Formatter | UPDATED: All string values now escaped for PGN safety |

#### Detail Phase Documents Needing Updates

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_3_NAME_FILE_PARSER.md` | Low | Add Round string format section (3.2.2) |
| `PHASE_6_PGN_OUTPUT.md` | Low | Add escape_pgn_string function |

#### Gap Resolution

- **Gap 12** (Round String Format): RESOLVED - Documented all round formats, added PGN escaping

---

### January 2026 - Error Recovery Strategy (v2.5)

Added comprehensive error recovery system to address Gap 13 from IMPLEMENTATION_GAPS.md.

#### Phases Affected by v2.5

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **7.3** | Error Recovery Strategy | NEW: Complete error recovery system |
| **7.3** | ScidError | EXTENDED: Added GameDecodeError, MoveDecodeError, is_recoverable() |
| **7.3** | GameProcessResult | NEW: Success/Partial/Failed enum for game processing |
| **7.3** | ConversionOptions | NEW: ErrorMode (Strict/Lenient/BestEffort), max_errors, include_partial |
| **7.3** | ConversionStats | NEW: Track success/partial/failed counts and error list |
| **7.3** | to_pgn_with_recovery | NEW: Main conversion method with error handling |
| **7.3** | decode_moves_with_recovery | NEW: Move-level error recovery |
| **8.1** | CLI Args | ADDED: --error-mode, --max-errors, --include-partial, --quiet flags |

#### Detail Phase Documents Needing Updates

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_7_PUBLIC_API.md` | Medium | Add Error Recovery section (7.3) |
| `PHASE_8_CLI_TOOL.md` | Low | Add error handling CLI flags |

#### Gap Resolution

- **Gap 13** (Error Recovery Mid-Game): RESOLVED - Added comprehensive error recovery with multiple modes

---

### January 2026 - Memory Mapping Support (v2.6)

Added optional memory-mapped file access to address Gap 14 from IMPLEMENTATION_GAPS.md.

#### Phases Affected by v2.6

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **7.4** | Streaming | NEW: GameFileReader for streaming game data access |
| **7.4** | ScidDatabase | UPDATED: Simplified API using streaming by default |
| **7.4** | Design | CHANGED: Removed FileAccessMode enum, streaming is now the only mode |

#### Detail Phase Documents Needing Updates

| Document | Priority | Changes Needed |
|----------|----------|----------------|
| `PHASE_7_PUBLIC_API.md` | Medium | Update to reflect streaming-only approach |

#### Gap Resolution

- **Gap 14** (Memory Mapping for Large DBs): RESOLVED - Added optional mmap with auto-detection

---

### January 2026 - Bible Verification Update (v2.0)

After thorough verification against SCID source code (`repomix-scidvspc.xml`), the following corrections were made to align this implementation plan with the verified `SCID_DATABASE_FORMAT.md` ("the Bible"):

#### Phases Affected by v2.0

| Phase | Section | Change Summary |
|-------|---------|----------------|
| **4.2** | Tag Parsing | Added Phase 4.3 for game data structure |
| **4.3** | Game Data Structure | NEW - Documents two-part comment encoding |
| **5.1** | Piece Numbering | FIXED: Was King=0, Queen=1, Ra1=2... Now: King=0, QR=1, QN=2, QB=3, Q=4, KB=5, KN=6, KR=7 |
| **5.1** | Capture Swap | ADDED: Critical algorithm for piece list management |
| **5.1** | FEN Initialization | ADDED: King-always-0 swap logic |
| **5.2** | King Move | FIXED: Castling values were 10/11, now 9=O-O-O, 10=O-O |
| **5.2** | King Move | FIXED: Null move (value 0) is valid, not an error |
| **5.2** | Pawn Move | FIXED: Now uses toSquareDiff table with color-dependent +/- |
| **5.2** | Knight Move | ADDED: Validation that values 0 and 9-15 are INVALID |
| **5.2** | Rook Move | ADDED: Complete implementation |
| **5.2** | Bishop Move | ADDED: Complete implementation with fylediff formula |

#### Critical Corrections in v2.0

1. **Piece Numbering Order** (WRONG before):
   - OLD: "King=0, Queen=1, Ra1=2, Rh1=3, Bc1=4, Bf1=5, Nb1=6, Ng1=7"
   - NEW: "King=0, QR=1 (A1), QN=2 (B1), QB=3 (C1), Q=4 (D1), KB=5 (F1), KN=6 (G1), KR=7 (H1)"

2. **King Castling Values** (WRONG before):
   - OLD: "10 = Kingside castle, 11 = Queenside castle"
   - NEW: "9 = Queenside castle (O-O-O), 10 = Kingside castle (O-O)"

3. **Pawn Move Encoding** (WRONG before):
   - OLD: Used `forward - 1`, `forward`, `forward + 1` for captures
   - NEW: Uses `toSquareDiff` table with `+diff` for White, `-diff` for Black

4. **Capture Swap Algorithm** (MISSING before):
   - When a piece is captured, the LAST piece takes the captured piece's slot number
   - This is critical for correct piece number tracking

5. **Comment Encoding** (NOT DOCUMENTED before):
   - Two-part system: Markers (0x0C) in move data, text at end of game data

#### Source of Truth

All corrections verified against:
- `SCID_DATABASE_FORMAT.md` (the Bible)
- SCID source code: `Position::StdStart()` lines 77808-77858
- SCID source code: `Position::DoSimpleMove()` lines 78903-78912
- SCID source code: `decodeKing()` lines 59114-59126
- SCID source code: `decodePawn()` lines 59326-59348
- SCID source code: `decodeBishop()` lines 59232-59262
- SCID source code: `encodeComments()` / `decodeComments()` lines 59656-59708
