# Phase 9B: Integration Tests & Validation - Real-World Testing

## Overview

While unit tests verify individual components work correctly in isolation, **integration tests** verify the system works correctly as a whole with real data. For a SCID to PGN converter, integration testing is critical because:

1. **Real-World Data Complexity**: Actual SCID databases contain edge cases, unusual positions, and data patterns that synthetic test data might miss
2. **End-to-End Verification**: Integration tests verify all components work together correctly from file reading through PGN output
3. **Performance Validation**: Real databases help identify performance bottlenecks with realistic workloads
4. **Compatibility Testing**: Testing with databases from different SCID versions ensures broad compatibility
5. **Regression Detection**: Integration tests catch bugs that only appear when components interact

This phase covers:
- Integration tests with complete database parsing
- Performance benchmarking with criterion
- Memory profiling for efficiency validation
- Fuzzing to discover edge cases
- Real-world database validation
- CI/CD pipeline integration

---

## Why Integration Testing Differs from Unit Testing

### Unit Tests (Phase 9A)
- Test individual functions
- Use synthetic/minimal data
- Run in milliseconds
- Mock external dependencies
- Focus on correctness of individual components

### Integration Tests (Phase 9B)
- Test complete workflows
- Use real-world data
- May run for seconds
- Use actual files and I/O
- Focus on system behavior

### Example Comparison

**Unit Test**:
```rust
#[test]
fn test_parse_game_entry() {
    let data = create_test_game_entry(1, 2, 1, 1, 0, 0, 100);
    let entry = GameIndexEntry::parse(&data).unwrap();
    assert_eq!(entry.white_id, 1);
}
```

**Integration Test**:
```rust
#[test]
fn test_parse_complete_database() {
    let reader = ScidReader::open("tests/data/five").unwrap();

    // Parse all games
    let mut count = 0;
    for game in reader.games() {
        let game = game.unwrap();
        let pgn = game.to_pgn().unwrap();

        // Verify PGN is valid
        assert!(pgn.contains("[Event"));
        assert!(pgn.contains("[White"));
        count += 1;
    }

    assert_eq!(count, reader.game_count());
}
```

---

## Test Data Reference

### Location and Datasets

All test data is in `tests/data/`. See `IMPLEMENTATION_PLAN.md` → "Test Data" section for complete documentation.

| Dataset | PGN Source | SCID Database | Purpose |
|---------|------------|---------------|---------|
| **one** | `one.pgn` | `one.si4/sg4/sn4` | Minimal integration test |
| **five** | `five.pgn` | `five.si4/sg4/sn4` | Full integration validation |

### PGN ↔ SCID Relationship

**Critical for Integration Testing**: Each SCID database was created by importing its corresponding PGN file. This enables round-trip validation:

```
┌─────────────┐     Import      ┌─────────────────────────┐
│  five.pgn   │  ─────────────► │  five.si4/sg4/sn4       │
│  (source)   │                 │  (SCID database)        │
└─────────────┘                 └─────────────────────────┘
       ▲                                    │
       │                                    │ Parse + Convert
       │      Compare (must match)          ▼
       └──────────────────────────  Generated PGN output
```

### Integration Test Requirements

1. **Parse all games** from both datasets without errors
2. **Generate PGN output** for each game
3. **Compare against source PGN** - content must match
4. **Verify no data loss** - all tags, moves, annotations preserved

---

## Reference: Integration Testing Best Practices

### Test Fixture Organization

Test data is organized in `tests/data/`:

```
tests/
├── data/
│   ├── one.pgn                 # Source PGN (1 game)
│   ├── one.si4                 # SCID index file
│   ├── one.sg4                 # SCID game file
│   ├── one.sn4                 # SCID name file
│   ├── five.pgn                # Source PGN (5 games)
│   ├── five.si4                # SCID index file
│   ├── five.sg4                # SCID game file
│   └── five.sn4                # SCID name file
├── integration_test.rs
├── benchmark_test.rs
└── fuzzing_test.rs
```

### Integration Test Structure

```rust
// tests/integration_test.rs

use scidtopgn_core::ScidReader;
use std::path::PathBuf;

// Helper to get test data path
fn test_data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

#[test]
fn test_complete_workflow() {
    // 1. Open database
    let reader = ScidReader::open(fixture_path("minimal.si4"))
        .expect("Should open database");

    // 2. Iterate games
    for (i, game_result) in reader.games().enumerate() {
        let game = game_result.expect("Should parse game");

        // 3. Generate PGN
        let pgn = game.to_pgn().expect("Should generate PGN");

        // 4. Validate output
        validate_pgn(&pgn, i);
    }
}

fn validate_pgn(pgn: &str, game_index: usize) {
    // PGN must have required tags
    assert!(pgn.contains("[Event"), "Game {} missing Event tag", game_index);
    assert!(pgn.contains("[White"), "Game {} missing White tag", game_index);
    assert!(pgn.contains("[Black"), "Game {} missing Black tag", game_index);

    // PGN must have result
    assert!(
        pgn.contains("1-0") || pgn.contains("0-1") ||
        pgn.contains("1/2-1/2") || pgn.contains("*"),
        "Game {} missing result",
        game_index
    );

    // PGN must have blank line between tags and moves
    assert!(pgn.contains("]\n\n"), "Game {} missing blank line", game_index);
}
```

---

## Task 9B.1: Integration Test Setup

### 9B.1.1: Create Test Fixture Database

**File**: Create `tests/fixtures/` directory with test databases

**Objective**: Create small, curated SCID databases for repeatable integration testing.

**Implementation**:

Since creating binary SCID files manually is complex, we'll write a fixture generator:

**File**: `tests/create_fixtures.rs` (test helper)

