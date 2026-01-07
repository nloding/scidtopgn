# Phase 7: Public API Design

## Overview

Phase 7 represents the **culmination of all previous phases**, creating the user-facing public API that makes our SCID parser library accessible, intuitive, and pleasant to use. This phase is about **interface design**, not algorithms - we're wrapping the robust internals from Phases 1-6 in an API that follows Rust best practices and provides an excellent developer experience.

**What Is a Public API?**

A public API is the set of types, functions, and methods that library users interact with directly. It's the **contract** between the library and its consumers. A well-designed API:
- **Hides complexity**: Users don't need to understand SCID binary formats
- **Guides correct usage**: Type system prevents misuse
- **Provides clear documentation**: Self-explanatory names and comprehensive docs
- **Performs well**: Efficient by default, with opt-in customization
- **Evolves gracefully**: Changes don't break existing code

**Why This Phase Is Critical**:

1. **First Impressions Matter**: Users judge library quality by API ergonomics
2. **Hard to Change**: Public APIs create backwards compatibility obligations
3. **Multiplies Effort**: Poor API design wastes time for every user
4. **Separates Great Libraries from Good Ones**: Implementation can be perfect, but bad API makes it unusable

**API Design Philosophy for This Project**:

```rust
// ✅ GOOD API - Simple, obvious, hard to misuse
let reader = ScidReader::open("database")?;
for game in reader.games() {
    println!("{}", game.to_pgn()?);
}

// ❌ BAD API - Complex, error-prone, requires deep knowledge
let mut si4 = File::open("database.si4")?;
let mut sn4 = File::open("database.sn4")?;
let mut sg4 = File::open("database.sg4")?;
let header = parse_si4_header(&mut si4)?;
let names = parse_name_database(sn4)?;
for i in 0..header.num_games {
    let mut entry_bytes = [0u8; 47];
    si4.read_exact(&mut entry_bytes)?;
    let entry = parse_game_index_entry(&entry_bytes)?;
    // ... 20 more lines to get a game ...
}
```

**Integration with Previous Phases**:

```
Phase 1 (Foundation)     → Error types, basic structures
Phase 2 (Index Parser)   → GameIndexEntry parsing
Phase 3 (Name Parser)    → NameDatabase parsing
Phase 4 (Game Parser)    → Game data parsing
Phase 5 (Move Decoder)   → Move decoding to shakmaty::Move
Phase 6 (PGN Output)     → PGN formatting
Phase 7 (Public API)     → Wraps everything in ScidReader
```

**Key Components**:

1. **ScidReader** - Main entry point, opens databases, provides access to games
2. **Game** - Represents a single parsed game with metadata and moves
3. **PgnOptions** - Configuration for PGN output format
4. **Prelude** - Convenient re-exports for ergonomic imports
5. **Documentation** - Comprehensive rustdoc comments and examples

**Design Principles**:

1. **Minimal Surface Area**: Expose only what users need
2. **Ergonomic Defaults**: Common cases work without configuration
3. **Progressive Disclosure**: Simple things simple, complex things possible
4. **Type Safety**: Use types to prevent errors at compile time
5. **Zero-Cost Abstractions**: Convenience shouldn't sacrifice performance
6. **Clear Ownership**: Obvious who owns data and files
7. **Composable**: Components work together naturally

**Success Criteria**:
- ✅ Open database with single function call
- ✅ Iterate games with simple for loop
- ✅ Convert to PGN with one method call
- ✅ Stream output for memory efficiency
- ✅ Clear error messages for all failure modes
- ✅ Comprehensive documentation with examples
- ✅ Pass Rust API guidelines checklist

---

## Rust API Design Guidelines Reference

### Official Guidelines

