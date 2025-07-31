# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**scidtopgn** is a Rust CLI tool that converts SCID chess databases (.si4/.sg4/.sn4) to PGN format. SCID (Shane's Chess Information Database) uses a proprietary binary format requiring specialized parsing.

## Build and Development Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run with development settings (first 10 games only)
cargo run -- database_name

# Run with all games
cargo run -- --max-games=0 database_name

# Run with custom output
cargo run -- -o output.pgn database_name

# Run comprehensive test suite
cargo test

# Run specific test suites
cargo test --test date_extraction_tests
cargo test --test comprehensive_date_test

# Run unit tests only  
cargo test --lib
```

## Architecture Overview

The codebase follows a modular Rust CLI structure:

- **src/main.rs**: CLI entry point using `clap` for argument parsing. Contains comprehensive documentation about recent major fixes.
- **src/scid/**: SCID database parsing modules
  - `database.rs`: Main coordination and ScidDatabase struct
  - `index.rs`: Parses .si4 index files (game metadata, dates, player IDs)
  - `names.rs`: Parses .sn4 name files (player/event/site names with front-coded compression)
  - `games.rs`: Parses .sg4 game files (chess moves, variations, comments)
  - `moves.rs`: Chess move encoding/decoding logic
  - `events.rs`: Event-related parsing
- **src/pgn/**: PGN export functionality
  - `exporter.rs`: Converts parsed SCID data to standard PGN format

## Test Data

When validating whether data was parsed from the SCID database files correctly, it can
be validated against the data in the the `test/data` directory.

**CRITICAL: NO FILES FROM THE `test/data` directory should be removed.**

- `five.pgn` - contains five chess games in PGN format, all of which are in the five.sng4|si4|sn4 database set
- `five.sg4` - game/move database, containing the five games from `five.pgn`
- `five.si4` - index database, containing the five games from `five.pgn`
- `five.sn4` - name database, containing the five games from `five.pgn`

## SCID File Format Structure

SCID databases consist of three files:
- **basename.si4**: Index file with game metadata (dates, player/event IDs, file offsets)
- **basename.sn4**: Name database with front-coded string compression
- **basename.sg4**: Game data (moves, annotations, variations)

## Critical Implementation Details

### SCID Database Format

There is no documentation for the SCID database files. Instead, you must reverse
engineer from the source code.
- si4 (index) code: https://github.com/nloding/scidvspc/blob/main/src/index.cpp
  - Reading an event date: https://github.com/nloding/scidvspc/blob/main/src/index.cpp#L120
  - Writing an event date: https://github.com/nloding/scidvspc/blob/main/src/index.cpp#L137
  - Reading the index: https://github.com/nloding/scidvspc/blob/main/src/index.cpp#L173
- sn4 (name) code: https://github.com/nloding/scidvspc/blob/main/src/namebase.cpp
  - Header: https://github.com/nloding/scidvspc/blob/main/src/namebase.cpp#L74
  - Read file: https://github.com/nloding/scidvspc/blob/main/src/namebase.cpp#L149
- sg4 (game) code: https://github.com/nloding/scidvspc/blob/main/src/gfile.cpp

### Date Parsing 🚨 CRITICAL ISSUE

**Status**: 🚨 **HARDCODED AND BROKEN** - Date parsing always returns "2022.12.19" regardless of actual game dates.

**⚠️ CRITICAL BUG**: The current implementation hardcodes the date pattern `0x0944cd93` and always returns it, making the date parsing completely non-functional for real-world databases containing games with different dates.

#### Root Cause Analysis
- **File**: `src/scid/index.rs` lines 209, 227, 236
- **Problem**: All games return hardcoded "2022.12.19" regardless of actual dates
- **False Positive**: Tests pass because test dataset only contains games from 2022.12.19
- **Impact**: Any real SCID database will show incorrect dates

#### Current Broken Implementation
```rust
// Line 209: Hardcoded pattern definition
let discovered_pattern = 0x0944cd93u32;

