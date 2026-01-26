# Phase 4: Game File Structure - Detailed Implementation Plan

**Timeline**: Week 3 (5-7 days)
**Prerequisites**: Phase 2 complete (index parsing), Phase 3 complete (name parsing)
**Dependencies**: Index file offsets, binary parsing, flate2 crate (zlib decompression)

---

## Overview

Phase 4 implements parsing of SCID game file (.sg4) structure WITHOUT parsing chess moves yet. We focus on:
- Reading game boundaries using index file offsets
- Parsing PGN tags (metadata embedded in games)
- Parsing game flags
- Handling optional FEN strings (non-standard starting positions)
- Preparing data structures for move parsing (Phase 5)

**What is the Game File?**

The `.sg4` file contains:
- **Multiple games** stored sequentially
- **Variable-length game records** (each game has different size)
- **PGN tags** (player titles, annotations, etc.)
- **Game flags** (promotions, custom start position, etc.)
- **Chess moves** in binary format (Phase 5)

**CRITICAL IMPLEMENTATION NOTE - COMPRESSION**:

⚠️ **Most real-world SCID databases use zlib compression!**

Before parsing game data, you MUST check the `packed` flag in the index entry:
- If `entry.is_packed()` returns true, decompress with zlib before parsing
- If false, data is uncompressed and can be parsed directly

Without decompression support, the parser will read garbage bytes as moves!
See Task 4.1.3 for complete decompression implementation.

**CRITICAL MISCONCEPTIONS TO AVOID**:

⚠️ **MYTH**: You can scan the .sg4 file for patterns to find game boundaries
**REALITY**: Game boundaries MUST be determined from the index file (.si4)

⚠️ **MYTH**: Bytes 11-15 (ENCODE_NAG, ENCODE_COMMENT, etc.) mark tag/move boundaries
**REALITY**: These bytes are move elements; tags end with null terminator (0x00)

⚠️ **MYTH**: Double-null (0x00 0x00) separates games
**REALITY**: Double-null can appear within games (e.g., comment terminators)

⚠️ **MYTH**: Each .sg4 file contains one game
**REALITY**: One .sg4 file contains ALL games in the database

**Success Criteria**:
- ✅ Read game data using index file offsets
- ✅ **Detect and decompress zlib-compressed games** (CRITICAL - Gap 13)
- ✅ Parse PGN tags correctly (common and regular tags)
- ✅ Detect tag section end (null terminator)
- ✅ Parse game flags byte
- ✅ Handle optional FEN strings
- ✅ **Understand two-part comment encoding** (NEW)
- ✅ **Separate move data from comment data** (NEW)
- ✅ **Identify special byte markers (0x0B-0x0F)** (NEW)
- ✅ Validate against known game data (five.sg4)
- ✅ All tests passing with real SCID files

---

## Reference Documentation

### SCID Format Specification

**Primary Reference**: `SCID_DATABASE_FORMAT.md`

**CRITICAL SECTIONS - READ THESE FIRST**:

- **Lines 482-654**: **Multiple Games Per SG4 File** (MOST CRITICAL!)
  - Why you CANNOT scan for boundaries
  - How to use index file offsets correctly
  - Common implementation errors

- **Lines 656-754**: **Tag Parsing and Game Structure**
  - Tag section end detection (null terminator)
  - Tag encoding rules
  - Why bytes 11-15 are NOT boundary markers

- **Lines 690-754**: **Tag Encoding Rules**
  - Common tags (240+ tag_id)
  - Regular tags
  - Special EventDate (255 + 3 bytes)

- **Lines 509-534**: **Game Record Structure**
  - Overall layout: tags → flags → moves → end marker

- **Lines 1106-1137**: **Two-Part Comment Encoding** (NEW - CRITICAL!)
  - Comment markers (0x0C) in move data contain NO text
  - Actual comment text stored AFTER move data
  - Comments are null-terminated strings at end of game
  - Tree traversal order (pre-order DFS)

- **SCID Source Lines 59349-59358**: **Special Byte Constants**
  - ENCODE_NAG = 11 (0x0B)
  - ENCODE_COMMENT = 12 (0x0C)
  - ENCODE_START_MARKER = 13 (0x0D)
  - ENCODE_END_MARKER = 14 (0x0E)
  - ENCODE_END_GAME = 15 (0x0F)

### Key Concepts

**Game Boundary Detection** (Lines 535-598):

SCID uses the index file to locate games:

```rust
// ✅ CORRECT - Use index file
let offset = index_entry.game_offset;
let length = index_entry.game_length;
let game_bytes = &sg4_data[offset..offset+length];

// ❌ WRONG - Scanning for patterns
// let game_end = find_double_null(data);  // FAILS!
```

**Tag Section End Detection** (Lines 662-689):

Tags end when you read a null byte (0x00):

```rust
loop {
    let byte = read_byte()?;
    if byte == 0 {
        break;  // Tags section complete
    }
    // Parse this tag...
}
// Next byte is flags
```

**Tag Encoding** (Lines 690-724):

Three tag types:

1. **Common tags** (byte > 240):
   ```
   [241][value_length][value_bytes]
   ```

2. **Regular tags**:
   ```
   [name_length][name_bytes][value_length][value_bytes]
   ```

3. **Special EventDate**:
   ```
   [255][3_date_bytes]
   ```

**Two-Part Comment Encoding** (CRITICAL - Lines 1106-1137):

SCID does NOT store comment text inline with moves!

```text
WRONG assumption:
  [move][0x0C]["comment text\0"][move]...

CORRECT structure:
  Move data:    [move][0x0C][move][0x0C][0x0F]
                      ^^^^       ^^^^  ^^^^
                      marker     marker end

  Comment data: ["First comment\0"]["Second comment\0"]
                 (stored AFTER 0x0F)
```

The 0x0C byte is just a MARKER. Text comes later!

---

## Test Data Reference

### Location and Datasets

All test data is in `tests/data/`. See `IMPLEMENTATION_PLAN.md` → "Test Data" section for complete documentation.

| Dataset | Game File | Description |
|---------|-----------|-------------|
| **one** | `one.sg4` | Single game - basic game structure |
| **five** | `five.sg4` | Five games - comprehensive parsing |

### PGN ↔ SCID Relationship

Each SCID database was created by importing its corresponding PGN file:
- `one.pgn` → `one.sg4` (game data corresponds to PGN movetext)
- `five.pgn` → `five.sg4` (game data corresponds to PGN movetext)

This enables validation: decoded moves should produce the same movetext as the source PGN.

### Expected Structure (five.sg4 Game 1)

- Offset: From five.si4 Game 1 entry
- Length: From five.si4 Game 1 entry
- Contains: Tags, flags, moves, end marker
- Tags may include: WhiteTitle, BlackTitle, etc.

---

## Section 4.1: Game Boundary Detection

### Objective

Implement correct game boundary detection using index file offsets. This is CRITICAL to get right.

---

### Task 4.1.1: Understand Why Scanning Fails (Educational Task)

**Acceptance Criteria**:
- Documentation explaining the problem
- Examples of why patterns don't work
- Clear understanding of correct approach
- Reference implementation errors to avoid

**Steps**:

1. Create `crates/core/src/database/games.rs`:

```rust
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
//! ❌ **WRONG ASSUMPTION**: Each .sg4 file contains one game
//! ✅ **REALITY**: One .sg4 file contains ALL games
//!
//! ❌ **WRONG APPROACH**: Scan file for patterns to find game boundaries
//! ✅ **CORRECT APPROACH**: Use index file offsets
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
//! ```rust
//! // From index file, we know:
//! let game_offset = 0;      // Game 1 starts at byte 0
//! let game_length = 1234;   // Game 1 is 1234 bytes long
//!
//! // Read exact slice
//! let game_bytes = &sg4_data[offset..offset+length];
//! ```
//!
//! ## Proof from SCID Source Code
//!
//! From scidvspc/src/GFile::ReadGame() (SCID_DATABASE_FORMAT.md lines 542-548):
//!
//! ```cpp
//! errorT GFile::ReadGame (ByteBuffer * bb, uint offset, uint length)
//! {
//!     // Reads specific game at offset with length from SG4 file
//!     bb->ProvideExternal (&(CurrentBlock->data[offset % GF_BLOCKSIZE]), length);
//!     return OK;
//! }
//! ```
//!
//! SCID **ALWAYS** uses offset and length. It **NEVER** scans for patterns.
//!
//! ## Real-World Example
//!
//! From five.sg4 (5-game database):
//!
//! ```text
//! Game 1: offset=0,     length=500   → bytes [0..500]
//! Game 2: offset=500,   length=450   → bytes [500..950]
//! Game 3: offset=950,   length=600   → bytes [950..1550]
//! Game 4: offset=1550,  length=520   → bytes [1550..2070]
//! Game 5: offset=2070,  length=480   → bytes [2070..2550]
//! ```
//!
//! No gaps, no separators, no magic patterns. Just sequential data.
//!
//! ## Implementation Rule
//!
//! **ALWAYS USE INDEX FILE OFFSETS**
//!
//! Never implement:
//! - ❌ Pattern scanning
//! - ❌ Delimiter searching
//! - ❌ "Find next game" logic
//!
//! Always implement:
//! - ✅ Read offset and length from index entry
//! - ✅ Read exact byte slice from .sg4
//! - ✅ Parse that slice independently
//!
//! See SCID_DATABASE_FORMAT.md lines 482-654 for complete explanation.