**Source**: Rust API Guidelines (https://rust-lang.github.io/api-guidelines/)

**Key Sections Relevant to This Phase**:

#### C-GOOD-ERR: Errors are well-documented and actionable

```rust
// ✅ GOOD - Specific error with context
pub enum ScidError {
    #[error("Failed to open {file}: {source}")]
    FileOpenError {
        file: String,
        source: std::io::Error,
    },
    // ...
}

// ❌ BAD - Vague error
pub enum ScidError {
    Error(String),
}
```

#### C-CALLER-CONTROL: Caller can decide where to allocate

```rust
// ✅ GOOD - Accept writer, caller controls allocation
pub fn write_pgn(&self, writer: impl Write) -> Result<()>

// ❌ BAD - Forces allocation, returns String
pub fn to_pgn(&self) -> Result<String>

// ✅ BEST - Provide both options!
pub fn to_pgn(&self) -> Result<String>  // Convenience
pub fn write_pgn(&self, writer: impl Write) -> Result<()>  // Efficiency
```

#### C-GENERIC: Use generics where beneficial

```rust
// ✅ GOOD - Accepts any path-like type
pub fn open(path: impl AsRef<Path>) -> Result<Self>

// ❌ BAD - Forces specific type
pub fn open(path: &str) -> Result<Self>
```

#### C-CONV: Provide conversion methods

```rust
// ✅ GOOD - Implements standard traits
impl Display for GameResult {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.to_pgn_string())
    }
}
```

#### C-ITER: Return iterators instead of collections

```rust
// ✅ GOOD - Returns iterator, caller chooses collection
pub fn games(&self) -> impl Iterator<Item = Result<Game>> + '_

// ❌ BAD - Forces Vec allocation
pub fn games(&self) -> Vec<Game>
```

### Builder Pattern for Options

**When to Use**: Complex configuration with many optional parameters

```rust
// ✅ GOOD - Builder pattern with defaults
let options = PgnOptions::builder()
    .compact(true)
    .include_comments(false)
    .line_width(100)
    .build();

// ✅ ALSO GOOD - Struct with defaults
let options = PgnOptions {
    compact: true,
    ..Default::default()
};
```

### Documentation Standards

**Minimum Documentation Requirements**:
- Module-level docs explaining purpose
- Public types with overview and examples
- Public functions with description, arguments, returns, errors, examples
- Edge cases and gotchas noted

**Example Format**:
```rust
/// Opens a SCID database from the specified path.
///
/// This function opens all three required files (.si4, .sn4, .sg4),
/// validates the database structure, and loads index entries and names
/// into memory for fast access.
///
/// # Arguments
///
/// * `path` - Base path to database (without extension). For example,
///            pass "database" to open "database.si4", "database.sn4",
///            and "database.sg4".
///
/// # Returns
///
/// Returns `ScidReader` on success, or `ScidError` if:
/// - Files don't exist or can't be opened
/// - Files are corrupted or invalid format
/// - Version is unsupported
///
/// # Examples
///
/// ```
/// use scidtopgn_core::ScidReader;
///
/// let reader = ScidReader::open("path/to/database")?;
/// println!("Loaded {} games", reader.game_count());
/// # Ok::<(), scidtopgn_core::ScidError>(())
/// ```
///
/// # Errors
///
/// Returns `ScidError::FileOpenError` if files cannot be opened.
/// Returns `ScidError::InvalidFormat` if magic bytes are incorrect.
pub fn open(path: impl AsRef<Path>) -> Result<Self> {
    // ...
}
```

---

## Reference Documentation

### Implementation Plan

**Reference**: `IMPLEMENTATION_PLAN.md`

**Phase 7 Section** (lines 1084-1231):
- ScidReader implementation (lines 1086-1208)
- Prelude module (lines 1211-1230)

### Previous Phases

**Dependencies** (what we're building on):
- **Phase 1**: Error types (`ScidError`, `Result`)
- **Phase 2**: `GameIndexEntry`, index parsing functions
- **Phase 3**: `NameDatabase`, name parsing functions
- **Phase 4**: Game file parsing, tag extraction
- **Phase 5**: Move decoding to `shakmaty::Move`
- **Phase 6**: `PgnFormatter`, `PgnOptions`

### Standard Library References

**File I/O**:
- `std::fs::File` - File handles
- `std::io::{Read, Write, BufReader}` - I/O traits
- `std::path::Path` - File paths

**Traits**:
- `Iterator` - Game iteration
- `Display` - String formatting
- `Default` - Default values
- `Clone` - Cloneable types

---

## Task Breakdown

### Section 7.1: Public API Design and Architecture

**Objective**: Design the complete public API surface before implementation, ensuring it meets Rust best practices and user needs.

---

#### Task 7.1.1: Educational - Understanding API Design Principles

**Objective**: Document the principles and patterns that will guide our API design.

**Background Education**:

**What Makes a Great Rust API?**

1. **Ergonomic**: Easy to use correctly, hard to use incorrectly
2. **Discoverable**: Types and methods are easy to find via IDE/docs
3. **Consistent**: Similar operations work similarly
4. **Efficient**: Fast by default, with control when needed
5. **Safe**: Type system prevents common mistakes
6. **Documented**: Every public item has clear documentation

**The Pit of Success**: Design APIs so the easiest path is also the correct path.

**API Design Anti-Patterns to Avoid**:

```rust
// ❌ ANTI-PATTERN 1: Boolean parameters (unclear at call site)
reader.convert(true, false, true);  // What do these mean?!

// ✅ SOLUTION: Use enum or builder
reader.convert(ConvertOptions {
    include_comments: true,
    include_variations: false,
    compact: true,
});

// ❌ ANTI-PATTERN 2: Many required parameters
fn format_game(entry: &Entry, names: &Names, tags: &Tags,
               moves: &[Move], opts: &Opts, fmt: &Fmt) -> String

// ✅ SOLUTION: Group related data
fn format_game(game: &Game, options: &PgnOptions) -> String

// ❌ ANTI-PATTERN 3: Unclear ownership
fn get_game(&mut self, index: usize) -> &Game  // Returns reference?
fn get_game(&self, index: usize) -> Game       // Returns owned?

// ✅ SOLUTION: Clear ownership with naming
fn game(&self, index: usize) -> Result<Game>  // Returns owned copy
fn game_ref(&self, index: usize) -> &Game     // Returns reference

// ❌ ANTI-PATTERN 4: Forced allocation
fn games(&self) -> Vec<Game>  // Always allocates Vec

// ✅ SOLUTION: Return iterator, caller chooses collection
fn games(&self) -> impl Iterator<Item = Result<Game>> + '_

// ❌ ANTI-PATTERN 5: Panic on errors
fn open(path: &str) -> ScidReader {
    // panics if file doesn't exist!
}

// ✅ SOLUTION: Return Result for fallible operations
fn open(path: impl AsRef<Path>) -> Result<ScidReader>
```

**Progressive Disclosure Pattern**:

```rust
// Level 1: Simplest possible usage
let reader = ScidReader::open("database")?;
for game in reader.games() {
    println!("{}", game?);
}

// Level 2: With options
let options = PgnOptions::default();
for game in reader.games() {
    let pgn = game?.to_pgn_with_options(&options)?;
    println!("{}", pgn);
}

// Level 3: Full control with streaming
let options = PgnOptions::builder()
    .compact(true)
    .line_width(100)
    .build();
let file = File::create("output.pgn")?;
reader.write_pgn(file, options)?;
```

**Memory Efficiency Considerations**:

```rust
// Small database (<1000 games): Use to_pgn()
let pgn_string = reader.to_pgn(options)?;  // Fine, allocates ~1MB
write!(output, "{}", pgn_string)?;

// Large database (>100,000 games): Use write_pgn()
let output = File::create("output.pgn")?;
reader.write_pgn(output, options)?;  // Streams, constant memory
```

**Acceptance Criteria**:
- [ ] Document all API design principles
- [ ] Provide examples of good vs bad patterns
- [ ] Explain progressive disclosure approach
- [ ] Define error handling strategy
- [ ] Document memory efficiency trade-offs

**No Implementation Required** - This is educational documentation.

---

#### Task 7.1.2: Design ScidReader API Surface

**Objective**: Design the complete public interface for ScidReader before implementation.

**API Surface Definition**:

```rust
/// Main entry point for reading SCID chess databases
///
/// ScidReader provides access to all games in a SCID database and
/// methods to convert them to PGN format. It handles opening the
/// three required files (.si4, .sn4, .sg4), parsing metadata, and
/// providing efficient access to individual games.
///
/// # Examples
///
/// Basic usage:
/// ```
/// use scidtopgn_core::ScidReader;
///
/// let reader = ScidReader::open("path/to/database")?;
/// println!("Database contains {} games", reader.game_count());
///
/// for game in reader.games() {
///     let game = game?;
///     println!("{}", game.to_pgn()?);
/// }
/// # Ok::<(), scidtopgn_core::ScidError>(())
/// ```
pub struct ScidReader {
    // Private fields - users don't need to know internals
}

impl ScidReader {
    // === Construction ===

    /// Opens a SCID database from the specified base path
    pub fn open(path: impl AsRef<Path>) -> Result<Self>;

    // === Metadata Access ===

    /// Returns the total number of games in the database
    pub fn game_count(&self) -> usize;

    /// Returns database description from header
    pub fn description(&self) -> &str;

    /// Returns database version number
    pub fn version(&self) -> u16;

    // === Game Access ===

    /// Returns a specific game by index (0-based)
    pub fn game(&self, index: usize) -> Result<Game>;

    /// Returns an iterator over all games
    pub fn games(&self) -> GameIterator<'_>;

    // === PGN Conversion ===

    /// Converts entire database to PGN string (in-memory)
    ///
    /// For large databases, prefer `write_pgn()` which streams output.
    pub fn to_pgn(&self, options: &PgnOptions) -> Result<String>;

    /// Writes database to PGN format (streaming, memory-efficient)
    pub fn write_pgn(&self, writer: impl Write, options: &PgnOptions) -> Result<()>;
}
```

**Design Decisions**:

1. **Private Fields**: Hide implementation details, allow future changes
2. **Generic Path**: `impl AsRef<Path>` accepts `&str`, `String`, `PathBuf`, `&Path`
3. **Borrowed Options**: `&PgnOptions` avoids unnecessary clones
4. **Iterator Pattern**: `games()` returns iterator, not `Vec<Game>`
5. **Both Convenience and Efficiency**: `to_pgn()` for small DBs, `write_pgn()` for large
6. **Clear Naming**: `game_count()` not `len()` (not a collection)

**Acceptance Criteria**:
- [ ] Complete API surface defined
- [ ] All methods documented with examples
- [ ] Design decisions justified
- [ ] Trade-offs explained
- [ ] Follows Rust API guidelines

**No Implementation Yet** - Design only.

---

### Section 7.2: ScidReader Implementation

**Objective**: Implement the core ScidReader type with file handling, parsing, and game access.

---

#### Task 7.2.1: Implement ScidReader::open()

**Objective**: Implement database opening with validation and error handling.

**Implementation Strategy**:
1. Accept generic path (any `AsRef<Path>`)
2. Validate all three files exist
3. Parse index header and entries
4. Parse name database
5. Keep game file handle open for later access
6. Return clear errors for all failure modes

**Reference**: IMPLEMENTATION_PLAN.md lines 1103-1131

**Acceptance Criteria**:
- [ ] Opens all three required files
- [ ] Validates magic bytes and version
- [ ] Parses index header
- [ ] Loads all index entries into memory
- [ ] Parses complete name database
- [ ] Returns specific errors for each failure mode
- [ ] Handles missing files gracefully
- [ ] Handles corrupted data gracefully

**Implementation**:

**File**: `crates/core/src/lib.rs`

```rust
use std::fs::File;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};