```rust
//! Test fixture generator
//!
//! This creates minimal SCID databases for integration testing.
//! Run with: cargo test --test create_fixtures -- --ignored

use byteorder::{BigEndian, WriteBytesExt};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
#[ignore] // Only run when explicitly requested
fn generate_minimal_fixture() {
    let fixture_dir = Path::new("tests/fixtures/minimal");
    fs::create_dir_all(fixture_dir).unwrap();

    // Create .si4 index file
    let mut si4 = File::create(fixture_dir.join("minimal.si4")).unwrap();

    // Header
    si4.write_all(b"\x53\x63\x69\x64\x20\x44\x42\x00").unwrap(); // Magic
    si4.write_u32::<BigEndian>(0x00040000).unwrap(); // Version 4.0
    si4.write_u32::<BigEndian>(0).unwrap(); // Base flags
    si4.write_u32::<BigEndian>(1).unwrap(); // Game count: 1
    si4.write_u32::<BigEndian>(0).unwrap(); // Auto-load

    // Padding to 256 bytes
    let zeros = vec![0u8; 256 - 24];
    si4.write_all(&zeros).unwrap();

    // One game entry (46 bytes)
    // Offset: 0, Length: 20
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u16::<BigEndian>(20).unwrap();

    // White ID: 0, Black ID: 1
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(1).unwrap();

    // Event: 0, Site: 0, Round: 0
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();
    si4.write_u8(0).unwrap();
    si4.write_u16::<BigEndian>(0).unwrap();

    // Result: 1-0 (0b00 in bits 6-7)
    si4.write_u8(0x00).unwrap();

    // Date: 2024.03.15
    let date = ((2024u32 & 0xFFF) << 9) | ((3u32 & 0x0F) << 5) | (15u32 & 0x1F);
    si4.write_u8((date >> 16) as u8).unwrap();
    si4.write_u16::<BigEndian>((date & 0xFFFF) as u16).unwrap();

    // ELO ratings: White 2800, Black 2750
    si4.write_u16::<BigEndian>(2800).unwrap();
    si4.write_u16::<BigEndian>(2750).unwrap();

    // Remaining fields: zeros
    let remaining = vec![0u8; 46 - 27];
    si4.write_all(&remaining).unwrap();

    // Create .sn4 name file
    let mut sn4 = File::create(fixture_dir.join("minimal.sn4")).unwrap();

    // Header: 2 players, 1 event, 1 site, 0 rounds
    sn4.write_u32::<BigEndian>(2).unwrap();
    sn4.write_u32::<BigEndian>(1).unwrap();
    sn4.write_u32::<BigEndian>(1).unwrap();
    sn4.write_u32::<BigEndian>(0).unwrap();

    // Player 0: "Carlsen, Magnus"
    let name = "Carlsen, Magnus";
    sn4.write_u8(((0 & 0x0F) | ((name.len() as u8) << 4))).unwrap();
    sn4.write_all(name.as_bytes()).unwrap();

    // Player 1: "Nakamura, Hikaru" (common prefix: "" - no compression)
    let name2 = "Nakamura, Hikaru";
    sn4.write_u8(((0 & 0x0F) | ((name2.len() as u8) << 4))).unwrap();
    sn4.write_all(name2.as_bytes()).unwrap();

    // Event 0: "Test Tournament"
    let event = "Test Tournament";
    sn4.write_u8(((0 & 0x0F) | ((event.len() as u8) << 4))).unwrap();
    sn4.write_all(event.as_bytes()).unwrap();

    // Site 0: "Online"
    let site = "Online";
    sn4.write_u8(((0 & 0x0F) | ((site.len() as u8) << 4))).unwrap();
    sn4.write_all(site.as_bytes()).unwrap();

    // Create .sg4 game file
    let mut sg4 = File::create(fixture_dir.join("minimal.sg4")).unwrap();

    // Standard start position
    sg4.write_u8(0x00).unwrap();

    // Moves: 1.e4 e5 2.Nf3 Nc6 1-0
    // e2-e4: pawn piece 12, move value 2 (two squares)
    sg4.write_u8(12).unwrap();
    sg4.write_u8(0x02).unwrap();

    // e7-e5: black pawn piece 28, move value 2
    sg4.write_u8(28).unwrap();
    sg4.write_u8(0x02).unwrap();

    // Nf3: white knight piece 6, move encoding
    sg4.write_u8(6).unwrap();
    sg4.write_u8(0x14).unwrap();

    // Nc6: black knight piece 22
    sg4.write_u8(22).unwrap();
    sg4.write_u8(0x15).unwrap();

    // End of game marker
    sg4.write_u8(0xFF).unwrap();
    sg4.write_u8(0xFF).unwrap();

    println!("Created minimal fixture at tests/fixtures/minimal/");
    println!("  - minimal.si4 (header + 1 game entry)");
    println!("  - minimal.sn4 (2 players, 1 event, 1 site)");
    println!("  - minimal.sg4 (1 game with 4 moves)");
}

#[test]
#[ignore]
fn generate_expected_pgn() {
    // Create expected PGN output for minimal fixture
    let expected = r#"[Event "Test Tournament"]
[Site "Online"]
[Date "2024.03.15"]
[Round "?"]
[White "Carlsen, Magnus"]
[Black "Nakamura, Hikaru"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 1-0
"#;

    let output_path = Path::new("tests/fixtures/expected/minimal.pgn");
    fs::create_dir_all(output_path.parent().unwrap()).unwrap();

    let mut file = File::create(output_path).unwrap();
    file.write_all(expected.as_bytes()).unwrap();

    println!("Created expected output at tests/fixtures/expected/minimal.pgn");
}
```

**Acceptance Criteria**:
- [ ] Fixture generator creates valid .si4/.sn4/.sg4 files
- [ ] Minimal fixture (1 game) created
- [ ] Expected PGN output saved
- [ ] Fixtures committed to git

