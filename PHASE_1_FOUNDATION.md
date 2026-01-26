# Phase 1: Foundation - Detailed Implementation Plan

**Timeline**: Week 1 (5-7 days)
**Prerequisites**: None - starting from scratch
**Dependencies**: Rust toolchain installed (1.70+)

---

## Overview

Phase 1 establishes the foundational structure for the SCID to PGN converter project. This includes creating the workspace skeleton, setting up all directories, configuring dependencies, and implementing core types and error handling. Upon completion, the project will compile successfully with `cargo build` and have a complete type system ready for parsing implementation.

**Success Criteria**:
- ✅ Workspace compiles without errors
- ✅ All modules present and properly organized
- ✅ Error types comprehensive and tested
- ✅ Core types complete with tests passing
- ✅ Git repository initialized with proper .gitignore
- ✅ Documentation skeleton in place

---

## Section 1.1: Project Structure Setup

### Objective

Create the complete workspace directory structure, configuration files, and build system setup. The workspace will contain two crates (`core` library and `cli` binary) with shared dependencies managed at the workspace level.

### Reference Documentation

- **PROPOSED_PROJECT_STRUCTURE.md**: Lines 20-73 (workspace structure)
- **PROPOSED_PROJECT_STRUCTURE.md**: Lines 232-296 (workspace configuration)

---

### Task 1.1.1: Initialize Git Repository

**Acceptance Criteria**:
- Git repository initialized in `/Users/nloding/code/scidtopgn/`
- `.gitignore` file present and configured for Rust projects
- Initial commit created with message "Initial commit: workspace structure"

**Steps**:

1. Navigate to project root:
   ```bash
   cd /Users/nloding/code/scidtopgn
   ```

2. Initialize git repository:
   ```bash
   git init
   ```

3. Create `.gitignore` file with the following content:
   ```gitignore
   # Rust build artifacts
   /target
   /*/target

   # Cargo lock file (keep for binary crates, ignore for libraries)
   # Cargo.lock  # Uncomment if pure library

   # IDE files
   .vscode/
   .idea/
   *.swp
   *.swo
   *~
   .DS_Store

   # Test databases (may be large)
   *.si4
   *.sn4
   *.sg4
   /test/data/*.si4
   /test/data/*.sn4
   /test/data/*.sg4

   # Generated documentation
   /target/doc

   # Backup files
   *.bak
   *.tmp

   # Editor configurations
   .vscode/
   .idea/

   # OS files
   .DS_Store
   Thumbs.db
   ```

4. Stage and commit:
   ```bash
   git add .gitignore
   git commit -m "Initial commit: add .gitignore"
   ```

**Validation**:
```bash
# Verify git is initialized
git status
# Should show: "On branch main" or "On branch master"

# Verify .gitignore exists
cat .gitignore | grep "target"
# Should show the target directory ignore rule
```

---

### Task 1.1.2: Create Workspace Root Configuration

**Acceptance Criteria**:
- `Cargo.toml` exists at workspace root
- Workspace resolver set to "2"
- Members include `crates/core` and `crates/cli`
- Workspace-level dependencies configured
- All metadata fields populated

**Steps**:

1. Create `Cargo.toml` at `/Users/nloding/code/scidtopgn/Cargo.toml` with exact content:

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/cli"]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]  # CHANGE THIS
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/scidtopgn"  # CHANGE THIS
rust-version = "1.70"

[workspace.dependencies]
# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Chess logic
shakmaty = "0.25"

# Binary parsing (if needed)
byteorder = "1.5"

# CLI (for cli crate only)
clap = { version = "4.5", features = ["derive"] }

# Development dependencies
[workspace.dev-dependencies]
criterion = "0.5"
pretty_assertions = "1.4"
```

2. Edit the `authors` and `repository` fields with your actual information.

**Validation**:
```bash
# Verify workspace structure is valid
cargo metadata --format-version 1 | grep "workspace_root"
# Should output the workspace root path