use crate::error::{Result, ScidError};
use crate::database::index::{Si4Header, GameIndexEntry, parse_si4_header, parse_game_index_entry};
use crate::database::names::{NameDatabase, parse_name_database};

/// Main entry point for reading SCID chess databases
///
/// `ScidReader` provides high-level access to SCID databases, handling
/// all parsing and providing convenient methods for game access and
/// PGN conversion.
///
/// # File Structure
///
/// SCID databases consist of three files with the same base name:
/// - `.si4` - Index file (metadata for all games)
/// - `.sn4` - Name file (player names, event names, etc.)
/// - `.sg4` - Game file (chess moves and annotations)
///
/// # Memory Usage
///
/// `ScidReader` loads the index and names into memory on open for fast
/// access. For a database with 100,000 games, expect ~50MB RAM usage.
/// Game data is read on-demand from the .sg4 file.
///
/// # Examples
///
/// Open database and print summary:
/// ```
/// use scidtopgn_core::ScidReader;
///
/// let reader = ScidReader::open("my_database")?;
/// println!("Database: {}", reader.description());
/// println!("Games: {}", reader.game_count());
/// println!("Version: {}", reader.version());
/// # Ok::<(), scidtopgn_core::ScidError>(())
/// ```
///
/// Iterate all games:
/// ```
/// use scidtopgn_core::{ScidReader, PgnOptions};
///
/// let reader = ScidReader::open("my_database")?;
/// for game in reader.games() {
///     let game = game?;
///     println!("{}", game.to_pgn(&PgnOptions::default())?);
/// }
/// # Ok::<(), scidtopgn_core::ScidError>(())
/// ```
#[derive(Debug)]
pub struct ScidReader {
    /// Index file header with database metadata
    header: Si4Header,

    /// All game index entries (loaded into memory for fast access)
    index_entries: Vec<GameIndexEntry>,

    /// Name database (players, events, sites, rounds)
    names: NameDatabase,

    /// Game file handle (kept open for reading game data on demand)
    sg4_file: File,

    /// Base path for error messages
    base_path: PathBuf,
}

impl ScidReader {
    /// Opens a SCID database from the specified path.
    ///
    /// The path should be the base name without extension. For example,
    /// to open files `database.si4`, `database.sn4`, and `database.sg4`,
    /// pass `"database"` as the path.
    ///
    /// # Arguments
    ///
    /// * `path` - Base path to database files (without extension).
    ///            Accepts `&str`, `String`, `&Path`, or `PathBuf`.
    ///
    /// # Returns
    ///
    /// Returns `ScidReader` on success, or `ScidError` if:
    /// - Any of the three required files don't exist
    /// - Files cannot be opened (permissions, etc.)
    /// - Files are corrupted or have invalid format
    /// - Database version is unsupported
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// // All of these work:
    /// let r1 = ScidReader::open("database")?;
    /// let r2 = ScidReader::open("path/to/database")?;
    /// let r3 = ScidReader::open(std::path::Path::new("database"))?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - `ScidError::FileNotFound` - One or more database files don't exist
    /// - `ScidError::InvalidMagic` - File doesn't have SCID magic bytes
    /// - `ScidError::UnsupportedVersion` - Database version not supported
    /// - `ScidError::Io` - I/O error reading files
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let base_path = path.as_ref().to_path_buf();

