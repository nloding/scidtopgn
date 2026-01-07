# SCID to PGN Project Structure Proposal

## Overview

This document proposes a clean, idiomatic Rust project structure for the SCID-to-PGN converter. The primary goal is to decode a SCID database and provide the output in PGN format, available both as a library for import into other applications and as a CLI tool.

## Design Principles

Based on Rust best practices and community conventions:

1. **Separation of Concerns** - Business logic in library, entry point in binary
2. **Workspace Organization** - Multiple crates sharing dependencies and build process
3. **Domain-Driven Modules** - Modules organized by domain concepts, not technical layers
4. **Centralized Error Handling** - Single source of truth for error types
5. **Clean Public API** - Re-exports create ergonomic library interface
6. **Testability First** - Unit, integration, and end-to-end tests from the start

## Proposed Structure

```
scidtopgn/
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock
├── README.md
├── LICENSE
│
├── crates/                       # Workspace members
│   ├── core/                     # Library crate - reusable parsing logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs           # Public API entry point + re-exports
│   │       │
│   │       ├── database/        # SCID database parsing
│   │       │   ├── mod.rs       # Module exports
│   │       │   ├── reader.rs    # ScidReader - main entry point
│   │       │   ├── files.rs     # Multi-file handling (.si4, .sg4, .sn4)
│   │       │   ├── index.rs     # Index parsing (.si4)
│   │       │   ├── games.rs     # Game data parsing (.sg4)
│   │       │   ├── names.rs     # Name data parsing (.sn4)
│   │       │   └── types.rs     # Database-specific types
│   │       │
│   │       ├── format/          # Output formatting
│   │       │   ├── mod.rs       # Module exports
│   │       │   ├── pgn.rs       # PGN generation
│   │       │   └── converter.rs # SCID → PGN conversion logic
│   │       │
│   │       ├── parser/          # Low-level parsing utilities
│   │       │   ├── mod.rs       # Module exports
│   │       │   ├── binary.rs    # Binary parsing helpers
│   │       │   └── encoding.rs  # Character encoding handling
│   │       │
│   │       ├── error.rs         # Centralized error types
│   │       ├── types.rs         # Shared type definitions
│   │       ├── prelude.rs       # Commonly used imports
│   │       │
│   │       └── tests/           # Integration tests
│   │           └── integration.rs
│   │
│   └── cli/                     # CLI tool - thin wrapper
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs          # Entry point - orchestration only
│           ├── args.rs          # CLI argument parsing
│           ├── output.rs        # Output formatting
│           └── config.rs        # CLI configuration
│
├── tests/                       # Workspace-level integration tests
│   └── cli_e2e.rs              # End-to-end CLI tests
│
├── examples/                    # Usage examples for library consumers
│   ├── simple_convert.rs       # Basic usage example
│   └── batch_processing.rs     # Batch conversion example
│
└── benches/                     # Performance benchmarks
    └── conversion_bench.rs      # Benchmark conversion logic
```

## Module Responsibilities

### Core Crate (`crates/core/`)

The core crate contains all parsing, conversion, and data structure logic. It's designed as a reusable library.

#### `database/` Module
**Responsibility**: All SCID-specific file format handling

- **reader.rs**: ScidReader implementation - main entry point for database access
- **files.rs**: Multi-file SCID database handling (.si4, .sg4, .sn4 coordination)
- **index.rs**: Index file parsing (.si4) and navigation
- **games.rs**: Game data file parsing (.sg4) - moves, positions, metadata
- **names.rs**: Name data file parsing (.sn4) - players, events, sites, etc.
- **types.rs**: Database-specific type definitions

#### `format/` Module
**Responsibility**: Output formatting and conversion

- **pgn.rs**: PGN format generation according to specification
- **converter.rs**: High-level SCID → PGN conversion orchestrator

#### `parser/` Module
**Responsibility**: Low-level parsing utilities

- **binary.rs**: Binary reading helpers (endianness, bit manipulation)
- **encoding.rs**: Character encoding support (if needed for SCID format)

#### `error.rs`
**Responsibility**: Centralized error handling

Define a comprehensive error type using `thiserror`:
- `IoError`: File I/O failures
- `ParseError`: SCID format parsing failures
- `EncodingError`: Character encoding issues
- `ValidationError`: Invalid data or corruption
- `FormatError`: PGN generation errors

