# Phase 8: CLI Tool

## Overview

Phase 8 creates the **command-line interface** that transforms our robust library from Phases 1-7 into a tool that users can actually run from their terminal. This phase is entirely about **user experience** - creating an interface that is intuitive, helpful, performant, and pleasant to use.

**What Is a CLI Tool?**

A Command-Line Interface (CLI) tool is a program designed to be run from a terminal or shell. Unlike graphical applications, CLIs communicate through text input and output, making them:
- **Scriptable**: Can be automated and integrated into workflows
- **Composable**: Can be chained with other tools via pipes
- **Remote-friendly**: Works over SSH and in headless environments
- **Efficient**: No GUI overhead, fast startup times

**Why This Phase Is Critical**:

The CLI is the **primary way users will interact** with our SCID parser. While the library (Phase 7) is technically powerful, most users won't write Rust code - they'll run a command. The CLI must be:
- **Discoverable**: `--help` shows clear usage
- **Forgiving**: Helpful error messages, not crashes
- **Fast**: Progress indication for long operations
- **Predictable**: Follows Unix conventions
- **Flexible**: Supports common workflows

**Unix Philosophy Applied**:

Our CLI follows the classic Unix philosophy:
1. **Do one thing well**: Convert SCID to PGN (not a full database manager)
2. **Work with text streams**: Output to stdout for composability
3. **Be silent on success**: Don't clutter output unless asked
4. **Fail gracefully**: Clear error messages, proper exit codes
5. **Expect to be part of a pipeline**: Support stdin/stdout patterns

**Example User Workflows**:

```bash
# Basic conversion to file
scidtopgn database -o output.pgn

# Pipe to other tools
scidtopgn database | grep "Carlsen" | head -20

# Compact format for small file size
scidtopgn database --compact -o small.pgn

# Convert specific game range
scidtopgn database --range 1-100 -o sample.pgn

# Show database info
scidtopgn database --info

# Verbose output with progress
scidtopgn large-database -v -o output.pgn
```

**CLI Design Goals**:

1. **Intuitive Defaults**: Common case works without flags
2. **Helpful Help**: `--help` shows clear examples
3. **Progressive Disclosure**: Simple by default, powerful when needed
4. **Fast Feedback**: Progress bars for long operations
5. **Composable Output**: Works well in pipes and scripts
6. **Cross-Platform**: Works on Windows, macOS, Linux
7. **Professional Polish**: Matches quality of tools like `ripgrep`, `fd`, `bat`

**Integration with Previous Phases**:

```
Phase 7 (Public API) → ScidReader, PgnOptions
Phase 6 (PGN Output) → Format options passed from CLI
Phase 8 (CLI Tool)   → Wraps library in terminal interface
```

**Technology Stack**:

- **clap v4** - Modern argument parsing with derives
- **indicatif** - Progress bars and spinners
- **anyhow** - Ergonomic error handling for applications
- **atty** - Terminal detection (stdout vs pipe)
- **colored** (optional) - Colorized output

**Success Criteria**:
- ✅ Convert database with single command
- ✅ Clear help text with examples
- ✅ Progress indicator for large databases
- ✅ Helpful error messages
- ✅ Output to file or stdout
- ✅ Support filtering options
- ✅ Works on all major platforms
- ✅ Fast startup (< 100ms for small databases)

---

## CLI Design Philosophy Reference

### Unix Tool Design Principles

**Source**: "The Art of Unix Programming" by Eric S. Raymond

**Key Principles for Our CLI**:

#### Rule of Silence: When a program has nothing interesting to say, it should say nothing

```bash
# ✅ GOOD - Silent success to stdout
$ scidtopgn database > output.pgn
$ echo $?
0

# ❌ BAD - Unnecessary noise
$ scidtopgn database > output.pgn
Converting database...
Processing game 1...
Processing game 2...
Done!
$ echo $?
0
```

**Our Approach**:
- Normal output (PGN) goes to stdout
- Status messages go to stderr
- Silent success when output is piped

#### Rule of Repair: When something goes wrong, make it easy to fix

```bash
# ✅ GOOD - Helpful error
$ scidtopgn nonexistent
Error: Database not found

  The database 'nonexistent' does not exist.

  Expected files:
    - nonexistent.si4 (index file)
    - nonexistent.sn4 (name file)
    - nonexistent.sg4 (game file)

  Try:
    - Check the file path is correct
    - Ensure all three files are present

# ❌ BAD - Cryptic error
$ scidtopgn nonexistent
thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: Os { code: 2, kind: NotFound, message: "No such file or directory" }'
```

#### Rule of Clarity: Clarity is better than cleverness

```bash
# ✅ GOOD - Clear, explicit options
scidtopgn database --output output.pgn --compact

# ❌ BAD - Cryptic shortcuts
scidtopgn database -o output.pgn -c
```

**Our Approach**: Support both long names (`--output`) and short aliases (`-o`)

#### Rule of Composition: Design programs to be connected with other programs

```bash
# ✅ GOOD - Works in pipelines
scidtopgn database | grep "Carlsen" | less
scidtopgn database | pgn-extract --output filtered.pgn

# ❌ BAD - Forces file output
scidtopgn database --output output.pgn
# Now must read file to process further
```

### Modern CLI Best Practices

**Source**: "Command Line Interface Guidelines" (clig.dev)

#### Provide a -h for help

```bash
$ scidtopgn -h
Convert SCID chess databases to PGN format

Usage: scidtopgn [OPTIONS] <DATABASE>

Arguments:
  <DATABASE>  Path to SCID database (without extension)

Options:
  -o, --output <FILE>  Output file (stdout if not specified)
  -c, --compact        Compact output (no line breaks)
  -r, --range <RANGE>  Convert specific game range (e.g., "1-100")
  -v, --verbose        Show progress and details
  -h, --help           Print help
  -V, --version        Print version
```