        // Build paths for all three files
        let si4_path = base_path.with_extension("si4");
        let sn4_path = base_path.with_extension("sn4");
        let sg4_path = base_path.with_extension("sg4");

        // Validate all files exist before opening
        if !si4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: si4_path.display().to_string(),
            });
        }
        if !sn4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sn4_path.display().to_string(),
            });
        }
        if !sg4_path.exists() {
            return Err(ScidError::FileNotFound {
                file: sg4_path.display().to_string(),
            });
        }

        // Open index file
        let mut si4_file = File::open(&si4_path)
            .map_err(|e| ScidError::FileOpenError {
                file: si4_path.display().to_string(),
                source: e,
            })?;

        // Parse index header
        let header = parse_si4_header(&mut si4_file)?;

        // Validate version
        if header.version != 400 {
            return Err(ScidError::UnsupportedVersion {
                version: header.version,
                supported: 400,
            });
        }

        // Parse all index entries
        let mut index_entries = Vec::with_capacity(header.num_games as usize);
        for game_num in 0..header.num_games {
            let mut entry_bytes = [0u8; 47];
            si4_file.read_exact(&mut entry_bytes)
                .map_err(|e| ScidError::ParseError {
                    file: si4_path.clone(),
                    offset: (182 + game_num * 47) as u64,
                    message: format!("Failed to read index entry {}: {}", game_num, e),
                })?;

            let entry = parse_game_index_entry(&entry_bytes)?;
            index_entries.push(entry);
        }

        // Open and parse name database
        let sn4_file = File::open(&sn4_path)
            .map_err(|e| ScidError::FileOpenError {
                file: sn4_path.display().to_string(),
                source: e,
            })?;

        let names = parse_name_database(sn4_file)?;

        // Open game file (keep open for game data access)
        let sg4_file = File::open(&sg4_path)
            .map_err(|e| ScidError::FileOpenError {
                file: sg4_path.display().to_string(),
                source: e,
            })?;

        Ok(ScidReader {
            header,
            index_entries,
            names,
            sg4_file,
            base_path,
        })
    }

    /// Returns the total number of games in the database.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// println!("Found {} games", reader.game_count());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn game_count(&self) -> usize {
        self.index_entries.len()
    }

    /// Returns the database description from the header.
    ///
    /// This is typically the database name or tournament name.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// println!("Database: {}", reader.description());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn description(&self) -> &str {
        &self.header.description
    }

    /// Returns the SCID database version number.
    ///
    /// Current supported version is 400.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// assert_eq!(reader.version(), 400);
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn version(&self) -> u16 {
        self.header.version
    }

    /// Returns reference to the complete database header.
    ///
    /// This provides access to all header fields including base type,
    /// auto-load game, and custom flags.
    pub fn header(&self) -> &Si4Header {
        &self.header
    }

    /// Returns reference to the name database.
    ///
    /// This provides access to all player names, event names, site names,
    /// and round names used in the database.
    pub fn names(&self) -> &NameDatabase {
        &self.names
    }
}
```

**Testing**:

**File**: `crates/core/tests/scid_reader_tests.rs`

```rust
use scidtopgn_core::{ScidReader, ScidError};

#[test]
fn test_open_valid_database() {
    let reader = ScidReader::open("test/data/five").unwrap();

    assert_eq!(reader.game_count(), 5);
    assert_eq!(reader.version(), 400);
    assert!(!reader.description().is_empty());
}

#[test]
fn test_open_missing_file() {
    let result = ScidReader::open("test/data/nonexistent");

    match result {
        Err(ScidError::FileNotFound { file }) => {
            assert!(file.contains("nonexistent"));
        }
        _ => panic!("Expected FileNotFound error"),
    }
}

#[test]
fn test_open_corrupted_file() {
    // Create temporary corrupted file
    use std::fs;
    use std::io::Write;

    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path().join("corrupted");

    // Write invalid magic bytes
    let mut si4 = fs::File::create(base_path.with_extension("si4")).unwrap();
    si4.write_all(b"INVALID_MAGIC").unwrap();

    let mut sn4 = fs::File::create(base_path.with_extension("sn4")).unwrap();
    sn4.write_all(b"Scid.sn\0").unwrap();

    let mut sg4 = fs::File::create(base_path.with_extension("sg4")).unwrap();
    sg4.write_all(b"").unwrap();

    let result = ScidReader::open(&base_path);

    match result {
        Err(ScidError::InvalidMagic { .. }) => {
            // Expected
        }
        _ => panic!("Expected InvalidMagic error"),
    }
}

#[test]
fn test_metadata_access() {
    let reader = ScidReader::open("test/data/five").unwrap();

    // Test game count
    assert_eq!(reader.game_count(), 5);

    // Test description
    let desc = reader.description();
    assert!(!desc.is_empty());

    // Test version
    assert_eq!(reader.version(), 400);

    // Test header access
    let header = reader.header();
    assert_eq!(header.num_games, 5);

    // Test names access
    let names = reader.names();
    assert!(names.players.len() > 0);
    assert!(names.events.len() > 0);
}

#[test]
fn test_open_different_path_types() {
    // Test with &str
    let _r1 = ScidReader::open("test/data/five").unwrap();

    // Test with String
    let _r2 = ScidReader::open("test/data/five".to_string()).unwrap();

    // Test with PathBuf
    use std::path::PathBuf;
    let path = PathBuf::from("test/data/five");
    let _r3 = ScidReader::open(&path).unwrap();

    // Test with &Path
    let _r4 = ScidReader::open(path.as_path()).unwrap();
}
```

**Validation Commands**:
```bash
cargo test --test scid_reader_tests
cargo test test_open_valid_database
cargo test test_open_missing_file
cargo test test_metadata_access
```

**Expected Output**:
```
running 5 tests
test test_open_valid_database ... ok
test test_open_missing_file ... ok
test test_open_corrupted_file ... ok
test test_metadata_access ... ok
test test_open_different_path_types ... ok

