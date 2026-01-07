# Phase 9A: Unit Tests - Comprehensive Testing Framework

## Overview

Unit testing is the foundation of software quality assurance. For a binary parser like our SCID to PGN converter, comprehensive unit tests are absolutely critical because:

1. **Binary Format Correctness**: A single bit error in parsing can cascade into completely incorrect game data
2. **Edge Case Coverage**: Chess has many special cases (en passant, castling, promotion) that must work perfectly
3. **Regression Prevention**: As we optimize or refactor, tests ensure we don't break existing functionality
4. **Documentation**: Good tests serve as executable documentation showing how code should be used
5. **Confidence**: High test coverage gives us confidence to refactor and optimize without fear

This phase focuses exclusively on **unit tests** - small, fast, isolated tests that verify individual functions and modules work correctly. We'll test every parsing function, every type conversion, and every edge case to achieve >90% code coverage.

---

## Why Unit Testing Matters for Binary Parsers

### The Cost of Parsing Errors

When parsing binary data, errors compound:

```rust
// WRONG: Off-by-one error in date parsing
fn parse_date_wrong(date_value: u32) -> (u16, u8, u8) {
    let year = ((date_value >> 9) & 0x1FF) as u16;  // Should be & 0xFFF
    let month = ((date_value >> 5) & 0x0F) as u8;
    let day = (date_value & 0x1F) as u8;
    (year, month, day)
}

// Result: Year 2024 becomes year 24, completely wrong!
```

Without unit tests, this error might only surface when a user notices games from "year 24" in their output - by which time you've released the bug to production.

### The Testing Pyramid

```
        /\
       /  \      E2E Tests (Few)
      /    \     - Full database conversion
     /------\    - CLI integration
    /        \
   /  Integration\ (Some)
  /    Tests      \
 /-----------------\
/                   \
/    Unit Tests      \ (Many)
/   (This Phase)      \
-----------------------
```

Unit tests form the base because they're:
- **Fast**: Run in milliseconds
- **Focused**: Test one thing at a time
- **Debuggable**: Easy to pinpoint failures
- **Comprehensive**: Cover all code paths

---

## Reference: Rust Testing Best Practices

### Cargo Test Framework

Rust has built-in testing support via `cargo test`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        assert_eq!(1 + 1, 2);
    }

    #[test]
    #[should_panic(expected = "overflow")]
    fn test_overflow() {
        let _x: u8 = 255 + 1;  // Should panic
    }

    #[test]
    fn test_result() -> Result<(), String> {
        if 2 + 2 == 4 {
            Ok(())
        } else {
            Err(String::from("Math is broken"))
        }
    }
}
```

### Test Organization Patterns

**Pattern 1: Inline Tests** (preferred for unit tests)
```rust
// src/index_parser.rs
pub struct Si4Header { /* ... */ }

impl Si4Header {
    pub fn parse(data: &[u8]) -> Result<Self> { /* ... */ }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_parsing() {
        // Tests go here, close to the code they test
    }
}
```

**Pattern 2: Separate Test Files** (for integration tests)
```rust
// tests/integration_test.rs
// Separate file for end-to-end testing
```

### Assertion Macros

```rust
// Equality
assert_eq!(actual, expected);
assert_ne!(actual, not_expected);

// Boolean conditions
assert!(condition);
assert!(condition, "Custom message: {}", value);

// Result handling
let result = some_function()?;  // Propagate errors in tests
assert!(result.is_ok());
assert!(result.is_err());

// Custom messages
assert_eq!(
    actual, expected,
    "Expected {}, but got {}. Context: {:?}",
    expected, actual, debug_info
);
```

### Test-Driven Development (TDD)

The TDD cycle:
1. **Red**: Write a failing test first
2. **Green**: Write minimal code to make it pass
3. **Refactor**: Improve the code while keeping tests green

For binary parsing, TDD is especially valuable because we have a specification (SCID_DATABASE_FORMAT.md) to test against.

---

## Task 9A.1: Test Framework Setup

### 9A.1.1: Configure Test Dependencies

**File**: `crates/core/Cargo.toml`

**Objective**: Add development dependencies for comprehensive testing.

**Implementation**:

```toml
[package]
name = "scidtopgn-core"
version = "0.1.0"
edition = "2021"

[dependencies]
byteorder = "1.5"
thiserror = "1.0"
shakmaty = "0.25"

[dev-dependencies]
# Property-based testing
proptest = "1.4"

# Better assertion messages
pretty_assertions = "1.4"

# Test fixtures and helpers
tempfile = "3.8"

# Benchmarking
criterion = "0.5"
```

**Explanation**:
- **proptest**: Property-based testing generates random inputs to find edge cases
- **pretty_assertions**: Shows colorful diffs when assertions fail
- **tempfile**: Creates temporary files for testing file I/O
- **criterion**: Micro-benchmarking (used later in Phase 9B)

**Acceptance Criteria**:
- [ ] Dependencies added to Cargo.toml
- [ ] `cargo test` runs successfully (even with no tests yet)
- [ ] `cargo test --all-features` works

**Validation**:
```bash
cd crates/core
cargo test --verbose
# Should output: "running 0 tests"
```

---

### 9A.1.2: Create Test Utilities Module

**File**: `crates/core/src/test_utils.rs`

**Objective**: Create reusable test helpers for generating test data.

**Implementation**:

```rust
//! Test utilities for creating test fixtures
//!
//! This module provides helper functions to generate valid SCID data
//! for testing purposes. Not compiled in release builds.

#![cfg(test)]

use byteorder::{BigEndian, WriteBytesExt};

/// Creates a minimal valid .si4 header
///
/// See SCID_DATABASE_FORMAT.md lines 75-127 for header specification
pub fn create_test_si4_header(game_count: u32) -> Vec<u8> {
    let mut header = Vec::new();

    // Magic bytes (offset 0-7)
    header.extend_from_slice(b"\x53\x63\x69\x64\x20\x44\x42\x00");  // "Scid DB\0"

    // Version (offset 8-11): 4.0
    header.write_u32::<BigEndian>(0x00040000).unwrap();

    // Base flags (offset 12-15): 0
    header.write_u32::<BigEndian>(0).unwrap();

    // Game count (offset 16-19)
    header.write_u32::<BigEndian>(game_count).unwrap();

    // Auto-load game number (offset 20-23): 0
    header.write_u32::<BigEndian>(0).unwrap();

    // Padding to 256 bytes
    header.resize(256, 0);

    header
}