#### Make the default the right thing for most users

```bash
# ✅ GOOD - Sensible defaults
$ scidtopgn database
# → Outputs to stdout with all tags and comments

# ❌ BAD - Requires flags for common case
$ scidtopgn database --format pgn --include-tags --include-comments
```

#### Be liberal in what you accept

```bash
# All of these should work:
scidtopgn database
scidtopgn database.si4
scidtopgn ./database
scidtopgn /path/to/database
```

#### Print errors to stderr

```bash
# Correct output routing:
Normal output → stdout (can be piped)
Progress/info → stderr (user sees it, pipes ignore it)
Errors       → stderr (user sees it, pipes ignore it)
```

### Exit Codes

**Standard Exit Codes**:
```
0   - Success
1   - General error
2   - Misuse of shell command (invalid arguments)
64  - Input data error (corrupted database)
65  - Input file not found
66  - Cannot create output file
70  - Internal software error
```

**Our Usage**:
```rust
std::process::exit(0);   // Success
std::process::exit(1);   // General error
std::process::exit(2);   // Invalid arguments
std::process::exit(65);  // File not found
```

---

## Reference Documentation

### Implementation Plan

**Reference**: `IMPLEMENTATION_PLAN.md`

**Phase 8 Section** (lines 1233-1343):
- CLI argument parsing (lines 1235-1282)
- Main CLI logic (lines 1285-1340)

### Clap Framework

**Crate**: `clap = { version = "4.4", features = ["derive"] }`

**Documentation**: https://docs.rs/clap/latest/clap/

**Key Features**:
- Derive macros for ergonomic argument definition
- Automatic help generation
- Argument validation
- Subcommands support
- Shell completion generation

**Basic Pattern**:
```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "scidtopgn")]
#[command(version, about, long_about = None)]
struct Args {
    /// Required positional argument
    database: String,

    /// Optional flag
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    // Use args...
}
```

### Indicatif (Progress Bars)

**Crate**: `indicatif = "0.17"`

**Basic Pattern**:
```rust
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new(total_games as u64);
pb.set_style(ProgressStyle::default_bar()
    .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
    .unwrap());

for game in reader.games() {
    // Process game...
    pb.inc(1);
}

pb.finish_with_message("Complete!");
```

### Anyhow (Error Handling)

**Crate**: `anyhow = "1.0"`

**Usage in main()**:
```rust
fn main() -> anyhow::Result<()> {
    // Any error type works
    let reader = ScidReader::open("database")?;  // ScidError
    let file = File::create("output.pgn")?;      // io::Error

    Ok(())
}
```

---

## Task Breakdown

### Section 8.1: CLI Design and Architecture

**Objective**: Design the complete CLI interface before implementation, ensuring excellent user experience.

---

#### Task 8.1.1: Educational - CLI Design Principles

**Objective**: Document the design principles that will guide our CLI implementation.

**Background Education**:

**What Makes a Great CLI?**

1. **Fast to Start**: No unnecessary initialization
2. **Clear Purpose**: Obvious what it does from `--help`
3. **Predictable**: Follows conventions users expect
4. **Forgiving**: Accepts multiple input formats
5. **Helpful**: Great error messages with suggestions
6. **Composable**: Works well with pipes and redirection
7. **Cross-Platform**: Same behavior on all OSes

**Anti-Patterns to Avoid**:

```bash
# ❌ ANTI-PATTERN 1: Requiring exact file extension
scidtopgn database.si4  # Why should user know internals?

# ✅ SOLUTION: Accept base name
scidtopgn database      # Tool handles extensions

# ❌ ANTI-PATTERN 2: No progress for long operations
$ scidtopgn huge-database -o output.pgn
# ... user waits 5 minutes wondering if it crashed ...

# ✅ SOLUTION: Show progress
$ scidtopgn huge-database -o output.pgn
Converting database...
[00:02:34] ████████████░░░░░░░░ 50000/100000 games

# ❌ ANTI-PATTERN 3: Mixing output and messages
$ scidtopgn database
Opening database...
[Event "Game 1"]
Converting...
[Site "New York"]
50% complete...

# ✅ SOLUTION: Separate streams
$ scidtopgn database 2>/dev/null  # Gets clean PGN
$ scidtopgn database -v -o file   # Progress to stderr

# ❌ ANTI-PATTERN 4: Cryptic errors
$ scidtopgn database
Error: ENOENT

# ✅ SOLUTION: Helpful errors
$ scidtopgn database
Error: Database files not found

  Could not find 'database.si4'

  Make sure the database files exist:
    ✗ database.si4 (index file)
    ✗ database.sn4 (name file)
    ✗ database.sg4 (game file)
```

**User Experience Hierarchy**:

```
1. Novice User
   → Needs: Clear help, examples, error messages
   → Usage: scidtopgn --help

2. Occasional User
   → Needs: Quick reference, sensible defaults
   → Usage: scidtopgn database -o output.pgn

3. Power User
   → Needs: All options, composability, speed
   → Usage: scidtopgn db --range 5000-10000 --compact | gzip > games.pgn.gz
```

**Acceptance Criteria**:
- [ ] Document all CLI design principles
- [ ] Provide examples of good vs bad CLIs
- [ ] Explain user experience hierarchy
- [ ] Define output stream conventions
- [ ] Document exit code strategy

**No Implementation Required** - This is educational documentation.

---

#### Task 8.1.2: Design Argument Structure

**Objective**: Design the complete argument structure before implementation.

**Arguments Design**:

```
USAGE:
    scidtopgn [OPTIONS] <DATABASE>

ARGS:
    <DATABASE>
        Path to SCID database (without extension)

        The database consists of three files (.si4, .sn4, .sg4).
        Specify the base path without extension.

        Examples:
          scidtopgn database
          scidtopgn /path/to/database
          scidtopgn ./my-database

OPTIONS:
    -o, --output <FILE>
        Output file (writes to stdout if not specified)

        Use '-' to explicitly write to stdout.

        Examples:
          -o output.pgn
          -o /path/to/output.pgn
          -o -  (stdout)

    -c, --compact
        Compact output (no line breaks in movetext)

        Reduces file size but makes PGN less readable.
        Useful for machine processing.

    --no-comments
        Exclude comments from output

        Default is to include comments if present.

    --no-variations
        Exclude variations from output

        Default is to include variations if present.

    -r, --range <RANGE>
        Convert only specific games (1-based indexing)

        Format: START-END or START or -END or START-

        Examples:
          --range 1-100     (first 100 games)
          --range 50        (only game 50)
          --range 100-      (from game 100 to end)
          --range -50       (first 50 games)

    --player <NAME>
        Filter games by player name (partial match)

        Matches White or Black player.
        Case-insensitive.

        Example:
          --player Carlsen

    --min-elo <RATING>
        Filter games where both players >= rating

        Example:
          --min-elo 2700

    -v, --verbose
        Show progress and details

        Displays progress bar for databases > 1000 games.
        Shows game count, time elapsed, etc.

    -q, --quiet
        Suppress all non-essential output

        Only errors are printed.
        Useful in scripts.

    --info
        Show database information and exit

        Displays:
          - Database description
          - Number of games
          - Version
          - File sizes
          - Player/event/site counts

    --error-mode <MODE>
        Error handling mode for game parsing failures

        Controls how the tool handles games that fail to parse:
          - strict:      Stop on first error (default)
          - lenient:     Skip failed games, continue processing
          - best-effort: Output partial games when possible

        Examples:
          --error-mode lenient
          --error-mode best-effort

    --max-errors <N>
        Maximum errors before stopping (lenient/best-effort mode)

        After this many errors, stop processing.
        Use 0 for unlimited. Default: 0

        Example:
          --error-mode lenient --max-errors 100

    --include-partial
        Include partially decoded games (best-effort mode)

        When a game fails mid-parse, output moves decoded so far.
        Adds a comment noting where parsing failed.

    --mmap
        Force memory-mapped file access

        Can improve performance for large databases by letting
        the OS handle file caching.

    --mmap-threshold <SIZE>
        Auto-enable memory mapping above this file size

        Files larger than threshold use memory mapping automatically.
        Specify with suffix: 50M, 1G, 500K. Default: 100M

        Example:
          --mmap-threshold 50M

    -h, --help
        Print help information

    -V, --version
        Print version information
```

**Design Decisions**:

1. **Required Positional Argument**: `DATABASE` is positional (no flag needed)
2. **Output Defaults to Stdout**: Enables piping to other tools
3. **Long Names Primary**: Clear over concise (but provide short aliases)
4. **Sensible Defaults**: Common case needs no flags
5. **Filtering Options**: Support common queries without complex syntax
6. **Progressive Verbosity**: Quiet by default, verbose when requested

**Acceptance Criteria**:
- [ ] Complete argument specification
- [ ] Examples for each argument
- [ ] Justification for defaults
- [ ] Help text designed

**No Implementation Yet** - Design only.

---

### Section 8.2: Argument Parsing Implementation

**Objective**: Implement argument parsing using clap with proper validation.

---

#### Task 8.2.1: Setup CLI Crate and Dependencies

**Objective**: Create the CLI crate structure and add dependencies.

**File Structure**:
```
crates/cli/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── args.rs
│   ├── error.rs
│   ├── progress.rs
│   └── filter.rs
```

**Acceptance Criteria**:
- [ ] CLI crate created in workspace
- [ ] All dependencies added to Cargo.toml
- [ ] Module structure created
- [ ] Builds successfully

**Implementation**:

**File**: `crates/cli/Cargo.toml`

```toml
[package]
name = "scidtopgn"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <email@example.com>"]
description = "Convert SCID chess databases to PGN format"
license = "MIT OR Apache-2.0"
repository = "https://github.com/yourusername/scidtopgn"

[[bin]]
name = "scidtopgn"
path = "src/main.rs"

[dependencies]
# Core library
scidtopgn-core = { path = "../core" }

# CLI framework
clap = { version = "4.4", features = ["derive", "cargo", "wrap_help"] }

# Error handling
anyhow = "1.0"
thiserror = "1.0"

# Progress indicators
indicatif = "0.17"

# Terminal detection
atty = "0.2"

# Optional: Colorized output
colored = "2.0"

[dev-dependencies]
assert_cmd = "2.0"  # CLI testing
predicates = "3.0"   # Assertion helpers
tempfile = "3.8"     # Temporary directories for tests
```

**File**: `crates/cli/src/main.rs` (skeleton)

```rust
mod args;
mod error;
mod progress;
mod filter;

use args::Args;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // TODO: Implement main logic

    Ok(())
}
```

**Validation Command**:
```bash
cargo build --bin scidtopgn
cargo run --bin scidtopgn -- --help
```

---

#### Task 8.2.2: Implement Args Struct with Clap

**Objective**: Implement complete argument parsing with clap derives.

**Acceptance Criteria**:
- [ ] All arguments defined
- [ ] Help text complete with examples
- [ ] Default values set correctly
- [ ] Argument validation implemented
- [ ] Version information included

**Implementation**:

**File**: `crates/cli/src/args.rs`