#### `types.rs`
**Responsibility**: Shared type definitions

Common types used across modules (piece representations, move notation, etc.)

#### `prelude.rs`
**Responsibility**: Convenient re-exports for library consumers

Re-exports commonly used types and traits for easier imports:
```rust
pub use crate::{ScidReader, ScidError, Result};
pub use crate::database::{Game, GameIndex};
pub use crate::format::{PgnOptions, PgnFormatter};
```

Allows users to write: `use scidtopgn_core::prelude::*;`

### CLI Crate (`crates/cli/`)

The CLI crate is a thin wrapper around the core library. It should contain minimal logic.

#### `main.rs`
**Responsibility**: Entry point and orchestration

- Parse command-line arguments
- Create `ScidReader` from core library
- Call conversion functions
- Handle top-level errors and display user-friendly messages
- Exit with appropriate status codes

#### `args.rs`
**Responsibility**: CLI argument parsing

Define command structure using a parsing library (e.g., clap):
- Input file path (required)
- Output destination (stdout or file)
- Optional filters (game range, player names, etc.)
- Output options (format options)

#### `output.rs`
**Responsibility**: Output handling

- Write to stdout or file
- Handle output encoding
- Progress reporting (optional)

#### `config.rs`
**Responsibility**: CLI configuration

- Aggregate configuration from args and environment
- Validate configuration
- Convert to core library options

## Public API Design

### Core Library API (`crates/core/src/lib.rs`)

```rust
// Re-export main types for convenience
pub use database::{Game, GameIndex, ScidDatabase};
pub use format::{PgnFormatter, PgnOptions};
pub use error::{ScidError, Result};

// Prelude module for convenient imports
pub mod prelude;

/// Main entry point for reading SCID databases
pub struct ScidReader {
    // Internal state
}

impl ScidReader {
    /// Open a SCID database from a file path
    pub fn open(path: impl AsRef<Path>) -> Result<Self> { /* ... */ }

    /// Get an iterator over all games in the database
    pub fn games(&self) -> impl Iterator<Item = Result<Game>> { /* ... */ }

    /// Get a specific game by index
    pub fn get_game(&self, index: usize) -> Result<Game> { /* ... */ }

    /// Convert entire database to PGN format (returns String)
    /// Note: For large databases, consider using write_pgn() instead
    pub fn to_pgn(&self, options: PgnOptions) -> Result<String> { /* ... */ }

    /// Write entire database to PGN format using a writer
    /// Memory-efficient for large databases
    pub fn write_pgn(&self, writer: impl std::io::Write, options: PgnOptions) -> Result<()> { /* ... */ }

    /// Get database metadata
    pub fn metadata(&self) -> &DatabaseMetadata { /* ... */ }
}

/// PGN formatting options
#[derive(Debug, Clone)]
pub struct PgnOptions {
    pub include_comments: bool,
    pub include_variations: bool,
    pub compact: bool,
    // ... other options
}

/// Custom error type for all SCID-related operations
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
        message: String
    },

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),
}
```

### Example Library Usage

```rust
use scidtopgn_core::prelude::*;
use std::fs::File;

// Simple conversion (returns String)
let reader = ScidReader::open("games.si4")?;
let pgn = reader.to_pgn(PgnOptions::default())?;
println!("{}", pgn);

// Memory-efficient conversion to file using writer
let reader = ScidReader::open("games.si4")?;
let output = File::create("output.pgn")?;
reader.write_pgn(output, PgnOptions::default())?;

// Iterate over games
let reader = ScidReader::open("games.si4")?;
for game in reader.games().take(10) {
    let game = game?;
    println!("{}: {}", game.white(), game.result());
}
```

## Workspace Configuration

### Root `Cargo.toml`

```toml
[workspace]
resolver = "2"
members = ["crates/core", "crates/cli"]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <you@example.com>"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/scidtopgn"
rust-version = "1.70"

[workspace.dependencies]
# Shared dependencies with version pinning
thiserror = "1.0"
anyhow = "1.0"
# Add other shared dependencies here
```

### Core `crates/core/Cargo.toml`