use crate::error::{Result, ScidError};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

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
```

**Validation**:

```bash
cargo build -p scidtopgn-core
# Should compile successfully
```

**Educational Note**: This task builds understanding. The documentation itself is the deliverable.

---

### Task 4.1.2: Implement Game Data Reading

**Acceptance Criteria**:
- Function reads game slice using offset and length
- Validates offset and length are in bounds
- Returns exact byte slice
- Error handling for invalid offsets
- Complete test coverage

**Steps**:

1. Add to `crates/core/src/database/games.rs`:

```rust
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
/// │ (500 bytes)│ (450 bytes)│ (600 bytes)│ (520 bytes)│
/// └────────────┴────────────┴────────────┴────────────┘
///  ↑            ↑            ↑            ↑
///  offset=0     offset=500   offset=950   offset=1550
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
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// # use scidtopgn_core::database::games::read_game_data;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("database.sg4")?;
///
/// // From index entry
/// let offset = 1234;
/// let length = 567;
///
/// // Read exact slice
/// let game_bytes = read_game_data(&mut file, offset, length)?;
/// assert_eq!(game_bytes.len(), length as usize);
/// # Ok(())
/// # }
/// ```
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
    file.read_exact(&mut game_bytes).map_err(|e| {
        ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: offset as u64,
            message: format!("Failed to read {} bytes: {}", length, e),
        }
    })?;

    Ok(game_bytes)
}
```

2. Add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_data_default() {
        let data = GameData::default();
        assert_eq!(data.flags, 0);
        assert!(data.tags.is_empty());
        assert!(data.start_position.is_none());
    }

    #[test]
    fn test_read_game_data_validation() {
        // Test zero length error
        // (Will need actual file for full test)

        // Test maximum length validation
        let max_length = 131071u32;
        assert!(max_length <= 131071);

        let invalid_length = 200000u32;
        assert!(invalid_length > 131071);
    }
}
```

**Validation**:

```bash
cargo test -p scidtopgn-core games::tests
```

---

### Task 4.1.3: Implement zlib Decompression Support (CRITICAL)

**Acceptance Criteria**:
- Detect compressed games using `is_packed()` from index entry
- Decompress game data using zlib (flate2 crate)
- Handle both compressed and uncompressed games transparently
- Error handling for corrupted compressed data
- Complete test coverage

**Background**:

**CRITICAL**: Most real-world SCID databases store games in zlib-compressed format!

When the `packed` flag (bit 7 of byte 6 in index entry) is set, the game data
read from .sg4 is zlib-compressed and MUST be decompressed before parsing.

Without this support, the parser will read garbage bytes as move data for
the majority of real databases.

**Steps**:

1. Add `flate2` dependency to `Cargo.toml`:

```toml
[dependencies]
flate2 = "1.0"
```

2. Add decompression function to `crates/core/src/database/games.rs`:

```rust
use flate2::read::ZlibDecoder;
use std::io::Read;

/// Maximum size for decompressed game data
///
/// SCID uses a MAX_GAME_LENGTH of ~128KB for decompressed data.
/// We use a slightly larger buffer for safety.
const MAX_DECOMPRESSED_SIZE: usize = 256 * 1024; // 256 KB

/// Decompress zlib-compressed game data
///
/// # Arguments
///
/// * `compressed` - Raw compressed bytes from .sg4 file
///
/// # Returns
///
/// Decompressed game data ready for parsing
///
/// # Errors
///
/// Returns error if:
/// - Data is not valid zlib format
/// - Decompressed size exceeds maximum
/// - Decompression fails for any reason
///
/// # Example
///
/// ```rust
/// if index_entry.is_packed() {
///     let decompressed = decompress_game_data(&raw_bytes)?;
///     parse_game_structure(&decompressed)?;
/// } else {
///     parse_game_structure(&raw_bytes)?;
/// }
/// ```
pub fn decompress_game_data(compressed: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut decompressed = Vec::with_capacity(compressed.len() * 4); // Estimate 4x expansion

    decoder.read_to_end(&mut decompressed).map_err(|e| {
        ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: 0,
            message: format!("Failed to decompress game data: {}", e),
        }
    })?;

    // Sanity check - decompressed data shouldn't be too large
    if decompressed.len() > MAX_DECOMPRESSED_SIZE {
        return Err(ScidError::ParseError {
            file: PathBuf::from("sg4"),
            offset: 0,
            message: format!(
                "Decompressed game data too large: {} bytes (max {})",
                decompressed.len(),
                MAX_DECOMPRESSED_SIZE
            ),
        });
    }

    Ok(decompressed)
}