/// Creates a test game index entry
///
/// See SCID_DATABASE_FORMAT.md lines 129-209 for index entry specification
pub fn create_test_game_entry(
    white_id: u32,
    black_id: u32,
    event_id: u32,
    site_id: u32,
    round_id: u32,
    offset: u32,
    length: u16,
) -> Vec<u8> {
    let mut entry = Vec::new();

    // Offset (3 bytes, big-endian)
    entry.write_u8((offset >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((offset & 0xFFFF) as u16).unwrap();

    // Length (2 bytes)
    entry.write_u16::<BigEndian>(length).unwrap();

    // White ID (3 bytes)
    entry.write_u8((white_id >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((white_id & 0xFFFF) as u16).unwrap();

    // Black ID (3 bytes)
    entry.write_u8((black_id >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((black_id & 0xFFFF) as u16).unwrap();

    // Event ID (3 bytes)
    entry.write_u8((event_id >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((event_id & 0xFFFF) as u16).unwrap();

    // Site ID (3 bytes)
    entry.write_u8((site_id >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((site_id & 0xFFFF) as u16).unwrap();

    // Round ID (3 bytes)
    entry.write_u8((round_id >> 16) as u8).unwrap();
    entry.write_u16::<BigEndian>((round_id & 0xFFFF) as u16).unwrap();

    // Remaining fields (result, date, eco, etc.) - use defaults
    entry.resize(46, 0);

    entry
}

/// Creates a test name file entry with front-coding
///
/// See SCID_DATABASE_FORMAT.md lines 315-470 for name file specification
pub fn create_test_name_entry(name: &str, previous_name: &str) -> Vec<u8> {
    let mut entry = Vec::new();

    // Calculate common prefix length
    let common_len = name.chars()
        .zip(previous_name.chars())
        .take_while(|(a, b)| a == b)
        .count();

    // Length byte: bits 0-3 = common length, bits 4-7 = new chars length
    let new_chars = &name[common_len..];
    let length_byte = ((common_len as u8) & 0x0F) | (((new_chars.len() as u8) & 0x0F) << 4);

    entry.push(length_byte);
    entry.extend_from_slice(new_chars.as_bytes());

    entry
}

/// Creates a minimal valid game data block
///
/// Returns bytes for a game with just 1.e4 e5 2.Nf3
pub fn create_test_game_data() -> Vec<u8> {
    let mut data = Vec::new();

    // Standard start position flag
    data.push(0x00);

    // Move: e2-e4 (pawn e-file, 2 squares forward)
    // Pawn pieces are 8-15, e-file pawn is piece 12
    // Move value: 2 squares forward = 0x02
    data.push(12);   // From piece number
    data.push(0x02); // Move encoding

    // Move: e7-e5 (black pawn)
    data.push(12);   // From piece number
    data.push(0x02); // Move encoding

    // Move: Ng1-f3 (knight)
    // Knight from g1 is piece 6
    data.push(6);
    data.push(0x14); // Knight move encoding

    // End of game marker
    data.push(0xFF);
    data.push(0xFF);

    data
}

/// Creates a complete minimal test database in memory
pub struct TestDatabase {
    pub si4_data: Vec<u8>,
    pub sn4_data: Vec<u8>,
    pub sg4_data: Vec<u8>,
}

impl TestDatabase {
    pub fn new_single_game() -> Self {
        let mut si4_data = create_test_si4_header(1);
        let game_data = create_test_game_data();

        // Add one game entry
        si4_data.extend(create_test_game_entry(
            1, // white_id
            2, // black_id
            1, // event_id
            1, // site_id
            0, // round_id
            0, // offset in .sg4
            game_data.len() as u16,
        ));

        // Create minimal name file
        let mut sn4_data = Vec::new();

        // Header: 3 frequencies (player, event, site) + 3 names each
        sn4_data.write_u32::<BigEndian>(3).unwrap(); // 3 name types

        // Player names frequency: 2 players
        sn4_data.write_u32::<BigEndian>(2).unwrap();

        // Event names frequency: 1 event
        sn4_data.write_u32::<BigEndian>(1).unwrap();

        // Site names frequency: 1 site
        sn4_data.write_u32::<BigEndian>(1).unwrap();

        // Round names frequency: 0 rounds
        sn4_data.write_u32::<BigEndian>(0).unwrap();

        // Player names
        sn4_data.extend(create_test_name_entry("Carlsen, Magnus", ""));
        sn4_data.extend(create_test_name_entry("Nakamura, Hikaru", "Carlsen, Magnus"));

        // Event name
        sn4_data.extend(create_test_name_entry("Test Tournament", ""));

        // Site name
        sn4_data.extend(create_test_name_entry("Online", ""));

        Self {
            si4_data,
            sn4_data,
            sg4_data: game_data,
        }
    }
}

/// Helper to create temporary test files
pub fn write_test_database(db: &TestDatabase) -> std::io::Result<tempfile::TempDir> {
    use std::fs::File;
    use std::io::Write;

    let dir = tempfile::tempdir()?;

    let si4_path = dir.path().join("test.si4");
    let mut si4_file = File::create(si4_path)?;
    si4_file.write_all(&db.si4_data)?;

    let sn4_path = dir.path().join("test.sn4");
    let mut sn4_file = File::create(sn4_path)?;
    sn4_file.write_all(&db.sn4_data)?;

    let sg4_path = dir.path().join("test.sg4");
    let mut sg4_file = File::create(sg4_path)?;
    sg4_file.write_all(&db.sg4_data)?;

    Ok(dir)
}
```

**Acceptance Criteria**:
- [ ] Test utilities module created
- [ ] Helper functions compile without warnings
- [ ] Functions generate valid test data

**Validation**:
```bash
cargo test --lib
# Should compile without errors
```

---

### 9A.1.3: Enable Test Utilities in Main Module

**File**: `crates/core/src/lib.rs`

**Objective**: Make test utilities available to all test modules.

**Implementation**:

Add to the end of `lib.rs`:

```rust
// Test utilities module (only compiled during testing)
#[cfg(test)]
mod test_utils;
```

**Acceptance Criteria**:
- [ ] Test utilities accessible from all test modules
- [ ] Module only compiles during tests (not in release builds)

---

## Task 9A.2: Index File Parser Tests

### 9A.2.1: Test Si4Header Parsing

**File**: `crates/core/src/index_parser.rs` (add to existing `#[cfg(test)] mod tests`)

**Objective**: Verify .si4 header parsing handles all valid and invalid cases.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_header_parse_valid() {
        let header_data = create_test_si4_header(42);
        let header = Si4Header::parse(&header_data).expect("Should parse valid header");

        assert_eq!(header.version, 0x00040000, "Version should be 4.0");
        assert_eq!(header.game_count, 42, "Game count should match");
        assert_eq!(header.auto_load, 0, "Auto-load should be 0");
    }

    #[test]
    fn test_header_parse_zero_games() {
        let header_data = create_test_si4_header(0);
        let header = Si4Header::parse(&header_data).expect("Empty database should be valid");

        assert_eq!(header.game_count, 0);
    }

    #[test]
    fn test_header_parse_large_database() {
        // Test with 1 million games
        let header_data = create_test_si4_header(1_000_000);
        let header = Si4Header::parse(&header_data).expect("Should handle large counts");

        assert_eq!(header.game_count, 1_000_000);
    }

    #[test]
    fn test_header_parse_invalid_magic() {
        let mut header_data = create_test_si4_header(1);

        // Corrupt magic bytes
        header_data[0] = b'X';

        let result = Si4Header::parse(&header_data);
        assert!(result.is_err(), "Should reject invalid magic bytes");

        if let Err(e) = result {
            let error_msg = format!("{}", e);
            assert!(
                error_msg.contains("magic") || error_msg.contains("Invalid"),
                "Error should mention magic bytes: {}",
                error_msg
            );
        }
    }

    #[test]
    fn test_header_parse_truncated() {
        let header_data = vec![0u8; 100]; // Only 100 bytes instead of 256

        let result = Si4Header::parse(&header_data);
        assert!(result.is_err(), "Should reject truncated header");
    }

    #[test]
    fn test_header_parse_empty() {
        let result = Si4Header::parse(&[]);
        assert!(result.is_err(), "Should reject empty data");
    }

    #[test]
    fn test_header_version_field() {
        let mut header_data = create_test_si4_header(1);

        // Set version to 3.9 (older version)
        use byteorder::{BigEndian, WriteBytesExt};
        let mut cursor = std::io::Cursor::new(&mut header_data[8..12]);
        cursor.write_u32::<BigEndian>(0x00039000).unwrap();

        let header = Si4Header::parse(&header_data).expect("Should parse older version");
        assert_eq!(header.version, 0x00039000);
    }
}
```

**Why These Tests Matter**:
- **test_header_parse_valid**: Happy path validation
- **test_header_parse_zero_games**: Edge case (empty database)
- **test_header_parse_large_database**: Ensures u32 handling works
- **test_header_parse_invalid_magic**: Security - reject corrupted files early
- **test_header_parse_truncated**: Prevents buffer overruns
- **test_header_parse_empty**: Edge case handling
- **test_header_version_field**: Version compatibility

**Acceptance Criteria**:
- [ ] All 7 tests pass
- [ ] Tests cover both success and failure cases
- [ ] Error messages are descriptive

**Validation**:
```bash
cargo test test_header --lib
# Should show: test result: ok. 7 passed; 0 failed
```

---

### 9A.2.2: Test GameIndexEntry Parsing

**File**: `crates/core/src/index_parser.rs` (add to tests module)

**Objective**: Verify game index entry parsing extracts all fields correctly.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_game_entry_parse_basic() {
        let entry_data = create_test_game_entry(
            100, // white_id
            200, // black_id
            50,  // event_id
            25,  // site_id
            10,  // round_id
            1000, // offset
            500,  // length
        );

        let entry = GameIndexEntry::parse(&entry_data)
            .expect("Should parse valid entry");

        assert_eq!(entry.offset, 1000, "Offset should be 1000");
        assert_eq!(entry.length, 500, "Length should be 500");
        assert_eq!(entry.white_id, 100, "White ID should be 100");
        assert_eq!(entry.black_id, 200, "Black ID should be 200");
        assert_eq!(entry.event_id, 50, "Event ID should be 50");
        assert_eq!(entry.site_id, 25, "Site ID should be 25");
        assert_eq!(entry.round_id, 10, "Round ID should be 10");
    }

    #[test]
    fn test_game_entry_parse_max_values() {
        // Test with maximum 24-bit values (2^24 - 1 = 16,777,215)
        let entry_data = create_test_game_entry(
            0xFFFFFF, // max white_id
            0xFFFFFF, // max black_id
            0xFFFFFF, // max event_id
            0xFFFFFF, // max site_id
            0xFFFFFF, // max round_id
            0xFFFFFF, // max offset (3 bytes)
            0xFFFF,   // max length (2 bytes)
        );

        let entry = GameIndexEntry::parse(&entry_data)
            .expect("Should handle maximum values");

        assert_eq!(entry.white_id, 0xFFFFFF);
        assert_eq!(entry.offset, 0xFFFFFF);
        assert_eq!(entry.length, 0xFFFF);
    }

    #[test]
    fn test_game_entry_parse_date() {
        // Test date parsing: 2024-03-15
        // See SCID_DATABASE_FORMAT.md lines 211-313
        let mut entry_data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);

        // Date is at offset 20-22 (3 bytes)
        // Format: year (12 bits) | month (4 bits) | day (5 bits)
        let year = 2024u32;
        let month = 3u32;
        let day = 15u32;

        let date_value = ((year & 0xFFF) << 9) | ((month & 0x0F) << 5) | (day & 0x1F);

        entry_data[20] = ((date_value >> 16) & 0xFF) as u8;
        entry_data[21] = ((date_value >> 8) & 0xFF) as u8;
        entry_data[22] = (date_value & 0xFF) as u8;

        let entry = GameIndexEntry::parse(&entry_data)
            .expect("Should parse date");

        assert_eq!(entry.date_year, 2024);
        assert_eq!(entry.date_month, 3);
        assert_eq!(entry.date_day, 15);
    }

    #[test]
    fn test_game_entry_parse_result() {
        // Test all possible results: 1-0, 0-1, 1/2-1/2, *
        let results = vec![
            (0b00, GameResult::WhiteWin),
            (0b01, GameResult::BlackWin),
            (0b10, GameResult::Draw),
            (0b11, GameResult::Unknown),
        ];

        for (result_bits, expected) in results {
            let mut entry_data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);

            // Result is in offset 5, upper 2 bits
            entry_data[5] = (entry_data[5] & 0x3F) | (result_bits << 6);

            let entry = GameIndexEntry::parse(&entry_data)
                .expect("Should parse result");

            assert_eq!(
                entry.result, expected,
                "Result bits {:02b} should map to {:?}",
                result_bits, expected
            );
        }
    }

    #[test]
    fn test_game_entry_parse_truncated() {
        let entry_data = vec![0u8; 20]; // Only 20 bytes instead of 46

        let result = GameIndexEntry::parse(&entry_data);
        assert!(result.is_err(), "Should reject truncated entry");
    }

    #[test]
    fn test_game_entry_parse_elo_ratings() {
        let mut entry_data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);

        // White ELO at offset 23-24 (2 bytes, big-endian)
        entry_data[23] = 0x09; // 2400 = 0x0960
        entry_data[24] = 0x60;

        // Black ELO at offset 25-26
        entry_data[25] = 0x0A; // 2700 = 0x0A8C
        entry_data[26] = 0x8C;

        let entry = GameIndexEntry::parse(&entry_data)
            .expect("Should parse ELO ratings");

        assert_eq!(entry.white_elo, 2400);
        assert_eq!(entry.black_elo, 2700);
    }

    #[test]
    fn test_game_entry_zero_elo() {
        let mut entry_data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);

        // Zero ELO (unknown)
        entry_data[23] = 0;
        entry_data[24] = 0;

        let entry = GameIndexEntry::parse(&entry_data)
            .expect("Should parse zero ELO");

        assert_eq!(entry.white_elo, 0);
    }
}
```

**Acceptance Criteria**:
- [ ] All 8 tests pass
- [ ] Date parsing tested with real date
- [ ] All result values tested
- [ ] ELO ratings extracted correctly
- [ ] Edge cases (max values, truncated) handled

**Validation**:
```bash
cargo test test_game_entry --lib
# Should show: test result: ok. 8 passed; 0 failed
```

---

## Task 9A.3: Name File Parser Tests

### 9A.3.1: Test Front-Coding Decompression

**File**: `crates/core/src/name_parser.rs` (add to tests module)

**Objective**: Verify front-coding decompression reconstructs names correctly.

**Reference**: SCID_DATABASE_FORMAT.md lines 356-470

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_front_coding_no_common_prefix() {
        // First name has no previous name
        let entry = create_test_name_entry("Carlsen, Magnus", "");

        let (name, bytes_read) = decompress_name(&entry, "")
            .expect("Should decompress first name");

        assert_eq!(name, "Carlsen, Magnus");
        assert_eq!(bytes_read, entry.len());
    }

    #[test]
    fn test_front_coding_common_prefix() {
        // "Carlsen, Magnus" -> "Carlsen, Mikhail"
        // Common prefix: "Carlsen, M" (10 chars)
        // New suffix: "ikhail" (6 chars)
        let previous = "Carlsen, Magnus";
        let entry = create_test_name_entry("Carlsen, Mikhail", previous);

        let (name, bytes_read) = decompress_name(&entry, previous)
            .expect("Should decompress with common prefix");

        assert_eq!(name, "Carlsen, Mikhail");
        assert!(bytes_read > 0);
    }

    #[test]
    fn test_front_coding_full_sequence() {
        // Test a realistic sequence of player names (sorted)
        let names = vec![
            "Anand, Viswanathan",
            "Aronian, Levon",
            "Carlsen, Magnus",
            "Caruana, Fabiano",
            "Ding, Liren",
        ];

        let mut previous = String::new();

        for expected in names {
            let entry = create_test_name_entry(expected, &previous);
            let (name, _) = decompress_name(&entry, &previous)
                .expect(&format!("Should decompress: {}", expected));

            assert_eq!(name, expected);
            previous = name;
        }
    }

    #[test]
    fn test_front_coding_no_common() {
        // Completely different names
        let entry = create_test_name_entry("Zyzzx", "Alpha");

        let (name, _) = decompress_name(&entry, "Alpha")
            .expect("Should handle no common prefix");

        assert_eq!(name, "Zyzzx");
    }

    #[test]
    fn test_front_coding_entire_name_common() {
        // New name is extension of previous
        let previous = "Smith";
        let entry = create_test_name_entry("Smith, John", previous);

        let (name, _) = decompress_name(&entry, previous)
            .expect("Should handle entire previous name as prefix");

        assert_eq!(name, "Smith, John");
    }

    #[test]
    fn test_front_coding_empty_name() {
        // Edge case: empty name
        let entry = create_test_name_entry("", "");

        let (name, _) = decompress_name(&entry, "")
            .expect("Should handle empty name");

        assert_eq!(name, "");
    }

    #[test]
    fn test_front_coding_max_prefix_length() {
        // 15 character common prefix (max that fits in 4 bits)
        let previous = "AAAAAAAAAAAAAAA_Previous";
        let current =  "AAAAAAAAAAAAAAA_Current";

        let entry = create_test_name_entry(current, previous);
        let (name, _) = decompress_name(&entry, previous)
            .expect("Should handle max prefix length");

        assert_eq!(name, current);
    }

    #[test]
    fn test_front_coding_non_ascii() {
        // Test with unicode characters (common in chess names)
        let previous = "Grünfeld, David";
        let current =  "Grünfeld, Ernst";

        let entry = create_test_name_entry(current, previous);
        let (name, _) = decompress_name(&entry, previous)
            .expect("Should handle unicode");

        assert_eq!(name, current);
    }
}
```

**Why These Tests Matter**:
- **test_front_coding_no_common_prefix**: First name in database
- **test_front_coding_common_prefix**: Typical case with shared prefix
- **test_front_coding_full_sequence**: Real-world sorted name sequence
- **test_front_coding_no_common**: No compression (different names)
- **test_front_coding_entire_name_common**: Previous name is substring
- **test_front_coding_empty_name**: Edge case
- **test_front_coding_max_prefix_length**: Tests 4-bit limit (15 chars)
- **test_front_coding_non_ascii**: Unicode support (common in chess)

**Acceptance Criteria**:
- [ ] All 8 tests pass
- [ ] Real-world name sequences work
- [ ] Unicode names handled correctly
- [ ] Edge cases (empty, max length) work

**Validation**:
```bash
cargo test test_front_coding --lib
# Should show: test result: ok. 8 passed; 0 failed
```

---

### 9A.3.2: Test NameDatabase Loading

**File**: `crates/core/src/name_parser.rs` (add to tests module)

**Objective**: Test loading complete name database with all name types.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_name_database_load_single_player() {
        use byteorder::{BigEndian, WriteBytesExt};

        let mut data = Vec::new();

        // Header: 1 player, 0 events, 0 sites, 0 rounds
        data.write_u32::<BigEndian>(1).unwrap(); // player count
        data.write_u32::<BigEndian>(0).unwrap(); // event count
        data.write_u32::<BigEndian>(0).unwrap(); // site count
        data.write_u32::<BigEndian>(0).unwrap(); // round count

        // One player name
        data.extend(create_test_name_entry("Kasparov, Garry", ""));

        let db = NameDatabase::parse(&data)
            .expect("Should load single player");

        assert_eq!(db.player_count(), 1);
        assert_eq!(db.get_player(0), Some("Kasparov, Garry"));
        assert_eq!(db.get_player(1), None);
    }

    #[test]
    fn test_name_database_load_multiple_types() {
        use byteorder::{BigEndian, WriteBytesExt};

        let mut data = Vec::new();

        // Header: 2 players, 1 event, 1 site, 0 rounds
        data.write_u32::<BigEndian>(2).unwrap();
        data.write_u32::<BigEndian>(1).unwrap();
        data.write_u32::<BigEndian>(1).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        // Player names
        data.extend(create_test_name_entry("Carlsen, Magnus", ""));
        data.extend(create_test_name_entry("Nakamura, Hikaru", "Carlsen, Magnus"));

        // Event name
        data.extend(create_test_name_entry("World Championship", ""));

        // Site name
        data.extend(create_test_name_entry("Dubai", ""));

        let db = NameDatabase::parse(&data)
            .expect("Should load all name types");

        assert_eq!(db.player_count(), 2);
        assert_eq!(db.event_count(), 1);
        assert_eq!(db.site_count(), 1);
        assert_eq!(db.round_count(), 0);

        assert_eq!(db.get_player(0), Some("Carlsen, Magnus"));
        assert_eq!(db.get_player(1), Some("Nakamura, Hikaru"));
        assert_eq!(db.get_event(0), Some("World Championship"));
        assert_eq!(db.get_site(0), Some("Dubai"));
    }

    #[test]
    fn test_name_database_large_count() {
        use byteorder::{BigEndian, WriteBytesExt};

        let mut data = Vec::new();

        // 1000 players
        data.write_u32::<BigEndian>(1000).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        // Generate 1000 names
        let mut previous = String::new();
        for i in 0..1000 {
            let name = format!("Player{:04}", i);
            data.extend(create_test_name_entry(&name, &previous));
            previous = name;
        }

        let db = NameDatabase::parse(&data)
            .expect("Should load 1000 players");

        assert_eq!(db.player_count(), 1000);
        assert_eq!(db.get_player(0), Some("Player0000"));
        assert_eq!(db.get_player(500), Some("Player0500"));
        assert_eq!(db.get_player(999), Some("Player0999"));
    }

    #[test]
    fn test_name_database_empty() {
        use byteorder::{BigEndian, WriteBytesExt};

        let mut data = Vec::new();

        // All counts zero
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();
        data.write_u32::<BigEndian>(0).unwrap();

        let db = NameDatabase::parse(&data)
            .expect("Empty database should be valid");

        assert_eq!(db.player_count(), 0);
        assert_eq!(db.event_count(), 0);
    }

    #[test]
    fn test_name_database_truncated_header() {
        let data = vec![0u8; 8]; // Only 2 u32s instead of 4

        let result = NameDatabase::parse(&data);
        assert!(result.is_err(), "Should reject truncated header");
    }
}
```

**Acceptance Criteria**:
- [ ] All 5 tests pass
- [ ] Single and multiple name types work
- [ ] Large databases (1000+ names) load correctly
- [ ] Empty database handled
- [ ] Truncated data rejected

**Validation**:
```bash
cargo test test_name_database --lib
# Should show: test result: ok. 5 passed; 0 failed
```

---

## Task 9A.4: Move Decoder Tests

### 9A.4.1: Test Piece-Specific Move Decoders

**File**: `crates/core/src/move_decoder.rs` (add to tests module)

**Objective**: Test each piece type's move decoder independently.

**Reference**: SCID_DATABASE_FORMAT.md lines 536-653 (move encoding)

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Color};
    use pretty_assertions::assert_eq;

    // ===== KING MOVE TESTS =====

    #[test]
    fn test_king_move_one_square() {
        // King on e1, move to e2 (one square up)
        let from = Square::E1;
        let move_value = 0x01; // One square in a specific direction

        let decoded = decode_king_move(from, move_value, Color::White)
            .expect("Should decode king move");

        assert_eq!(decoded.from, from);
        assert_eq!(decoded.to, Square::E2);
        assert_eq!(decoded.piece_type, PieceType::King);
    }

    #[test]
    fn test_king_castling_kingside_white() {
        // White kingside castling: e1 -> g1
        let from = Square::E1;
        let move_value = 0x0E; // Kingside castle encoding

        let decoded = decode_king_move(from, move_value, Color::White)
            .expect("Should decode kingside castle");

        assert_eq!(decoded.from, Square::E1);
        assert_eq!(decoded.to, Square::G1);
        assert!(decoded.is_castling);
    }

    #[test]
    fn test_king_castling_queenside_white() {
        // White queenside castling: e1 -> c1
        let from = Square::E1;
        let move_value = 0x0F; // Queenside castle encoding

        let decoded = decode_king_move(from, move_value, Color::White)
            .expect("Should decode queenside castle");

        assert_eq!(decoded.from, Square::E1);
        assert_eq!(decoded.to, Square::C1);
        assert!(decoded.is_castling);
    }

    #[test]
    fn test_king_castling_black() {
        // Black kingside castling: e8 -> g8
        let from = Square::E8;
        let move_value = 0x0E;

        let decoded = decode_king_move(from, move_value, Color::Black)
            .expect("Should decode black castle");

        assert_eq!(decoded.from, Square::E8);
        assert_eq!(decoded.to, Square::G8);
    }

    // ===== QUEEN MOVE TESTS =====

    #[test]
    fn test_queen_move_horizontal() {
        // Queen on d1, move to h1 (4 squares right)
        let from = Square::D1;
        let move_value = 0x04; // Horizontal move, 4 squares
        let mut stream = ByteStream::new(&[]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode horizontal queen move");

        assert_eq!(decoded.from, Square::D1);
        assert_eq!(decoded.to, Square::H1);
    }

    #[test]
    fn test_queen_move_vertical() {
        // Queen on d1, move to d8 (7 squares up)
        let from = Square::D1;
        let move_value = 0x17; // Vertical encoding
        let mut stream = ByteStream::new(&[]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode vertical queen move");

        assert_eq!(decoded.from, Square::D1);
        assert_eq!(decoded.to, Square::D8);
    }

    #[test]
    fn test_queen_move_diagonal_short() {
        // Queen diagonal move (1-byte encoding)
        let from = Square::D4;
        let move_value = 0x20; // Diagonal direction
        let mut stream = ByteStream::new(&[]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode short diagonal");

        assert_eq!(decoded.from, Square::D4);
        // Destination depends on exact encoding
    }

    #[test]
    fn test_queen_move_diagonal_long() {
        // Queen long diagonal (2-byte encoding required)
        let from = Square::A1;
        let move_value = 0xFF; // Marker for 2-byte diagonal

        // Second byte: distance and direction
        let second_byte = 0x07; // 7 squares diagonally
        let mut stream = ByteStream::new(&[second_byte]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode long diagonal from stream");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H8); // a1 to h8 is 7 squares
    }

    // ===== ROOK MOVE TESTS =====

    #[test]
    fn test_rook_move_horizontal() {
        // Rook on a1, move to h1
        let from = Square::A1;
        let move_value = 0x07; // 7 squares right

        let decoded = decode_rook_move(from, move_value)
            .expect("Should decode rook move");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H1);
    }

    #[test]
    fn test_rook_move_vertical() {
        // Rook on a1, move to a8
        let from = Square::A1;
        let move_value = 0x17; // Vertical encoding

        let decoded = decode_rook_move(from, move_value)
            .expect("Should decode vertical rook move");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::A8);
    }

    // ===== BISHOP MOVE TESTS =====

    #[test]
    fn test_bishop_move_diagonal() {
        // Bishop on c1, move to f4 (3 squares diagonally)
        let from = Square::C1;
        let move_value = 0x03; // 3 squares in diagonal direction

        let decoded = decode_bishop_move(from, move_value)
            .expect("Should decode bishop move");

        assert_eq!(decoded.from, Square::C1);
        assert_eq!(decoded.to, Square::F4);
    }

    #[test]
    fn test_bishop_move_long_diagonal() {
        // Bishop on a1, move to h8 (7 squares)
        let from = Square::A1;
        let move_value = 0x07;

        let decoded = decode_bishop_move(from, move_value)
            .expect("Should decode long diagonal");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H8);
    }

    // ===== KNIGHT MOVE TESTS =====

    #[test]
    fn test_knight_all_possible_moves() {
        // Knight has exactly 8 possible moves from center
        let from = Square::D4;

        // All 8 knight move encodings
        let expected_targets = vec![
            (0x00, Square::E6),  // 2 up, 1 right
            (0x01, Square::F5),  // 1 up, 2 right
            (0x02, Square::F3),  // 1 down, 2 right
            (0x03, Square::E2),  // 2 down, 1 right
            (0x04, Square::C2),  // 2 down, 1 left
            (0x05, Square::B3),  // 1 down, 2 left
            (0x06, Square::B5),  // 1 up, 2 left
            (0x07, Square::C6),  // 2 up, 1 left
        ];

        for (move_value, expected_to) in expected_targets {
            let decoded = decode_knight_move(from, move_value)
                .expect(&format!("Should decode knight move {}", move_value));

            assert_eq!(decoded.from, from);
            assert_eq!(
                decoded.to, expected_to,
                "Knight move {} should go to {}",
                move_value, expected_to
            );
        }
    }

    #[test]
    fn test_knight_edge_squares() {
        // Knight on a1 (corner) - only 2 possible moves
        let from = Square::A1;

        let decoded = decode_knight_move(from, 0x00)
            .expect("Should decode from corner");

        assert_eq!(decoded.from, Square::A1);
        // Should be b3 or c2
        assert!(
            decoded.to == Square::B3 || decoded.to == Square::C2,
            "Should be legal knight move from corner"
        );
    }

    // ===== PAWN MOVE TESTS =====

    #[test]
    fn test_pawn_move_one_square() {
        // White pawn e2 -> e3
        let from = Square::E2;
        let move_value = 0x01; // One square forward

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode pawn move");

        assert_eq!(decoded.from, Square::E2);
        assert_eq!(decoded.to, Square::E3);
        assert!(!decoded.is_capture);
        assert_eq!(decoded.promotion, None);
    }

    #[test]
    fn test_pawn_move_two_squares() {
        // White pawn e2 -> e4 (initial two-square move)
        let from = Square::E2;
        let move_value = 0x02; // Two squares forward

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode two-square pawn move");

        assert_eq!(decoded.from, Square::E2);
        assert_eq!(decoded.to, Square::E4);
    }

    #[test]
    fn test_pawn_capture_left() {
        // White pawn e4 captures d5
        let from = Square::E4;
        let move_value = 0x11; // Capture left encoding

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode pawn capture");

        assert_eq!(decoded.from, Square::E4);
        assert_eq!(decoded.to, Square::D5);
        assert!(decoded.is_capture);
    }

    #[test]
    fn test_pawn_capture_right() {
        // White pawn e4 captures f5
        let from = Square::E4;
        let move_value = 0x12; // Capture right encoding

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode pawn capture right");

        assert_eq!(decoded.from, Square::E4);
        assert_eq!(decoded.to, Square::F5);
        assert!(decoded.is_capture);
    }

    #[test]
    fn test_pawn_promotion_queen() {
        // White pawn e7 -> e8=Q
        let from = Square::E7;
        let move_value = 0x81; // Promotion to queen

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode promotion");

        assert_eq!(decoded.from, Square::E7);
        assert_eq!(decoded.to, Square::E8);
        assert_eq!(decoded.promotion, Some(PieceType::Queen));
    }

    #[test]
    fn test_pawn_promotion_knight() {
        // Underpromotion to knight
        let from = Square::E7;
        let move_value = 0x84; // Promotion to knight

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode knight promotion");

        assert_eq!(decoded.promotion, Some(PieceType::Knight));
    }

    #[test]
    fn test_pawn_en_passant() {
        // White pawn e5 captures en passant on d6
        let from = Square::E5;
        let move_value = 0xE1; // En passant encoding

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode en passant");

        assert_eq!(decoded.from, Square::E5);
        assert_eq!(decoded.to, Square::D6);
        assert!(decoded.is_capture);
        assert!(decoded.is_en_passant);
    }

    #[test]
    fn test_pawn_black_moves() {
        // Black pawn e7 -> e6
        let from = Square::E7;
        let move_value = 0x01;

        let decoded = decode_pawn_move(from, move_value, Color::Black)
            .expect("Should decode black pawn move");

        assert_eq!(decoded.from, Square::E7);
        assert_eq!(decoded.to, Square::E6); // Down for black
    }
}
```

**Acceptance Criteria**:
- [ ] All 27 piece-specific tests pass
- [ ] King moves include castling
- [ ] Queen diagonal uses ByteStream correctly
- [ ] All 8 knight moves tested
- [ ] Pawn promotions and en passant work
- [ ] Both colors tested

**Validation**:
```bash
cargo test decode_king --lib
cargo test decode_queen --lib
cargo test decode_rook --lib
cargo test decode_bishop --lib
cargo test decode_knight --lib
cargo test decode_pawn --lib
# All should pass
```

---

### 9A.4.2: Test ScidPosition Integration

**File**: `crates/core/src/move_decoder.rs` (add to tests module)

**Objective**: Test ScidPosition wrapper that combines shakmaty with piece tracking.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_scid_position_initial() {
        let pos = ScidPosition::new();

        // Verify starting position
        assert_eq!(pos.side_to_move(), Color::White);
        assert!(pos.can_castle_kingside(Color::White));
        assert!(pos.can_castle_queenside(Color::White));

        // Check piece mapping exists
        assert_eq!(pos.piece_count(), 32); // All pieces present
    }

    #[test]
    fn test_scid_position_piece_numbers() {
        let pos = ScidPosition::new();

        // White king is piece 0, on e1
        assert_eq!(pos.piece_square(0), Some(Square::E1));

        // White queen is piece 1, on d1
        assert_eq!(pos.piece_square(1), Some(Square::D1));

        // White e-pawn is piece 12, on e2
        assert_eq!(pos.piece_square(12), Some(Square::E2));

        // Black king is piece 16, on e8
        assert_eq!(pos.piece_square(16), Some(Square::E8));
    }

    #[test]
    fn test_scid_position_make_move() {
        let mut pos = ScidPosition::new();

        // Make 1.e4
        let piece_num = 12; // e-pawn
        let move_value = 0x02; // two squares forward

        pos.make_move(piece_num, move_value)
            .expect("Should make legal move");

        // Piece 12 should now be on e4
        assert_eq!(pos.piece_square(12), Some(Square::E4));

        // Side to move should be black
        assert_eq!(pos.side_to_move(), Color::Black);
    }

    #[test]
    fn test_scid_position_capture_updates_pieces() {
        let mut pos = ScidPosition::new();

        // Play fool's mate: 1.f3 e5 2.g4 Qh4#
        pos.make_move(13, 0x01).unwrap(); // f2-f3
        pos.make_move(28, 0x02).unwrap(); // e7-e5
        pos.make_move(14, 0x02).unwrap(); // g2-g4

        // Black queen captures g4
        pos.make_move(17, 0x40).unwrap(); // Qd8-h4+ (capturing g4)

        // White g-pawn (piece 14) should be captured (removed from board)
        assert_eq!(pos.piece_square(14), None);
    }

    #[test]
    fn test_scid_position_castling() {
        let mut pos = ScidPosition::new();

        // Set up position for castling: clear pieces between king and rook
        // (This requires making moves to get to castling position)

        // 1.e4 e5 2.Nf3 Nf6 3.Bc4 Bc5 4.O-O
        pos.make_move(12, 0x02).unwrap();
        pos.make_move(28, 0x02).unwrap();
        pos.make_move(6, 0x01).unwrap();
        pos.make_move(22, 0x01).unwrap();
        pos.make_move(5, 0x02).unwrap();
        pos.make_move(21, 0x02).unwrap();

        // White castles kingside
        pos.make_move(0, 0x0E).unwrap(); // King castles

        // King should be on g1, rook on f1
        assert_eq!(pos.piece_square(0), Some(Square::G1));
        assert_eq!(pos.piece_square(7), Some(Square::F1)); // Rook moved
    }

    #[test]
    fn test_scid_position_promotion() {
        // Create position with pawn on 7th rank
        let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
        let mut pos = ScidPosition::from_fen(fen)
            .expect("Should parse FEN");

        // Promote e7-e8=Q
        pos.make_move(12, 0x81).unwrap();

        // Piece should be promoted to queen
        assert_eq!(pos.piece_square(12), Some(Square::E8));
        assert_eq!(pos.piece_type(12), Some(PieceType::Queen));
    }

    #[test]
    fn test_scid_position_checkmate_detection() {
        // Fool's mate position
        let mut pos = ScidPosition::new();

        pos.make_move(13, 0x01).unwrap(); // f2-f3
        pos.make_move(28, 0x02).unwrap(); // e7-e5
        pos.make_move(14, 0x02).unwrap(); // g2-g4
        pos.make_move(17, 0x40).unwrap(); // Qd8-h4#

        assert!(pos.is_checkmate());
        assert!(!pos.is_stalemate());
    }

    #[test]
    fn test_scid_position_stalemate_detection() {
        // Simple stalemate position: K vs K+Q
        let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
        let pos = ScidPosition::from_fen(fen)
            .expect("Should parse FEN");

        assert!(pos.is_stalemate());
        assert!(!pos.is_checkmate());
    }
}
```

**Acceptance Criteria**:
- [ ] All 8 integration tests pass
- [ ] Piece number tracking works after moves
- [ ] Captures remove pieces from mapping
- [ ] Castling updates both king and rook
- [ ] Promotion changes piece type
- [ ] Checkmate and stalemate detected

**Validation**:
```bash
cargo test test_scid_position --lib
# Should show: test result: ok. 8 passed; 0 failed
```

---

## Task 9A.5: PGN Formatter Tests

### 9A.5.1: Test SAN Generation

**File**: `crates/core/src/pgn_formatter.rs` (add to tests module)

**Objective**: Verify SAN notation is generated correctly via shakmaty.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Chess, Position, Move, Square};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_san_simple_pawn_move() {
        let mut gen = SanGenerator::new();

        // 1.e4
        let chess_move = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };

        let san = gen.move_to_san(&chess_move)
            .expect("Should generate SAN");

        assert_eq!(san, "e4");
    }

    #[test]
    fn test_san_piece_move() {
        let mut gen = SanGenerator::new();

        // 1.e4 e5 2.Nf3
        gen.move_to_san(&Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        }).unwrap();

        gen.move_to_san(&Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E5,
            capture: None,
            promotion: None,
        }).unwrap();

        let knight_move = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            to: Square::F3,
            capture: None,
            promotion: None,
        };

        let san = gen.move_to_san(&knight_move)
            .expect("Should generate knight SAN");

        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_san_capture() {
        let mut gen = SanGenerator::new();

        // Set up: 1.e4 d5 2.exd5
        gen.move_to_san(&Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        }).unwrap();

        gen.move_to_san(&Move::Normal {
            role: Role::Pawn,
            from: Square::D7,
            to: Square::D5,
            capture: None,
            promotion: None,
        }).unwrap();

        let capture = Move::Normal {
            role: Role::Pawn,
            from: Square::E4,
            to: Square::D5,
            capture: Some(Role::Pawn),
            promotion: None,
        };

        let san = gen.move_to_san(&capture)
            .expect("Should generate capture SAN");

        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_san_castling_kingside() {
        // Position after 1.e4 e5 2.Nf3 Nf6 3.Bc4 Bc5 4.O-O
        let mut gen = SanGenerator::new();

        // Set up position (abbreviated for test)
        // ... make preparatory moves ...

        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::H1,
        };

        let san = gen.move_to_san(&castle)
            .expect("Should generate castling SAN");

        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_san_castling_queenside() {
        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::A1,
        };

        // (Assuming position is set up for queenside castling)
        let san = San::from_move(&Chess::default(), &castle).to_string();

        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_san_promotion() {
        // Pawn promotion: e7-e8=Q
        let promotion = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E8,
            capture: None,
            promotion: Some(Role::Queen),
        };

        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let pos = Chess::from_setup(&fen.parse().unwrap())
            .expect("Should parse FEN");

        let san = San::from_move(&pos, &promotion).to_string();

        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_san_underpromotion() {
        // Promote to knight: e8=N
        let promotion = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E8,
            capture: None,
            promotion: Some(Role::Knight),
        };

        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let pos = Chess::from_setup(&fen.parse().unwrap()).unwrap();

        let san = San::from_move(&pos, &promotion).to_string();

        assert_eq!(san, "e8=N");
    }

    #[test]
    fn test_san_check() {
        // Move that gives check
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let pos = Chess::from_setup(&fen.parse().unwrap()).unwrap();

        // Qh4+ gives check
        let check_move = Move::Normal {
            role: Role::Queen,
            from: Square::D8,
            to: Square::H4,
            capture: None,
            promotion: None,
        };

        let san = San::from_move(&pos, &check_move).to_string();

        assert!(san.ends_with('+'), "Check should end with +: {}", san);
    }

    #[test]
    fn test_san_checkmate() {
        // Fool's mate final position
        let fen = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3";
        let pos = Chess::from_setup(&fen.parse().unwrap()).unwrap();

        // This position is already checkmate
        assert!(pos.is_checkmate());

        // The move that led here would have "#"
        // (Testing the notation, not making the move from this position)
    }

    #[test]
    fn test_san_disambiguation_file() {
        // When two rooks can move to same square, disambiguate by file
        let fen = "4k3/8/8/8/8/8/8/R3K2R w KQ - 0 1";
        let pos = Chess::from_setup(&fen.parse().unwrap()).unwrap();

        // Rook from a1 to a8
        let rook_move = Move::Normal {
            role: Role::Rook,
            from: Square::A1,
            to: Square::A8,
            capture: None,
            promotion: None,
        };

        let san = San::from_move(&pos, &rook_move).to_string();

        // Should be Ra8 or Raa8 depending on whether h1 rook can also go to a8
        assert!(san.starts_with('R'));
        assert!(san.contains("a8"));
    }

    #[test]
    fn test_san_disambiguation_rank() {
        // When two pieces on same file can move to same square
        let fen = "4k3/8/8/8/3R4/8/3R4/4K3 w - - 0 1";
        let pos = Chess::from_setup(&fen.parse().unwrap()).unwrap();

        // Rook from d2 to d6
        let rook_move = Move::Normal {
            role: Role::Rook,
            from: Square::D2,
            to: Square::D6,
            capture: None,
            promotion: None,
        };

        let san = San::from_move(&pos, &rook_move).to_string();

        // Should be R2d6 (rank disambiguation)
        assert!(san.contains('2'), "Should disambiguate by rank: {}", san);
    }
}
```

**Acceptance Criteria**:
- [ ] All 11 SAN tests pass
- [ ] Simple moves generate correct notation
- [ ] Captures include "x"
- [ ] Castling generates O-O and O-O-O
- [ ] Promotions include "=Q", "=N", etc.
- [ ] Check and checkmate marked
- [ ] Disambiguation works

**Validation**:
```bash
cargo test test_san --lib
# Should show: test result: ok. 11 passed; 0 failed
```

---

### 9A.5.2: Test PGN Tag Formatting

**File**: `crates/core/src/pgn_formatter.rs` (add to tests module)

**Objective**: Verify PGN tags are formatted per specification.

**Reference**: PGN specification for Seven Tag Roster

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_seven_tag_roster_basic() {
        let roster = SevenTagRoster {
            event: "World Championship".to_string(),
            site: "Dubai".to_string(),
            date: "2024.03.15".to_string(),
            round: "1".to_string(),
            white: "Carlsen, Magnus".to_string(),
            black: "Nepomniachtchi, Ian".to_string(),
            result: "1-0".to_string(),
        };

        let formatted = roster.format();

        assert!(formatted.contains("[Event \"World Championship\"]"));
        assert!(formatted.contains("[Site \"Dubai\"]"));
        assert!(formatted.contains("[Date \"2024.03.15\"]"));
        assert!(formatted.contains("[Round \"1\"]"));
        assert!(formatted.contains("[White \"Carlsen, Magnus\"]"));
        assert!(formatted.contains("[Black \"Nepomniachtchi, Ian\"]"));
        assert!(formatted.contains("[Result \"1-0\"]"));
    }

    #[test]
    fn test_tag_order_is_correct() {
        let roster = SevenTagRoster {
            event: "Test".to_string(),
            site: "Online".to_string(),
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white: "Player1".to_string(),
            black: "Player2".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Must be in exact order
        assert_eq!(lines.len(), 7);
        assert!(lines[0].starts_with("[Event"));
        assert!(lines[1].starts_with("[Site"));
        assert!(lines[2].starts_with("[Date"));
        assert!(lines[3].starts_with("[Round"));
        assert!(lines[4].starts_with("[White"));
        assert!(lines[5].starts_with("[Black"));
        assert!(lines[6].starts_with("[Result"));
    }

    #[test]
    fn test_tag_escaping_quotes() {
        let roster = SevenTagRoster {
            event: "Match \"The Candidates\"".to_string(),
            site: "Online".to_string(),
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white: "Player1".to_string(),
            black: "Player2".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.format();

        // Quotes should be escaped
        assert!(formatted.contains("Match \\\"The Candidates\\\""));
    }

    #[test]
    fn test_tag_unknown_values() {
        let roster = SevenTagRoster {
            event: "?".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "?".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.format();

        // Unknown values should be represented as "?"
        assert!(formatted.contains("[Event \"?\"]"));
        assert!(formatted.contains("[Date \"????.??.??\"]"));
    }

    #[test]
    fn test_supplemental_tags() {
        let mut tags = SupplementalTags::new();
        tags.insert("WhiteElo", "2800");
        tags.insert("BlackElo", "2750");
        tags.insert("ECO", "C42");

        let formatted = tags.format();

        assert!(formatted.contains("[WhiteElo \"2800\"]"));
        assert!(formatted.contains("[BlackElo \"2750\"]"));
        assert!(formatted.contains("[ECO \"C42\"]"));
    }

    #[test]
    fn test_supplemental_tags_sorted() {
        let mut tags = SupplementalTags::new();
        tags.insert("ZZZ", "last");
        tags.insert("AAA", "first");
        tags.insert("MMM", "middle");

        let formatted = tags.format();
        let lines: Vec<&str> = formatted.lines().collect();

        // Should be alphabetically sorted
        assert!(lines[0].contains("AAA"));
        assert!(lines[1].contains("MMM"));
        assert!(lines[2].contains("ZZZ"));
    }

    #[test]
    fn test_date_formatting() {
        // Test various date formats
        let test_cases = vec![
            ((2024, 3, 15), "2024.03.15"),
            ((2024, 12, 31), "2024.12.31"),
            ((2024, 1, 1), "2024.01.01"),
            ((0, 0, 0), "????.??.??"), // Unknown date
            ((2024, 0, 0), "2024.??.??"), // Year only
        ];

        for ((year, month, day), expected) in test_cases {
            let formatted = format_pgn_date(year, month, day);
            assert_eq!(
                formatted, expected,
                "Date ({}, {}, {}) should format as {}",
                year, month, day, expected
            );
        }
    }

    #[test]
    fn test_result_formatting() {
        let results = vec![
            (GameResult::WhiteWin, "1-0"),
            (GameResult::BlackWin, "0-1"),
            (GameResult::Draw, "1/2-1/2"),
            (GameResult::Unknown, "*"),
        ];

        for (result, expected) in results {
            let formatted = format_result(result);
            assert_eq!(formatted, expected);
        }
    }
}
```

**Acceptance Criteria**:
- [ ] All 8 tag formatting tests pass
- [ ] Seven Tag Roster in correct order
- [ ] Quotes escaped properly
- [ ] Unknown values handled ("?")
- [ ] Supplemental tags sorted
- [ ] Dates formatted correctly

**Validation**:
```bash
cargo test test_seven_tag --lib
cargo test test_tag --lib
cargo test test_date --lib
cargo test test_result --lib
# All should pass
```

---

### 9A.5.3: Test Complete PGN Generation

**File**: `crates/core/src/pgn_formatter.rs` (add to tests module)

**Objective**: Test full PGN document generation with tags and moves.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    // ... previous tests ...

    #[test]
    fn test_complete_pgn_simple_game() {
        // Create a simple game: 1.e4 e5 2.Nf3 1-0
        let index_entry = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);
        let names = create_test_name_database();
        let game_data = create_simple_game_data(); // e4 e5 Nf3

        let pgn = PgnFormatter::format_game(
            &index_entry,
            &names,
            &game_data,
            &PgnOptions::default(),
        ).expect("Should format PGN");

        // Should have tags
        assert!(pgn.contains("[Event"));
        assert!(pgn.contains("[White"));
        assert!(pgn.contains("[Black"));

        // Should have moves
        assert!(pgn.contains("1. e4 e5"));
        assert!(pgn.contains("2. Nf3"));

        // Should have result
        assert!(pgn.contains("1-0"));

        // Should have blank line between tags and moves
        assert!(pgn.contains("]\n\n1. "));
    }

    #[test]
    fn test_pgn_compact_format() {
        let options = PgnOptions {
            compact: true,
            ..Default::default()
        };

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 100),
            &create_test_name_database(),
            &create_simple_game_data(),
            &options,
        ).expect("Should format compact PGN");

        // Compact format: all moves on one line
        let movetext = extract_movetext(&pgn);
        assert!(!movetext.contains('\n'), "Compact should be single line");
    }

    #[test]
    fn test_pgn_verbose_format() {
        let options = PgnOptions {
            verbose: true,
            ..Default::default()
        };

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 100),
            &create_test_name_database(),
            &create_simple_game_data(),
            &options,
        ).expect("Should format verbose PGN");

        // Verbose format: each move on separate line
        let movetext = extract_movetext(&pgn);
        let lines = movetext.lines().count();
        assert!(lines > 1, "Verbose should have multiple lines");
    }

    #[test]
    fn test_pgn_line_wrapping() {
        // Create a long game (many moves)
        let long_game = create_long_game_data(80); // 80 moves

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 1000),
            &create_test_name_database(),
            &long_game,
            &PgnOptions::default(),
        ).expect("Should format long game");

        // Lines should be wrapped at ~80 characters
        let movetext = extract_movetext(&pgn);
        for line in movetext.lines() {
            assert!(
                line.len() <= 85,
                "Line too long ({} chars): {}",
                line.len(),
                line
            );
        }
    }

    #[test]
    fn test_pgn_with_comments() {
        let game_data = create_game_with_comments();

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 200),
            &create_test_name_database(),
            &game_data,
            &PgnOptions::default(),
        ).expect("Should format with comments");

        // Should contain comment markers
        assert!(pgn.contains("{") && pgn.contains("}"));
    }

    #[test]
    fn test_pgn_with_variations() {
        let game_data = create_game_with_variations();

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 300),
            &create_test_name_database(),
            &game_data,
            &PgnOptions::default(),
        ).expect("Should format with variations");

        // Should contain variation markers
        assert!(pgn.contains("(") && pgn.contains(")"));
    }

    #[test]
    fn test_pgn_special_characters_escaped() {
        // Create game with special characters in event name
        let mut names = create_test_name_database();
        names.set_event(0, "Test \"Special\" Event");

        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 0, 1, 0, 0, 100),
            &names,
            &create_simple_game_data(),
            &PgnOptions::default(),
        ).expect("Should escape special chars");

        assert!(pgn.contains("\\\""));
    }

    #[test]
    fn test_pgn_result_in_tags_matches_movetext() {
        let pgn = PgnFormatter::format_game(
            &create_test_game_entry(1, 2, 1, 1, 0, 0, 100),
            &create_test_name_database(),
            &create_simple_game_data(),
            &PgnOptions::default(),
        ).expect("Should format PGN");

        // Extract result from tag
        let tag_result = extract_tag_value(&pgn, "Result");

        // Result should also appear at end of movetext
        assert!(
            pgn.ends_with(&tag_result) || pgn.ends_with(&format!("{}\n", tag_result)),
            "Result should appear at end of movetext"
        );
    }
}