```rust
use clap::Parser;
use std::path::PathBuf;

/// Convert SCID chess databases to PGN format
///
/// scidtopgn reads SCID database files (.si4, .sn4, .sg4) and converts
/// them to standard PGN (Portable Game Notation) format.
///
/// Examples:
///   # Convert to stdout
///   scidtopgn database
///
///   # Convert to file
///   scidtopgn database -o output.pgn
///
///   # Compact output
///   scidtopgn database --compact -o small.pgn
///
///   # Convert specific games
///   scidtopgn database --range 1-100 -o first_100.pgn
///
///   # Show database info
///   scidtopgn database --info
#[derive(Parser, Debug)]
#[command(name = "scidtopgn")]
#[command(version)]
#[command(about, long_about = None)]
#[command(after_help = "\
Examples:
  scidtopgn database                        Convert to stdout
  scidtopgn database -o output.pgn          Convert to file
  scidtopgn database --compact              Compact format
  scidtopgn database --range 1-100          First 100 games
  scidtopgn database --player Carlsen       Games with Carlsen
  scidtopgn database --info                 Show database info

For more information, visit: https://github.com/yourusername/scidtopgn
")]
pub struct Args {
    /// Path to SCID database (without extension)
    ///
    /// The database consists of three files:
    ///   - DATABASE.si4 (index file)
    ///   - DATABASE.sn4 (name file)
    ///   - DATABASE.sg4 (game file)
    ///
    /// Specify the base path without extension.
    #[arg(value_name = "DATABASE")]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub input: PathBuf,

    /// Output file (writes to stdout if not specified)
    ///
    /// Use '-' to explicitly write to stdout.
    /// Output is automatically streamed for memory efficiency.
    #[arg(short, long)]
    #[arg(value_name = "FILE")]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    pub output: Option<PathBuf>,

    /// Compact output (no line breaks in movetext)
    ///
    /// Reduces file size by removing line breaks within games.
    /// Makes PGN less readable but better for machine processing.
    #[arg(short, long)]
    pub compact: bool,

    /// Exclude comments from output
    ///
    /// By default, comments are included if present in the database.
    #[arg(long)]
    pub no_comments: bool,

    /// Exclude variations from output
    ///
    /// By default, variations are included if present in the database.
    #[arg(long)]
    pub no_variations: bool,

    /// Convert only specific games (1-based indexing)
    ///
    /// Format: START-END, START, -END, or START-
    ///
    /// Examples:
    ///   --range 1-100    First 100 games
    ///   --range 50       Only game 50
    ///   --range 100-     From game 100 to end
    ///   --range -50      First 50 games
    #[arg(short, long)]
    #[arg(value_name = "RANGE")]
    pub range: Option<String>,

    /// Filter games by player name (partial match, case-insensitive)
    ///
    /// Matches either White or Black player.
    /// Partial matches are supported.
    #[arg(long)]
    #[arg(value_name = "NAME")]
    pub player: Option<String>,

    /// Filter games where both players have rating >= RATING
    #[arg(long)]
    #[arg(value_name = "RATING")]
    pub min_elo: Option<u16>,

    /// Show progress and details
    ///
    /// Displays progress bar for databases with > 1000 games.
    /// Shows game count, conversion speed, and time elapsed.
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress all non-essential output
    ///
    /// Only errors are printed to stderr.
    /// Useful for scripts and automation.
    #[arg(short, long)]
    #[arg(conflicts_with = "verbose")]
    pub quiet: bool,

    // === Error Recovery Options (Gap 13) ===

    /// Error handling mode for game parsing failures
    ///
    /// Controls how the tool handles games that fail to parse:
    ///   - strict:     Stop on first error (default)
    ///   - lenient:    Skip failed games, continue processing
    ///   - best-effort: Output partial games when possible
    ///
    /// Examples:
    ///   --error-mode lenient     Skip bad games
    ///   --error-mode best-effort Include partial games
    #[arg(long)]
    #[arg(value_name = "MODE")]
    #[arg(value_parser = ["strict", "lenient", "best-effort"])]
    #[arg(default_value = "strict")]
    pub error_mode: String,

    /// Maximum errors before stopping (lenient/best-effort mode only)
    ///
    /// After this many errors, stop processing even in lenient mode.
    /// Use 0 for unlimited errors.
    ///
    /// Example:
    ///   --error-mode lenient --max-errors 100
    #[arg(long)]
    #[arg(value_name = "N")]
    #[arg(default_value = "0")]
    pub max_errors: usize,

    /// Include partially decoded games in output (best-effort mode)
    ///
    /// When a game fails mid-parse, output the moves decoded so far.
    /// Adds a comment noting where parsing failed.
    /// Only effective with --error-mode best-effort.
    #[arg(long)]
    pub include_partial: bool,

    // === Memory Mapping Options (Gap 14) ===

    /// Force memory-mapped file access for large databases
    ///
    /// Memory mapping can improve performance for large databases
    /// by letting the OS handle file caching efficiently.
    /// By default, files are read entirely into memory.
    #[arg(long)]
    pub mmap: bool,

    /// Auto-enable memory mapping above this file size
    ///
    /// Files larger than this threshold automatically use memory mapping.
    /// Specify size with suffix: 50M, 1G, 500K
    /// Default: 100M (100 megabytes)
    ///
    /// Examples:
    ///   --mmap-threshold 50M    Enable mmap for files > 50MB
    ///   --mmap-threshold 1G     Enable mmap for files > 1GB
    #[arg(long)]
    #[arg(value_name = "SIZE")]
    #[arg(default_value = "100M")]
    pub mmap_threshold: String,

    /// Show database information and exit
    ///
    /// Displays database metadata without converting:
    ///   - Description
    ///   - Number of games
    ///   - Version
    ///   - File sizes
    ///   - Player/event/site counts
    #[arg(long)]
    pub info: bool,
}

impl Args {
    /// Validate arguments and return errors if invalid
    pub fn validate(&self) -> Result<(), String> {
        // Validate range format if specified
        if let Some(ref range) = self.range {
            Self::parse_range(range)?;
        }

        // Validate output path can be created
        if let Some(ref output) = self.output {
            if output.as_os_str() != "-" {
                if let Some(parent) = output.parent() {
                    if !parent.exists() {
                        return Err(format!(
                            "Output directory does not exist: {}",
                            parent.display()
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse range string into (start, end) tuple
    ///
    /// Returns (Some(start), Some(end)) for inclusive range.
    /// Uses None to indicate "from beginning" or "to end".
    pub fn parse_range(range: &str) -> Result<(Option<usize>, Option<usize>), String> {
        // Handle single number: "50" → (Some(50), Some(50))
        if !range.contains('-') {
            let num = range.parse::<usize>()
                .map_err(|_| format!("Invalid game number: {}", range))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            return Ok((Some(num), Some(num)));
        }

        let parts: Vec<&str> = range.split('-').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid range format: {} (use START-END)", range));
        }

        let start = if parts[0].is_empty() {
            None  // "-50" means from beginning
        } else {
            let num = parts[0].parse::<usize>()
                .map_err(|_| format!("Invalid start number: {}", parts[0]))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            Some(num)
        };

        let end = if parts[1].is_empty() {
            None  // "100-" means to end
        } else {
            let num = parts[1].parse::<usize>()
                .map_err(|_| format!("Invalid end number: {}", parts[1]))?;
            if num == 0 {
                return Err("Game numbers are 1-based (use 1, not 0)".to_string());
            }
            Some(num)
        };

        // Validate start <= end if both specified
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return Err(format!("Invalid range: start ({}) > end ({})", s, e));
            }
        }

        Ok((start, end))
    }

    /// Check if output is to stdout
    pub fn is_stdout(&self) -> bool {
        self.output.is_none() ||
        self.output.as_ref().map(|p| p.as_os_str() == "-").unwrap_or(false)
    }
}
```