**Validation**:
```bash
# Generate fixtures
cargo test --test create_fixtures -- --ignored --nocapture

# Verify files created
ls -lh tests/fixtures/minimal/
# Should show: minimal.si4, minimal.sn4, minimal.sg4

ls -lh tests/fixtures/expected/
# Should show: minimal.pgn
```

---

### 9B.1.2: Create Integration Test Infrastructure

**File**: `tests/integration_test.rs`

**Objective**: Set up integration test framework with helper functions.

**Implementation**:

```rust
//! Integration tests for SCID to PGN conversion
//!
//! These tests use complete SCID databases to verify end-to-end functionality.

use scidtopgn_core::{ScidReader, PgnOptions};
use std::path::{Path, PathBuf};
use std::fs;

/// Get path to test fixture
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Get path to expected output
fn expected_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expected")
        .join(name)
}

/// Load expected PGN content
fn load_expected_pgn(name: &str) -> String {
    fs::read_to_string(expected_path(name))
        .expect(&format!("Should load expected PGN: {}", name))
}

/// Normalize PGN for comparison (ignore whitespace differences)
fn normalize_pgn(pgn: &str) -> String {
    pgn.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Validate PGN has required structure
fn validate_pgn_structure(pgn: &str, context: &str) {
    assert!(
        pgn.contains("[Event"),
        "{}: Missing Event tag",
        context
    );
    assert!(
        pgn.contains("[Site"),
        "{}: Missing Site tag",
        context
    );
    assert!(
        pgn.contains("[Date"),
        "{}: Missing Date tag",
        context
    );
    assert!(
        pgn.contains("[Round"),
        "{}: Missing Round tag",
        context
    );
    assert!(
        pgn.contains("[White"),
        "{}: Missing White tag",
        context
    );
    assert!(
        pgn.contains("[Black"),
        "{}: Missing Black tag",
        context
    );
    assert!(
        pgn.contains("[Result"),
        "{}: Missing Result tag",
        context
    );

    // Must have blank line between tags and movetext
    assert!(
        pgn.contains("]\n\n") || pgn.contains("]\r\n\r\n"),
        "{}: Missing blank line after tags",
        context
    );
}

/// Compare PGN output with expected
fn assert_pgn_matches_expected(actual: &str, expected: &str, context: &str) {
    let actual_normalized = normalize_pgn(actual);
    let expected_normalized = normalize_pgn(expected);

    if actual_normalized != expected_normalized {
        eprintln!("=== PGN Mismatch in {} ===", context);
        eprintln!("Expected:\n{}", expected);
        eprintln!("\nActual:\n{}", actual);
        eprintln!("=========================");
        panic!("PGN output does not match expected for {}", context);
    }
}

// Tests will be added below
```

**Acceptance Criteria**:
- [ ] Helper functions compile
- [ ] Fixture loading works
- [ ] PGN validation functions ready
- [ ] Comparison utilities ready

---

## Task 9B.2: Core Integration Tests

### 9B.2.1: Test Complete Database Parsing

**File**: `tests/integration_test.rs` (add to existing file)

**Objective**: Verify complete workflow from opening database to PGN generation.

**Implementation**:

```rust
// Add to tests/integration_test.rs

#[test]
fn test_open_minimal_database() {
    let db_path = fixture_path("minimal/minimal.si4");

    let reader = ScidReader::open(&db_path)
        .expect("Should open minimal database");

    assert_eq!(reader.game_count(), 1, "Should have 1 game");
}

#[test]
fn test_parse_minimal_game_metadata() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).expect("Should get first game");

    // Verify metadata
    assert_eq!(game.white(), "Carlsen, Magnus");
    assert_eq!(game.black(), "Nakamura, Hikaru");
    assert_eq!(game.event(), "Test Tournament");
    assert_eq!(game.site(), "Online");
    assert_eq!(game.date(), "2024.03.15");
}

#[test]
fn test_parse_minimal_game_moves() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();

    // Should have 4 moves: e4 e5 Nf3 Nc6
    assert_eq!(game.moves().len(), 4, "Should have 4 moves");
}

#[test]
fn test_generate_pgn_minimal() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().expect("Should generate PGN");

    // Validate structure
    validate_pgn_structure(&pgn, "minimal game");

    // Verify tags
    assert!(pgn.contains("Carlsen, Magnus"));
    assert!(pgn.contains("Nakamura, Hikaru"));
    assert!(pgn.contains("Test Tournament"));

    // Verify moves
    assert!(pgn.contains("1. e4 e5"));
    assert!(pgn.contains("2. Nf3 Nc6"));
}

#[test]
fn test_pgn_matches_expected() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();
    let actual_pgn = game.to_pgn().unwrap();

    let expected_pgn = load_expected_pgn("minimal.pgn");

    assert_pgn_matches_expected(&actual_pgn, &expected_pgn, "minimal game");
}

#[test]
fn test_iterate_all_games() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let mut count = 0;
    for game_result in reader.games() {
        let game = game_result.expect("Should parse game");
        let pgn = game.to_pgn().expect("Should generate PGN");

        validate_pgn_structure(&pgn, &format!("game {}", count));
        count += 1;
    }

    assert_eq!(count, 1, "Should iterate over 1 game");
}

#[test]
fn test_multiple_game_access() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    // Access same game multiple times
    let game1 = reader.game(0).unwrap();
    let game2 = reader.game(0).unwrap();

    assert_eq!(game1.white(), game2.white());
    assert_eq!(game1.black(), game2.black());
    assert_eq!(game1.moves().len(), game2.moves().len());
}

#[test]
fn test_game_out_of_bounds() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    // Try to access non-existent game
    let result = reader.game(999);
    assert!(result.is_err(), "Should fail for out-of-bounds index");
}

#[test]
fn test_pgn_options_compact() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let options = PgnOptions {
        compact: true,
        ..Default::default()
    };

    let mut output = Vec::new();
    reader.write_pgn(&mut output, &options)
        .expect("Should write compact PGN");

    let pgn = String::from_utf8(output).unwrap();

    // Compact format: moves should be on one line
    let lines: Vec<&str> = pgn.lines()
        .skip_while(|line| line.starts_with('[') || line.is_empty())
        .collect();

    assert_eq!(lines.len(), 1, "Compact format should have single line of moves");
}

#[test]
fn test_pgn_options_verbose() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let options = PgnOptions {
        verbose: true,
        ..Default::default()
    };

    let mut output = Vec::new();
    reader.write_pgn(&mut output, &options)
        .expect("Should write verbose PGN");

    let pgn = String::from_utf8(output).unwrap();

    // Verbose format: each move pair on separate line
    let move_lines: Vec<&str> = pgn.lines()
        .skip_while(|line| line.starts_with('[') || line.is_empty())
        .collect();

    assert!(move_lines.len() > 1, "Verbose format should have multiple lines");
}
```