# Verify resolver is set
grep 'resolver = "2"' Cargo.toml
# Should show the resolver line
```

**Notes**:
- Resolver "2" is required for feature unification in workspaces
- All version numbers must match across workspace members
- Shakmaty version 0.25+ required for latest API

---

### Task 1.1.3: Create Core Library Crate Structure

**Acceptance Criteria**:
- Directory `crates/core/` exists
- `crates/core/Cargo.toml` properly configured
- `crates/core/src/lib.rs` exists
- All module files created with placeholder content
- Module tree properly declared in `lib.rs`

**Steps**:

1. Create core crate directory structure:
   ```bash
   mkdir -p crates/core/src/database
   mkdir -p crates/core/src/format
   mkdir -p crates/core/src/parser
   mkdir -p crates/core/tests
   ```

2. Create `crates/core/Cargo.toml`:
   ```toml
   [package]
   name = "scidtopgn-core"
   version.workspace = true
   edition.workspace = true
   authors.workspace = true
   license.workspace = true
   repository.workspace = true
   rust-version.workspace = true
   description = "Library for parsing SCID chess databases and converting to PGN"
   keywords = ["chess", "scid", "pgn", "database", "parser"]
   categories = ["parser-implementations", "data-structures"]

   [dependencies]
   thiserror.workspace = true
   shakmaty.workspace = true
   byteorder.workspace = true

   [dev-dependencies]
   pretty_assertions.workspace = true
   ```

3. Create `crates/core/src/lib.rs`:
   ```rust
   //! SCID Database Parser and PGN Converter
   //!
   //! This library provides functionality to parse SCID (Shane's Chess Information Database)
   //! files and convert them to standard PGN (Portable Game Notation) format.
   //!
   //! # Quick Start
   //!
   //! ```no_run
   //! use scidtopgn_core::prelude::*;
   //!
   //! # fn main() -> Result<(), Box<dyn std::error::Error>> {
   //! let reader = ScidReader::open("database.si4")?;
   //! let pgn = reader.to_pgn(PgnOptions::default())?;
   //! println!("{}", pgn);
   //! # Ok(())
   //! # }
   //! ```

   // Public modules
   pub mod database;
   pub mod format;
   pub mod parser;

   // Core types and errors
   mod error;
   mod types;
   pub mod prelude;

   // Re-exports for convenience
   pub use error::{Result, ScidError};
   pub use types::GameResult;

   // Main API (will be implemented in later phases)
   // pub use database::reader::ScidReader;
   // pub use format::converter::PgnOptions;
   ```

4. Create module files in `crates/core/src/database/`:
   ```bash
   touch crates/core/src/database/mod.rs
   touch crates/core/src/database/reader.rs
   touch crates/core/src/database/files.rs
   touch crates/core/src/database/index.rs
   touch crates/core/src/database/games.rs
   touch crates/core/src/database/names.rs
   touch crates/core/src/database/types.rs
   ```

5. Create `crates/core/src/database/mod.rs`:
   ```rust
   //! SCID database file parsing
   //!
   //! This module handles parsing all three SCID file types:
   //! - `.si4` - Index file with game metadata
   //! - `.sn4` - Name file with player/event/site/round names
   //! - `.sg4` - Game file with chess moves and annotations

   // Module declarations
   pub mod reader;
   pub mod files;
   pub mod index;
   pub mod games;
   pub mod names;
   pub mod types;

   // Re-exports
   // pub use reader::ScidReader;
   // pub use index::GameIndexEntry;
   // pub use types::*;
   ```

6. Create module files in `crates/core/src/format/`:
   ```bash
   touch crates/core/src/format/mod.rs
   touch crates/core/src/format/pgn.rs
   touch crates/core/src/format/converter.rs
   ```

7. Create `crates/core/src/format/mod.rs`:
   ```rust
   //! PGN output formatting
   //!
   //! This module handles conversion of parsed SCID data to standard PGN format.

   pub mod pgn;
   pub mod converter;

   // Re-exports
   // pub use converter::PgnOptions;
   // pub use pgn::PgnFormatter;
   ```

8. Create module files in `crates/core/src/parser/`:
   ```bash
   touch crates/core/src/parser/mod.rs
   touch crates/core/src/parser/binary.rs
   touch crates/core/src/parser/encoding.rs
   ```

9. Create `crates/core/src/parser/mod.rs`:
   ```rust
   //! Low-level parsing utilities
   //!
   //! Binary parsing helpers and encoding support for SCID format.

   pub mod binary;
   pub mod encoding;

   // Re-exports
   // pub use binary::*;
   ```

10. Create placeholder content for each module file (reader.rs, files.rs, etc.):
    ```rust
    // Placeholder - will be implemented in Phase 2+
    ```

**Validation**:
```bash
# Verify directory structure
ls -R crates/core/src/

# Should show:
# crates/core/src/:
# database  format  lib.rs  parser
#
# crates/core/src/database:
# files.rs  games.rs  index.rs  mod.rs  names.rs  reader.rs  types.rs
#
# crates/core/src/format:
# converter.rs  mod.rs  pgn.rs
#
# crates/core/src/parser:
# binary.rs  encoding.rs  mod.rs