// Helper functions for tests
fn extract_movetext(pgn: &str) -> &str {
    // Find first blank line (separates tags from moves)
    if let Some(pos) = pgn.find("\n\n") {
        &pgn[pos + 2..]
    } else {
        pgn
    }
}

fn extract_tag_value(pgn: &str, tag_name: &str) -> String {
    let pattern = format!("[{} \"", tag_name);
    if let Some(start) = pgn.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = pgn[value_start..].find("\"]") {
            return pgn[value_start..value_start + end].to_string();
        }
    }
    String::new()
}
```

**Acceptance Criteria**:
- [ ] All 8 complete PGN tests pass
- [ ] Tags and movetext properly separated
- [ ] Compact and verbose formats work
- [ ] Line wrapping at ~80 characters
- [ ] Comments and variations included
- [ ] Special characters escaped
- [ ] Result appears in both tags and movetext

**Validation**:
```bash
cargo test test_complete_pgn --lib
cargo test test_pgn --lib
# All should pass
```

---

## Task 9A.6: Error Handling Tests

### 9A.6.1: Test Error Types and Messages

**File**: `crates/core/src/errors.rs` (add to tests module)

**Objective**: Verify all error types produce helpful error messages.

**Implementation**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_parse_error_display() {
        let err = ScidError::ParseError {
            message: "Invalid header magic".to_string(),
            offset: 0,
        };

        let display = format!("{}", err);
        assert!(display.contains("Parse error"));
        assert!(display.contains("Invalid header magic"));
        assert!(display.contains("offset 0"));
    }

    #[test]
    fn test_io_error_wrapping() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let scid_err = ScidError::from(io_err);

        let display = format!("{}", scid_err);
        assert!(display.contains("I/O error") || display.contains("File not found"));
    }

    #[test]
    fn test_invalid_move_error() {
        let err = ScidError::InvalidMove {
            piece_num: 12,
            move_value: 0xFF,
            position: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("Invalid move"));
        assert!(display.contains("piece 12"));
        assert!(display.contains("0xFF"));
    }

    #[test]
    fn test_corrupt_database_error() {
        let err = ScidError::CorruptDatabase {
            reason: "Game offset points beyond file end".to_string(),
        };

        let display = format!("{}", err);
        assert!(display.contains("Corrupt database"));
        assert!(display.contains("Game offset"));
    }

    #[test]
    fn test_error_is_send_sync() {
        // Errors must be Send + Sync for use in multi-threaded contexts
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ScidError>();
        assert_sync::<ScidError>();
    }

    #[test]
    fn test_error_source_chain() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "Access denied");
        let scid_err = ScidError::from(io_err);

        // Should preserve error source for debugging
        assert!(scid_err.source().is_some());
    }
}
```