**Acceptance Criteria**:
- [ ] All 10 integration tests pass
- [ ] Database opens successfully
- [ ] Metadata extracted correctly
- [ ] Moves parsed correctly
- [ ] PGN generated matches expected
- [ ] Iterator works
- [ ] Error cases handled
- [ ] PGN options work

**Validation**:
```bash
cargo test --test integration_test
# Should show: test result: ok. 10 passed; 0 failed
```

---

### 9B.2.2: Test Special Chess Positions

**File**: `tests/integration_test.rs` (add tests)

**Objective**: Verify special moves (castling, en passant, promotion) work in real games.

**Implementation**:

First, create a fixture with special moves (or use a real PGN file and convert to SCID):

```rust
#[test]
fn test_castling_kingside() {
    // This requires a fixture with kingside castling
    // For now, we'll test the pattern

    let db_path = fixture_path("special_moves/castling.si4");

    // Skip test if fixture doesn't exist yet
    if !db_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().unwrap();

    // PGN should contain kingside castling notation
    assert!(
        pgn.contains("O-O") && !pgn.contains("O-O-O"),
        "Should have kingside castling (O-O)"
    );
}

#[test]
fn test_castling_queenside() {
    let db_path = fixture_path("special_moves/castling_queen.si4");

    if !db_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().unwrap();

    assert!(
        pgn.contains("O-O-O"),
        "Should have queenside castling (O-O-O)"
    );
}

#[test]
fn test_pawn_promotion() {
    let db_path = fixture_path("special_moves/promotion.si4");

    if !db_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().unwrap();

    // Should contain promotion notation (e8=Q or similar)
    assert!(
        pgn.contains("=Q") || pgn.contains("=R") ||
        pgn.contains("=B") || pgn.contains("=N"),
        "Should have promotion notation"
    );
}

#[test]
fn test_en_passant() {
    let db_path = fixture_path("special_moves/en_passant.si4");

    if !db_path.exists() {
        eprintln!("Skipping test: fixture not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().unwrap();

    // En passant is just a pawn capture, so verify pawn captures exist
    // (Specific notation like "exd6 e.p." depends on PGN format)
    validate_pgn_structure(&pgn, "en passant game");
}
```

**Acceptance Criteria**:
- [ ] Castling tests written (may skip if fixtures not ready)
- [ ] Promotion tests written
- [ ] En passant tests written
- [ ] Tests pass when fixtures available

---

## Task 9B.3: Performance Benchmarking

### 9B.3.1: Setup Criterion Benchmarks

**File**: `benches/parsing_benchmarks.rs` (new file)

**Objective**: Measure parsing performance with criterion.

**First, update Cargo.toml**:

```toml
# In crates/core/Cargo.toml

[[bench]]
name = "parsing_benchmarks"
harness = false  # Use criterion's harness
```

**Implementation**:

```rust
//! Performance benchmarks for SCID parsing
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use scidtopgn_core::{ScidReader, PgnOptions};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Benchmark: Opening a database (header + index parsing)
fn bench_open_database(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");

    c.bench_function("open_database", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            black_box(reader);
        });
    });
}

/// Benchmark: Parsing a single game
fn bench_parse_single_game(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    c.bench_function("parse_single_game", |b| {
        b.iter(|| {
            let game = reader.game(black_box(0)).unwrap();
            black_box(game);
        });
    });
}

/// Benchmark: Generating PGN for a single game
fn bench_generate_pgn(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();

    c.bench_function("generate_pgn", |b| {
        b.iter(|| {
            let pgn = game.to_pgn().unwrap();
            black_box(pgn);
        });
    });
}

/// Benchmark: Complete workflow (parse + generate PGN)
fn bench_complete_workflow(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");

    c.bench_function("complete_workflow", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            let game = reader.game(0).unwrap();
            let pgn = game.to_pgn().unwrap();
            black_box(pgn);
        });
    });
}

/// Benchmark: Iterating all games
fn bench_iterate_all_games(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");

    c.bench_function("iterate_all_games", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            let mut count = 0;
            for game in reader.games() {
                count += 1;
                black_box(game.unwrap());
            }
            black_box(count);
        });
    });
}

/// Benchmark: Throughput with different database sizes
fn bench_throughput_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    // Test with different sizes if fixtures exist
    let sizes = vec![
        ("minimal", 1),
        ("medium", 100),     // If fixture exists
        ("large", 10000),    // If fixture exists
    ];

    for (name, game_count) in sizes {
        let path = fixture_path(&format!("{}/{}.si4", name, name));

        if !path.exists() {
            eprintln!("Skipping benchmark: {} (fixture not found)", name);
            continue;
        }

        group.throughput(Throughput::Elements(game_count as u64));

        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &path,
            |b, path| {
                b.iter(|| {
                    let reader = ScidReader::open(black_box(path)).unwrap();
                    let mut count = 0;
                    for game in reader.games() {
                        let game = game.unwrap();
                        let _pgn = game.to_pgn().unwrap();
                        count += 1;
                    }
                    black_box(count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Memory allocation during parsing
fn bench_memory_allocation(c: &mut Criterion) {
    let db_path = fixture_path("minimal/minimal.si4");

    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();

            // This measures allocations for game parsing
            for game in reader.games() {
                let game = game.unwrap();
                black_box(game);
            }
        });
    });
}

/// Benchmark: PGN options (compact vs verbose)
fn bench_pgn_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("pgn_formats");
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    group.bench_function("compact", |b| {
        let options = PgnOptions {
            compact: true,
            ..Default::default()
        };

        b.iter(|| {
            let mut output = Vec::new();
            reader.write_pgn(black_box(&mut output), &options).unwrap();
            black_box(output);
        });
    });

    group.bench_function("verbose", |b| {
        let options = PgnOptions {
            verbose: true,
            ..Default::default()
        };

        b.iter(|| {
            let mut output = Vec::new();
            reader.write_pgn(black_box(&mut output), &options).unwrap();
            black_box(output);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_open_database,
    bench_parse_single_game,
    bench_generate_pgn,
    bench_complete_workflow,
    bench_iterate_all_games,
    bench_throughput_by_size,
    bench_memory_allocation,
    bench_pgn_formats,
);

criterion_main!(benches);
```