# Verify crate builds (will have warnings about unused modules)
cargo build -p scidtopgn-core
# Should complete with 0 errors (warnings OK)
```

---

### Task 1.1.4: Create CLI Binary Crate Structure

**Acceptance Criteria**:
- Directory `crates/cli/` exists
- `crates/cli/Cargo.toml` properly configured
- `crates/cli/src/main.rs` exists with minimal placeholder
- All CLI module files created

**Steps**:

1. Create CLI crate directory structure:
   ```bash
   mkdir -p crates/cli/src
   mkdir -p crates/cli/tests
   ```

2. Create `crates/cli/Cargo.toml`:
   ```toml
   [package]
   name = "scidtopgn"
   version.workspace = true
   edition.workspace = true
   authors.workspace = true
   license.workspace = true
   repository.workspace = true
   rust-version.workspace = true
   description = "CLI tool for converting SCID chess databases to PGN format"
   keywords = ["chess", "scid", "pgn", "cli"]
   categories = ["command-line-utilities"]

   [[bin]]
   name = "scidtopgn"
   path = "src/main.rs"

   [dependencies]
   scidtopgn-core = { path = "../core" }
   anyhow.workspace = true
   clap.workspace = true
   ```

3. Create `crates/cli/src/main.rs`:
   ```rust
   //! SCID to PGN Converter - Command Line Interface
   //!
   //! This binary provides a command-line interface to the scidtopgn-core library.

   use anyhow::Result;

   fn main() -> Result<()> {
       println!("SCID to PGN Converter v{}", env!("CARGO_PKG_VERSION"));
       println!("Implementation in progress...");

       Ok(())
   }
   ```

4. Create CLI module files:
   ```bash
   touch crates/cli/src/args.rs
   touch crates/cli/src/output.rs
   touch crates/cli/src/config.rs
   ```

5. Add placeholder content to each module file:
   ```rust
   // Placeholder - will be implemented in Phase 8
   ```

**Validation**:
```bash
# Verify CLI builds
cargo build -p scidtopgn

# Run the CLI placeholder
cargo run -p scidtopgn
# Should print version and "Implementation in progress..."