**Acceptance Criteria**:
- [ ] All 6 error tests pass
- [ ] Error messages are descriptive
- [ ] Errors implement Send + Sync
- [ ] Error source chain preserved

---

## Task 9A.7: Property-Based Testing

### 9A.7.1: Proptest for Roundtrip Testing

**File**: `crates/core/tests/proptests.rs` (new file)

**Objective**: Use property-based testing to find edge cases automatically.

**Implementation**:

```rust
//! Property-based tests using proptest
//!
//! These tests generate random inputs to verify properties hold
//! for all possible inputs, not just hand-picked test cases.

use proptest::prelude::*;
use scidtopgn_core::*;

// ===== Property: Date Encoding/Decoding Roundtrip =====

proptest! {
    #[test]
    fn prop_date_roundtrip(
        year in 0u16..4096,
        month in 0u8..13,
        day in 0u8..32
    ) {
        // Encode date to SCID format
        let encoded = encode_date(year, month, day);

        // Decode back
        let (decoded_year, decoded_month, decoded_day) = decode_date(encoded);

        // Should match (within valid ranges)
        prop_assert_eq!(decoded_year, year & 0xFFF);
        prop_assert_eq!(decoded_month, month & 0x0F);
        prop_assert_eq!(decoded_day, day & 0x1F);
    }
}

// ===== Property: Name Front-Coding =====

proptest! {
    #[test]
    fn prop_front_coding_roundtrip(
        name in "[a-zA-Z, ]{1,50}",
        previous in "[a-zA-Z, ]{0,50}"
    ) {
        // Encode with front-coding
        let encoded = encode_name(&name, &previous);

        // Decode back
        let (decoded, _) = decompress_name(&encoded, &previous)
            .expect("Should decode");

        // Should match original
        prop_assert_eq!(decoded, name);
    }
}

// ===== Property: ELO Rating Encoding =====

proptest! {
    #[test]
    fn prop_elo_roundtrip(elo in 0u16..3000) {
        let encoded = encode_elo(elo);
        let decoded = decode_elo(encoded);

        prop_assert_eq!(decoded, elo);
    }
}

// ===== Property: Move Number Does Not Overflow =====

proptest! {
    #[test]
    fn prop_move_number_safe(move_num in 1u32..10000) {
        // Formatting large move numbers should not panic
        let formatted = format_move_number(move_num);

        // Should contain the number
        prop_assert!(formatted.contains(&move_num.to_string()));
    }
}

// ===== Property: All Valid Move Encodings Decode =====

proptest! {
    #[test]
    fn prop_pawn_move_valid(
        from_file in 0u8..8,
        from_rank in 0u8..8,
        move_value in 0u8..=255
    ) {
        let from_square = Square::from_coords(
            File::from_index(from_file),
            Rank::from_index(from_rank)
        );

        // Attempt to decode (may fail for invalid moves, that's ok)
        let result = decode_pawn_move(from_square, move_value, Color::White);

        // If it succeeds, decoded move should be valid
        if let Ok(decoded) = result {
            prop_assert_ne!(decoded.from, decoded.to);
            prop_assert!(decoded.to.is_valid());
        }
    }
}

// ===== Property: PGN Tags Are Never Empty =====

proptest! {
    #[test]
    fn prop_pgn_tags_not_empty(
        event in "[a-zA-Z ]{1,30}",
        site in "[a-zA-Z ]{1,30}",
        white in "[a-zA-Z, ]{1,30}",
        black in "[a-zA-Z, ]{1,30}"
    ) {
        let roster = SevenTagRoster {
            event,
            site,
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white,
            black,
            result: "1-0".to_string(),
        };

        let formatted = roster.format();

        // Every tag line should have content
        for line in formatted.lines() {
            prop_assert!(line.len() > 10); // Minimum: [X "?"]
            prop_assert!(line.contains('['));
            prop_assert!(line.contains(']'));
            prop_assert!(line.contains('"'));
        }
    }
}
```