**Acceptance Criteria**:
- [ ] Benchmark suite compiles
- [ ] All benchmarks run successfully
- [ ] Performance baseline established
- [ ] Throughput measured

**Validation**:
```bash
cargo bench

# Expected output:
# open_database          time:   [150.23 µs 152.41 µs 154.89 µs]
# parse_single_game      time:   [12.456 µs 12.789 µs 13.123 µs]
# generate_pgn           time:   [45.678 µs 46.234 µs 46.890 µs]
# complete_workflow      time:   [210.45 µs 215.67 µs 221.34 µs]
# iterate_all_games      time:   [25.678 µs 26.123 µs 26.678 µs]

# Performance targets (these are aspirational):
# - Open database: <200 µs
# - Parse single game: <20 µs
# - Generate PGN: <50 µs
# - Throughput: >1000 games/second
```

---

### 9B.3.2: Profile Hot Paths

**File**: `benches/profiling.rs`

**Objective**: Identify performance bottlenecks using profiling.

**Implementation**:

```bash
# Install profiling tools
cargo install cargo-flamegraph

# Generate flame graph (on Linux/macOS)
cargo flamegraph --bench parsing_benchmarks

# This creates flamegraph.svg showing where time is spent

# On any platform, use perf with criterion
cargo bench -- --profile-time=10
```

**Analysis checklist**:
- [ ] Index parsing takes <10% of total time
- [ ] Name decompression takes <15% of total time
- [ ] Move decoding takes <30% of total time
- [ ] PGN formatting takes <20% of total time
- [ ] No unexpected hot paths

---

## Task 9B.4: Memory Profiling

### 9B.4.1: Measure Memory Usage

**File**: `benches/memory_benchmark.rs`

**Objective**: Verify memory usage is reasonable for large databases.

**Implementation**:

```rust
//! Memory usage benchmarks
//!
//! These tests measure peak memory usage during parsing.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Custom allocator to track memory usage
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let size = layout.size();
            let old = ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let new = old + size;

            // Update peak
            let mut peak = PEAK_ALLOCATED.load(Ordering::Relaxed);
            while new > peak {
                match PEAK_ALLOCATED.compare_exchange_weak(
                    peak,
                    new,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

#[cfg(test)]
mod tests {
    use super::*;
    use scidtopgn_core::ScidReader;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn reset_memory_tracking() {
        ALLOCATED.store(0, Ordering::Relaxed);
        PEAK_ALLOCATED.store(0, Ordering::Relaxed);
    }

    fn get_peak_memory() -> usize {
        PEAK_ALLOCATED.load(Ordering::Relaxed)
    }

    #[test]
    fn test_memory_usage_single_game() {
        reset_memory_tracking();

        let db_path = fixture_path("minimal/minimal.si4");
        let reader = ScidReader::open(&db_path).unwrap();

        let game = reader.game(0).unwrap();
        let _pgn = game.to_pgn().unwrap();

        let peak = get_peak_memory();

        println!("Peak memory for single game: {} KB", peak / 1024);

        // Single game should use <100 KB
        assert!(
            peak < 100 * 1024,
            "Single game used {} KB, expected <100 KB",
            peak / 1024
        );
    }

    #[test]
    fn test_memory_usage_iterator() {
        reset_memory_tracking();

        let db_path = fixture_path("minimal/minimal.si4");
        let reader = ScidReader::open(&db_path).unwrap();

        // Iterate all games (memory should not grow linearly)
        for game in reader.games() {
            let game = game.unwrap();
            let _pgn = game.to_pgn().unwrap();
        }

        let peak = get_peak_memory();

        println!("Peak memory for iterator: {} KB", peak / 1024);

        // Iterator should not accumulate memory
        // (If we had 1000 games, peak should still be low)
        assert!(
            peak < 1024 * 1024, // <1 MB
            "Iterator used {} MB, should stay low",
            peak / (1024 * 1024)
        );
    }
}
```

**Acceptance Criteria**:
- [ ] Memory tracking compiles
- [ ] Single game uses <100 KB
- [ ] Iterator doesn't accumulate memory
- [ ] Peak memory reasonable