test result: ok. 5 passed
```

---

#### Task 7.2.2: Implement Game Struct

**Objective**: Define the Game type that represents a parsed chess game.

**Design Decisions**:
- Should be self-contained (no references to ScidReader)
- Should provide convenient access to all game data
- Should support easy PGN conversion

**Acceptance Criteria**:
- [ ] Contains all game data (index, tags, moves)
- [ ] Provides accessor methods for common fields
- [ ] Implements Display for basic output
- [ ] Supports conversion to PGN

**Implementation**:

**File**: `crates/core/src/game.rs`

```rust
use crate::database::index::GameIndexEntry;
use crate::database::names::NameDatabase;
use crate::format::pgn::{PgnFormatter, PgnOptions};
use crate::error::Result;
use shakmaty::Move as ChessMove;
use std::collections::HashMap;
use std::fmt;

/// Represents a single parsed chess game
///
/// A `Game` contains all data for one game from a SCID database:
/// - Metadata from the index file (players, date, result, ratings, etc.)
/// - Additional tags from the game file
/// - Decoded chess moves
///
/// # Examples
///
/// ```
/// use scidtopgn_core::ScidReader;
///
/// let reader = ScidReader::open("database")?;
/// let game = reader.game(0)?;
///
/// println!("White: {}", game.white());
/// println!("Black: {}", game.black());
/// println!("Result: {}", game.result());
/// println!("Moves: {}", game.move_count());
/// # Ok::<(), scidtopgn_core::ScidError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Game {
    /// Index entry with metadata
    pub(crate) index: GameIndexEntry,

    /// Additional PGN tags from game file
    pub(crate) custom_tags: HashMap<String, String>,

    /// Starting position (FEN) if non-standard
    pub(crate) start_position: Option<String>,

    /// Decoded chess moves
    pub(crate) moves: Vec<ChessMove>,

    /// Reference to name database for tag lookup
    /// Note: We store indices, not references, to avoid lifetime issues
    names_cache: GameNames,
}

/// Cached names for a game (avoids need to reference NameDatabase)
#[derive(Debug, Clone)]
struct GameNames {
    white: String,
    black: String,
    event: String,
    site: String,
    round: String,
}