**Testing**:

**File**: `crates/cli/tests/args_tests.rs`

```rust
use scidtopgn::args::Args;
use clap::Parser;

#[test]
fn test_basic_args() {
    let args = Args::try_parse_from(&["scidtopgn", "database"]).unwrap();
    assert_eq!(args.input.to_str().unwrap(), "database");
    assert!(args.output.is_none());
}

#[test]
fn test_output_arg() {
    let args = Args::try_parse_from(&[
        "scidtopgn", "database", "-o", "output.pgn"
    ]).unwrap();

    assert_eq!(args.output.unwrap().to_str().unwrap(), "output.pgn");
}

#[test]
fn test_compact_flag() {
    let args = Args::try_parse_from(&[
        "scidtopgn", "database", "--compact"
    ]).unwrap();

    assert!(args.compact);
}

#[test]
fn test_range_parsing() {
    // Single number
    let (start, end) = Args::parse_range("50").unwrap();
    assert_eq!(start, Some(50));
    assert_eq!(end, Some(50));

    // Range
    let (start, end) = Args::parse_range("10-20").unwrap();
    assert_eq!(start, Some(10));
    assert_eq!(end, Some(20));

    // From beginning
    let (start, end) = Args::parse_range("-100").unwrap();
    assert_eq!(start, None);
    assert_eq!(end, Some(100));

    // To end
    let (start, end) = Args::parse_range("50-").unwrap();
    assert_eq!(start, Some(50));
    assert_eq!(end, None);
}

#[test]
fn test_invalid_range() {
    assert!(Args::parse_range("0").is_err());  // 0-based
    assert!(Args::parse_range("20-10").is_err());  // start > end
    assert!(Args::parse_range("abc").is_err());  // invalid
}

#[test]
fn test_help_output() {
    let result = Args::try_parse_from(&["scidtopgn", "--help"]);
    // Should error with help message (clap behavior)
    assert!(result.is_err());
}

#[test]
fn test_version_output() {
    let result = Args::try_parse_from(&["scidtopgn", "--version"]);
    assert!(result.is_err());  // Clap exits with version
}

#[test]
fn test_conflicting_verbose_quiet() {
    let result = Args::try_parse_from(&[
        "scidtopgn", "database", "-v", "-q"
    ]);

    // Should fail - verbose and quiet conflict
    assert!(result.is_err());
}
```

**Validation Commands**:
```bash
cargo test --package scidtopgn args_tests
cargo run --bin scidtopgn -- --help
cargo run --bin scidtopgn -- --version
cargo run --bin scidtopgn -- database --range 1-100 --help
```

**Expected --help Output**:
```
Convert SCID chess databases to PGN format

Usage: scidtopgn [OPTIONS] <DATABASE>

Arguments:
  <DATABASE>  Path to SCID database (without extension)

Options:
  -o, --output <FILE>       Output file (writes to stdout if not specified)
  -c, --compact             Compact output (no line breaks in movetext)
      --no-comments         Exclude comments from output
      --no-variations       Exclude variations from output
  -r, --range <RANGE>       Convert only specific games (1-based indexing)
      --player <NAME>       Filter games by player name
      --min-elo <RATING>    Filter games where both players have rating >= RATING
  -v, --verbose             Show progress and details
  -q, --quiet               Suppress all non-essential output
      --info                Show database information and exit
      --error-mode <MODE>   Error handling: strict, lenient, best-effort [default: strict]
      --max-errors <N>      Stop after N errors (lenient/best-effort) [default: 0]
      --include-partial     Include partially decoded games (best-effort)
      --mmap                Force memory-mapped file access
      --mmap-threshold <SIZE>  Auto-enable mmap above size [default: 100M]
  -h, --help                Print help
  -V, --version             Print version

Examples:
  scidtopgn database                        Convert to stdout
  scidtopgn database -o output.pgn          Convert to file
  scidtopgn database --compact              Compact format
  scidtopgn database --range 1-100          First 100 games
  scidtopgn database --player Carlsen       Games with Carlsen
  scidtopgn database --info                 Show database info
  scidtopgn database --error-mode lenient   Skip failed games
  scidtopgn large.db --mmap -o output.pgn   Memory-map large database

For more information, visit: https://github.com/yourusername/scidtopgn
```

---

### Section 8.3: Main CLI Logic Implementation