**Why Property Testing Matters**:
- **Finds edge cases**: Automatically discovers inputs you didn't think to test
- **Proves properties**: Verifies invariants hold for ALL inputs, not just examples
- **Regression detection**: When a property test fails, it found a real bug
- **Documentation**: Properties describe what the code MUST do

**Acceptance Criteria**:
- [ ] All 6 property tests pass with 100+ generated cases each
- [ ] Roundtrip properties verified
- [ ] No panics on random inputs
- [ ] Edge cases found and handled

**Validation**:
```bash
cargo test --test proptests
# Should show: test result: ok. 6 passed; 0 failed
# (Each test runs 100+ cases by default)
```

---

## Success Metrics

### Code Coverage Target

**Goal**: Achieve >90% code coverage for core library

**Measurement**:
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --out Html --output-dir coverage
# Open coverage/index.html in browser

# Coverage should show:
# - src/index_parser.rs: >95%
# - src/name_parser.rs: >95%
# - src/move_decoder.rs: >90% (complex logic)
# - src/pgn_formatter.rs: >90%
# - Overall: >90%
```

### Test Execution Performance

**Goal**: All unit tests complete in <5 seconds

**Validation**:
```bash
time cargo test --lib

# Should output:
# test result: ok. 100+ passed; 0 failed; 0 ignored; 0 measured
# real    0m3.5s  (under 5 seconds)
```

### Test Count

**Minimum**:
- Index parser: 15 tests
- Name parser: 13 tests
- Move decoder: 35 tests
- PGN formatter: 27 tests
- Error handling: 6 tests
- Property tests: 6 tests
- **Total: 102+ unit tests**

---

## Common Pitfalls

### Pitfall 1: Not Testing Edge Cases

**WRONG**:
```rust
#[test]
fn test_parse_date() {
    let date = parse_date(0x012345);
    assert_eq!(date.year, 2024);
}
```

**RIGHT**:
```rust
#[test]
fn test_parse_date_normal() {
    let date = parse_date(encode_date(2024, 3, 15));
    assert_eq!(date, (2024, 3, 15));
}