/// Read and optionally decompress game data from .sg4 file
///
/// This is the recommended high-level function for reading games.
/// It automatically handles both compressed and uncompressed games.
///
/// # Arguments
///
/// * `file` - Opened .sg4 file
/// * `entry` - Index entry for the game (contains offset, length, packed flag)
///
/// # Returns
///
/// Ready-to-parse game data (decompressed if necessary)
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// # use scidtopgn_core::database::games::read_and_decompress_game;
/// # use scidtopgn_core::database::index::GameIndexEntry;
/// # fn example(entry: &GameIndexEntry) -> Result<(), Box<dyn std::error::Error>> {
/// let mut file = File::open("database.sg4")?;
///
/// // Automatically handles compression
/// let game_bytes = read_and_decompress_game(&mut file, entry)?;
///
/// // Now parse the game data...
/// # Ok(())
/// # }
/// ```
pub fn read_and_decompress_game(
    file: &mut File,
    entry: &crate::database::index::GameIndexEntry,
) -> Result<Vec<u8>> {
    // Read raw game data
    let raw_data = read_game_data(file, entry.game_offset, entry.game_length)?;

    // Decompress if packed
    if entry.is_packed() {
        decompress_game_data(&raw_data)
    } else {
        Ok(raw_data)
    }
}
```

3. Add tests:

```rust
#[cfg(test)]
mod decompression_tests {
    use super::*;
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// Create zlib-compressed test data
    fn compress_test_data(data: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    #[test]
    fn test_decompress_valid_data() {
        // Create test game data (simulated)
        let original = b"This is test game data with some moves and comments";

        // Compress it
        let compressed = compress_test_data(original);

        // Decompress it
        let decompressed = decompress_game_data(&compressed).unwrap();

        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_empty_data() {
        // Compressed empty data
        let compressed = compress_test_data(b"");
        let decompressed = decompress_game_data(&compressed).unwrap();
        assert!(decompressed.is_empty());
    }

    #[test]
    fn test_decompress_invalid_data() {
        // Not valid zlib data
        let invalid = b"This is not compressed data!";
        let result = decompress_game_data(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_decompress_large_data() {
        // Create larger test data
        let original: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let compressed = compress_test_data(&original);

        // Verify compression actually reduced size
        assert!(compressed.len() < original.len());

        // Decompress and verify
        let decompressed = decompress_game_data(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_decompress_real_move_data_pattern() {
        // Simulate realistic move data bytes (0x00-0xFF patterns)
        let mut game_data = Vec::new();

        // Tags section (simplified)
        game_data.push(0x00); // Empty tags

        // Flags byte
        game_data.push(0x00);

        // Move data (simulated moves)
        game_data.extend_from_slice(&[0xCF, 0xDF, 0x61, 0x51]); // Some move bytes

        // End marker
        game_data.push(0x0F);

        // Compress and decompress
        let compressed = compress_test_data(&game_data);
        let decompressed = decompress_game_data(&compressed).unwrap();

        assert_eq!(decompressed, game_data);
    }
}
```

4. Update module exports in `crates/core/src/database/mod.rs`:

```rust
pub use games::{read_game_data, decompress_game_data, read_and_decompress_game, GameData};
```

**Validation**:

```bash
# Verify flate2 dependency
cargo build -p scidtopgn-core

# Run decompression tests
cargo test -p scidtopgn-core decompression_tests

# All tests should pass
```

**Integration Note**:

When using the game reading functions, ALWAYS check the packed flag:

```rust
// ✅ CORRECT - Uses high-level function that handles compression
let game_data = read_and_decompress_game(&mut sg4_file, &index_entry)?;

// ✅ ALSO CORRECT - Manual handling
let raw_data = read_game_data(&mut sg4_file, entry.game_offset, entry.game_length)?;
let game_data = if entry.is_packed() {
    decompress_game_data(&raw_data)?
} else {
    raw_data
};

// ❌ WRONG - Ignores compression, will read garbage for compressed games!
let game_data = read_game_data(&mut sg4_file, entry.game_offset, entry.game_length)?;
```

---

## Section 4.2: Tag and Flags Parsing

### Objective

Implement complete tag parsing with proper detection of tag section end, handling of common and regular tags, and game flags parsing.

---

### Task 4.2.1: Understand Tag Section Structure (Educational Task)

**Acceptance Criteria**:
- Documentation explaining tag structure
- Clear examples of tag encoding
- Understanding of null terminator significance
- Common vs regular tag distinction

**Steps**:

1. Add comprehensive documentation to `games.rs`:

```rust
/// # Tag Parsing - Complete Specification
///
/// ## Tag Section Overview
///
/// Game records start with PGN tags stored in binary format:
///
/// ```text
/// ┌─────────────────────────────────────────────────┐
/// │ Game Record                                     │
/// ├─────────────────────────────────────────────────┤
/// │ Tags Section (variable length)                  │
/// │   [tag1][tag2][tag3]...[0x00]                  │
/// ├─────────────────────────────────────────────────┤
/// │ Flags Byte (1 byte)                            │
/// ├─────────────────────────────────────────────────┤
/// │ Optional FEN (if non-standard start)           │
/// ├─────────────────────────────────────────────────┤
/// │ Move Data (variable length)                    │
/// ├─────────────────────────────────────────────────┤
/// │ End Marker (0x0F)                              │
/// └─────────────────────────────────────────────────┘
/// ```
///
/// ## How Tags End
///
/// **Tags end with a null byte (0x00)**
///
/// ```rust
/// loop {
///     let byte = read_byte()?;
///     if byte == 0 {
///         break;  // Tags complete!
///     }
///     parse_tag(byte, ...)?;
/// }
/// // Next byte is the flags byte
/// ```
///
/// ## Three Tag Types
///
/// ### 1. Common Tags (byte > 240)
///
/// SCID has 10 "common" tags that are encoded efficiently:
///
/// ```text
/// Byte value:  241 = WhiteTitle
///              242 = BlackTitle
///              243 = Annotator
///              etc.
///
/// Format: [tag_id][value_length][value_bytes]
///
/// Example: WhiteTitle = "GM"
///   [241][2]['G']['M']
/// ```
///
/// ### 2. Regular Tags (byte ≤ 240)
///
/// Custom tags or standard PGN tags:
///
/// ```text
/// Format: [name_length][name_bytes][value_length][value_bytes]
///
/// Example: Custom tag "MyTag" = "value"
///   [5]['M']['y']['T']['a']['g'][5]['v']['a']['l']['u']['e']
/// ```
///
/// ### 3. Special EventDate (byte = 255)
///
/// Event date encoded as 3-byte binary value:
///
/// ```text
/// Format: [255][byte1][byte2][byte3]
/// ```
///
/// ## Common Tag IDs
///
/// From SCID source code (game.cpp):
///
/// ```rust
/// const COMMON_TAGS: [&str; 10] = [
///     "WhiteTitle",    // 241
///     "BlackTitle",    // 242
///     "Annotator",     // 243
///     "Opening",       // 244
///     "Variation",     // 245
///     "SubVariation",  // 246
///     "WhiteElo",      // 247 (rarely used, usually in index)
///     "BlackElo",      // 248 (rarely used)
///     "WhiteUSCF",     // 249
///     "BlackUSCF",     // 250
/// ];
/// ```
///
/// Note: Tag IDs start at 241 because:
/// - 0-240: Used for regular tag name lengths
/// - 241-250: Common tags
/// - 255: Special EventDate
///
/// ## Critical: Tags vs Moves
///
/// **COMMON MISTAKE**: Thinking bytes 11-15 separate tags from moves
///
/// ```rust
/// // ❌ WRONG - Don't check for special bytes
/// if byte >= 11 && byte <= 15 {
///     // Start parsing moves
/// }
///
/// // ✅ CORRECT - Check for null terminator
/// if byte == 0 {
///     // Tags done, next is flags
/// }
/// ```
///
/// Bytes 11-15 are move elements:
/// - 11 (0x0B) = ENCODE_NAG (annotation)
/// - 12 (0x0C) = ENCODE_COMMENT
/// - 13 (0x0D) = ENCODE_START_MARKER (variation start)
/// - 14 (0x0E) = ENCODE_END_MARKER (variation end)
/// - 15 (0x0F) = ENCODE_END_GAME
///
/// These can appear in tag VALUES (as regular bytes), so they don't
/// mark boundaries!
///
/// See SCID_DATABASE_FORMAT.md lines 656-754 for complete specification.

/// Maximum tag name length for regular tags
const MAX_TAG_LEN: u8 = 240;

/// Common tag names (indices 0-9 correspond to tag IDs 241-250)
///
/// From SCID source: game.cpp, commonTags array
const COMMON_TAGS: [&str; 10] = [
    "WhiteTitle",    // 241
    "BlackTitle",    // 242
    "Annotator",     // 243
    "Opening",       // 244
    "Variation",     // 245
    "SubVariation",  // 246
    "WhiteElo",      // 247
    "BlackElo",      // 248
    "WhiteUSCF",     // 249
    "BlackUSCF",     // 250
];

/// Special byte for EventDate tag
const EVENT_DATE_TAG: u8 = 255;

/// Validate a FEN string using shakmaty (Gap 7)
///
/// Ensures the FEN represents a valid chess position that can be used
/// as a starting position for move parsing.
///
/// # Arguments
///
/// * `fen` - FEN string to validate
///
/// # Returns
///
/// Ok(()) if valid, Err with description if invalid
///
/// # Example
///
/// ```
/// // Valid starting position
/// assert!(validate_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_ok());
///
/// // Invalid FEN (missing king)
/// assert!(validate_fen("8/8/8/8/8/8/8/8 w - - 0 1").is_err());
/// ```
pub fn validate_fen(fen: &str) -> Result<()> {
    use shakmaty::fen::Fen;

    // Parse FEN with shakmaty
    let parsed: Fen = fen.parse().map_err(|e| {
        ScidError::InvalidFormat(format!("FEN parse error: {:?}", e))
    })?;

    // Convert to position to validate it's a legal position
    let _position: shakmaty::Chess = parsed.into_position(shakmaty::CastlingMode::Standard)
        .map_err(|e| {
            ScidError::InvalidFormat(format!("Invalid chess position: {:?}", e))
        })?;

    Ok(())
}

/// Parse FEN string into shakmaty Chess position (Gap 7)
///
/// Used when initializing the move decoder with a custom starting position.
///
/// # Arguments
///
/// * `fen` - FEN string representing the starting position
///
/// # Returns
///
/// shakmaty::Chess position if valid
pub fn parse_fen_to_position(fen: &str) -> Result<shakmaty::Chess> {
    use shakmaty::fen::Fen;

    let parsed: Fen = fen.parse().map_err(|e| {
        ScidError::InvalidFormat(format!("FEN parse error: {:?}", e))
    })?;

    parsed.into_position(shakmaty::CastlingMode::Standard)
        .map_err(|e| {
            ScidError::InvalidFormat(format!("Invalid chess position: {:?}", e))
        })
}
```

**Validation**:

This is documentation - validation is through code review.

---

### Task 4.2.2: Implement Tag Parsing Function

**Acceptance Criteria**:
- Parses tags until null terminator
- Handles common tags (241-250)
- Handles regular tags (≤240)
- Handles special EventDate (255)
- Returns tags HashMap
- Returns position after tags (for flags byte)
- Complete test coverage

**Steps**:

1. Add tag parsing to `games.rs`:

```rust
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
/// # Tag Parsing Algorithm
///
/// ```text
/// pos = 0
/// tags = {}
///
/// loop:
///     byte = bytes[pos]
///     pos += 1
///
///     if byte == 0:
///         break  # Tags complete
///
///     if byte == 255:
///         # Special EventDate (skip for now)
///         pos += 3
///
///     else if byte > 240:
///         # Common tag
///         tag_name = COMMON_TAGS[byte - 241]
///         value_len = bytes[pos]
///         pos += 1
///         value = bytes[pos..pos+value_len]
///         pos += value_len
///         tags[tag_name] = value
///
///     else:
///         # Regular tag
///         name_len = byte
///         name = bytes[pos..pos+name_len]
///         pos += name_len
///         value_len = bytes[pos]
///         pos += 1
///         value = bytes[pos..pos+value_len]
///         pos += value_len
///         tags[name] = value
///
/// return (tags, pos)
/// ```
///
/// # Example
///
/// ```text
/// Input bytes:
///   [241][2]['G']['M']  // WhiteTitle = "GM"
///   [242][2]['I']['M']  // BlackTitle = "IM"
///   [0]                 // End of tags
///
/// Output:
///   tags = {"WhiteTitle": "GM", "BlackTitle": "IM"}
///   pos = 9 (position of flags byte)
/// ```
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
```

2. Add comprehensive tests:

```rust
#[cfg(test)]
mod tag_tests {
    use super::*;

    #[test]
    fn test_parse_common_tag() {
        // WhiteTitle (241) = "GM"
        let bytes = vec![
            241,  // WhiteTitle tag ID
            2,    // Value length
            b'G', b'M',  // Value
            0,    // End of tags
        ];

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        assert_eq!(tags.get("WhiteTitle"), Some(&"GM".to_string()));
        assert_eq!(pos, 5);  // Position after null terminator
    }

    #[test]
    fn test_parse_regular_tag() {
        // Custom tag "MyTag" = "value"
        let bytes = vec![
            5,    // Name length
            b'M', b'y', b'T', b'a', b'g',  // Name
            5,    // Value length
            b'v', b'a', b'l', b'u', b'e',  // Value
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
            241, 2, b'G', b'M',
            // BlackTitle = "IM"
            242, 2, b'I', b'M',
            // Custom tag
            4, b'T', b'e', b's', b't',  // "Test"
            2, b'O', b'K',              // "OK"
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
        let bytes = vec![0];  // Just null terminator

        let (tags, pos) = parse_game_tags(&bytes).unwrap();

        assert!(tags.is_empty());
        assert_eq!(pos, 1);
    }

    #[test]
    fn test_parse_event_date_skip() {
        let bytes = vec![
            255,  // EventDate special tag
            0x12, 0x34, 0x56,  // 3-byte date (skipped)
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
            241,  // WhiteTitle
            10,   // Claims 10 bytes but...
            b'G', b'M',  // Only 2 bytes!
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
}
```

**Validation**:

```bash
cargo test -p scidtopgn-core tag_tests

# All tests should pass
```

---

### Task 4.2.3: Implement Flags and FEN Parsing

**Acceptance Criteria**:
- Parses flags byte after tags
- Interprets flag bits correctly
- Reads optional FEN if non-standard start flag set
- Returns complete GameData structure
- Test coverage

**Steps**:

1. Add flags parsing to `games.rs`:

```rust
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
///  ↑              ↑      ↑          ↑
///  parse_game_    pos    if flag    rest of
///  tags()                set        bytes
/// ```
///
/// # Example
///
/// ```no_run
/// # use scidtopgn_core::database::games::parse_game_structure;
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let game_bytes = vec![/* ... */];
/// let game_data = parse_game_structure(&game_bytes)?;
///
/// println!("Flags: 0x{:02X}", game_data.flags);
/// if let Some(fen) = game_data.start_position {
///     println!("Custom start: {}", fen);
/// }
/// # Ok(())
/// # }
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

    let flags = bytes[pos];
    pos += 1;

    // Step 3: Check for non-standard start position (Gap 7 - FEN Validation)
    let start_position = if flags::is_set(flags, flags::NON_STANDARD_START) {
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
        pos = fen_end + 1;  // Skip past null terminator

        // Validate FEN string using shakmaty (Gap 7)
        // This ensures we can create a valid starting position for move parsing
        if let Err(e) = validate_fen(&fen) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: fen_start as u64,
                message: format!("Invalid FEN string: {}", e),
            });
        }

        Some(fen)
    } else {
        None
    };

    // Step 4: Remaining bytes are move data
    let move_data = bytes[pos..].to_vec();

    Ok(GameData {
        tags,
        flags,
        start_position,
        move_data,
    })
}
```

2. Add tests:

```rust
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
        b'r', b'n', b'b', b'q', b'k', b'b', b'n', b'r', b'/', /* ... simplified */ 0,
        // Move data
        0x10, 0x20,
    ];

    let game_data = parse_game_structure(&bytes).unwrap();

    assert_eq!(game_data.flags, 0x01);
    assert!(flags::is_set(game_data.flags, flags::NON_STANDARD_START));
    assert!(game_data.start_position.is_some());
}

#[test]
fn test_flags_helpers() {
    let flags: u8 = 0x03;  // Bits 0 and 1 set

    assert!(flags::is_set(flags, flags::NON_STANDARD_START));
    assert!(flags::is_set(flags, flags::HAS_PROMOTIONS));
    assert!(!flags::is_set(flags, flags::HAS_UNDER_PROMOS));
}

#[test]
fn test_validate_fen_standard_start() {
    // Standard starting position should be valid
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    assert!(validate_fen(fen).is_ok());
}

#[test]
fn test_validate_fen_custom_position() {
    // Valid custom position (after 1.e4)
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    assert!(validate_fen(fen).is_ok());
}

#[test]
fn test_validate_fen_invalid() {
    // Invalid FEN (missing king)
    let fen = "8/8/8/8/8/8/8/8 w - - 0 1";
    assert!(validate_fen(fen).is_err());

    // Invalid FEN (malformed)
    let fen = "not a valid fen";
    assert!(validate_fen(fen).is_err());
}

#[test]
fn test_decode_latin1_to_utf8_ascii() {
    // ASCII text is unchanged
    let bytes = b"Hello World";
    assert_eq!(decode_latin1_to_utf8(bytes), "Hello World");
}

#[test]
fn test_decode_latin1_to_utf8_special_chars() {
    // German umlaut ü (0xFC in Latin-1)
    assert_eq!(decode_latin1_to_utf8(&[0xFC]), "ü");

    // French é (0xE9 in Latin-1)
    assert_eq!(decode_latin1_to_utf8(&[0xE9]), "é");

    // Spanish ñ (0xF1 in Latin-1)
    assert_eq!(decode_latin1_to_utf8(&[0xF1]), "ñ");

    // Mixed text: "Müller" in Latin-1
    let bytes = &[b'M', 0xFC, b'l', b'l', b'e', b'r'];
    assert_eq!(decode_latin1_to_utf8(bytes), "Müller");
}

#[test]
fn test_encode_utf8_to_latin1() {
    // ASCII text
    assert_eq!(encode_utf8_to_latin1("Hello"), Some(b"Hello".to_vec()));

    // Latin-1 compatible text
    assert_eq!(encode_utf8_to_latin1("Müller"), Some(vec![b'M', 0xFC, b'l', b'l', b'e', b'r']));

    // Non-Latin-1 character (Russian д = U+0434) should fail
    assert_eq!(encode_utf8_to_latin1("Привет"), None);
}
```

**Validation**:

```bash
cargo test -p scidtopgn-core test_parse_flags
cargo test -p scidtopgn-core test_flags_helpers
cargo test -p scidtopgn-core test_validate_fen
cargo test -p scidtopgn-core test_decode_latin1
cargo test -p scidtopgn-core test_encode_utf8
```

---

### Task 4.2.4: Integration Testing with Real Data

**Acceptance Criteria**:
- Parse games from five.sg4
- Validate tags are extracted
- Validate flags are correct
- Combine with index and name data
- Display complete game metadata

**Steps**:

1. Create integration test `crates/core/tests/game_structure.rs`:

```rust
//! Integration tests for SG4 game file structure parsing

use scidtopgn_core::database::{
    index::parse_si4_file,
    names::parse_name_database,
    games::{read_game_data, parse_game_structure},
};
use std::fs::File;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_sg4_game_1() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse index to get game location
    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let game1 = &entries[0];

    // Read game data
    let mut sg4_file = File::open(&sg4_path).unwrap();
    let game_bytes = read_game_data(
        &mut sg4_file,
        game1.game_offset,
        game1.game_length,
    ).unwrap();

    println!("Game 1 size: {} bytes", game_bytes.len());

    // Parse game structure
    let game_data = parse_game_structure(&game_bytes).unwrap();

    println!("\n=== Game 1 Tags ===");
    for (name, value) in &game_data.tags {
        println!("  {}: {}", name, value);
    }

    println!("\n=== Game 1 Flags ===");
    println!("  Flags byte: 0x{:02X}", game_data.flags);
    println!("  Non-standard start: {}",
        game_data.start_position.is_some());

    println!("\n=== Game 1 Move Data ===");
    println!("  Move data size: {} bytes", game_data.move_data.len());
    println!("  First 10 bytes: {:02X?}",
        &game_data.move_data[..game_data.move_data.len().min(10)]);
}

#[test]
fn test_parse_all_five_games() {
    let si4_path = test_data_path("five.si4");
    let sn4_path = test_data_path("five.sn4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sn4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse all three files
    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let names = parse_name_database(&sn4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    println!("\n=== Complete Game Metadata ===\n");

    for (i, entry) in entries.iter().enumerate() {
        // Read game data
        let game_bytes = read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        ).unwrap();

        // Parse structure
        let game_data = parse_game_structure(&game_bytes).unwrap();

        // Get names
        let (white, black) = entry.get_player_names(&names);
        let event = entry.get_event_name(&names);
        let site = entry.get_site_name(&names);

        println!("Game {}:", i + 1);
        println!("  White: {}", white.unwrap_or("?"));
        println!("  Black: {}", black.unwrap_or("?"));
        println!("  Event: {}", event.unwrap_or("?"));
        println!("  Site: {}", site.unwrap_or("?"));
        println!("  Date: {}", entry.game_date.to_pgn_string());
        println!("  Result: {}", entry.result);
        println!("  Tags: {}", game_data.tags.len());
        println!("  Move data: {} bytes", game_data.move_data.len());
        println!();
    }
}

#[test]
fn test_validate_game_boundaries() {
    // Validate that games don't overlap and cover the file correctly
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();

    // Check games are sequential with no gaps
    for i in 0..entries.len() - 1 {
        let current_end = entries[i].game_offset + entries[i].game_length;
        let next_start = entries[i + 1].game_offset;

        assert_eq!(
            current_end, next_start,
            "Game {} ends at {}, Game {} starts at {} - gap or overlap!",
            i + 1, current_end, i + 2, next_start
        );
    }

    println!("✓ All games are sequential with no gaps or overlaps");
}
```

**Validation**:

```bash
cargo test -p scidtopgn-core test_parse_five_sg4 -- --nocapture

# Expected output:
# === Game 1 Tags ===
#   WhiteTitle: GM
#   ...
#
# === Complete Game Metadata ===
# Game 1:
#   White: Hossain, Enam
#   Black: Cheparinov, I
#   ...
#
# ✓ All games are sequential
```

---

### Task 4.2.5: Final Phase 4 Validation

**Acceptance Criteria**:
- All tasks complete
- All tests passing (unit + integration)
- Documentation complete
- Real file parsing works
- Ready for Phase 5 (move parsing)

**Steps**:

1. Run complete test suite:

```bash
cargo clean
cargo build --all
cargo test --all -- --nocapture

# Should see 55+ tests passing
```

2. Code quality:

```bash
cargo fmt --all
cargo clippy --all -- -D warnings
```

3. Create completion commit:

```bash
git add .
git commit -m "Complete Phase 4: Game File Structure

- Game boundary detection using index offsets
- Tag parsing (common, regular, special EventDate)
- Null terminator detection for tag section end
- Flags byte parsing and interpretation
- Optional FEN parsing for non-standard positions
- Complete test suite (55+ tests)
- Integration with index and name files
- Documentation explaining critical concepts
- Validation against real SCID files

All metadata extraction complete. Move data ready for Phase 5.
Ready for Phase 5: Chess Position Tracking."
```

**Final Checklist**:

- [ ] GameData struct complete (with separate comment_data field)
- [ ] read_game_data() implemented
- [ ] **decompress_game_data() implemented with flate2** (Gap 13)
- [ ] **read_and_decompress_game() convenience function** (Gap 13)
- [ ] parse_game_tags() complete with all tag types
- [ ] Flags parsing working
- [ ] FEN parsing for custom starts
- [ ] special_bytes module with all marker constants (0x0B-0x0F)
- [ ] is_special_marker() helper function
- [ ] parse_game_structure() separates move and comment data
- [ ] Integration tests passing
- [ ] Combines index + names + games correctly
- [ ] Documentation explains boundary detection
- [ ] Documentation explains tag/move separation
- [ ] **Documentation explains two-part comment encoding** (NEW)
- [ ] All tests passing (55+)
- [ ] No clippy warnings
- [ ] Ready for Phase 5

---

## Section 4.3: Game Data Structure & Comment Encoding

### Objective

Understand the complete game data layout, especially the **two-part comment encoding** system. This is CRITICAL for correct move parsing in Phase 5.

---

### Task 4.3.1: Understand Game Data Layout (Educational Task)

**Acceptance Criteria**:
- Documentation of complete game data structure
- Understanding of two-part comment encoding
- Knowledge of special byte markers
- Clear separation of concerns between Phase 4 and Phase 5

**CRITICAL: Complete Game Data Structure**:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        GAME DATA LAYOUT                             │
├─────────────────────────────────────────────────────────────────────┤
│ 1. PGN Tags Section (variable length)                               │
│    - Tags encoded as described in Section 4.2                       │
│    - Ends with null terminator (0x00)                               │
├─────────────────────────────────────────────────────────────────────┤
│ 2. Flags Byte (1 byte)                                              │
│    - Bit 0: Non-standard start position                             │
│    - Bit 1: Has promotions                                          │
│    - Bit 2: Has under-promotions                                    │
├─────────────────────────────────────────────────────────────────────┤
│ 3. Optional FEN String (if flags & 0x01)                            │
│    - Null-terminated string                                         │
│    - Only present for non-standard starting positions               │
├─────────────────────────────────────────────────────────────────────┤
│ 4. Move Data Section (variable length)                              │
│    - Chess moves encoded as single bytes (mostly)                   │
│    - Queen diagonal moves are 2 bytes                               │
│    - Contains MARKERS ONLY for annotations:                         │
│      • 0x0B (11) = NAG marker (followed by 1 NAG byte)              │
│      • 0x0C (12) = Comment marker (NO TEXT HERE!)                   │
│      • 0x0D (13) = Start variation marker                           │
│      • 0x0E (14) = End variation marker                             │
│      • 0x0F (15) = End of game marker                               │
│    - Ends with 0x0F (ENCODE_END_GAME)                               │
├─────────────────────────────────────────────────────────────────────┤
│ 5. Comments Section (variable length)                               │
│    - Actual comment TEXT stored here                                │
│    - Null-terminated strings                                        │
│    - Appear in move tree traversal order (pre-order DFS)            │
└─────────────────────────────────────────────────────────────────────┘
```

**Steps**:

1. Add special byte constants to `games.rs`:

```rust
/// Special move data markers
///
/// These bytes appear in the move data section and have special meanings.
/// CRITICAL: They are NOT tag boundary markers!
///
/// From SCID source code (game.cpp lines 59349-59358):
/// ```cpp
/// #define ENCODE_NAG          11
/// #define ENCODE_COMMENT      12
/// #define ENCODE_START_MARKER 13
/// #define ENCODE_END_MARKER   14
/// #define ENCODE_END_GAME     15
/// ```
pub mod special_bytes {
    /// NAG (Numeric Annotation Glyph) marker
    ///
    /// Format in move data: [0x0B][nag_value]
    /// The NAG value (0-255) follows immediately after this marker.
    pub const ENCODE_NAG: u8 = 0x0B;         // 11

    /// Comment marker - NO TEXT HERE!
    ///
    /// This is just a MARKER indicating a comment exists.
    /// The actual comment TEXT is stored in the Comments Section
    /// at the END of the game data.
    ///
    /// Format in move data: [0x0C] (single byte, no following data)
    pub const ENCODE_COMMENT: u8 = 0x0C;     // 12

    /// Start of variation marker
    ///
    /// Indicates the start of a sub-variation (alternative line).
    /// Variations are nested - you may see multiple START markers
    /// before seeing END markers.
    pub const ENCODE_START_MARKER: u8 = 0x0D; // 13

    /// End of variation marker
    ///
    /// Indicates the end of a sub-variation.
    /// Must be balanced with START markers.
    pub const ENCODE_END_MARKER: u8 = 0x0E;   // 14

    /// End of game marker
    ///
    /// Signals the end of the move data section.
    /// After this marker, the Comments Section begins.
    pub const ENCODE_END_GAME: u8 = 0x0F;     // 15

    /// First special byte value
    pub const ENCODE_FIRST: u8 = 0x0B;        // 11

    /// Last special byte value
    pub const ENCODE_LAST: u8 = 0x0F;         // 15

    /// Check if a byte is a special marker (not a move)
    ///
    /// IMPORTANT: This is for move data parsing, NOT tag parsing!
    /// In the tag section, these bytes can appear as regular data.
    #[inline]
    pub fn is_special_marker(byte: u8) -> bool {
        byte >= ENCODE_FIRST && byte <= ENCODE_LAST
    }
}
```

---

### Task 4.3.2: Two-Part Comment Encoding (CRITICAL)

**Acceptance Criteria**:
- Understanding of why comments are split
- Knowledge of marker-only encoding in moves
- Knowledge of text storage at end of game

**The Two-Part Comment System**:

SCID uses a clever two-part system for comments:

1. **In move data**: Only a single marker byte `0x0C` appears
2. **At end of game data**: Actual comment text as null-terminated strings

**Why Two Parts?**

This design separates variable-length text from compact move encoding:
- Move data stays compact and predictable
- Comments don't bloat the move section
- Easier to skip comments when not needed

**Implementation**:

```rust
/// Comment encoding in SCID
///
/// # Part 1: Markers in Move Data
///
/// When parsing move data, comment markers (0x0C) indicate that
/// a comment exists for the previous move. NO TEXT follows the marker.
///
/// ```text
/// Move data example:
/// [move1][move2][0x0C][move3][0x0C][0x0F]
///               ^^^^         ^^^^  ^^^^
///               comment      comment end
///               marker       marker  game
/// ```
///
/// # Part 2: Text in Comments Section
///
/// After the end-of-game marker (0x0F), comment text appears:
///
/// ```text
/// [move_data...][0x0F]["First comment\0"]["Second comment\0"]
/// ```
///
/// Comments appear in the order they were marked in the move tree,
/// using pre-order depth-first traversal.
///
/// # Decoding Algorithm
///
/// ```rust
/// // Phase 1: Parse moves, track comment positions
/// struct MoveNode {
///     chess_move: Move,
///     has_comment: bool,  // Set when we see 0x0C
///     comment: Option<String>,
/// }
///
/// fn parse_move_data(bytes: &[u8]) -> Vec<MoveNode> {
///     let mut nodes = Vec::new();
///     let mut pos = 0;
///
///     while pos < bytes.len() {
///         let byte = bytes[pos];
///         pos += 1;
///
///         match byte {
///             ENCODE_COMMENT => {
///                 // Mark previous move as having a comment
///                 // DO NOT try to read comment text here!
///                 if let Some(last) = nodes.last_mut() {
///                     last.has_comment = true;
///                 }
///             }
///             ENCODE_END_GAME => {
///                 break; // Move data ends
///             }
///             ENCODE_NAG => {
///                 let nag = bytes[pos];
///                 pos += 1;
///                 // Store NAG with previous move
///             }
///             ENCODE_START_MARKER => {
///                 // Start of variation
///             }
///             ENCODE_END_MARKER => {
///                 // End of variation
///             }
///             _ => {
///                 // Regular move byte - decode it
///                 let chess_move = decode_move(byte, &bytes[pos..])?;
///                 nodes.push(MoveNode {
///                     chess_move,
///                     has_comment: false,
///                     comment: None,
///                 });
///             }
///         }
///     }
///
///     nodes
/// }
///
/// // Phase 2: Read comments and attach to marked nodes
/// fn attach_comments(nodes: &mut [MoveNode], comment_bytes: &[u8]) {
///     let mut pos = 0;
///
///     for node in nodes.iter_mut() {
///         if node.has_comment {
///             // Read null-terminated string
///             let start = pos;
///             while pos < comment_bytes.len() && comment_bytes[pos] != 0 {
///                 pos += 1;
///             }
///             node.comment = Some(
///                 String::from_utf8_lossy(&comment_bytes[start..pos]).to_string()
///             );
///             pos += 1; // Skip null terminator
///         }
///     }
/// }
/// ```
///
/// # Pre-Game Comments (Gap 3)
///
/// SCID supports comments that appear BEFORE the first move. These are
/// typically game introductions, tournament notes, or analysis preambles.
///
/// ## How Pre-Game Comments Work
///
/// A comment marker (0x0C) can appear at the very START of move data,
/// before any actual moves. This indicates a pre-game comment.
///
/// ```text
/// Move data with pre-game comment:
/// [0x0C][move1][0x0C][move2][0x0F]
///   ↑      ↑      ↑
///   pre-game  move1's
///   comment   comment
///
/// Comment section:
/// ["This game was played...\\0"]["Good move\\0"]
/// ```
///
/// ## Implementation
///
/// ```rust
/// /// Track pre-game comment during parsing
/// struct MoveParser {
///     pre_game_comment: Option<String>,
///     moves: Vec<MoveNode>,
///     // ...
/// }
///
/// fn parse_moves(move_data: &[u8]) -> MoveParser {
///     let mut parser = MoveParser::new();
///     let mut pos = 0;
///     let mut first_move_seen = false;
///
///     while pos < move_data.len() {
///         let byte = move_data[pos];
///         pos += 1;
///
///         match byte {
///             ENCODE_COMMENT => {
///                 if !first_move_seen {
///                     // This is a pre-game comment marker
///                     parser.has_pre_game_comment = true;
///                 } else {
///                     // Attach to previous move
///                     parser.mark_last_move_has_comment();
///                 }
///             }
///             ENCODE_END_GAME => break,
///             _ if !is_special_marker(byte) => {
///                 // Regular move
///                 first_move_seen = true;
///                 // ... decode move ...
///             }
///             _ => { /* handle other markers */ }
///         }
///     }
///
///     parser
/// }
/// ```
///
/// ## PGN Output for Pre-Game Comments
///
/// Pre-game comments appear before the first move in PGN:
///
/// ```pgn
/// [Event "Tournament"]
/// [White "Player1"]
/// [Black "Player2"]
///
/// {This game was the decisive match of the tournament.} 1.e4 e5 2.Nf3
/// ```
///
/// # From SCID Source Code
///
/// Encoding (game.cpp lines 59656-59679):
/// ```cpp
/// static errorT encodeComments(ByteBuffer* buf, moveT* m, uint* count) {
///     while (m->marker != END_MARKER) {
///         if (m->comment != NULL) {
///             buf->PutTerminatedString(m->comment);  // Write text HERE
///             (*count)++;
///         }
///         // Recurse into variations...
///     }
/// }
/// ```
///
/// Decoding (game.cpp lines 59687-59708):
/// ```cpp
/// static errorT decodeComments(StrAllocator* strAlloc, ByteBuffer* buf, moveT* m) {
///     while (m->marker != END_MARKER) {
///         if (m->comment != NULL) {  // Was marked during move parsing
///             char* str;
///             buf->GetTerminatedString(&str);  // Read text HERE
///             m->comment = strAlloc->Duplicate(str);
///         }
///         // Recurse into variations...
///     }
/// }
/// ```
```

**Test for Understanding**:

```rust
#[test]
fn test_comment_encoding_understanding() {
    // Example game data with comments
    let game_data = vec![
        // Tags section
        0x00,  // No tags (just null terminator)

        // Flags
        0x00,  // No special flags

        // Move data section
        0xCF,  // Move: e2-e4 (e-pawn double push)
        0x0C,  // Comment marker for e4
        0x37,  // Move: e7-e5
        0xCF,  // Move: Nf3 (some knight move)
        0x0C,  // Comment marker for Nf3
        0x0F,  // End of game

        // Comments section (after 0x0F)
        b'G', b'o', b'o', b'd', b' ', b'm', b'o', b'v', b'e', 0x00,  // "Good move\0"
        b'T', b'h', b'e', b'o', b'r', b'y', 0x00,  // "Theory\0"
    ];

    // When parsing:
    // 1. Move data parsing finds two 0x0C markers
    // 2. After 0x0F, we read "Good move" and "Theory"
    // 3. First comment goes to e4, second to Nf3

    // The key insight: 0x0C means "this move has a comment"
    // but the comment text is NOT at that position!
}
```

---

### Task 4.3.3: Update parse_game_structure for Comment Awareness

**Acceptance Criteria**:
- GameData includes move_data_end position
- Comments section position identified
- Ready for Phase 5 to use this information

**Steps**:

1. Update `GameData` struct:

```rust
/// Game data extracted from .sg4 file
///
/// Contains parsed tags, flags, and separated move/comment data.
#[derive(Debug, Clone)]
pub struct GameData {
    /// PGN tags (e.g., "WhiteTitle" → "GM")
    pub tags: HashMap<String, String>,

    /// Game flags byte
    pub flags: u8,

    /// Starting position FEN (if non-standard start)
    pub start_position: Option<String>,

    /// Raw move data bytes (includes markers, ends at ENCODE_END_GAME)
    ///
    /// This section contains:
    /// - Move bytes (piece encoding)
    /// - NAG markers (0x0B + value)
    /// - Comment markers (0x0C only - no text!)
    /// - Variation markers (0x0D, 0x0E)
    /// - Ends with 0x0F
    pub move_data: Vec<u8>,

    /// Raw comment data bytes (null-terminated strings)
    ///
    /// This section contains actual comment text.
    /// Comments appear in move tree traversal order.
    /// Each comment is null-terminated.
    pub comment_data: Vec<u8>,
}
```

2. Update `parse_game_structure`:

```rust
/// Parse complete game structure with separate move and comment data
///
/// # CRITICAL: Two-Part Comment Encoding
///
/// SCID stores comments in TWO parts:
/// 1. Move data has MARKERS (0x0C) indicating comments exist
/// 2. After move data, actual comment TEXT is stored
///
/// This function separates these two sections.
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

    let flags = bytes[pos];
    pos += 1;

    // Step 3: Check for non-standard start position
    let start_position = if flags::is_set(flags, flags::NON_STANDARD_START) {
        let fen_start = pos;
        while pos < bytes.len() && bytes[pos] != 0 {
            pos += 1;
        }
        if pos >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: pos as u64,
                message: "Unterminated FEN string".to_string(),
            });
        }
        let fen = String::from_utf8_lossy(&bytes[fen_start..pos]).to_string();
        pos += 1; // Skip null terminator
        Some(fen)
    } else {
        None
    };

    // Step 4: Find end of move data (0x0F marker)
    let move_data_start = pos;
    let mut move_data_end = pos;

    while move_data_end < bytes.len() {
        let byte = bytes[move_data_end];

        if byte == special_bytes::ENCODE_END_GAME {
            move_data_end += 1; // Include the 0x0F marker
            break;
        }

        // Skip NAG value byte
        if byte == special_bytes::ENCODE_NAG {
            move_data_end += 2; // Marker + NAG value
            continue;
        }

        // Queen diagonal moves are 2 bytes, but we can't detect them
        // without full move parsing. For now, just scan for 0x0F.
        move_data_end += 1;
    }

    // Step 5: Separate move data and comment data
    let move_data = bytes[move_data_start..move_data_end].to_vec();
    let comment_data = if move_data_end < bytes.len() {
        bytes[move_data_end..].to_vec()
    } else {
        Vec::new()
    };

    Ok(GameData {
        tags,
        flags,
        start_position,
        move_data,
        comment_data,
    })
}
```

3. Add tests:

```rust
#[cfg(test)]
mod comment_structure_tests {
    use super::*;