**Objective**: Implement the main program logic that ties everything together.

---

#### Task 8.3.1: Implement Main Function

**Objective**: Create the main() function that orchestrates the conversion process.

**Acceptance Criteria**:
- [ ] Opens database
- [ ] Applies filters
- [ ] Converts games
- [ ] Writes output
- [ ] Handles errors gracefully
- [ ] Shows progress when appropriate

**Implementation**:

**File**: `crates/cli/src/main.rs`

```rust
mod args;
mod progress;
mod filter;

use args::Args;
use clap::Parser;
use scidtopgn_core::prelude::*;
use std::fs::File;
use std::io::{self, Write};
use anyhow::{Context, Result};

fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Validate arguments
    args.validate()
        .context("Invalid arguments")?;

    // Open SCID database with appropriate access mode
    if !args.quiet {
        eprintln!("Opening database: {}", args.input.display());
    }

    // Determine file access mode based on flags (Gap 14)
    let open_options = if args.mmap {
        OpenOptions::memory_mapped()
    } else {
        OpenOptions::auto_detect()
            .mmap_threshold(parse_size_string(&args.mmap_threshold)?)
    };

    let reader = ScidReader::open_with_options(&args.input, open_options)
        .with_context(|| format!("Failed to open database: {}", args.input.display()))?;

    // Show info and exit if requested
    if args.info {
        return show_database_info(&reader);
    }

    // Print database summary (unless quiet)
    if !args.quiet {
        eprintln!("Database: {}", reader.description());
        eprintln!("Total games: {}", reader.game_count());
    }

    // Configure PGN options
    let pgn_options = PgnOptions {
        include_comments: !args.no_comments,
        include_variations: !args.no_variations,
        compact: args.compact,
        line_width: if args.compact { usize::MAX } else { 80 },
        include_supplemental_tags: true,
    };

    // Determine output destination
    let output: Box<dyn Write> = if args.is_stdout() {
        Box::new(io::stdout())
    } else {
        let path = args.output.as_ref().unwrap();
        let file = File::create(path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;

        if !args.quiet {
            eprintln!("Writing to: {}", path.display());
        }

        Box::new(file)
    };

    // Convert and write
    convert_database(&reader, output, &args, &pgn_options)?;

    if !args.quiet {
        eprintln!("Conversion complete!");
    }

    Ok(())
}

/// Show database information
fn show_database_info(reader: &ScidReader) -> Result<()> {
    println!("Database Information");
    println!("====================");
    println!();
    println!("Description: {}", reader.description());
    println!("Version:     {}", reader.version());
    println!("Total games: {}", reader.game_count());
    println!();
    println!("Names:");
    println!("  Players:   {}", reader.names().players.len());
    println!("  Events:    {}", reader.names().events.len());
    println!("  Sites:     {}", reader.names().sites.len());
    println!("  Rounds:    {}", reader.names().rounds.len());

    Ok(())
}

/// Parse size string like "100M", "1G", "500K" to bytes
fn parse_size_string(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with('G') {
        (&s[..s.len()-1], 1024 * 1024 * 1024)
    } else if s.ends_with('M') {
        (&s[..s.len()-1], 1024 * 1024)
    } else if s.ends_with('K') {
        (&s[..s.len()-1], 1024)
    } else {
        (s.as_str(), 1)
    };

    let num: u64 = num_str.parse()
        .with_context(|| format!("Invalid size: {}", s))?;

    Ok(num * multiplier)
}

/// Parse error mode string to ErrorMode enum
fn parse_error_mode(s: &str) -> ErrorMode {
    match s.to_lowercase().as_str() {
        "lenient" => ErrorMode::Lenient,
        "best-effort" => ErrorMode::BestEffort,
        _ => ErrorMode::Strict,
    }
}

/// Convert database to PGN
fn convert_database(
    reader: &ScidReader,
    mut output: Box<dyn Write>,
    args: &Args,
    options: &PgnOptions,
) -> Result<()> {
    // Determine which games to process
    let total_games = reader.game_count();
    let (start_idx, end_idx) = if let Some(ref range) = args.range {
        let (start, end) = Args::parse_range(range)?;
        let start = start.unwrap_or(1).saturating_sub(1);  // Convert to 0-based
        let end = end.unwrap_or(total_games);
        (start, end.min(total_games))
    } else {
        (0, total_games)
    };

    let games_to_process = end_idx - start_idx;

    if !args.quiet && games_to_process == 0 {
        eprintln!("Warning: No games selected");
        return Ok(());
    }

    // Create progress bar if verbose and processing many games
    let progress = if args.verbose && games_to_process > 100 && !args.is_stdout() {
        Some(progress::create_progress_bar(games_to_process))
    } else {
        None
    };

    // Configure error handling (Gap 13)
    let error_mode = parse_error_mode(&args.error_mode);
    let max_errors = if args.max_errors == 0 { usize::MAX } else { args.max_errors };

    // Process games
    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let mut partial = 0;

    for (idx, game_result) in reader.games().enumerate().skip(start_idx).take(games_to_process) {
        // Parse game with error recovery
        let game = match game_result {
            Ok(g) => g,
            Err(e) => {
                errors += 1;

                match error_mode {
                    ErrorMode::Strict => {
                        return Err(e).with_context(|| format!("Game {} failed", idx + 1));
                    }
                    ErrorMode::Lenient => {
                        if !args.quiet {
                            eprintln!("Warning: Skipping game {}: {}", idx + 1, e);
                        }
                        skipped += 1;
                        if errors >= max_errors {
                            return Err(anyhow::anyhow!(
                                "Maximum errors ({}) reached, stopping", max_errors
                            ));
                        }
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        continue;
                    }
                    ErrorMode::BestEffort => {
                        // Try to get partial game if available
                        if args.include_partial {
                            if let Some(partial_game) = e.partial_game() {
                                if !args.quiet {
                                    eprintln!("Warning: Partial game {}: {}", idx + 1, e);
                                }
                                partial += 1;
                                // Output partial game with error comment
                                let pgn = partial_game.to_pgn_with_comment(
                                    &format!("{{ Parsing stopped: {} }}", e)
                                );
                                output.write_all(pgn.as_bytes())
                                    .context("Failed to write output")?;
                                if let Some(ref pb) = progress {
                                    pb.inc(1);
                                }
                                continue;
                            }
                        }
                        // No partial available, skip like lenient
                        if !args.quiet {
                            eprintln!("Warning: Skipping game {}: {}", idx + 1, e);
                        }
                        skipped += 1;
                        if errors >= max_errors {
                            return Err(anyhow::anyhow!(
                                "Maximum errors ({}) reached, stopping", max_errors
                            ));
                        }
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        continue;
                    }
                }
            }
        };

        // Apply filters
        if !filter::should_include_game(&game, args) {
            skipped += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }

        // Convert to PGN and write
        let pgn = game.to_pgn_with_options(options)
            .with_context(|| format!("Failed to convert game {}", idx + 1))?;

        output.write_all(pgn.as_bytes())
            .context("Failed to write output")?;

        processed += 1;

        if let Some(ref pb) = progress {
            pb.inc(1);
            pb.set_message(format!("OK: {} | Skip: {} | Err: {}", processed, skipped, errors));
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message(format!(
            "Complete! Converted: {} | Skipped: {} | Errors: {} | Partial: {}",
            processed, skipped, errors, partial
        ));
    }

    if !args.quiet {
        eprintln!("Processed: {} games", processed);
        if skipped > 0 {
            eprintln!("Skipped:   {} games (filtered)", skipped);
        }
        if errors > 0 {
            eprintln!("Errors:    {} games", errors);
        }
        if partial > 0 {
            eprintln!("Partial:   {} games", partial);
        }
    }

    Ok(())
}

/// Error handling mode (Gap 13)
#[derive(Debug, Clone, Copy, PartialEq)]
enum ErrorMode {
    /// Stop on first error
    Strict,
    /// Skip failed games, continue processing
    Lenient,
    /// Output partial games when possible
    BestEffort,
}
```