# Verify binary was created
ls -lh target/debug/scidtopgn
# Should show the binary file
```

---

### Task 1.1.5: Create README and Documentation

**Acceptance Criteria**:
- `README.md` exists with project description
- License files present (MIT and Apache-2.0)
- Basic project documentation structure in place

**Steps**:

1. Create `README.md` at workspace root:
   ```markdown
   # SCID to PGN Converter

   A Rust library and CLI tool for parsing SCID (Shane's Chess Information Database) files and converting them to standard PGN (Portable Game Notation) format.

   ## Project Status

   🚧 **Under Active Development** - Phase 1 Complete

   ## Features (Planned)

   - Parse SCID database files (.si4, .sn4, .sg4)
   - Extract game metadata (players, dates, ratings, results)
   - Decode chess moves with full position tracking
   - Generate standard PGN output
   - Support for variations and annotations
   - Memory-efficient streaming for large databases
   - Both library and CLI interfaces

   ## Architecture

   This project uses a workspace structure with two crates:

   - `scidtopgn-core` - Library for SCID parsing and PGN generation
   - `scidtopgn` - Command-line interface

   ## Building

   ```bash
   # Build all crates
   cargo build

   # Build release version
   cargo build --release

   # Run tests
   cargo test

   # Run CLI (when implemented)
   cargo run --bin scidtopgn -- --help
   ```

   ## Development

   See implementation plan documents for detailed development roadmap:

   - `IMPLEMENTATION_PLAN.md` - Overall project plan
   - `PHASE_1_FOUNDATION.md` - Phase 1 detailed plan
   - `SCID_DATABASE_FORMAT.md` - SCID format specification
   - `PROPOSED_PROJECT_STRUCTURE.md` - Architecture design

   ## Dependencies

   - [shakmaty](https://github.com/niklasf/shakmaty) - Chess move generation and validation
   - [thiserror](https://github.com/dtolnay/thiserror) - Error handling
   - [clap](https://github.com/clap-rs/clap) - CLI argument parsing

   ## License

   Licensed under either of:

   - Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
   - MIT license ([LICENSE-MIT](LICENSE-MIT))

   at your option.
   ```

2. Create `LICENSE-MIT`:
   ```
   MIT License

   Copyright (c) 2025 [Your Name]

   Permission is hereby granted, free of charge, to any person obtaining a copy
   of this software and associated documentation files (the "Software"), to deal
   in the Software without restriction, including without limitation the rights
   to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
   copies of the Software, and to permit persons to whom the Software is
   furnished to do so, subject to the following conditions:

   The above copyright notice and this permission notice shall be included in all
   copies or substantial portions of the Software.

   THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
   IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
   FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
   AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
   LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
   OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
   SOFTWARE.
   ```

3. Create `LICENSE-APACHE` (copy from https://www.apache.org/licenses/LICENSE-2.0.txt)

4. Commit all structure files:
   ```bash
   git add .
   git commit -m "Add workspace structure and documentation"
   ```

**Validation**:
```bash
# Verify README exists and is readable
cat README.md | head -n 5
# Should show project title and description

# Verify licenses exist
ls -lh LICENSE-*
# Should show both license files
```

---

### Task 1.1.6: Verify Complete Workspace Build

**Acceptance Criteria**:
- `cargo build` completes successfully for entire workspace
- `cargo check` passes with no errors
- `cargo test` runs (even with no tests yet)
- No compilation errors (warnings are acceptable)

**Steps**:

1. Clean build from workspace root:
   ```bash
   cargo clean
   cargo build
   ```

2. Verify both crates compile:
   ```bash
   cargo build -p scidtopgn-core
   cargo build -p scidtopgn
   ```

3. Check for errors:
   ```bash
   cargo check --all
   ```

4. Run test suite (will be empty but should pass):
   ```bash
   cargo test --all
   ```

**Expected Output**:
```
   Compiling scidtopgn-core v0.1.0 (/Users/nloding/code/scidtopgn/crates/core)
   Compiling scidtopgn v0.1.0 (/Users/nloding/code/scidtopgn/crates/cli)
    Finished dev [unoptimized + debuginfo] target(s) in X.XXs
```

**Validation**:
```bash
# Check build status
echo $?
# Should output: 0 (success)

# Verify target directory structure
ls -d target/debug/scidtopgn target/debug/libscidtopgn_core.*
# Should show both the binary and library artifacts
```

**Common Issues**:

- **Issue**: "failed to resolve: use of undeclared crate or module"
  - **Fix**: Ensure all `mod.rs` files properly declare their child modules
  - **Fix**: Verify `lib.rs` declares all top-level modules

- **Issue**: "could not find `Cargo.toml`"
  - **Fix**: Ensure you're in the workspace root directory
  - **Fix**: Check workspace member paths are correct

- **Issue**: Dependency version mismatch
  - **Fix**: Ensure all version numbers use `.workspace = true`
  - **Fix**: Verify workspace dependencies are properly declared

---

## Section 1.2: Core Types and Error Handling

### Objective

Implement the foundational type system and error handling that all other modules will depend on. This includes error types using `thiserror`, chess-related type re-exports from `shakmaty`, and SCID-specific types.

### Reference Documentation

- **SCID_DATABASE_FORMAT.md**: Lines 70-240 (index file structure, for understanding types needed)
- **SCID_DATABASE_FORMAT.md**: Lines 1249-1323 (example data structures)
- **IMPLEMENTATION_PLAN.md**: Lines 35-103 (error and type definitions)

---

### Task 1.2.1: Implement Error Types

**Acceptance Criteria**:
- `crates/core/src/error.rs` exists and is complete
- Uses `thiserror` for derive macros
- All error variants documented
- Error types include file path context where relevant
- `Result<T>` type alias defined
- Compilation successful with no warnings

**Steps**:

1. Create `crates/core/src/error.rs` with complete implementation:

```rust
//! Error types for SCID database parsing
//!
//! This module defines all error types that can occur during SCID file parsing
//! and PGN conversion. All errors use the `thiserror` crate for ergonomic
//! error handling.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for SCID operations
///
/// All fallible operations in this library return this Result type.
pub type Result<T> = std::result::Result<T, ScidError>;

/// Errors that can occur during SCID database parsing and conversion
#[derive(Debug, Error)]
pub enum ScidError {
    /// I/O error occurred while reading files
    ///
    /// This wraps standard library I/O errors that occur when reading
    /// SCID database files from disk.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid SCID file format detected
    ///
    /// This error occurs when a file doesn't match the expected SCID format,
    /// such as having an incorrect magic number or version.
    ///
    /// # Examples
    ///
    /// - Wrong magic bytes (not "Scid.si\0" or "Scid.sn\0")
    /// - Unsupported SCID version number
    /// - Truncated file header
    #[error("Invalid SCID file: {0}")]
    InvalidFormat(String),

    /// Parse error at specific location in file
    ///
    /// This error provides detailed context about where parsing failed,
    /// including the file being parsed, byte offset, and description.
    ///
    /// This is the most common error type and includes rich context for debugging.
    #[error("Parse error in {file:?} at offset {offset}: {message}")]
    ParseError {
        /// Path to the file being parsed
        file: PathBuf,
        /// Byte offset where the error occurred
        offset: u64,
        /// Description of what went wrong
        message: String,
    },

    /// Invalid game index requested
    ///
    /// Attempted to access a game that doesn't exist in the database.
    ///
    /// # Examples
    ///
    /// - Requesting game 100 when database only has 50 games
    /// - Negative game index (if using signed integers)
    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),

    /// Character encoding error
    ///
    /// Occurs when name strings or comments contain invalid UTF-8 sequences.
    /// SCID files should use UTF-8, but older databases may have encoding issues.
    #[error("Encoding error: {0}")]
    Encoding(String),

    /// Data validation error
    ///
    /// The file structure is valid but the data doesn't make sense.
    ///
    /// # Examples
    ///
    /// - Date with month > 12
    /// - ELO rating > 4095 (exceeds 12-bit limit)
    /// - Checksum mismatch
    #[error("Validation error: {0}")]
    Validation(String),

    /// Unsupported feature or format variation
    ///
    /// The file uses a SCID feature that isn't yet implemented.
    #[error("Unsupported feature: {0}")]
    Unsupported(String),
}

impl ScidError {
    /// Create a parse error with context
    ///
    /// Convenience constructor for the most common error type.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::error::ScidError;
    /// use std::path::PathBuf;
    ///
    /// let error = ScidError::parse(
    ///     PathBuf::from("test.si4"),
    ///     182,
    ///     "Invalid game count"
    /// );
    /// ```
    pub fn parse(file: PathBuf, offset: u64, message: impl Into<String>) -> Self {
        ScidError::ParseError {
            file,
            offset,
            message: message.into(),
        }
    }

    /// Create an invalid format error
    ///
    /// Convenience constructor for format validation errors.
    pub fn invalid_format(message: impl Into<String>) -> Self {
        ScidError::InvalidFormat(message.into())
    }

    /// Create a validation error
    ///
    /// Convenience constructor for data validation errors.
    pub fn validation(message: impl Into<String>) -> Self {
        ScidError::Validation(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ScidError::InvalidFormat("wrong magic".to_string());
        assert_eq!(err.to_string(), "Invalid SCID file: wrong magic");
    }

    #[test]
    fn test_parse_error_display() {
        let err = ScidError::ParseError {
            file: PathBuf::from("test.si4"),
            offset: 100,
            message: "bad data".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("test.si4"));
        assert!(display.contains("100"));
        assert!(display.contains("bad data"));
    }

    #[test]
    fn test_parse_constructor() {
        let err = ScidError::parse(PathBuf::from("game.sg4"), 500, "invalid move");
        match err {
            ScidError::ParseError { file, offset, message } => {
                assert_eq!(file, PathBuf::from("game.sg4"));
                assert_eq!(offset, 500);
                assert_eq!(message, "invalid move");
            }
            _ => panic!("Wrong error variant"),
        }
    }

    #[test]
    fn test_io_error_conversion() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let scid_err: ScidError = io_err.into();
        assert!(matches!(scid_err, ScidError::Io(_)));
    }

    #[test]
    fn test_result_type_alias() {
        fn example_fn() -> Result<i32> {
            Ok(42)
        }
        assert_eq!(example_fn().unwrap(), 42);
    }
}
```

2. Update `crates/core/src/lib.rs` to expose error module:
   ```rust
   // In the "Core types and errors" section
   mod error;
   mod types;
   pub mod prelude;

   // Re-exports for convenience
   pub use error::{Result, ScidError};
   ```

**Validation**:
```bash
# Compile error module
cargo build -p scidtopgn-core

# Run error tests
cargo test -p scidtopgn-core error

# Should see output like:
# running 5 tests
# test error::tests::test_error_display ... ok
# test error::tests::test_io_error_conversion ... ok
# test error::tests::test_parse_constructor ... ok
# test error::tests::test_parse_error_display ... ok
# test error::tests::test_result_type_alias ... ok
```

---

### Task 1.2.2: Implement Core Types

**Acceptance Criteria**:
- `crates/core/src/types.rs` exists and is complete
- Re-exports shakmaty chess types
- SCID-specific types implemented
- All types have documentation
- Unit tests pass
- Derives appropriate traits

**Steps**:

1. Create `crates/core/src/types.rs`:

```rust
//! Core type definitions for SCID database structures
//!
//! This module defines types used throughout the library. Chess-related types
//! are re-exported from the `shakmaty` library, while SCID-specific types are
//! defined here.

// Re-export shakmaty types for chess operations
pub use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};

/// Game date representation
///
/// Dates in SCID are stored as packed binary values. This struct provides
/// a more ergonomic representation.
///
/// # SCID Format Reference
///
/// Game dates are stored in the index file (.si4) at offset 25-28 in each
/// 47-byte game entry. They use 20-bit absolute encoding:
/// - Bits 19-9: Year (0-2047)
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// See SCID_DATABASE_FORMAT.md lines 225-252 for complete specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameDate {
    /// Year (e.g., 2022)
    pub year: u16,
    /// Month (1-12)
    pub month: u8,
    /// Day (1-31)
    pub day: u8,
}

impl GameDate {
    /// Create a new GameDate
    ///
    /// # Panics
    ///
    /// Panics in debug mode if month or day are invalid.
    /// In release mode, invalid values are stored as-is.
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        debug_assert!(month >= 1 && month <= 12, "Invalid month: {}", month);
        debug_assert!(day >= 1 && day <= 31, "Invalid day: {}", day);
        debug_assert!(year < 2048, "Year too large: {}", year);

        GameDate { year, month, day }
    }

    /// Check if the date is valid
    ///
    /// Returns true if month and day are in valid ranges.
    /// Does not check if the specific day is valid for the month
    /// (e.g., doesn't validate Feb 30).
    pub fn is_valid(&self) -> bool {
        self.month >= 1 && self.month <= 12 && self.day >= 1 && self.day <= 31 && self.year < 2048
    }

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
    ///
    /// ```
    /// use scidtopgn_core::types::GameDate;
    ///
    /// let date = GameDate::new(2022, 12, 25);
    /// assert_eq!(date.to_pgn_string(), "2022.12.25");
    ///
    /// let partial = GameDate { year: 1997, month: 5, day: 0 };
    /// assert_eq!(partial.to_pgn_string(), "1997.05.??");
    ///
    /// let unknown = GameDate { year: 0, month: 0, day: 0 };
    /// assert_eq!(unknown.to_pgn_string(), "????.??.??");
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

impl Default for GameDate {
    fn default() -> Self {
        GameDate {
            year: 0,
            month: 0,
            day: 0,
        }
    }
}

/// Game result
///
/// Represents the outcome of a chess game. This is stored in the index file
/// as part of the variation counts field (bits 15-12).
///
/// # SCID Format Reference
///
/// Results are encoded in the upper 4 bits of a 16-bit field at offset 21-22
/// in each game index entry:
/// - 0 = Unknown/Ongoing (*)
/// - 1 = White wins (1-0)
/// - 2 = Black wins (0-1)
/// - 3 = Draw (1/2-1/2)
///
/// See SCID_DATABASE_FORMAT.md lines 174-187 for complete specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameResult {
    /// White wins (1-0)
    WhiteWins,
    /// Black wins (0-1)
    BlackWins,
    /// Draw (1/2-1/2)
    Draw,
    /// Unknown or ongoing (*)
    Unknown,
}