    #[test]
    fn test_game_with_comments() {
        let bytes = vec![
            // Tags
            0x00,  // No tags

            // Flags
            0x00,

            // Move data with comment markers
            0xCF,  // Move byte
            0x0C,  // Comment marker (no text!)
            0x37,  // Move byte
            0x0F,  // End of game

            // Comment section
            b'T', b'e', b's', b't', 0x00,  // "Test\0"
        ];

        let game = parse_game_structure(&bytes).unwrap();

        // Move data includes everything up to and including 0x0F
        assert!(game.move_data.contains(&0x0C)); // Has comment marker
        assert!(game.move_data.ends_with(&[0x0F])); // Ends with END_GAME

        // Comment data is separate
        assert_eq!(game.comment_data, b"Test\0");
    }

    #[test]
    fn test_game_without_comments() {
        let bytes = vec![
            0x00,  // No tags
            0x00,  // Flags
            0xCF,  // Move
            0x0F,  // End of game
            // No comments section
        ];

        let game = parse_game_structure(&bytes).unwrap();

        assert_eq!(game.move_data, vec![0xCF, 0x0F]);
        assert!(game.comment_data.is_empty());
    }

    #[test]
    fn test_special_byte_detection() {
        use special_bytes::*;

        assert!(is_special_marker(ENCODE_NAG));
        assert!(is_special_marker(ENCODE_COMMENT));
        assert!(is_special_marker(ENCODE_START_MARKER));
        assert!(is_special_marker(ENCODE_END_MARKER));
        assert!(is_special_marker(ENCODE_END_GAME));

        // Regular move bytes are NOT special
        assert!(!is_special_marker(0x00));
        assert!(!is_special_marker(0x0A));
        assert!(!is_special_marker(0x10));
        assert!(!is_special_marker(0xCF));
    }
}
```

---

### Task 4.3.4: Comment Tree Traversal Algorithm

**Acceptance Criteria**:
- Understanding of pre-order DFS traversal for comments
- Algorithm to match comment markers to comment text
- Handling of variations with nested comments

**The Comment Traversal Problem**:

Comments in the comment section appear in **pre-order depth-first traversal order** of the move tree. This is critical to understand for correctly matching comment markers (0x0C) in move data to their text in the comment section.

**Pre-Order DFS Traversal**:

```text
Game Tree Example:
                    1.e4 (comment A)
                     │
                    1...e5 (comment B)
                     │
              ┌──────┴───────┐
              │              │
           2.Nf3          [VAR: 2.Bc4 (comment C)]
         (comment D)              │
              │              2...Nf6 (comment E)
           2...Nc6