impl Game {
    /// Create game from components (internal use)
    pub(crate) fn new(
        index: GameIndexEntry,
        custom_tags: HashMap<String, String>,
        start_position: Option<String>,
        moves: Vec<ChessMove>,
        names: &NameDatabase,
    ) -> Self {
        // Cache names so Game doesn't need to reference NameDatabase
        let names_cache = GameNames {
            white: names.players.get(index.white_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            black: names.players.get(index.black_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            event: names.events.get(index.event_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            site: names.sites.get(index.site_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
            round: names.rounds.get(index.round_id as usize)
                .cloned()
                .unwrap_or_else(|| "?".to_string()),
        };

        Game {
            index,
            custom_tags,
            start_position,
            moves,
            names_cache,
        }
    }

    // === Player Information ===

    /// Returns the White player name
    pub fn white(&self) -> &str {
        &self.names_cache.white
    }

    /// Returns the Black player name
    pub fn black(&self) -> &str {
        &self.names_cache.black
    }

    /// Returns White's rating (0 if unknown)
    pub fn white_elo(&self) -> u16 {
        self.index.white_elo
    }

    /// Returns Black's rating (0 if unknown)
    pub fn black_elo(&self) -> u16 {
        self.index.black_elo
    }

    // === Event Information ===

    /// Returns the event/tournament name
    pub fn event(&self) -> &str {
        &self.names_cache.event
    }

    /// Returns the site/location name
    pub fn site(&self) -> &str {
        &self.names_cache.site
    }

    /// Returns the round identifier
    pub fn round(&self) -> &str {
        &self.names_cache.round
    }

    /// Returns the game date
    pub fn date(&self) -> &crate::types::GameDate {
        &self.index.game_date
    }

    // === Game Data ===

    /// Returns the game result
    pub fn result(&self) -> &crate::types::GameResult {
        &self.index.result
    }

    /// Returns the number of half-moves (plies) in the game
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    /// Returns reference to all moves
    pub fn moves(&self) -> &[ChessMove] {
        &self.moves
    }

    /// Returns the ECO (Encyclopedia of Chess Openings) code
    pub fn eco_code(&self) -> u16 {
        self.index.eco_code
    }

    /// Returns the starting position FEN if non-standard
    pub fn start_position(&self) -> Option<&str> {
        self.start_position.as_deref()
    }

    /// Returns custom PGN tags from the game file
    pub fn custom_tags(&self) -> &HashMap<String, String> {
        &self.custom_tags
    }

    // === PGN Conversion ===

    /// Converts game to PGN format with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let game = reader.game(0)?;
    /// let pgn = game.to_pgn()?;
    /// println!("{}", pgn);
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn to_pgn(&self) -> Result<String> {
        self.to_pgn_with_options(&PgnOptions::default())
    }

    /// Converts game to PGN format with custom options
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::{ScidReader, PgnOptions};
    ///
    /// let reader = ScidReader::open("database")?;
    /// let game = reader.game(0)?;
    ///
    /// let options = PgnOptions {
    ///     compact: true,
    ///     ..Default::default()
    /// };
    ///
    /// let pgn = game.to_pgn_with_options(&options)?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn to_pgn_with_options(&self, options: &PgnOptions) -> Result<String> {
        // Create temporary NameDatabase for formatter
        // (This is a bit inefficient, but keeps Game independent)
        let temp_names = NameDatabase {
            players: vec![self.white().to_string(), self.black().to_string()],
            events: vec![self.event().to_string()],
            sites: vec![self.site().to_string()],
            rounds: vec![self.round().to_string()],
        };

        // Use index IDs 0 since we're using temporary database
        let mut temp_index = self.index.clone();
        temp_index.white_id = 0;
        temp_index.black_id = 1;
        temp_index.event_id = 0;
        temp_index.site_id = 0;
        temp_index.round_id = 0;

        // Create GameData for formatter
        let game_data = crate::database::games::GameData {
            tags: self.custom_tags.clone(),
            flags: 0, // Not used by formatter
            start_position: self.start_position.clone(),
            moves: self.moves.clone(),
        };

        PgnFormatter::format_game(&temp_index, &temp_names, &game_data, options)
    }
}

impl fmt::Display for Game {
    /// Formats game as PGN with default options
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let game = reader.game(0)?;
    /// println!("{}", game);  // Prints PGN
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.to_pgn() {
            Ok(pgn) => write!(f, "{}", pgn),
            Err(e) => write!(f, "Error formatting game: {}", e),
        }
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_accessors() {
        let reader = ScidReader::open("test/data/five").unwrap();
        let game = reader.game(0).unwrap();

        // Test player names
        assert!(!game.white().is_empty());
        assert!(!game.black().is_empty());

        // Test event info
        assert!(!game.event().is_empty());

        // Test game data
        assert!(game.move_count() > 0);
        assert!(game.moves().len() > 0);
    }

    #[test]
    fn test_game_to_pgn() {
        let reader = ScidReader::open("test/data/five").unwrap();
        let game = reader.game(0).unwrap();

        let pgn = game.to_pgn().unwrap();

        // Verify PGN structure
        assert!(pgn.contains("[Event"));
        assert!(pgn.contains("[White"));
        assert!(pgn.contains("[Black"));
        assert!(pgn.contains("1."));  // Move numbers
    }

    #[test]
    fn test_game_display() {
        let reader = ScidReader::open("test/data/five").unwrap();
        let game = reader.game(0).unwrap();

        let display_output = format!("{}", game);
        assert!(display_output.contains("[Event"));
    }
}
```

---

### Section 7.3: Game Iteration and Access

**Objective**: Implement efficient game iteration and access patterns.

---

#### Task 7.3.1: Implement game() Method

**Objective**: Provide indexed access to individual games.

**Implementation**:

**File**: `crates/core/src/lib.rs` (ScidReader impl continued)

```rust
impl ScidReader {
    /// Returns a specific game by index.
    ///
    /// Games are numbered from 0 to `game_count() - 1`.
    ///
    /// # Arguments
    ///
    /// * `index` - Zero-based game index
    ///
    /// # Returns
    ///
    /// Returns the parsed `Game` or an error if:
    /// - Index is out of bounds
    /// - Game data is corrupted
    /// - Move parsing fails
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    ///
    /// // Get first game
    /// let game = reader.game(0)?;
    /// println!("{}: {}", game.white(), game.black());
    ///
    /// // Get last game
    /// let last_game = reader.game(reader.game_count() - 1)?;
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `ScidError::InvalidGameIndex` if index >= game_count().
    pub fn game(&self, index: usize) -> Result<Game> {
        // Validate index
        if index >= self.index_entries.len() {
            return Err(ScidError::InvalidGameIndex {
                index,
                max: self.index_entries.len(),
            });
        }

        let entry = &self.index_entries[index];

        // Read game data from SG4 file
        // Clone file handle so multiple reads don't interfere
        let mut sg4_file = self.sg4_file.try_clone()
            .map_err(|e| ScidError::Io(e))?;

        let game_bytes = crate::database::games::read_game_data(
            &mut sg4_file,
            entry.game_offset,
            entry.game_length,
        )?;

        // Parse game (tags, moves, etc.)
        let game_data = crate::database::games::parse_game(
            game_bytes,
            entry.start_position.as_deref(),
        )?;

        // Create Game object
        Ok(Game::new(
            entry.clone(),
            game_data.tags,
            game_data.start_position,
            game_data.moves,
            &self.names,
        ))
    }
}
```

---

#### Task 7.3.2: Implement games() Iterator

**Objective**: Provide iterator interface for efficient game traversal.

**Implementation**:

```rust
/// Iterator over games in a SCID database
///
/// This iterator lazily parses games on demand, minimizing memory usage.
pub struct GameIterator<'a> {
    reader: &'a ScidReader,
    index: usize,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<Game>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.reader.game_count() {
            None
        } else {
            let game = self.reader.game(self.index);
            self.index += 1;
            Some(game)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.reader.game_count() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for GameIterator<'a> {
    fn len(&self) -> usize {
        self.reader.game_count() - self.index
    }
}

impl ScidReader {
    /// Returns an iterator over all games in the database.
    ///
    /// Games are parsed lazily as the iterator is consumed, so this is
    /// memory-efficient even for large databases.
    ///
    /// # Examples
    ///
    /// Basic iteration:
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// for game in reader.games() {
    ///     let game = game?;
    ///     println!("{} vs {}", game.white(), game.black());
    /// }
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// Collecting into Vec:
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let games: Result<Vec<_>, _> = reader.games().collect();
    /// let games = games?;
    /// println!("Loaded {} games", games.len());
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    ///
    /// Filter and map:
    /// ```
    /// use scidtopgn_core::ScidReader;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let high_rated_games = reader.games()
    ///     .filter_map(|g| g.ok())
    ///     .filter(|g| g.white_elo() >= 2700 && g.black_elo() >= 2700)
    ///     .collect::<Vec<_>>();
    /// # Ok::<(), scidtopgn_core::ScidError>(())
    /// ```
    pub fn games(&self) -> GameIterator<'_> {
        GameIterator {
            reader: self,
            index: 0,
        }
    }
}
```

**Testing**:

```rust
#[test]
fn test_game_iteration() {
    let reader = ScidReader::open("test/data/five").unwrap();

    let mut count = 0;
    for game in reader.games() {
        let game = game.unwrap();
        assert!(!game.white().is_empty());
        count += 1;
    }

    assert_eq!(count, 5);
}

#[test]
fn test_iterator_size_hint() {
    let reader = ScidReader::open("test/data/five").unwrap();
    let mut iter = reader.games();

    assert_eq!(iter.len(), 5);
    iter.next();
    assert_eq!(iter.len(), 4);
}

#[test]
fn test_collect_games() {
    let reader = ScidReader::open("test/data/five").unwrap();

    let games: Result<Vec<_>, _> = reader.games().collect();
    let games = games.unwrap();

    assert_eq!(games.len(), 5);
}

#[test]
fn test_filter_games() {
    let reader = ScidReader::open("test/data/five").unwrap();

    // Filter games with ELO > 2300
    let high_rated = reader.games()
        .filter_map(|g| g.ok())
        .filter(|g| g.white_elo() >= 2300 || g.black_elo() >= 2300)
        .count();

    assert!(high_rated > 0);
}
```

---

### Section 7.4: PGN Conversion Methods

**Objective**: Implement both in-memory and streaming PGN conversion.

---

#### Task 7.4.1: Implement to_pgn() - In-Memory Conversion

**Implementation**:

```rust
impl ScidReader {
    /// Converts entire database to PGN format (in-memory).
    ///
    /// This method is convenient for small databases but allocates
    /// the entire PGN output in memory. For large databases (>10,000 games),
    /// prefer `write_pgn()` which streams output.
    ///
    /// # Arguments
    ///
    /// * `options` - PGN formatting options
    ///
    /// # Memory Usage
    ///
    /// Expect ~500 bytes per game. A 100,000 game database will
    /// allocate ~50MB of RAM for the output string.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::{ScidReader, PgnOptions};
    ///
    /// let reader = ScidReader::open("database")?;
    /// let options = PgnOptions::default();
    /// let pgn = reader.to_pgn(&options)?;
    ///
    /// // Write to file
    /// std::fs::write("output.pgn", pgn)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn to_pgn(&self, options: &PgnOptions) -> Result<String> {
        let mut pgn = String::new();

        for game in self.games() {
            let game = game?;
            let game_pgn = game.to_pgn_with_options(options)?;
            pgn.push_str(&game_pgn);
        }

        Ok(pgn)
    }
}
```

---

#### Task 7.4.2: Implement write_pgn() - Streaming Conversion

**Implementation**:

```rust
impl ScidReader {
    /// Writes database to PGN format (streaming, memory-efficient).
    ///
    /// This method streams games to the writer one at a time, using
    /// constant memory regardless of database size. Prefer this over
    /// `to_pgn()` for large databases.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `std::io::Write` (File, stdout, Vec<u8>, etc.)
    /// * `options` - PGN formatting options
    ///
    /// # Examples
    ///
    /// Write to file:
    /// ```
    /// use scidtopgn_core::{ScidReader, PgnOptions};
    /// use std::fs::File;
    ///
    /// let reader = ScidReader::open("database")?;
    /// let file = File::create("output.pgn")?;
    /// let options = PgnOptions::default();
    ///
    /// reader.write_pgn(file, &options)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Write to stdout:
    /// ```
    /// use scidtopgn_core::{ScidReader, PgnOptions};
    ///
    /// let reader = ScidReader::open("database")?;
    /// let options = PgnOptions::default();
    ///
    /// reader.write_pgn(std::io::stdout(), &options)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// Write to buffer:
    /// ```
    /// use scidtopgn_core::{ScidReader, PgnOptions};
    ///
    /// let reader = ScidReader::open("database")?;
    /// let mut buffer = Vec::new();
    /// let options = PgnOptions::default();
    ///
    /// reader.write_pgn(&mut buffer, &options)?;
    /// let pgn_string = String::from_utf8(buffer)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn write_pgn(&self, mut writer: impl std::io::Write, options: &PgnOptions) -> Result<()> {
        for game in self.games() {
            let game = game?;
            let game_pgn = game.to_pgn_with_options(options)?;
            writer.write_all(game_pgn.as_bytes())?;
        }