impl GameResult {
    /// Create from SCID encoding value
    ///
    /// # Arguments
    ///
    /// * `value` - The 4-bit result encoding from SCID (0-3)
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::types::GameResult;
    ///
    /// assert_eq!(GameResult::from_scid(1), GameResult::WhiteWins);
    /// assert_eq!(GameResult::from_scid(2), GameResult::BlackWins);
    /// assert_eq!(GameResult::from_scid(3), GameResult::Draw);
    /// assert_eq!(GameResult::from_scid(0), GameResult::Unknown);
    /// ```
    pub fn from_scid(value: u8) -> Self {
        match value {
            1 => GameResult::WhiteWins,
            2 => GameResult::BlackWins,
            3 => GameResult::Draw,
            _ => GameResult::Unknown,
        }
    }

    /// Convert to PGN result string
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::types::GameResult;
    ///
    /// assert_eq!(GameResult::WhiteWins.to_pgn(), "1-0");
    /// assert_eq!(GameResult::BlackWins.to_pgn(), "0-1");
    /// assert_eq!(GameResult::Draw.to_pgn(), "1/2-1/2");
    /// assert_eq!(GameResult::Unknown.to_pgn(), "*");
    /// ```
    pub fn to_pgn(&self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unknown => "*",
        }
    }
}