**Validation**:
```bash
cargo test --test memory_benchmark -- --nocapture

# Expected output:
# Peak memory for single game: 45 KB
# Peak memory for iterator: 128 KB
```

---

## Task 9B.5: Fuzzing

### 9B.5.1: Setup Cargo Fuzz

**Objective**: Use fuzzing to discover edge cases and crashes.

**Implementation**:

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Initialize fuzzing in the project
cd crates/core
cargo fuzz init

# This creates: fuzz/fuzz_targets/
```

**File**: `fuzz/fuzz_targets/parse_index.rs`

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use scidtopgn_core::index_parser::Si4Header;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as header
    // Should never panic, only return error
    let _ = Si4Header::parse(data);
});
```

**File**: `fuzz/fuzz_targets/parse_names.rs`

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use scidtopgn_core::name_parser::NameDatabase;

fuzz_target!(|data: &[u8]| {
    // Try to parse arbitrary bytes as name database
    let _ = NameDatabase::parse(data);
});
```

**File**: `fuzz/fuzz_targets/decode_moves.rs`

```rust
#![no_main]

use libfuzzer_sys::fuzz_target;
use scidtopgn_core::move_decoder::ScidPosition;

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }

    let mut pos = ScidPosition::new();

    // Try to decode random bytes as moves
    for chunk in data.chunks(2) {
        let piece_num = chunk[0];
        let move_value = chunk.get(1).copied().unwrap_or(0);

        // Should not panic on invalid moves
        let _ = pos.make_move(piece_num, move_value);
    }
});
```

**Acceptance Criteria**:
- [ ] Fuzzing setup complete
- [ ] Three fuzz targets created
- [ ] Can run fuzz tests

**Validation**:
```bash
# Run fuzzer for 60 seconds
cargo fuzz run parse_index -- -max_total_time=60

# Should output:
# #1234567: cov: 234 ft: 567 corp: 89 exec/s: 12345
# ...no crashes...

# Run all fuzz targets
cargo fuzz run parse_names -- -max_total_time=60
cargo fuzz run decode_moves -- -max_total_time=60
```

If fuzzing finds a crash, it will save the input to `fuzz/artifacts/`. Fix the bug and re-run.

---

## Task 9B.6: Real-World Validation

### 9B.6.1: Test with Large Databases

**Objective**: Validate with real SCID databases (100,000+ games).

**Implementation**:

Since we can't include large databases in the repository, create a test that uses them if available:

**File**: `tests/large_database_test.rs`

```rust
//! Tests with large real-world databases
//!
//! These tests require large SCID databases to be placed in tests/fixtures/large/
//! They are ignored by default and only run when explicitly requested.

use scidtopgn_core::{ScidReader, PgnOptions};
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::Instant;

fn large_db_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("large")
        .join("large.si4")
}