**File**: `crates/cli/src/filter.rs`

```rust
use scidtopgn_core::Game;
use crate::args::Args;

/// Check if game should be included based on filters
pub fn should_include_game(game: &Game, args: &Args) -> bool {
    // Player filter
    if let Some(ref player) = args.player {
        let player_lower = player.to_lowercase();
        let white_match = game.white().to_lowercase().contains(&player_lower);
        let black_match = game.black().to_lowercase().contains(&player_lower);

        if !white_match && !black_match {
            return false;
        }
    }

    // Minimum ELO filter
    if let Some(min_elo) = args.min_elo {
        if game.white_elo() < min_elo || game.black_elo() < min_elo {
            return false;
        }
    }

    true
}
```

**File**: `crates/cli/src/progress.rs`

```rust
use indicatif::{ProgressBar, ProgressStyle};

/// Create progress bar with style
pub fn create_progress_bar(total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);

    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ")
    );

    pb
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Mixing stdout and stderr

**Problem**: Printing status messages to stdout makes output unpipeable.

**Example**:
```rust
// ❌ WRONG - status to stdout
println!("Processing game {}...", i);
println!("{}", pgn);  // Mixed with status!
```

**Solution**: Status to stderr, data to stdout:
```rust
// ✅ CORRECT
eprintln!("Processing game {}...", i);  // stderr
println!("{}", pgn);  // stdout (pipeable)
```

---

### Pitfall 2: Not handling broken pipes

**Problem**: Panicking when output pipe is closed (e.g., `| head -10`).

**Solution**: Handle BrokenPipe error gracefully:
```rust
match output.write_all(data) {
    Ok(_) => {},
    Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
        // Pipe closed, exit silently
        return Ok(());
    }
    Err(e) => return Err(e.into()),
}
```

---

### Pitfall 3: Progress bar interfering with output

**Problem**: Progress bar corrupts piped output.

**Solution**: Only show progress when output is to file:
```rust
let show_progress = args.verbose && !args.is_stdout();
```

---

### Pitfall 4: Poor error messages

**Problem**: Showing internal errors to users.

**Example**:
```rust
// ❌ WRONG
Error: Os { code: 2, kind: NotFound, message: "No such file or directory" }
```

**Solution**: Use anyhow's context:
```rust
// ✅ CORRECT
reader.open(&path)
    .with_context(|| format!("Failed to open database: {}", path.display()))?;

// Produces:
// Error: Failed to open database: mydb
//
// Caused by:
//     File not found: mydb.si4
```

---

### Pitfall 5: Not setting exit codes

**Problem**: Always exiting with 0, even on errors.

**Solution**: Let anyhow handle exit codes:
```rust
fn main() -> Result<()> {  // Returns Result
    // Errors automatically cause exit code 1
    do_work()?;
    Ok(())
}
```

---

## Success Metrics

### Phase 8 Completion Criteria

**Code Completeness**:
- [x] Args struct with all options
- [x] Argument validation
- [x] Main conversion logic
- [x] Progress indicators
- [x] Filtering implementation
- [x] Error handling

**CLI Quality**:
- [ ] Follows Unix philosophy
- [ ] Clear help text with examples
- [ ] Helpful error messages
- [ ] Progress for long operations
- [ ] Works with pipes
- [ ] Cross-platform compatibility

**Testing**:
- [ ] Unit tests for argument parsing (15+ tests)
- [ ] Integration tests (actual CLI runs)
- [ ] Error message validation
- [ ] Help text validation

**User Experience**:
- [ ] < 100ms startup for small databases
- [ ] Progress bar for large databases
- [ ] Clear error messages with suggestions
- [ ] Works in scripts and interactively

**Validation Commands**:
```bash
# Build CLI
cargo build --release --bin scidtopgn

