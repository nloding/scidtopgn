# Implementation Guide - Task-by-Task Orchestration

This guide provides a sequential checklist for implementing the SCID to PGN converter. Work through tasks one at a time, verifying each before proceeding.

## How to Use This Guide

1. **Work on ONE task at a time** - Don't try to implement an entire phase
2. **Read the task section** in the referenced phase document
3. **Implement the code** as specified
4. **Run tests** to verify before moving on
5. **Check off completed tasks** in this file

## Prerequisites

- Rust toolchain installed (1.70+)
- Test data files in `tests/data/` (five.si4, five.sn4, five.sg4)
- Access to phase documents for detailed specifications

---

## Phase 1: Foundation

**Document**: `PHASE_1_FOUNDATION.md`
**Goal**: Project setup, error types, core data structures

### Section 1.1: Project Structure

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 1.1.1 | Initialize Git Repository | 36 | [X] |
| 1.1.2 | Create Workspace Root Configuration | 115 | [X] |
| 1.1.3 | Create Core Library Crate Structure | 181 | [X] |
| 1.1.4 | Create CLI Binary Crate Structure | 367 | [X] |
| 1.1.5 | Create README and Documentation | 451 | [X] |
| 1.1.6 | Verify Complete Workspace Build | 574 | [X] |

**Verification**: `cargo build` succeeds with no errors

### Section 1.2: Core Types

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 1.2.1 | Implement Error Types | 654 | [X] |
| 1.2.2 | Implement Core Types (GameDate, GameResult) | 883 | [X] |
| 1.2.3 | Implement Prelude Module | 1222 | [X] |
| 1.2.4 | Create Comprehensive Test Suite | 1309 | [X] |
| 1.2.5 | Final Phase 1 Validation | 1361 | [X] |

**Verification**: `cargo test -p scidtopgn-core` passes all tests

---

## Phase 2: Index File Parser (.si4)

**Document**: `PHASE_2_INDEX_FILE_PARSER.md`
**Goal**: Parse SI4 header and game index entries
**Dependencies**: Phase 1 complete

### Section 2.1: SI4 Header Parsing

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 2.1.1 | Create Header Data Structure (Si4Header) | 94 | [X] |
| 2.1.2 | Implement Header Parsing Function | 775 | [X] |
| 2.1.3 | Create Header Parsing Tests | 927 | [X] |
| 2.1.4 | Add Test Data Files | 1091 | [X] |

**Verification**: `cargo test -p scidtopgn-core si4_header` passes

### Section 2.2: Game Index Entry Parsing

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 2.2.1 | Create Game Index Entry Structure | 1175 | [X] |
| 2.2.2 | Implement Entry Parsing - Part 1 (Simple Fields) | 1540 | [X] |
| 2.2.3 | Implement Entry Parsing - Part 2 (Packed IDs) | 1688 | [X] |
| 2.2.4 | Implement Date Parsing (CRITICAL) | 1804 | [X] |
| 2.2.5 | Implement Remaining Fields (Result, ELO, Counts) | 2027 | [ ] |
| 2.2.6 | Integration Testing with Real Data | 2508 | [ ] |
| 2.2.7 | Create Helper Functions and Documentation | 2605 | [ ] |
| 2.2.8 | Final Phase 2 Validation | 2771 | [ ] |

**Verification**: `cargo test -p scidtopgn-core index` passes (50+ tests)

**Key Types Created**:
- `Si4Header` - Header structure with custom_flag_names
- `GameIndexEntry` - 47-byte entry with all fields
- `RatingType` - Enum for rating systems (Elo, USCF, etc.)
- `MaterialSignature` - Final position material encoding
- `game_flags` module - Flag constants
- Helper methods: `is_packed()`, `material_signature()`, `eco_to_string()`

---

## Phase 3: Name File Parser (.sn4)

**Document**: `PHASE_3_NAME_FILE_PARSER.md`
**Goal**: Parse name database with front-coding decompression
**Dependencies**: Phase 1 complete

### Section 3.1: SN4 Header

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 3.1.1 | Create Header Data Structure | 101 | [ ] |
| 3.1.2 | Implement Header Parsing Function | 472 | [ ] |