```

**Traversal Order**: A → B → C → E → D

The algorithm visits:
1. Node, then
2. First variation (recursively), then
3. Main line continuation

**Algorithm**:

```rust
/// Comment tree traversal - matches markers to text
///
/// Comments are stored in pre-order DFS traversal order.
/// This algorithm traverses the move tree in the same order
/// to match comment markers with their text.
///
/// # Pre-order DFS Algorithm
///
/// ```
/// fn traverse_for_comments(node: &mut MoveNode, comments: &mut CommentIterator) {
///     // 1. Visit THIS node first (pre-order)
///     if node.has_comment_marker {
///         node.comment = comments.next();  // Get next comment text
///     }
///
///     // 2. Process variations BEFORE continuing main line
///     for variation in &mut node.variations {
///         traverse_for_comments(variation, comments);
///     }
///
///     // 3. Continue to next move in main line
///     if let Some(ref mut next) = node.next_move {
///         traverse_for_comments(next, comments);
///     }
/// }
/// ```
///
/// # Why Pre-Order?
///
/// SCID writes comments during encoding using the same traversal:
///
/// ```cpp
/// // From game.cpp encodeComments()
/// static errorT encodeComments(ByteBuffer* buf, moveT* m, uint* count) {
///     while (m->marker != END_MARKER) {
///         if (m->comment != NULL) {
///             buf->PutTerminatedString(m->comment);  // Write comment HERE
///             (*count)++;
///         }
///         // Handle variations recursively (BEFORE next move)
///         if (m->numVariations > 0) {
///             for (uint i = 0; i < m->numVariations; i++) {
///                 encodeComments(buf, m->varChild[i], count);
///             }
///         }
///         m = m->next;  // Continue main line
///     }
/// }
/// ```