#[test]
fn test_parse_date_unknown() {
    let date = parse_date(0); // All zeros = unknown
    assert_eq!(date, (0, 0, 0));
}

#[test]
fn test_parse_date_year_only() {
    let date = parse_date(encode_date(2024, 0, 0));
    assert_eq!(date, (2024, 0, 0));
}

#[test]
fn test_parse_date_max_values() {
    let date = parse_date(encode_date(4095, 15, 31));
    // Should handle maximum bit values
}
```

### Pitfall 2: Testing Implementation, Not Behavior

**WRONG**:
```rust
#[test]
fn test_internal_state() {
    let parser = Parser::new();
    assert_eq!(parser.internal_buffer.len(), 0); // Testing internals!
}
```

**RIGHT**:
```rust
#[test]
fn test_parser_behavior() {
    let parser = Parser::new();
    let result = parser.parse(&valid_data);
    assert!(result.is_ok()); // Testing observable behavior
}
```

### Pitfall 3: Unclear Test Names

**WRONG**:
```rust
#[test]
fn test1() { /* ... */ }

#[test]
fn test_parsing() { /* Too vague */ }
```

**RIGHT**:
```rust
#[test]
fn test_header_parse_valid() { /* Clear! */ }

#[test]
fn test_header_parse_invalid_magic() { /* Specific! */ }