### Section 3.2: Name Database Parsing

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 3.2.1 | Implement Variable-Length Integer Reading | 640 | [ ] |
| 3.2.2 | Implement String Cleaning Function | 752 | [ ] |
| 3.2.3 | Implement Front-Coding Decompression | 841 | [ ] |
| 3.2.4 | Implement Complete Name Database Parser | 1079 | [ ] |
| 3.2.5 | Integration Testing with Real Data | 1240 | [ ] |
| 3.2.6 | Connect Names to Index Entries | 1390 | [ ] |
| 3.2.7 | Performance Testing and Optimization | 1511 | [ ] |
| 3.2.8 | Final Phase 3 Validation | 1575 | [ ] |
| 3.2.9 | Name ID Lookup and Edge Cases | 1670 | [ ] |
| 3.2.10 | Round String Formats and PGN Escaping | 1793 | [ ] |

**Verification**: `cargo test -p scidtopgn-core names` passes

**Key Types Created**:
- `Sn4Header` - Name file header
- `NameDatabase` - Player, event, site, round names
- `NameType` enum - Player, Event, Site, Round

---

## Phase 4: Game File Structure (.sg4)

**Document**: `PHASE_4_GAME_FILE_STRUCTURE.md`
**Goal**: Parse game data, tags, comments (NOT moves yet)
**Dependencies**: Phase 2 complete (need index offsets)

### Section 4.1: Game Boundary Detection

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 4.1.1 | Understand Why Scanning Fails (Educational) | 197 | [ ] |
| 4.1.2 | Implement Game Data Reading | 349 | [ ] |
| 4.1.3 | Implement zlib Decompression (CRITICAL) | 492 | [ ] |

**Verification**: Can read raw game bytes using index offsets

### Section 4.2: Tag and Flags Parsing

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 4.2.1 | Understand Tag Section Structure (Educational) | 768 | [ ] |
| 4.2.2 | Implement Tag Parsing Function | 1000 | [ ] |
| 4.2.3 | Implement Flags and FEN Parsing | 1331 | [ ] |
| 4.2.4 | Integration Testing with Real Data | 1601 | [ ] |
| 4.2.5 | Final Phase 4 Validation | 1771 | [ ] |

### Section 4.3: Game Data Structure & Comments

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 4.3.1 | Understand Game Data Layout (Educational) | 1850 | [ ] |
| 4.3.2 | Two-Part Comment Encoding (CRITICAL) | 1967 | [ ] |
| 4.3.3 | Update parse_game_structure for Comments | 2236 | [ ] |
| 4.3.4 | Comment Tree Traversal Algorithm | 2443 | [ ] |
| 4.3.5 | NAG Handling | 2651 | [ ] |
| 4.3.6 | Variation Data Structures | 2755 | [ ] |
| 4.3.7 | Integration Testing for Comment Separation | 3012 | [ ] |
| 4.3.8 | Phase 4.3 Completion Checklist | 3108 | [ ] |

**Verification**: `cargo test -p scidtopgn-core games` passes (55+ tests)

**Key Types Created**:
- `GameData` - Tags, flags, move_data, comment_data
- `decompress_game_data()` - zlib decompression
- `read_and_decompress_game()` - High-level game reading
- `special_bytes` module - 0x0B-0x0F markers

---

## Phase 5: Move Parsing

**Document**: `PHASE_5_MOVE_PARSING.md`
**Goal**: Decode SCID binary moves to shakmaty::Move
**Dependencies**: Phase 4 complete

### Section 5.1: Position Tracking

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 5.1.1 | Design Piece Numbering System | 147 | [ ] |
| 5.1.2 | Implement ScidPosition Wrapper | 538 | [ ] |
| 5.1.3 | Chess960 Support | 834 | [ ] |

### Section 5.2: Move Decoding

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 5.2.1 | Implement ByteStream for Multi-Byte Moves | 1121 | [ ] |
| 5.2.2 | Implement Piece-Specific Move Decoders | 1348 | [ ] |
| 5.2.3 | Implement High-Level Move Decoder | 1876 | [ ] |
| 5.2.4 | Game Tree and Variation Data Structures | 2162 | [ ] |