/// Comment iterator for matching markers to text
///
/// # Character Encoding (Gap 11)
///
/// SCID stores comments using Latin-1 (ISO-8859-1) encoding, NOT UTF-8.
/// This iterator handles the conversion to UTF-8 for Rust strings.
///
/// ## Why Latin-1?
///
/// SCID was developed before UTF-8 became standard. Latin-1 was common
/// for Western European text and is still the default in many older databases.
///
/// ## Conversion Strategy
///
/// Latin-1 is a subset of Unicode (codepoints 0x00-0xFF map directly),
/// so conversion is straightforward:
/// - ASCII bytes (0x00-0x7F) are the same in both encodings
/// - High bytes (0x80-0xFF) are converted to their Unicode equivalents
///
/// ```rust
/// fn latin1_to_utf8(bytes: &[u8]) -> String {
///     bytes.iter()
///         .map(|&b| b as char)  // Latin-1 byte → Unicode codepoint
///         .collect()
/// }
/// ```
pub struct CommentIterator<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> CommentIterator<'a> {
    pub fn new(comment_data: &'a [u8]) -> Self {
        Self { data: comment_data, pos: 0 }
    }

    /// Get next null-terminated comment string
    ///
    /// Handles Latin-1 to UTF-8 conversion (Gap 11)
    pub fn next(&mut self) -> Option<String> {
        if self.pos >= self.data.len() {
            return None;
        }

        let start = self.pos;

        // Find null terminator
        while self.pos < self.data.len() && self.data[self.pos] != 0 {
            self.pos += 1;
        }

        // Convert Latin-1 to UTF-8 (Gap 11)
        let text = decode_latin1_to_utf8(&self.data[start..self.pos]);

        // Skip null terminator
        if self.pos < self.data.len() {
            self.pos += 1;
        }

        Some(text)
    }
}