impl Default for GameResult {
    fn default() -> Self {
        GameResult::Unknown
    }
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_pgn())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_date_creation() {
        let date = GameDate::new(2022, 12, 19);
        assert_eq!(date.year, 2022);
        assert_eq!(date.month, 12);
        assert_eq!(date.day, 19);
    }

    #[test]
    fn test_game_date_validation() {
        assert!(GameDate::new(2022, 12, 19).is_valid());
        assert!(GameDate::new(2047, 1, 1).is_valid());
        assert!(!GameDate::new(2022, 13, 1).is_valid());
        assert!(!GameDate::new(2022, 0, 1).is_valid());
        assert!(!GameDate::new(2022, 12, 0).is_valid());
        assert!(!GameDate::new(2022, 12, 32).is_valid());
    }

    #[test]
    fn test_game_date_pgn_string() {
        let date = GameDate::new(2022, 12, 19);
        assert_eq!(date.to_pgn_string(), "2022.12.19");

        let date2 = GameDate::new(2000, 1, 5);
        assert_eq!(date2.to_pgn_string(), "2000.01.05");
    }

    #[test]
    fn test_game_date_unknown_components() {
        // Unknown day
        let date = GameDate { year: 2022, month: 5, day: 0 };
        assert_eq!(date.to_pgn_string(), "2022.05.??");

        // Unknown month and day
        let date2 = GameDate { year: 1997, month: 0, day: 0 };
        assert_eq!(date2.to_pgn_string(), "1997.??.??");

        // Fully unknown
        let date3 = GameDate { year: 0, month: 0, day: 0 };
        assert_eq!(date3.to_pgn_string(), "????.??.??");
    }

    #[test]
    fn test_game_date_default() {
        let date = GameDate::default();
        assert_eq!(date.year, 0);
        assert_eq!(date.month, 0);
        assert_eq!(date.day, 0);
    }

    #[test]
    fn test_game_result_from_scid() {
        assert_eq!(GameResult::from_scid(0), GameResult::Unknown);
        assert_eq!(GameResult::from_scid(1), GameResult::WhiteWins);
        assert_eq!(GameResult::from_scid(2), GameResult::BlackWins);
        assert_eq!(GameResult::from_scid(3), GameResult::Draw);
        assert_eq!(GameResult::from_scid(99), GameResult::Unknown);
    }

    #[test]
    fn test_game_result_to_pgn() {
        assert_eq!(GameResult::WhiteWins.to_pgn(), "1-0");
        assert_eq!(GameResult::BlackWins.to_pgn(), "0-1");
        assert_eq!(GameResult::Draw.to_pgn(), "1/2-1/2");
        assert_eq!(GameResult::Unknown.to_pgn(), "*");
    }

    #[test]
    fn test_game_result_display() {
        assert_eq!(GameResult::WhiteWins.to_string(), "1-0");
        assert_eq!(GameResult::Draw.to_string(), "1/2-1/2");
    }

    #[test]
    fn test_game_result_default() {
        assert_eq!(GameResult::default(), GameResult::Unknown);
    }

    #[test]
    fn test_shakmaty_reexports() {
        // Verify we can use re-exported types
        let _square: Square = Square::A1;
        let _color: Color = Color::White;
        let _role: Role = Role::King;
    }
}
```

2. Update `crates/core/src/lib.rs`:
   ```rust
   // Re-exports for convenience
   pub use error::{Result, ScidError};
   pub use types::GameResult;  // Add this line
   ```

**Validation**:
```bash
# Compile types module
cargo build -p scidtopgn-core