// Lines 227 & 236: Always returns hardcoded pattern
discovered_pattern  // ALWAYS returns 2022.12.19
```

#### Research Findings (COMPLETED)
**SCID Date Encoding Format** (from official source code analysis):
- **Format**: 32-bit unsigned integer with no year offset
- **Encoding**: `DATE_MAKE(year, month, day) = ((year << 9) | (month << 5) | day)`
- **Bit Layout**:
  - Bits 0-4: Day (5 bits)
  - Bits 5-8: Month (4 bits)  
  - Bits 9-19: Year (11 bits, supports up to year 2047)
- **Critical Finding**: NO year offset exists - years stored as actual values (2022 stored as 2022)
- **Current +1408 offset**: Completely incorrect and not part of SCID specification

#### Why Tests Pass (False Positive)
- Test data (`test/data/five.*`) contains only games with date "2022.12.19"
- Hardcoded return value matches expected test output
- Creates illusion that implementation works correctly

#### Will Fail With Real Data
```
Actual SCID Database:
- Game 1: 2020.03.15 → Shows: 2022.12.19 ❌
- Game 2: 2021.07.22 → Shows: 2022.12.19 ❌
- Game 3: 2023.11.30 → Shows: 2022.12.19 ❌
```

#### Required Fix
- Remove hardcoded pattern and implement proper field offset reading
- Remove incorrect +1408 year offset (not part of SCID spec)
- Implement fixed-position date reading from 47-byte game index
- Create test data with multiple different dates for validation

#### Test Coverage
Comprehensive unit tests validate:
- Core bit-field extraction logic
- Date formatting and PGN compatibility  
- Edge case handling (invalid dates)
- Pattern decoding accuracy
- Integration with test dataset

### Name File Format
The .sn4 format uses:
- 44-byte header with magic "Scid.sn\0"
- Variable-length encoding for IDs and frequencies
- Front-coded string compression
- Control character cleaning required

## Development Status

### ✅ Working Features
- **Date parsing** 🚨 **HARDCODED** - shows "2022.12.19" for ALL games (hardcoded, not actually parsed)
- **Name extraction** ✅ **FULLY WORKING** - complete names like "Michael", not partial "ichael"
- **Basic PGN header generation** - exports proper PGN format with correct dates
- **CLI interface** - comprehensive argument parsing with clap
- **Development mode** - `--max-games=10` default for rapid testing
- **Comprehensive test suite** - unit and integration tests validating all core functionality

### ❌ Known Issues
- **🚨 CRITICAL: Date parsing hardcoded** - Always returns "2022.12.19" regardless of actual game dates (see `DATE_PARSING_ISSUE.md`)
- Game data parsing from .sg4 files ("failed to fill whole buffer" error)
- Move notation conversion not implemented
- Limited game export capabilities

## Testing Strategy

Use the development mode for rapid iteration:
```bash
# Test with limited games for faster feedback
cargo run -- database_name  # Uses --max-games=10 default

# Test with all games when ready
cargo run -- --max-games=0 database_name
```

## Major Fixes Implemented

The project has undergone significant debugging with comprehensive fixes:

### 1. Date Parsing Bug 🚨 **HARDCODED - NOT ACTUALLY FIXED**
- **Previous Problem**: Dates showing garbage values like "52298.152.207" instead of readable dates
- **Current Problem**: All games hardcoded to return "2022.12.19" regardless of actual dates
- **Root Cause**: Implementation hardcodes specific pattern instead of reading actual date fields
- **Current State**: Tests pass but create false positive - only works for test dataset
- **Critical Issue**: Will fail with any real-world database containing different dates
- **Location**: `src/scid/index.rs` lines 209, 227, 236 - see `DATE_PARSING_ISSUE.md` for full analysis
- **Remediation Plan**: See `TODO_FIX_DATE_PARSING.md` for detailed 5-phase fix plan

### 2. Name Extraction Bug ✅ **FULLY WORKING**  
- **Problem**: Partial name extraction where "Michael" became "ichael"
- **Root Cause**: Incorrect SCID .sn4 front-coded string parsing
- **Solution**: Proper implementation based on SCID source code analysis
- **Result**: Complete names extracted correctly
- **Location**: `src/scid/names.rs` with control character cleaning

### 3. Comprehensive Test Suite ✅ **NEW**
- **Unit tests**: Core logic validation in `src/scid/index.rs`
- **Integration tests**: End-to-end workflow validation in `tests/`  
- **Coverage**: Date parsing, name extraction, PGN format compatibility
- **Validation**: Against five.pgn source of truth dataset

These fixes are thoroughly documented in the codebase with extensive comments and test coverage for future reference.

## Critical Documentation Files

For the hardcoded date parsing issue discovered in July 2025:
- **`DATE_PARSING_ISSUE.md`** - Comprehensive analysis of the hardcoded date parsing problem, root cause, and impact assessment
- **`TODO_FIX_DATE_PARSING.md`** - Detailed 5-phase remediation plan with specific tasks and success criteria
- **Key Finding**: SCID uses no year offset (research confirmed from official source code)

## Next Development Steps

### 🚨 CRITICAL PRIORITY
1. **Fix hardcoded date parsing** - Implement proper SCID date field reading (see `TODO_FIX_DATE_PARSING.md`)
   - Remove hardcoded pattern `0x0944cd93`
   - Remove incorrect +1408 year offset
   - Implement fixed-offset date reading from 47-byte game index
   - Create test data with multiple different dates

### Standard Priority
2. Fix .sg4 game data reading ("failed to fill whole buffer" errors)
3. Implement chess move parsing and PGN notation conversion
4. Add comprehensive error handling for malformed game data
5. Performance optimization for large databases (1.8M+ games)

## Dependencies

- `clap = { version = "4.0", features = ["derive"] }` for CLI argument parsing

No additional test framework or linting tools are currently configured.