        Ok(())
    }
}
```

**Testing**:

```rust
#[test]
fn test_to_pgn_small_database() {
    let reader = ScidReader::open("test/data/five").unwrap();
    let options = PgnOptions::default();
    let pgn = reader.to_pgn(&options).unwrap();

    // Should contain all 5 games
    assert_eq!(pgn.matches("[Event").count(), 5);
}

#[test]
fn test_write_pgn_to_file() {
    use std::fs::File;
    use std::io::Read;

    let reader = ScidReader::open("test/data/five").unwrap();
    let options = PgnOptions::default();

    let temp_dir = tempfile::tempdir().unwrap();
    let output_path = temp_dir.path().join("output.pgn");

    let file = File::create(&output_path).unwrap();
    reader.write_pgn(file, &options).unwrap();

    // Verify file contents
    let mut contents = String::new();
    File::open(&output_path).unwrap().read_to_string(&mut contents).unwrap();

    assert_eq!(contents.matches("[Event").count(), 5);
}

#[test]
fn test_write_pgn_to_buffer() {
    let reader = ScidReader::open("test/data/five").unwrap();
    let options = PgnOptions::default();

    let mut buffer = Vec::new();
    reader.write_pgn(&mut buffer, &options).unwrap();

    let pgn = String::from_utf8(buffer).unwrap();
    assert_eq!(pgn.matches("[Event").count(), 5);
}
```

---

### Section 7.5: Prelude Module

**Objective**: Create convenient re-exports for library users.

**Implementation**:

**File**: `crates/core/src/prelude.rs`

```rust
//! Convenience re-exports for common types and functions.
//!
//! The prelude makes it easy to get started with the library by
//! importing the most commonly used types in one line:
//!
//! ```
//! use scidtopgn_core::prelude::*;
//!
//! let reader = ScidReader::open("database")?;
//! let options = PgnOptions::default();
//! let pgn = reader.to_pgn(&options)?;
//! # Ok::<(), ScidError>(())
//! ```

// Re-export main types
pub use crate::ScidReader;
pub use crate::Game;
pub use crate::ScidError;
pub use crate::Result;

// Re-export format types
pub use crate::format::pgn::PgnOptions;

// Re-export common data types
pub use crate::types::{GameDate, GameResult};

// Re-export commonly used shakmaty types for convenience
pub use shakmaty::{Color, Role, Square, Move as ChessMove};
```

**File**: `crates/core/src/lib.rs` (module declarations)

```rust
pub mod error;
pub mod types;
pub mod database;
pub mod parser;
pub mod format;
pub mod game;
pub mod prelude;

// Re-export main types at crate root for convenience
pub use error::{ScidError, Result};
pub use game::Game;