#[test]
fn test_king_castling_kingside_white() { /* Descriptive! */ }
```

### Pitfall 4: Not Using Test Helpers

**WRONG**:
```rust
#[test]
fn test_game1() {
    let mut data = Vec::new();
    data.push(0x53);
    data.push(0x63);
    // ... 50 lines of manual data construction
}

#[test]
fn test_game2() {
    let mut data = Vec::new();
    data.push(0x53);
    data.push(0x63);
    // ... same 50 lines duplicated!
}
```

**RIGHT**:
```rust
#[test]
fn test_game1() {
    let data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);
    // Use helper!
}

#[test]
fn test_game2() {
    let data = create_test_game_entry(3, 4, 2, 1, 0, 100, 200);
    // Reuse helper!
}
```

### Pitfall 5: Not Testing Error Cases

**WRONG**:
```rust
#[test]
fn test_parse() {
    let result = parse(&valid_data);
    assert!(result.is_ok());
    // Only testing happy path!
}
```

**RIGHT**:
```rust
#[test]
fn test_parse_valid() {
    let result = parse(&valid_data);
    assert!(result.is_ok());
}

#[test]
fn test_parse_truncated() {
    let result = parse(&truncated_data);
    assert!(result.is_err());
}

#[test]
fn test_parse_corrupted_magic() {
    let result = parse(&corrupted_data);
    assert!(result.is_err());

    // Also verify error message
    if let Err(e) = result {
        assert!(format!("{}", e).contains("magic"));
    }
}
```

---

## Validation Commands

After implementing all tests:

```bash
# Run all unit tests
cargo test --lib