### Section 5.3: Integration

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 5.3.1 | Connect Move Decoder to Game Parser | 2666 | [ ] |
| 5.3.1.1 | Pre-Game Comment Handling (Gap 3) | 2860 | [ ] |
| 5.3.2 | Comprehensive Real-World Validation | 3020 | [ ] |

**Verification**: `cargo test -p scidtopgn-core moves` passes (50+ tests)

**Key Types Created**:
- `ScidPosition` - Position + piece mapping
- `ScidMoveDecoder` - Main decoder
- `ByteStream` - Multi-byte move handling
- Lookup tables for all piece types

---

## Phase 6: PGN Output

**Document**: `PHASE_6_PGN_OUTPUT.md`
**Goal**: Generate valid PGN from decoded games
**Dependencies**: Phase 5 complete

### Section 6.1: SAN Generation

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 6.1.1 | Educational - Understanding SAN | 289 | [ ] |
| 6.1.2 | Implement SAN Generator Wrapper | 425 | [ ] |

### Section 6.2: PGN Tag Formatting

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 6.2.1 | Implement Seven Tag Roster | 912 | [ ] |
| 6.2.2 | Implement Supplemental Tags | 1167 | [ ] |

### Section 6.3: Move List Formatting

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 6.3.1 | Implement Move Number Formatting | 1545 | [ ] |

### Section 6.4: Complete PGN

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 6.4.1 | Implement Complete PGN Formatter | 1832 | [ ] |

**Verification**: `cargo test -p scidtopgn-core pgn` passes (45+ tests)

**Key Types Created**:
- `SevenTagRoster` - Required PGN tags
- `SupplementalTags` - Optional tags with rating types
- `PgnFormatter` - Complete PGN generation
- `rating_tag_name()` - Rating type-aware tag names

---

## Phase 7: Public API

**Document**: `PHASE_7_PUBLIC_API.md`
**Goal**: User-friendly API wrapping all internals
**Dependencies**: Phases 1-6 complete

### Section 7.1: API Design

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.1.1 | Educational - API Design Principles | 309 | [ ] |
| 7.1.2 | Design ScidReader API Surface | 417 | [ ] |

### Section 7.2: ScidReader Implementation

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.2.1 | Implement ScidReader::open() | 513 | [ ] |
| 7.2.2 | Implement Game Struct | 934 | [ ] |

### Section 7.3: Game Access

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.3.1 | Implement game() Method | 1271 | [ ] |
| 7.3.2 | Implement games() Iterator | 1357 | [ ] |

### Section 7.4: PGN Conversion

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.4.1 | Implement to_pgn() | 1507 | [ ] |
| 7.4.2 | Implement write_pgn() | 1557 | [ ] |
| 7.4.3 | Streaming Game Data Access | 1673 | [ ] |

### Section 7.5: Prelude

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.5 | Implement Prelude Module | 1784 | [ ] |

### Section 7.6: Error Recovery

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.6.1 | Error Types and Recoverability | 1932 | [ ] |
| 7.6.2 | Error Mode Configuration | 1992 | [ ] |
| 7.6.3 | ScidReader Methods with Recovery | 2120 | [ ] |

### Section 7.7: File Access Modes

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 7.7.1 | File Access Mode Configuration | 2274 | [ ] |
| 7.7.2 | ScidReader with Configurable Access | 2397 | [ ] |
| 7.7.3 | Feature Flag Configuration | 2613 | [ ] |

**Verification**: `cargo test -p scidtopgn-core` passes (25+ API tests)

**Key Types Exported**:
- `ScidReader` - Main entry point
- `Game` - Single game with metadata
- `PgnOptions` - Output configuration
- `RatingType`, `MaterialSignature` - From Phase 2
- Prelude module for convenient imports

---

## Phase 8: CLI Tool

**Document**: `PHASE_8_CLI_TOOL.md`
**Goal**: Command-line interface for conversion
**Dependencies**: Phase 7 complete