# Run type tests
cargo test -p scidtopgn-core types::tests

# Should see output like:
# running 10 tests
# test types::tests::test_game_date_creation ... ok
# test types::tests::test_game_date_default ... ok
# test types::tests::test_game_date_pgn_string ... ok
# test types::tests::test_game_date_unknown_components ... ok
# test types::tests::test_game_date_validation ... ok
# test types::tests::test_game_result_default ... ok
# test types::tests::test_game_result_display ... ok
# test types::tests::test_game_result_from_scid ... ok
# test types::tests::test_game_result_to_pgn ... ok
# test types::tests::test_shakmaty_reexports ... ok
```

---

### Task 1.2.3: Implement Prelude Module

**Acceptance Criteria**:
- `crates/core/src/prelude.rs` exists
- Re-exports commonly used types
- Includes shakmaty types
- Documentation explains usage
- Can be imported with `use scidtopgn_core::prelude::*;`

**Steps**:

1. Create `crates/core/src/prelude.rs`:

```rust
//! Convenience re-exports for common types
//!
//! The prelude module provides convenient access to the most commonly used types
//! and traits in the library. Import this module to get everything you need:
//!
//! ```
//! use scidtopgn_core::prelude::*;
//! ```
//!
//! # What's Included
//!
//! - Error types: `ScidError`, `Result`
//! - Core types: `GameResult`, `GameDate`
//! - Shakmaty chess types: `Color`, `Square`, `Move`, `Position`, etc.
//! - Main API types (when implemented): `ScidReader`, `PgnOptions`

// Core library types
pub use crate::error::{Result, ScidError};
pub use crate::types::GameResult;
pub use crate::types::GameDate;

// Shakmaty chess types (most commonly used)
pub use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};

// Main API types (will be uncommented in later phases)
// pub use crate::database::reader::ScidReader;
// pub use crate::database::index::GameIndexEntry;
// pub use crate::format::converter::PgnOptions;
// pub use crate::format::pgn::PgnFormatter;
```

2. Verify prelude can be imported:

Create a test file `crates/core/tests/prelude_test.rs`:

```rust
//! Integration test for prelude module

use scidtopgn_core::prelude::*;

#[test]
fn test_prelude_imports() {
    // Verify error types are available
    let _result: Result<()> = Ok(());
    let _error = ScidError::invalid_format("test");

    // Verify GameResult is available
    let _result = GameResult::WhiteWins;
    assert_eq!(_result.to_pgn(), "1-0");

    // Verify GameDate is available
    let _date = GameDate::new(2022, 12, 19);
    assert_eq!(_date.to_pgn_string(), "2022.12.19");

    // Verify shakmaty types are available
    let _square = Square::E4;
    let _color = Color::White;
    let _role = Role::Queen;
}
```

**Validation**:
```bash
# Run integration test
cargo test -p scidtopgn-core prelude_test

# Should output:
# running 1 test
# test test_prelude_imports ... ok
```

---

### Task 1.2.4: Create Comprehensive Test Suite

**Acceptance Criteria**:
- All error variants tested
- All type methods tested
- Edge cases covered
- Documentation examples tested
- 100% code coverage on error and types modules

**Steps**:

1. Verify all tests pass:
   ```bash
   cargo test -p scidtopgn-core
   ```

2. Run with coverage (optional, requires cargo-tarpaulin):
   ```bash
   cargo install cargo-tarpaulin
   cargo tarpaulin -p scidtopgn-core --lib
   ```

3. Check documentation examples compile:
   ```bash
   cargo test -p scidtopgn-core --doc
   ```

**Expected Output**:
```
running 16 tests
test error::tests::test_error_display ... ok
test error::tests::test_io_error_conversion ... ok
test error::tests::test_parse_constructor ... ok
test error::tests::test_parse_error_display ... ok
test error::tests::test_result_type_alias ... ok
test types::tests::test_game_date_creation ... ok
test types::tests::test_game_date_default ... ok
test types::tests::test_game_date_pgn_string ... ok
test types::tests::test_game_date_unknown_components ... ok
test types::tests::test_game_date_validation ... ok
test types::tests::test_game_result_default ... ok
test types::tests::test_game_result_display ... ok
test types::tests::test_game_result_from_scid ... ok
test types::tests::test_game_result_to_pgn ... ok
test types::tests::test_shakmaty_reexports ... ok
test prelude_test::test_prelude_imports ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

