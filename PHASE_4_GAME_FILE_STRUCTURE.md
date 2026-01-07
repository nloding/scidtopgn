# Phase 4: Game File Structure - Detailed Implementation Plan

**Timeline**: Week 3 (5-7 days)
**Prerequisites**: Phase 2 complete (index parsing), Phase 3 complete (name parsing)
**Dependencies**: Index file offsets, binary parsing

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
- ✅ Parse PGN tags correctly (common and regular tags)
- ✅ Detect tag section end (null terminator)
- ✅ Parse game flags byte
- ✅ Handle optional FEN strings
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

---

## Test Data Reference

We will validate against `test/data/five.sg4`:

**Expected Structure** (Game 1):
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

    // Step 3: Check for non-standard start position
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
```

**Validation**:

```bash
cargo test -p scidtopgn-core test_parse_flags
cargo test -p scidtopgn-core test_flags_helpers
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
        .join("../../test/data")
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

# Should see 40+ tests passing
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
- Complete test suite (40+ tests)
- Integration with index and name files
- Documentation explaining critical concepts
- Validation against real SCID files

All metadata extraction complete. Move data ready for Phase 5.
Ready for Phase 5: Chess Position Tracking."
```

**Final Checklist**:

- [ ] GameData struct complete
- [ ] read_game_data() implemented
- [ ] parse_game_tags() complete with all tag types
- [ ] Flags parsing working
- [ ] FEN parsing for custom starts
- [ ] Integration tests passing
- [ ] Combines index + names + games correctly
- [ ] Documentation explains boundary detection
- [ ] Documentation explains tag/move separation
- [ ] All tests passing (40+)
- [ ] No clippy warnings
- [ ] Ready for Phase 5

---

## Success Metrics

1. **Parsing Accuracy**:
   - ✅ All games from five.sg4 parse without error
   - ✅ Tags extracted correctly
   - ✅ No overlap or gaps in game boundaries
   - ✅ Complete metadata available

2. **Code Quality**:
   - ✅ 40+ tests passing
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

---

## Next Steps

After Phase 4:

1. **Phase 5: Chess Position Tracking with Shakmaty** - Parse chess moves
2. Use shakmaty for position validation
3. Convert SCID moves to shakmaty Moves

Phase 4 provides complete game structure. Phase 5 will parse actual moves.

---

**Phase 4 Complete**: Game File Structure ✅