```toml
[package]
name = "scidtopgn-core"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "Library for parsing SCID chess databases and converting to PGN"
keywords = ["chess", "scid", "pgn", "database"]
categories = ["parser-implementations", "data-structures"]

[dependencies]
thiserror.workspace = true

[dev-dependencies]
# Testing dependencies
```

### CLI `crates/cli/Cargo.toml`

```toml
[package]
name = "scidtopgn"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "CLI tool for converting SCID chess databases to PGN format"

[[bin]]
name = "scidtopgn"
path = "src/main.rs"

[dependencies]
scidtopgn-core = { path = "../core" }
anyhow.workspace = true
clap = { version = "4.0", features = ["derive"] }
```

## Testing Strategy

### Unit Tests
- **Location**: Inline in each module file using `#[cfg(test)]`
- **Purpose**: Test individual functions and methods
- **Example**: Test binary parsing, move notation conversion, etc.

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        // Test header parsing logic
    }
}
```

### Integration Tests
- **Location**: `crates/core/tests/integration.rs`
- **Purpose**: Test interaction between modules
- **Coverage**: Full database parsing and conversion flows

```rust
use scidtopgn_core::ScidReader;

#[test]
fn test_full_conversion() {
    let reader = ScidReader::open("test_data/test.si4").unwrap();
    let pgn = reader.to_pgn(Default::default()).unwrap();
    assert!(pgn.len() > 0);
}
```

### End-to-End Tests
- **Location**: `tests/cli_e2e.rs`
- **Purpose**: Test CLI tool as a whole
- **Coverage**: Argument parsing, file I/O, exit codes

### Benchmarks
- **Location**: `benches/conversion_bench.rs`
- **Purpose**: Performance testing
- **Coverage**: Critical paths (parsing, conversion)

## Benefits of This Structure

### 1. **Clean Separation of Concerns**
- Core library contains all business logic
- CLI is a thin wrapper focused on user interaction
- Easy to add additional consumers (web server, GUI, etc.)

### 2. **Library-First Design**
- The core functionality is immediately reusable
- Other projects can depend on `scidtopgn-core`
- Well-documented public API

### 3. **Workspace Benefits**
- Single `Cargo.lock` ensures consistent dependencies
- Shared `Cargo.toml` reduces duplication
- `cargo build`/`cargo test` works across all crates
- Easy to add new crates (e.g., `web` or `ffi`) in the future

### 4. **Maintainability**
- Modules organized by domain concepts
- Clear boundaries between components
- Easy to locate and modify specific functionality

### 5. **Testability**
- Each module can be tested in isolation
- Integration tests verify component interaction
- E2E tests validate complete user workflows

### 6. **Extensibility**
- Easy to add new output formats (e.g., JSON, XML) in `format/`
- Easy to add new database parsers in `database/`
- Clear extension points for features

### 7. **Community Standards**
- Follows Rust project structure conventions
- Uses idiomatic patterns (thiserror, pub use, etc.)
- Familiar structure for Rust contributors

## Dependencies

### Recommended Core Dependencies
- `thiserror` - Error handling
- `byteorder` (if needed) - Binary parsing with endianness support
- `encoding_rs` (if needed) - Character encoding

### Recommended CLI Dependencies
- `anyhow` - Error handling in application code
- `clap` - Command-line argument parsing

### Recommended Dev Dependencies
- `criterion` - Benchmarking
- `pretty_assertions` - Better test failure messages

## Building and Running

```bash
# Build all crates
cargo build

# Run CLI
cargo run --bin scidtopgn -- --input games.si4

# Run tests
cargo test

# Run benchmarks
cargo bench

# Build release version
cargo build --release
```

## Documentation

- Inline documentation for all public APIs
- Examples in `examples/` directory
- `README.md` with quick start guide
- API docs generated via `cargo doc --open`

## Conclusion

This structure provides a solid foundation for a clean, maintainable, and extensible SCID-to-PGN converter. The workspace-based approach with separate library and CLI crates follows Rust best practices and ensures the core functionality can be reused in other projects while providing a convenient CLI tool for end users.

The domain-driven module organization (database, format, parser) makes the codebase intuitive to navigate, while centralized error handling and a clean public API contribute to long-term maintainability.

This structure scales well with future additions such as:
- Additional output formats
- Web API interface
- Foreign function interface (FFI) bindings
- GUI application