# Run specific test module
cargo test --lib index_parser

# Run tests with output
cargo test --lib -- --nocapture

# Run tests in parallel (default)
cargo test --lib --jobs 8

# Run property tests
cargo test --test proptests

# Measure code coverage
cargo tarpaulin --lib --out Html
# Open coverage/index.html

# Check that tests compile without warnings
cargo test --lib 2>&1 | grep warning
# Should have no output

# Benchmark test execution time
time cargo test --lib --release

# Run tests with verbose output
cargo test --lib --verbose
```

Expected output:
```
running 102 tests
test index_parser::tests::test_game_entry_parse_basic ... ok
test index_parser::tests::test_game_entry_parse_date ... ok
test index_parser::tests::test_game_entry_parse_elo_ratings ... ok
...
test pgn_formatter::tests::test_complete_pgn_simple_game ... ok
test pgn_formatter::tests::test_san_checkmate ... ok

test result: ok. 102 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 6 tests (property tests)
test prop_date_roundtrip ... ok (100 cases)
test prop_front_coding_roundtrip ... ok (100 cases)
test prop_elo_roundtrip ... ok (100 cases)
...

test result: ok. 6 passed; 0 failed
```

---

## Summary

This phase creates **102+ comprehensive unit tests** covering:

1. **Index file parsing** (15 tests)
   - Header parsing with valid/invalid data
   - Game entry parsing with all fields
   - Edge cases and error handling

2. **Name file parsing** (13 tests)
   - Front-coding decompression
   - Multiple name types
   - Large databases
   - Unicode support

3. **Move decoding** (35 tests)
   - All piece types (K, Q, R, B, N, P)
   - Special moves (castling, en passant, promotion)
   - ScidPosition integration
   - Piece number tracking

4. **PGN formatting** (27 tests)
   - SAN generation
   - Tag formatting
   - Complete PGN documents
   - Compact/verbose formats
   - Comments and variations

5. **Error handling** (6 tests)
   - Descriptive error messages
   - Error type coverage
   - Send/Sync requirements

6. **Property-based testing** (6 tests)
   - Roundtrip properties
   - Random input validation
   - Edge case discovery

**Success criteria**:
- ✅ >90% code coverage
- ✅ All tests pass
- ✅ Tests run in <5 seconds
- ✅ No compilation warnings
- ✅ Clear, descriptive test names
- ✅ Both success and failure cases tested

This comprehensive test suite ensures the SCID parser is **correct, robust, and production-ready**.

---

## Next Steps

After completing Phase 9A (Unit Tests), proceed to:

**Phase 9B: Integration Tests & Validation**
- Integration tests with real databases
- Performance benchmarking with criterion
- Memory profiling
- Fuzzing for robustness
- Real-world database validation
- CI/CD pipeline setup

The unit tests created in this phase provide the foundation for confidence in the codebase, enabling us to build integration tests and performance validation on top of proven, correct components.