#[test]
#[ignore] // Only run with: cargo test --test large_database_test -- --ignored
fn test_parse_large_database() {
    let db_path = large_db_path();

    if !db_path.exists() {
        eprintln!("Skipping: large database not found at {:?}", db_path);
        eprintln!("To run this test, place a large SCID database at that location.");
        return;
    }

    println!("Opening large database...");
    let start = Instant::now();

    let reader = ScidReader::open(&db_path)
        .expect("Should open large database");

    let open_time = start.elapsed();
    println!("Opened in {:?}", open_time);
    println!("Game count: {}", reader.game_count());

    // Parse all games
    println!("Parsing all games...");
    let start = Instant::now();

    let mut success_count = 0;
    let mut error_count = 0;

    for (i, game_result) in reader.games().enumerate() {
        match game_result {
            Ok(game) => {
                // Verify we can generate PGN
                match game.to_pgn() {
                    Ok(_pgn) => success_count += 1,
                    Err(e) => {
                        eprintln!("Game {}: PGN generation failed: {}", i, e);
                        error_count += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Game {}: Parse failed: {}", i, e);
                error_count += 1;
            }
        }

        if (i + 1) % 10000 == 0 {
            println!("  Processed {} games...", i + 1);
        }
    }

    let parse_time = start.elapsed();

    println!("\nResults:");
    println!("  Success: {}", success_count);
    println!("  Errors: {}", error_count);
    println!("  Total time: {:?}", parse_time);
    println!("  Games/second: {:.2}", success_count as f64 / parse_time.as_secs_f64());

    // Success criteria: <1% error rate
    let error_rate = error_count as f64 / (success_count + error_count) as f64;
    assert!(
        error_rate < 0.01,
        "Error rate too high: {:.2}%",
        error_rate * 100.0
    );

    // Performance criteria: >500 games/second
    let games_per_sec = success_count as f64 / parse_time.as_secs_f64();
    assert!(
        games_per_sec > 500.0,
        "Performance too slow: {:.2} games/sec (expected >500)",
        games_per_sec
    );
}

#[test]
#[ignore]
fn test_export_large_database_to_pgn() {
    let db_path = large_db_path();

    if !db_path.exists() {
        eprintln!("Skipping: large database not found");
        return;
    }

    let output_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("output")
        .join("large_export.pgn");

    std::fs::create_dir_all(output_path.parent().unwrap()).unwrap();

    println!("Exporting to PGN: {:?}", output_path);
    let start = Instant::now();

    let reader = ScidReader::open(&db_path).unwrap();
    let output_file = File::create(&output_path).unwrap();
    let mut writer = BufWriter::new(output_file);

    reader.write_pgn(&mut writer, &PgnOptions::default())
        .expect("Should export to PGN");

    writer.flush().unwrap();

    let export_time = start.elapsed();

    println!("Export completed in {:?}", export_time);
    println!("Games/second: {:.2}", reader.game_count() as f64 / export_time.as_secs_f64());

    // Verify file was created and has content
    let metadata = std::fs::metadata(&output_path).unwrap();
    println!("Output file size: {} MB", metadata.len() / (1024 * 1024));

    assert!(metadata.len() > 0, "Output file should not be empty");
}

#[test]
#[ignore]
fn test_memory_usage_large_database() {
    let db_path = large_db_path();

    if !db_path.exists() {
        eprintln!("Skipping: large database not found");
        return;
    }

    // This test should be run with: cargo test --test large_database_test -- --ignored --nocapture
    // While monitoring memory with: top, htop, or Activity Monitor

    println!("Starting memory test...");
    println!("Monitor memory usage with 'top' or similar tool");
    println!("Press Ctrl+C to stop");

    let reader = ScidReader::open(&db_path).unwrap();
    println!("Database opened. Game count: {}", reader.game_count());

    // Iterate without storing results (should not accumulate memory)
    for (i, game_result) in reader.games().enumerate() {
        let game = game_result.unwrap();
        let _pgn = game.to_pgn().unwrap();

        if (i + 1) % 1000 == 0 {
            println!("Processed {} games (memory should be stable)", i + 1);
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    println!("Test complete. Memory should not have grown linearly.");
}
```

**Acceptance Criteria**:
- [ ] Large database test written
- [ ] Can parse 100,000+ games
- [ ] Error rate <1%
- [ ] Performance >500 games/sec
- [ ] Memory usage stable

**Validation**:
```bash
# Place a large SCID database at tests/fixtures/large/large.si4
# (Download from public chess databases)

cargo test --test large_database_test -- --ignored --nocapture

# Expected output:
# Opening large database...
# Opened in 123.45ms
# Game count: 154321
# Parsing all games...
#   Processed 10000 games...
#   Processed 20000 games...
#   ...
# Results:
#   Success: 154200
#   Errors: 121
#   Total time: 245.67s
#   Games/second: 627.89
```

---

## Task 9B.7: CI/CD Integration

### 9B.7.1: GitHub Actions Workflow

**File**: `.github/workflows/ci.yml`

**Objective**: Automate testing on every commit.

**Implementation**:

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: ${{ matrix.rust }}
        override: true
        components: rustfmt, clippy

    - name: Cache cargo registry
      uses: actions/cache@v3
      with:
        path: ~/.cargo/registry
        key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo index
      uses: actions/cache@v3
      with:
        path: ~/.cargo/git
        key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

    - name: Cache cargo build
      uses: actions/cache@v3
      with:
        path: target
        key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

    - name: Check formatting
      run: cargo fmt -- --check

    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings

    - name: Build
      run: cargo build --verbose

    - name: Run unit tests
      run: cargo test --lib --verbose

    - name: Run integration tests
      run: cargo test --test integration_test --verbose

    - name: Run doc tests
      run: cargo test --doc --verbose

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Install tarpaulin
      run: cargo install cargo-tarpaulin

    - name: Generate coverage
      run: cargo tarpaulin --out Xml --verbose

    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v3
      with:
        files: ./cobertura.xml
        fail_ci_if_error: true

  benchmark:
    name: Performance Benchmarks
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
        override: true

    - name: Run benchmarks
      run: cargo bench --no-fail-fast

    - name: Store benchmark results
      uses: benchmark-action/github-action-benchmark@v1
      with:
        tool: 'cargo'
        output-file-path: target/criterion/output.txt
        github-token: ${{ secrets.GITHUB_TOKEN }}
        auto-push: true

  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install cargo-audit
      run: cargo install cargo-audit

    - name: Run security audit
      run: cargo audit
```

**Acceptance Criteria**:
- [ ] CI workflow file created
- [ ] Tests run on push/PR
- [ ] Multiple OS support (Linux, Windows, macOS)
- [ ] Code coverage uploaded
- [ ] Benchmarks tracked

**Validation**:
```bash
# Commit and push
git add .github/workflows/ci.yml
git commit -m "Add CI/CD workflow"
git push

# Check GitHub Actions tab to see tests running
```

---

## Success Metrics

### Integration Test Coverage

**Goal**: All major workflows covered

**Checklist**:
- [x] Database opening
- [x] Complete game parsing
- [x] PGN generation
- [x] Iterator functionality
- [x] Error handling
- [x] PGN format options
- [x] Special moves (if fixtures available)
- [x] Large database handling (if available)

### Performance Targets

**Goal**: Meet or exceed performance targets

| Operation | Target | Measurement |
|-----------|--------|-------------|
| Open database | <200 µs | `cargo bench` |
| Parse single game | <20 µs | `cargo bench` |
| Generate PGN | <50 µs | `cargo bench` |
| Complete workflow | <300 µs | `cargo bench` |
| Throughput | >500 games/s | Large DB test |

### Memory Efficiency

**Goal**: Constant memory usage regardless of database size

| Scenario | Target | Measurement |
|----------|--------|-------------|
| Single game | <100 KB | Memory benchmark |
| Iterator | <1 MB | Memory benchmark |
| Large DB (100k games) | <10 MB | Monitor during test |

### Code Quality

**Goal**: High quality, maintainable code

- [ ] Code coverage >90%
- [ ] All clippy warnings resolved
- [ ] No security vulnerabilities (cargo audit)
- [ ] Benchmarks tracked over time
- [ ] CI passes on all platforms

---

## Common Pitfalls

### Pitfall 1: Not Testing with Real Data

**WRONG**:
```rust
#[test]
fn test_integration() {
    // Only using synthetic test data
    let data = create_minimal_test_data();
    // ...
}
```

**RIGHT**:
```rust
#[test]
fn test_integration_real_database() {
    // Use actual SCID database file
    let reader = ScidReader::open("tests/fixtures/real_db.si4").unwrap();

    // Test with real-world complexity
    for game in reader.games() {
        // ...
    }
}
```

### Pitfall 2: Ignoring Performance Regressions

**WRONG**:
```bash
# Run benchmarks but don't track results
cargo bench
# Forget about it
```

**RIGHT**:
```bash
# Run benchmarks and save baseline
cargo bench -- --save-baseline main

# After changes, compare
cargo bench -- --baseline main

# Fail if performance degrades >10%
```

### Pitfall 3: Not Testing Error Cases in Integration

**WRONG**:
```rust
#[test]
fn test_parse_all() {
    for game in reader.games() {
        let game = game.unwrap(); // Panic on error!
        // ...
    }
}
```

**RIGHT**:
```rust
#[test]
fn test_parse_all_with_error_tracking() {
    let mut errors = Vec::new();

    for (i, result) in reader.games().enumerate() {
        match result {
            Ok(game) => { /* process */ },
            Err(e) => {
                errors.push((i, e));
            }
        }
    }

    // Report errors
    if !errors.is_empty() {
        eprintln!("Failed to parse {} games:", errors.len());
        for (i, e) in &errors {
            eprintln!("  Game {}: {}", i, e);
        }
    }

    // Allow small error rate
    assert!(errors.len() < 10, "Too many parsing errors");
}
```

### Pitfall 4: Not Measuring Memory in Integration Tests

**WRONG**:
```rust
#[test]
fn test_large_db() {
    // Parse entire database
    let all_games: Vec<_> = reader.games()
        .collect(); // Accumulates ALL games in memory!

    // This uses O(n) memory instead of O(1)
}
```

**RIGHT**:
```rust
#[test]
fn test_large_db_streaming() {
    // Process one at a time
    for game in reader.games() {
        let game = game.unwrap();
        process_game(&game);
        // Game dropped here, memory freed
    }

    // Memory usage stays constant
}
```

### Pitfall 5: Flaky Integration Tests

**WRONG**:
```rust
#[test]
fn test_with_timing() {
    let start = Instant::now();
    parse_database();

    assert!(start.elapsed() < Duration::from_secs(1));
    // FLAKY: May fail on slower CI machines!
}
```

**RIGHT**:
```rust
#[test]
fn test_with_reasonable_timeout() {
    let start = Instant::now();
    parse_database();

    // Use generous timeout for CI
    assert!(
        start.elapsed() < Duration::from_secs(30),
        "Parsing took too long (likely indicates a bug)"
    );
}

// For strict performance testing, use benchmarks instead
```

---

## Validation Commands

Comprehensive validation checklist:

```bash
# 1. Run all unit tests
cargo test --lib

# 2. Run all integration tests
cargo test --test integration_test
cargo test --test large_database_test -- --ignored  # If large DB available

# 3. Run benchmarks
cargo bench

# 4. Check code coverage
cargo tarpaulin --out Html --output-dir coverage
# Open coverage/index.html - should show >90%

# 5. Run fuzzer (for 5 minutes each)
cargo fuzz run parse_index -- -max_total_time=300
cargo fuzz run parse_names -- -max_total_time=300
cargo fuzz run decode_moves -- -max_total_time=300

# 6. Check for memory leaks (with valgrind on Linux)
valgrind --leak-check=full cargo test

# 7. Security audit
cargo audit

# 8. Code quality checks
cargo fmt -- --check
cargo clippy -- -D warnings

# 9. Documentation tests
cargo test --doc

# 10. Cross-platform check (if applicable)
cargo build --target x86_64-pc-windows-gnu
cargo build --target x86_64-apple-darwin

# 11. Release build verification
cargo build --release
cargo test --release
./target/release/scidtopgn --help

# 12. Stress test (if large DB available)
time ./target/release/scidtopgn tests/fixtures/large/large.si4 -o /tmp/output.pgn
# Should complete without errors or excessive memory
```

Expected final status:
```
✅ 102+ unit tests passing
✅ 15+ integration tests passing
✅ Code coverage >90%
✅ All benchmarks within targets
✅ No fuzzer crashes
✅ No memory leaks
✅ No security vulnerabilities
✅ CI passing on all platforms
✅ Documentation complete
```

---

## Summary

Phase 9B completes the testing suite with:

1. **Integration Tests** (15+ tests)
   - Complete database parsing workflows
   - Real-world data validation
   - Error handling in realistic scenarios
   - PGN format options verification

2. **Performance Benchmarking**
   - Criterion-based micro-benchmarks
   - Throughput measurements
   - Performance regression tracking
   - Profile-guided optimization

3. **Memory Profiling**
   - Peak memory measurements
   - Memory leak detection
   - Streaming verification (constant memory)

4. **Fuzzing**
   - Three fuzz targets (index, names, moves)
   - Crash detection
   - Edge case discovery

5. **Real-World Validation**
   - Large database testing (100,000+ games)
   - Error rate measurement (<1%)
   - Performance validation (>500 games/sec)
   - Export verification

6. **CI/CD Pipeline**
   - Automated testing on push/PR
   - Cross-platform support
   - Code coverage tracking
   - Security auditing
   - Benchmark tracking

**Combined with Phase 9A**, this provides:
- **117+ total tests** (102 unit + 15 integration)
- **>90% code coverage**
- **Performance benchmarks** for all operations
- **Memory profiling** ensuring efficiency
- **Fuzzing** for robustness
- **Real-world validation** with large databases
- **Automated CI/CD** for ongoing quality

The SCID to PGN converter is now **production-ready** with comprehensive testing at all levels: unit, integration, performance, and real-world validation.

---

## Next Phase

After completing Phase 9B, proceed to:

**Phase 10: Documentation & Polish**
- Complete API documentation with rustdoc
- Create usage examples
- Write comprehensive README
- Add code examples
- Final performance optimization
- Release preparation

The testing foundation ensures we can refactor, optimize, and enhance the codebase with confidence that nothing breaks.