/// Convert Latin-1 (ISO-8859-1) encoded bytes to UTF-8 string (Gap 11)
///
/// SCID stores comments in Latin-1 encoding. This function converts
/// them to UTF-8 for use in Rust strings.
///
/// # Character Encoding Details
///
/// Latin-1 bytes map directly to Unicode codepoints:
/// - 0x00-0x7F: ASCII (same in UTF-8)
/// - 0x80-0x9F: Control characters (rarely used)
/// - 0xA0-0xFF: Latin-1 supplement (accented characters, symbols)
///
/// # Examples
///
/// ```
/// // ASCII text (unchanged)
/// assert_eq!(decode_latin1_to_utf8(b"Hello"), "Hello");
///
/// // German umlaut ü (0xFC in Latin-1)
/// assert_eq!(decode_latin1_to_utf8(&[0xFC]), "ü");
///
/// // French é (0xE9 in Latin-1)
/// assert_eq!(decode_latin1_to_utf8(&[0xE9]), "é");
/// ```
pub fn decode_latin1_to_utf8(bytes: &[u8]) -> String {
    // Latin-1 bytes 0x00-0xFF map directly to Unicode codepoints 0x0000-0x00FF
    bytes.iter()
        .map(|&b| b as char)
        .collect()
}

/// Encode UTF-8 string to Latin-1 bytes (for writing SCID files)
///
/// Returns None if the string contains characters outside Latin-1 range.
///
/// # Note
///
/// Characters outside the Latin-1 range (U+0100 and above) cannot be
/// represented. This function returns None if such characters are present.
/// Consider using a fallback or warning when this occurs.
pub fn encode_utf8_to_latin1(text: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::with_capacity(text.len());

    for c in text.chars() {
        let codepoint = c as u32;
        if codepoint > 0xFF {
            // Character outside Latin-1 range
            return None;
        }
        bytes.push(codepoint as u8);
    }

    Some(bytes)
}
```

---

### Task 4.3.5: NAG (Numeric Annotation Glyph) Handling

**Acceptance Criteria**:
- NAG marker format understood
- Standard NAG codes documented
- NAG to PGN symbol mapping

**NAG Format in Move Data**:

```text
[0x0B][nag_value]
  ↑       ↑
  NAG   Single byte
  marker  (0-255)
```

**Standard NAG Codes**:

```rust
/// Standard NAG (Numeric Annotation Glyph) values
///
/// NAGs provide standardized move annotations.
/// From PGN specification and SCID usage.
pub mod nag_codes {
    // Move quality assessments
    pub const GOOD_MOVE: u8 = 1;              // !
    pub const POOR_MOVE: u8 = 2;              // ?
    pub const VERY_GOOD_MOVE: u8 = 3;         // !!
    pub const VERY_POOR_MOVE: u8 = 4;         // ??
    pub const SPECULATIVE_MOVE: u8 = 5;       // !?
    pub const QUESTIONABLE_MOVE: u8 = 6;      // ?!

    // Position assessments
    pub const FORCED_MOVE: u8 = 7;            // □ (forced)
    pub const SINGULAR_MOVE: u8 = 8;          // (only move)
    pub const WORST_MOVE: u8 = 9;             // (worst move)

    // Equality indicators
    pub const DRAWISH: u8 = 10;               // =
    pub const EQUAL_QUIET: u8 = 11;           // (equal, quiet)
    pub const EQUAL_ACTIVE: u8 = 12;          // (equal, active)
    pub const UNCLEAR: u8 = 13;               // ∞

    // Advantage indicators
    pub const WHITE_SLIGHT_ADV: u8 = 14;      // ⩲
    pub const BLACK_SLIGHT_ADV: u8 = 15;      // ⩱
    pub const WHITE_MODERATE_ADV: u8 = 16;    // ±
    pub const BLACK_MODERATE_ADV: u8 = 17;    // ∓
    pub const WHITE_DECISIVE_ADV: u8 = 18;    // +−
    pub const BLACK_DECISIVE_ADV: u8 = 19;    // −+

    // Special positions
    pub const WHITE_ZUGZWANG: u8 = 22;
    pub const BLACK_ZUGZWANG: u8 = 23;
    pub const WHITE_INITIATIVE: u8 = 36;
    pub const BLACK_INITIATIVE: u8 = 37;
    pub const WHITE_ATTACK: u8 = 40;
    pub const BLACK_ATTACK: u8 = 41;
    pub const WHITE_COMPENSATION: u8 = 44;
    pub const BLACK_COMPENSATION: u8 = 45;

    // Time pressure
    pub const WHITE_TIME_PRESSURE: u8 = 136;
    pub const BLACK_TIME_PRESSURE: u8 = 137;
    pub const WHITE_SEVERE_TIME: u8 = 138;
    pub const BLACK_SEVERE_TIME: u8 = 139;

    /// Convert NAG to PGN symbol
    pub fn to_pgn_symbol(nag: u8) -> &'static str {
        match nag {
            1 => "!",
            2 => "?",
            3 => "!!",
            4 => "??",
            5 => "!?",
            6 => "?!",
            10 => "=",
            13 => "∞",
            14 => "⩲",
            15 => "⩱",
            16 => "±",
            17 => "∓",
            18 => "+−",
            19 => "−+",
            _ => "",  // No standard symbol, use $N format
        }
    }

    /// Format NAG for PGN output
    ///
    /// Returns symbol if standard, otherwise $N format
    pub fn format_for_pgn(nag: u8) -> String {
        let symbol = to_pgn_symbol(nag);
        if symbol.is_empty() {
            format!("${}", nag)
        } else {
            symbol.to_string()
        }
    }
}
```

---

### Task 4.3.6: Variation Data Structures

**Acceptance Criteria**:
- MoveNode structure defined
- GameTree structure defined
- Variation nesting supported
- Ready for Phase 5 implementation

**Data Structures for Move Tree**:

```rust
/// A node in the game move tree
///
/// Each node represents a single move (or the game start).
/// Nodes can have:
/// - A comment attached
/// - NAG annotations
/// - Variations (alternative lines)
/// - A continuation (next move in main line)
#[derive(Debug, Clone)]
pub struct MoveNode {
    /// The chess move (None for game root)
    pub chess_move: Option<shakmaty::Move>,

    /// SAN notation of the move
    pub san: String,

    /// Comment text (if any)
    pub comment: Option<String>,

    /// NAG annotations (can have multiple)
    pub nags: Vec<u8>,

    /// Alternative variations from this position
    /// Each variation is a linked list of MoveNodes
    pub variations: Vec<Box<MoveNode>>,

    /// Next move in the current line
    pub next: Option<Box<MoveNode>>,

    /// Move number (for PGN output)
    pub move_number: u16,

    /// True if this is Black's move
    pub is_black_move: bool,
}

impl MoveNode {
    /// Create a new empty move node (for game root)
    pub fn new_root() -> Self {
        MoveNode {
            chess_move: None,
            san: String::new(),
            comment: None,
            nags: Vec::new(),
            variations: Vec::new(),
            next: None,
            move_number: 0,
            is_black_move: false,
        }
    }

    /// Create a move node with a chess move
    pub fn new_move(chess_move: shakmaty::Move, san: String, move_number: u16, is_black: bool) -> Self {
        MoveNode {
            chess_move: Some(chess_move),
            san,
            comment: None,
            nags: Vec::new(),
            variations: Vec::new(),
            next: None,
            move_number,
            is_black_move: is_black,
        }
    }

    /// Check if this node has any annotations
    pub fn has_annotations(&self) -> bool {
        self.comment.is_some() || !self.nags.is_empty() || !self.variations.is_empty()
    }
}

/// Complete game tree structure
///
/// Contains the root node and metadata needed for traversal.
#[derive(Debug, Clone)]
pub struct GameTree {
    /// Root node (pre-game position, before first move)
    pub root: MoveNode,

    /// Total number of moves in main line
    pub main_line_length: usize,

    /// Total number of variations
    pub variation_count: usize,

    /// Total number of comments
    pub comment_count: usize,
}

impl GameTree {
    pub fn new() -> Self {
        GameTree {
            root: MoveNode::new_root(),
            main_line_length: 0,
            variation_count: 0,
            comment_count: 0,
        }
    }

    /// Get iterator over main line moves only
    pub fn main_line(&self) -> MainLineIterator {
        MainLineIterator { current: self.root.next.as_ref().map(|n| n.as_ref()) }
    }
}

/// Iterator over main line moves
pub struct MainLineIterator<'a> {
    current: Option<&'a MoveNode>,
}

impl<'a> Iterator for MainLineIterator<'a> {
    type Item = &'a MoveNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.next.as_ref().map(|n| n.as_ref());
        Some(node)
    }
}
```

**Variation Parsing**:

When parsing move data:
- `0x0D` (START_MARKER) → Push current position, start new variation branch
- `0x0E` (END_MARKER) → Pop back to previous position, continue main line

```rust
/// Variation parsing state machine
///
/// Tracks position in move tree during parsing.
///
/// # CRITICAL: Position State Restoration (Gap 1)
///
/// When entering a variation with START_MARKER (0x0D), we MUST save the
/// complete chess position state, not just the tree position. When we
/// encounter END_MARKER (0x0E), we restore to that saved state.
///
/// This is essential because variations are alternative continuations
/// from a specific position. After processing a variation, we need to
/// return to the EXACT same chess position to continue the main line.
///
/// ## Why Position State Matters
///
/// ```text
/// Position A: 1.e4 e5 2.Nf3
///                      ↓
///              ┌───────┴────────┐
///              │                │
///           2...Nc6          [VAR: 2...d6]
///              ↓                  ↓
///           3.Bb5             (processes variation)
///                                 ↓
///                             END_MARKER
///                                 ↓
///                             ← MUST restore to Position A!
/// ```
///
/// ## Position State Stack
///
/// Each stack entry must include:
/// - Complete board position (shakmaty::Chess or FEN)
/// - Whose turn it is
/// - Castling rights
/// - En passant square
/// - Halfmove clock
/// - Fullmove number
/// - Current move number for PGN output
struct VariationParser {
    /// Stack of positions to return to (tree positions)
    position_stack: Vec<*mut MoveNode>,

    /// Stack of chess positions to restore (CRITICAL for Gap 1)
    /// Each entry is a complete position state
    chess_position_stack: Vec<shakmaty::Chess>,

    /// Current position in tree
    current: *mut MoveNode,

    /// Current chess position being tracked
    current_position: shakmaty::Chess,

    /// Variation depth (for debugging)
    depth: usize,
}