### Task 1.2.5: Final Phase 1 Validation

**Acceptance Criteria**:
- All tasks from 1.1 and 1.2 complete
- Zero compilation errors
- All tests passing
- Documentation complete
- Git history clean with meaningful commits

**Steps**:

1. Run complete validation suite:
   ```bash
   # Clean build
   cargo clean
   cargo build --all

   # Run all tests
   cargo test --all

   # Check formatting
   cargo fmt --all -- --check

   # Run clippy
   cargo clippy --all -- -D warnings

   # Build documentation
   cargo doc --no-deps --all
   ```

2. Verify git status:
   ```bash
   git status
   # Should show clean working directory or only expected files

   git log --oneline
   # Should show commits for structure and implementation
   ```

3. Create Phase 1 completion commit:
   ```bash
   git add .
   git commit -m "Complete Phase 1: Foundation

   - Workspace structure with core and cli crates
   - Error handling with thiserror
   - Core types (GameDate, GameResult)
   - Shakmaty integration for chess types
   - Comprehensive test suite (16 tests passing)
   - Full documentation

   All acceptance criteria met. Ready for Phase 2."
   ```

**Final Checklist**:

- [ ] Workspace builds successfully
- [ ] Core library builds successfully
- [ ] CLI binary builds successfully
- [ ] All 16+ tests passing
- [ ] No clippy warnings
- [ ] Documentation builds
- [ ] README exists and is complete
- [ ] Licenses present
- [ ] Git repository clean
- [ ] All files committed

---

## Success Metrics

Upon completion of Phase 1, the following must be true:

1. **Build System**:
   - `cargo build` completes in < 60 seconds
   - Zero compilation errors
   - Zero clippy warnings with `-D warnings`

2. **Test Suite**:
   - All tests pass (minimum 16 tests)
   - Test coverage > 90% on error and types modules
   - Documentation examples compile and run

3. **Code Quality**:
   - All public items documented
   - No TODO comments in committed code
   - Consistent code formatting (`cargo fmt`)

4. **Project Structure**:
   - All directories present per specification
   - All module files created
   - README complete and accurate

---

## Common Pitfalls and Solutions

### Pitfall 1: Workspace Member Path Errors

**Symptom**: `cargo build` fails with "couldn't find crate `scidtopgn-core`"

**Solution**:
1. Verify `members` array in root `Cargo.toml` matches actual directories
2. Ensure paths use forward slashes on all platforms
3. Run `cargo metadata` to verify workspace structure

### Pitfall 2: Dependency Version Mismatches

**Symptom**: Different versions of `thiserror` or `shakmaty` in different crates

**Solution**:
1. Use `workspace = true` for all shared dependencies
2. Verify `[workspace.dependencies]` section in root `Cargo.toml`
3. Run `cargo tree` to inspect dependency versions

### Pitfall 3: Module Visibility Issues

**Symptom**: "private module" errors when trying to use types

**Solution**:
1. Ensure modules are declared with `pub mod` in parent
2. Verify types are re-exported in `lib.rs`
3. Check prelude includes necessary re-exports

### Pitfall 4: Test Failures in CI

**Symptom**: Tests pass locally but fail in CI/CD

**Solution**:
1. Run `cargo test --all-features` locally
2. Check for platform-specific code
3. Verify no hard-coded paths

---

## Next Steps

After completing Phase 1, proceed to:

1. **Phase 2: Index File Parser** - Parse .si4 files and extract game metadata
2. Review Phase 1 code with team (if applicable)
3. Update project board/tracking

Phase 1 provides the foundation for all subsequent phases. Do not proceed until all acceptance criteria are met and all tests pass.

---

## Appendix: Quick Reference Commands

```bash
# Build everything
cargo build --all

# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Check without building
cargo check --all

# Format code
cargo fmt --all

# Lint code
cargo clippy --all -- -D warnings

# Build documentation
cargo doc --no-deps --open

# Run specific test
cargo test -p scidtopgn-core test_game_date_creation

# Clean build
cargo clean && cargo build --all

# Show dependency tree
cargo tree

# Update dependencies
cargo update

# Run CLI (placeholder)
cargo run -p scidtopgn
```

---

**Phase 1 Complete**: Foundation established ✅