# Run tests
cargo test --package scidtopgn
cargo test --test integration

# Test CLI manually
./target/release/scidtopgn --help
./target/release/scidtopgn --version
./target/release/scidtopgn test/data/five
./target/release/scidtopgn test/data/five -o output.pgn
./target/release/scidtopgn test/data/five --info
./target/release/scidtopgn test/data/five --range 1-3
./target/release/scidtopgn test/data/five --player Carlsen

# Test piping
./target/release/scidtopgn test/data/five | head -20
./target/release/scidtopgn test/data/five | grep "Event"

# Test error handling
./target/release/scidtopgn nonexistent
./target/release/scidtopgn test/data/five --range 0
./target/release/scidtopgn test/data/five --range 10-5

# Test error recovery modes (Gap 13)
./target/release/scidtopgn test/data/five --error-mode strict
./target/release/scidtopgn test/data/five --error-mode lenient
./target/release/scidtopgn test/data/five --error-mode best-effort --include-partial
./target/release/scidtopgn test/data/five --error-mode lenient --max-errors 10

# Test memory mapping (Gap 14)
./target/release/scidtopgn large-database --mmap -o output.pgn
./target/release/scidtopgn large-database --mmap-threshold 50M -o output.pgn
```

**Expected Output Examples**:

```bash
$ scidtopgn database -o output.pgn
Opening database: database
Database: My Chess Games
Total games: 5
Writing to: output.pgn
Conversion complete!

$ scidtopgn large-database -v -o output.pgn
Opening database: large-database
Database: Mega Database 2024
Total games: 100000
Writing to: output.pgn
[00:02:15] ████████████████████ 100000/100000 Converted: 99500 | Skipped: 500
Complete! Converted: 99500 | Skipped: 500

$ scidtopgn nonexistent
Opening database: nonexistent
Error: Failed to open database: nonexistent

Caused by:
    Database files not found

    Expected files:
      ✗ nonexistent.si4 (index file)
      ✗ nonexistent.sn4 (name file)
      ✗ nonexistent.sg4 (game file)

$ scidtopgn corrupted-database --error-mode lenient -o output.pgn
Opening database: corrupted-database
Database: Partially Corrupted DB
Total games: 1000
Writing to: output.pgn
Warning: Skipping game 45: Invalid move byte 0xFF at position 23
Warning: Skipping game 178: Unexpected end of move data
Warning: Skipping game 512: Invalid piece number 16
Processed: 997 games
Skipped:   0 games (filtered)
Errors:    3 games

$ scidtopgn large-database --mmap --verbose -o output.pgn
Opening database: large-database
Database: Mega Database 2024
Total games: 500000
Using memory-mapped file access
Writing to: output.pgn
[00:05:32] ████████████████████ 500000/500000 OK: 499850 | Skip: 0 | Err: 150
Complete! Converted: 499850 | Skipped: 0 | Errors: 150 | Partial: 0
```

---

## Next Phase Preview

**Phase 9: Testing & Validation** will ensure production readiness by:
1. Comprehensive unit test coverage (>90%)
2. Integration tests with large databases
3. Performance benchmarks
4. Memory profiling
5. Fuzzing for robustness
6. Real-world validation

**Phase 10: Documentation & Polish** will prepare for release by:
1. Complete README with installation instructions
2. Usage guide with examples
3. Contributing guidelines
4. Performance tuning
5. Release checklist
6. Packaging for distribution

---

**Phase 8 delivers a professional command-line tool that makes our SCID parser accessible to all users.** By following CLI best practices and focusing on user experience, we create a tool that's not just functional, but actually pleasant to use.

---

## Revision History

### Version 1.1 (January 2026) - Error Recovery and Memory Mapping

This version adds CLI support for error recovery (Gap 13) and memory mapping (Gap 14).

**New CLI Flags (Gap 13 - Error Recovery)**:

| Flag | Description |
|------|-------------|
| `--error-mode <MODE>` | Error handling: `strict`, `lenient`, `best-effort` (default: strict) |
| `--max-errors <N>` | Stop after N errors in lenient/best-effort mode (default: 0 = unlimited) |
| `--include-partial` | Output partially decoded games (best-effort mode) |

**Error Modes Explained**:
- **strict**: Stop on first error (traditional behavior)
- **lenient**: Skip failed games, continue processing others
- **best-effort**: Output partial games when possible, with error comments

**New CLI Flags (Gap 14 - Memory Mapping)**:

| Flag | Description |
|------|-------------|
| `--mmap` | Force memory-mapped file access |
| `--mmap-threshold <SIZE>` | Auto-enable mmap above size (default: 100M) |

**Size Format**: Supports suffixes K, M, G (e.g., `50M`, `1G`, `500K`)

**Changes Summary**:

| Section | Change |
|---------|--------|
| Task 8.1.2 | Added error recovery and memory mapping to argument design |
| Task 8.2.2 | Added new flags to Args struct |
| Task 8.3.1 | Updated main() with OpenOptions for memory mapping |
| Task 8.3.1 | Rewrote convert_database() with error mode handling |
| Validation | Added test commands for new flags |

**Example Usage**:

```bash
# Process large database with memory mapping and lenient error handling
scidtopgn mega-database.db \
    --mmap \
    --error-mode lenient \
    --max-errors 100 \
    -o output.pgn

# Get partial games from corrupted database
scidtopgn corrupted.db \
    --error-mode best-effort \
    --include-partial \
    -o recovered.pgn
```

**Reference**: IMPLEMENTATION_PLAN.md Phases 7.3 and 7.4, IMPLEMENTATION_GAPS.md Gaps 13 and 14

---

### Version 1.0 (Initial)

- Original Phase 8 implementation plan
- Core CLI structure with clap
- Basic conversion workflow
- Progress indicators
- Filtering options