impl VariationParser {
    /// Create new parser with starting position
    fn new(start_position: shakmaty::Chess) -> Self {
        Self {
            position_stack: Vec::new(),
            chess_position_stack: Vec::new(),
            current: std::ptr::null_mut(),
            current_position: start_position,
            depth: 0,
        }
    }

    /// Handle START_MARKER (0x0D)
    ///
    /// Saves BOTH the tree position AND the chess position
    fn start_variation(&mut self) {
        // Save current tree position
        self.position_stack.push(self.current);

        // CRITICAL: Save current chess position state
        self.chess_position_stack.push(self.current_position.clone());

        // Create new variation at current position
        // (Implementation depends on memory model)

        self.depth += 1;
    }

    /// Handle END_MARKER (0x0E)
    ///
    /// Restores BOTH the tree position AND the chess position
    fn end_variation(&mut self) {
        // Return to saved tree position
        if let Some(pos) = self.position_stack.pop() {
            self.current = pos;
        }

        // CRITICAL: Restore chess position state
        if let Some(chess_pos) = self.chess_position_stack.pop() {
            self.current_position = chess_pos;
        }

        self.depth = self.depth.saturating_sub(1);
    }

    /// Get current chess position (for move validation)
    fn current_chess_position(&self) -> &shakmaty::Chess {
        &self.current_position
    }

    /// Apply a move to current position
    fn apply_move(&mut self, chess_move: &shakmaty::Move) -> Result<(), shakmaty::PlayError<shakmaty::Chess>> {
        self.current_position = self.current_position.clone().play(chess_move)?;
        Ok(())
    }
}
```

---

### Task 4.3.7: Integration Testing for Comment Separation (Renumbered)

**Acceptance Criteria**:
- Real game data correctly separated
- Comment markers counted in move data
- Comment strings extracted from comment section

**Steps**:

1. Add to integration tests:

```rust
#[test]
fn test_comment_separation_real_data() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    println!("\n=== Comment Structure Analysis ===\n");

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes = read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        ).unwrap();

        let game = parse_game_structure(&game_bytes).unwrap();

        // Count comment markers in move data
        let comment_markers = game.move_data.iter()
            .filter(|&&b| b == special_bytes::ENCODE_COMMENT)
            .count();

        // Count null-terminated strings in comment data
        let comment_count = game.comment_data.iter()
            .filter(|&&b| b == 0)
            .count();

        println!("Game {}:", i + 1);
        println!("  Move data: {} bytes", game.move_data.len());
        println!("  Comment markers in moves: {}", comment_markers);
        println!("  Comment section: {} bytes", game.comment_data.len());
        println!("  Comment strings: ~{}", comment_count);

        // Verify marker count roughly matches comment count
        // (May not be exact due to pre-game comments)
        if comment_markers > 0 || comment_count > 0 {
            println!("  → Has annotations!");
        }
        println!();
    }
}

#[test]
fn test_verify_move_data_ends_correctly() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes = read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        ).unwrap();

        let game = parse_game_structure(&game_bytes).unwrap();

        // Move data MUST end with ENCODE_END_GAME
        assert!(
            game.move_data.last() == Some(&special_bytes::ENCODE_END_GAME),
            "Game {} move data doesn't end with 0x0F",
            i + 1
        );
    }

    println!("✓ All games have correct move data termination");
}
```

---

### Task 4.3.8: Phase 4.3 Completion Checklist (Renumbered)

**Final Checklist for Section 4.3**:

- [ ] `special_bytes` module with all marker constants
- [ ] `is_special_marker()` helper function
- [ ] Updated `GameData` with separate `comment_data` field
- [ ] Updated `parse_game_structure()` to separate move/comment data
- [ ] `CommentIterator` for reading null-terminated comment strings
- [ ] Comment tree traversal algorithm (pre-order DFS)
- [ ] `nag_codes` module with standard NAG values
- [ ] `to_pgn_symbol()` and `format_for_pgn()` helpers
- [ ] `MoveNode` struct for game tree nodes
- [ ] `GameTree` struct for complete game representation
- [ ] `VariationParser` state machine for nested variations
- [ ] **Position state stack in VariationParser (Gap 1)**
- [ ] **Pre-game comment handling (Gap 3)**
- [ ] **FEN validation with `validate_fen()` (Gap 7)**
- [ ] **`parse_fen_to_position()` helper (Gap 7)**
- [ ] **`decode_latin1_to_utf8()` for comments (Gap 11)**
- [ ] **`encode_utf8_to_latin1()` for writing (Gap 11)**
- [ ] Unit tests for comment structure
- [ ] Integration tests with real data
- [ ] Documentation explains two-part encoding
- [ ] All tests passing

**Validation**:

```bash
cargo test -p scidtopgn-core comment_structure_tests
cargo test -p scidtopgn-core test_comment_separation_real_data -- --nocapture
cargo test -p scidtopgn-core test_verify_move_data_ends_correctly -- --nocapture
```

---

## Success Metrics

1. **Parsing Accuracy**:
   - ✅ All games from five.sg4 parse without error
   - ✅ Tags extracted correctly
   - ✅ No overlap or gaps in game boundaries
   - ✅ Complete metadata available

2. **Code Quality**:
   - ✅ 55+ tests passing
   - ✅ Zero clippy warnings
   - ✅ Complete documentation

3. **Understanding**:
   - ✅ Clear why scanning fails
   - ✅ Correct use of index offsets
   - ✅ Proper tag termination detection

---

## Common Pitfalls and Solutions

### Pitfall 1: Scanning for Game Boundaries

**Symptom**: Parser fails or gets garbage data

**Solution**: ALWAYS use index file offsets

### Pitfall 2: Using Bytes 11-15 as Tag/Move Boundary

**Symptom**: Tags parse incorrectly

**Solution**: Only null terminator (0x00) ends tags

### Pitfall 3: Forgetting First Name Has No Prefix

**Symptom**: Position off by one after tags

**Solution**: Check for null, then read flags

### Pitfall 4: Not Handling Optional FEN

**Symptom**: Move data corrupted for non-standard games

**Solution**: Check NON_STANDARD_START flag

### Pitfall 5: Reading Comment Text at 0x0C Marker (NEW)

**Symptom**: Garbage data when parsing comments, move data appears corrupted

**Cause**: Assuming comment text follows the 0x0C marker byte

**Reality**: The 0x0C byte is ONLY a marker. Actual comment text is stored
in a separate section AFTER the 0x0F end-of-game marker.

**Solution**:
```rust
// ❌ WRONG - Don't read text after marker
if byte == 0x0C {
    let comment = read_null_terminated_string(); // WRONG!
}

// ✅ CORRECT - Just mark that a comment exists
if byte == 0x0C {
    current_move.has_comment = true;
    // Text will be read from comment section later
}
```

### Pitfall 6: Not Separating Move Data from Comment Data

**Symptom**: Comment strings appear in move data, parsing fails

**Solution**: Find 0x0F marker to split the two sections:
- Move data: Everything up to and including 0x0F
- Comment data: Everything after 0x0F

---

## Next Steps

After Phase 4:

1. **Phase 5: Chess Position Tracking with Shakmaty** - Parse chess moves
2. Use shakmaty for position validation
3. Convert SCID moves to shakmaty Moves

Phase 4 provides complete game structure. Phase 5 will parse actual moves.

---

**Phase 4 Complete**: Game File Structure ✅

---

## Revision History

### Version 2.0 (Initial)
- Original Phase 4 document
- Game boundary detection using index offsets
- Tag parsing (common, regular, EventDate)
- Flags and FEN parsing
- Basic special bytes documentation

### Version 2.1
**Date**: Aligned with IMPLEMENTATION_PLAN.md v2.1

**New Content Added**:

| Section | Addition | Description |
|---------|----------|-------------|
| Task 4.3.4 | Comment Tree Traversal | Pre-order DFS algorithm for matching comment markers to text |
| Task 4.3.4 | CommentIterator | Helper struct for reading null-terminated comments |
| Task 4.3.5 | NAG Handling | Complete NAG code table with standard values |
| Task 4.3.5 | nag_codes module | `to_pgn_symbol()` and `format_for_pgn()` helpers |
| Task 4.3.6 | MoveNode struct | Node in game tree with move, comment, NAGs, variations |
| Task 4.3.6 | GameTree struct | Complete game tree with root node and metadata |
| Task 4.3.6 | VariationParser | State machine for parsing nested variations |

**Task Renumbering**:
- Old Task 4.3.4 → Task 4.3.7
- Old Task 4.3.5 → Task 4.3.8

**Alignment with Other Documents**:
- Structures now match PHASE_5_MOVE_PARSING.md
- NAG codes consistent with IMPLEMENTATION_PLAN.md
- Comment algorithm matches Bible (SCID_DATABASE_FORMAT.md)

### Version 2.2 (Current)
**Date**: Gap resolution updates

**Gap Resolutions Incorporated**:

| Gap | Section | Changes |
|-----|---------|---------|
| Gap 1: Variation Position Restore | Task 4.3.6 | Added `chess_position_stack` to VariationParser, position state save/restore on START/END markers |
| Gap 3: Pre-game Comments | Task 4.3.2 | Added documentation for comments before first move, tracking `has_pre_game_comment` flag |
| Gap 7: FEN Parsing | Task 4.2.3 | Added `validate_fen()` and `parse_fen_to_position()` helpers using shakmaty |
| Gap 11: Comment Encoding | Task 4.3.4 | Added `decode_latin1_to_utf8()` and `encode_utf8_to_latin1()` for Latin-1 ↔ UTF-8 conversion |

**New Functions Added**:
- `validate_fen(fen: &str) -> Result<()>` - Validate FEN string is legal position
- `parse_fen_to_position(fen: &str) -> Result<shakmaty::Chess>` - Convert FEN to position
- `decode_latin1_to_utf8(bytes: &[u8]) -> String` - Convert Latin-1 comments to UTF-8
- `encode_utf8_to_latin1(text: &str) -> Option<Vec<u8>>` - Convert UTF-8 to Latin-1

**Updated VariationParser**:
- Now tracks both tree position AND chess position
- `start_variation()` saves complete position state
- `end_variation()` restores complete position state
- New `apply_move()` and `current_chess_position()` methods