// ScidReader is the main entry point
mod reader;
pub use reader::ScidReader;
```

**Testing Usage**:

```rust
#[test]
fn test_prelude_import() {
    use scidtopgn_core::prelude::*;

    let reader = ScidReader::open("test/data/five").unwrap();
    let game = reader.game(0).unwrap();
    let options = PgnOptions::default();
    let _pgn = game.to_pgn_with_options(&options).unwrap();
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Returning References That Outlive File Handles

**Problem**: Trying to return references to data read from files that are then closed.

**Example**:
```rust
// ❌ WRONG - file closes, reference becomes invalid
pub fn game(&self, index: usize) -> Result<&Game> {
    let game_bytes = read_game_data(...)?;
    // ... parse game ...
    Ok(&game)  // Returns reference to local data!
}
```

**Solution**: Return owned data, not references:
```rust
// ✅ CORRECT - returns owned Game
pub fn game(&self, index: usize) -> Result<Game> {
    let game_bytes = read_game_data(...)?;
    // ... parse and return owned Game ...
    Ok(game)
}
```

---

### Pitfall 2: Not Cloning File Handle for Concurrent Reads

**Problem**: Using same file handle for multiple reads causes position conflicts.

**Example**:
```rust
// ❌ WRONG - iterator modifies shared file position
pub fn games(&self) -> impl Iterator {
    (0..self.game_count())
        .map(|i| read_game_data(&mut self.sg4_file, ...))  // Conflict!
}
```

**Solution**: Clone file handle for each read:
```rust
// ✅ CORRECT - each read gets independent file handle
pub fn game(&self, index: usize) -> Result<Game> {
    let mut file = self.sg4_file.try_clone()?;  // Independent handle
    let game_bytes = read_game_data(&mut file, ...)?;
    // ...
}
```

---

### Pitfall 3: Exposing Internal Implementation Details

**Problem**: Public API exposes internal types that might change.

**Example**:
```rust
// ❌ WRONG - exposes internal Si4Header type
pub fn get_header(&self) -> &Si4Header {
    &self.header
}
```

**Solution**: Expose only essential data:
```rust
// ✅ CORRECT - exposes only necessary information
pub fn description(&self) -> &str {
    &self.header.description
}

pub fn game_count(&self) -> usize {
    self.header.num_games as usize
}
```

---

### Pitfall 4: Not Handling Out-of-Bounds Access

**Problem**: Panicking on invalid indices instead of returning errors.

**Example**:
```rust
// ❌ WRONG - panics on invalid index
pub fn game(&self, index: usize) -> Game {
    let entry = &self.index_entries[index];  // Panics!
    // ...
}
```

**Solution**: Validate and return Result:
```rust
// ✅ CORRECT - returns error for invalid index
pub fn game(&self, index: usize) -> Result<Game> {
    if index >= self.index_entries.len() {
        return Err(ScidError::InvalidGameIndex { index, max: self.game_count() });
    }
    let entry = &self.index_entries[index];
    // ...
}
```

---

### Pitfall 5: Iterator That Doesn't Handle Errors

**Problem**: Iterator swallows errors or panics on bad data.

**Example**:
```rust
// ❌ WRONG - panics on parse error
pub fn games(&self) -> impl Iterator<Item = Game> + '_ {
    (0..self.game_count())
        .map(|i| self.game(i).unwrap())  // Panics!
}
```

**Solution**: Iterator returns Result:
```rust
// ✅ CORRECT - iterator yields Result<Game>
pub fn games(&self) -> impl Iterator<Item = Result<Game>> + '_ {
    (0..self.game_count())
        .map(move |i| self.game(i))  // Returns Result
}
```

---

## Success Metrics

### Phase 7 Completion Criteria

**Code Completeness**:
- [x] ScidReader implemented
- [x] Game struct implemented
- [x] game() method implemented
- [x] games() iterator implemented
- [x] to_pgn() implemented
- [x] write_pgn() implemented
- [x] Prelude module implemented

**API Quality**:
- [ ] Follows Rust API Guidelines checklist
- [ ] All public items documented with rustdoc
- [ ] Examples in all documentation
- [ ] Clear error messages
- [ ] No public implementation details exposed

**Testing**:
- [ ] All unit tests passing (25+ tests)
- [ ] Integration tests with real databases passing
- [ ] Examples compile and run
- [ ] Documentation examples tested (via doctest)

**Documentation**:
- [ ] Module-level docs complete
- [ ] All public types documented
- [ ] All public methods documented
- [ ] Usage examples provided
- [ ] Common pitfalls documented

**Validation Commands**:
```bash
# Run all Phase 7 tests
cargo test --lib scid_reader
cargo test --lib game
cargo test --test integration

# Test documentation examples
cargo test --doc

# Check documentation coverage
cargo doc --open
cargo doc --no-deps --document-private-items

# Run examples
cargo run --example basic_usage
cargo run --example streaming_output

# Lint check
cargo clippy -- -D warnings
```

**Expected Final Output**:
```
=== PHASE 7 VALIDATION SUMMARY ===
Total test cases: 28
Passing: 28
Failing: 0

API Quality Checklist:
✅ C-GOOD-ERR: Errors well-documented and actionable
✅ C-CALLER-CONTROL: Provides both to_pgn() and write_pgn()
✅ C-GENERIC: Uses impl AsRef<Path>
✅ C-CONV: Implements Display for Game
✅ C-ITER: Returns iterator, not Vec
✅ All public items documented
✅ All docs include examples
✅ No panics in public API

Integration Test Results:
- Open and parse 5-game database: PASS
- Iterate all games: PASS (5/5)
- Convert to PGN: PASS (2.1 KB output)
- Stream to file: PASS
- Handle invalid files: PASS (proper errors)

Example Usage:
```rust
use scidtopgn_core::prelude::*;

let reader = ScidReader::open("database")?;
println!("Loaded {} games", reader.game_count());

for game in reader.games() {
    let game = game?;
    println!("{} vs {} ({})",
        game.white(), game.black(), game.result());
}
```

✅ Phase 7 Complete - Library Ready for Release!
```

---

## Next Steps

**Phase 8: CLI Tool** will build on the library API by:
1. Creating command-line argument parser with clap
2. Implementing main CLI logic
3. Adding progress indicators for large databases
4. Supporting output to file or stdout
5. Adding filtering options (date range, player name, etc.)
6. Comprehensive CLI testing

**Phase 9: Testing & Validation** will ensure production readiness:
1. Comprehensive unit test coverage
2. Integration tests with large databases
3. Performance benchmarks
4. Memory profiling
5. Fuzzing for robustness
6. Real-world validation with known databases

**Phase 10: Documentation & Polish** will prepare for release:
1. Complete README with examples
2. CHANGELOG documentation
3. Contributing guidelines
4. Performance tuning
5. Final API review
6. Release preparation

---

**Phase 7 represents the bridge between our robust internal implementation and the world of library users.** A well-designed public API makes the difference between a library that's technically correct and one that's actually pleasant to use. By following Rust best practices and providing comprehensive documentation, we create an API that users will enjoy working with.