### Section 8.1: CLI Design

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 8.1.1 | Educational - CLI Design Principles | 339 | [ ] |
| 8.1.2 | Design Argument Structure | 428 | [ ] |

### Section 8.2: Argument Parsing

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 8.2.1 | Setup CLI Crate and Dependencies | 599 | [ ] |
| 8.2.2 | Implement Args Struct with Clap | 693 | [ ] |

### Section 8.3: Main Logic

| Task | Description | Line | Status |
|------|-------------|------|--------|
| 8.3.1 | Implement Main Function | 1132 | [ ] |

**Verification**: `cargo run -p scidtopgn -- --help` works

---

## Phase 9A: Unit Tests

**Document**: `PHASE_9A_UNIT_TESTS.md`
**Goal**: Comprehensive unit test coverage
**Dependencies**: Phases 1-8 complete

| Task | Description | Status |
|------|-------------|--------|
| 9A.1 | Test Framework Setup | [ ] |
| 9A.2 | Index File Parser Tests | [ ] |
| 9A.3 | Name File Parser Tests | [ ] |
| 9A.4 | Move Decoder Tests | [ ] |
| 9A.5 | PGN Formatter Tests | [ ] |
| 9A.6 | Error Handling Tests | [ ] |
| 9A.7 | Property-Based Testing | [ ] |

**Verification**: `cargo test` passes with >80% coverage

---

## Phase 9B: Integration & Validation

**Document**: `PHASE_9B_INTEGRATION_VALIDATION.md`
**Goal**: End-to-end testing with real databases
**Dependencies**: Phase 9A complete

| Task | Description | Status |
|------|-------------|--------|
| 9B.1 | Integration Test Framework | [ ] |
| 9B.2 | Real Database Validation | [ ] |
| 9B.3 | PGN Output Validation | [ ] |
| 9B.4 | Performance Benchmarks | [ ] |

**Verification**: All test databases convert correctly

---

## Phase 10: Documentation & Polish

**Document**: `PHASE_10_DOCUMENTATION_POLISH.md`
**Goal**: Final documentation, examples, release prep
**Dependencies**: Phases 9A-9B complete

| Task | Description | Status |
|------|-------------|--------|
| 10.1 | API Documentation | [ ] |
| 10.2 | README and Examples | [ ] |
| 10.3 | CHANGELOG | [ ] |
| 10.4 | Release Preparation | [ ] |

**Verification**: `cargo doc --open` shows complete docs

---

## Quick Reference: Critical Tasks

These tasks are marked CRITICAL and must be implemented correctly:

| Task | Why Critical |
|------|--------------|
| 4.1.3 | zlib decompression - most databases are compressed |
| 4.3.2 | Two-part comment encoding - comments stored separately from moves |
| 2.2.4 | Date parsing - complex bit packing |
| 5.2.2 | Move decoders - piece-specific lookup tables |

---

## Verification Commands

```bash
# After each phase, run these to verify:

# Build check
cargo build

# Run all tests
cargo test

# Run specific phase tests
cargo test -p scidtopgn-core index    # Phase 2
cargo test -p scidtopgn-core names    # Phase 3
cargo test -p scidtopgn-core games    # Phase 4
cargo test -p scidtopgn-core moves    # Phase 5
cargo test -p scidtopgn-core pgn      # Phase 6

# Check for warnings
cargo clippy

# Format code
cargo fmt
```

---

## Progress Summary

| Phase | Tasks | Completed | Status |
|-------|-------|-----------|--------|
| 1 | 11 | 0 | Not Started |
| 2 | 12 | 0 | Not Started |
| 3 | 12 | 0 | Not Started |
| 4 | 16 | 0 | Not Started |
| 5 | 10 | 0 | Not Started |
| 6 | 5 | 0 | Not Started |
| 7 | 13 | 0 | Not Started |
| 8 | 5 | 0 | Not Started |
| 9A | 7 | 0 | Not Started |
| 9B | 4 | 0 | Not Started |
| 10 | 4 | 0 | Not Started |
| **Total** | **99** | **0** | **0%** |
