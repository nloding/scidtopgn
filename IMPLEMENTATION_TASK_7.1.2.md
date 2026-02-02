# Task 7.1.2: Design ScidReader API Surface

## Objective

Design the complete public interface for ScidReader before implementation.

## API Surface Definition

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
    pub fn to_pgn(&self, options: &PgnOptions) -> Result<String>;

    /// Writes database to PGN format (streaming, memory-efficient)
    pub fn write_pgn(&self, writer: impl Write, options: &PgnOptions) -> Result<()>;
}
```

## Design Decisions

### 1. Private Fields

**Decision**: Hide implementation details with private fields.

**Justification**:
- Allows future changes to implementation without breaking API
- Prevents users from accessing internal structures directly
- Provides cleaner, more focused public API
- Matches Rust idioms for library design

**Trade-offs**:
- Slightly less flexibility for power users
- Future API changes may be needed for advanced use cases

### 2. Generic Path

**Decision**: Accept `impl AsRef<Path>` instead of specific types.

```rust
pub fn open(path: impl AsRef<Path>) -> Result<Self>
```

**Justification**:
- Users can pass `&str`, `String`, `PathBuf`, or `&Path`
- One method covers all path-like types
- Zero-cost abstraction (no dynamic dispatch)

**Trade-offs**:
- May be slightly slower for string paths (minor)

### 3. Borrowed Options

**Decision**: Accept `&PgnOptions` rather than ownership.

```rust
pub fn to_pgn(&self, options: &PgnOptions) -> Result<String>
pub fn write_pgn(&self, writer: impl Write, options: &PgnOptions) -> Result<()>
```

**Justification**:
- Avoids unnecessary clones for configuration
- Options typically reused across multiple calls
- Clear ownership: caller owns options, not library

**Trade-offs**:
- Options must outlive reader (natural lifetime)

### 4. Iterator Pattern

**Decision**: Return iterator instead of collection.

```rust
pub fn games(&self) -> impl Iterator<Item = Result<Game>> + '_
```

**Justification**:
- Users choose collection strategy (Vec, VecDeque, etc.)
- Streaming access, O(1) memory per game
- Works with any collection type
- Aligns with Rust collections API (like `std::vec::IntoIter`)

**Trade-offs**:
- Slightly more verbose than `vec![]`
- Requires explicit handling of `Result`

### 5. Both Convenience and Efficiency

**Decision**: Provide both `to_pgn()` and `write_pgn()`.

```rust
pub fn to_pgn(&self, options: &PgnOptions) -> Result<String>  // Convenience
pub fn write_pgn(&self, writer: impl Write, options: &PgnOptions) -> Result<()>  // Efficiency
```

**Justification**:
- Small databases: `to_pgn()` is simpler and convenient
- Large databases: `write_pgn()` is memory-efficient
- Users choose based on their needs
- Matches `std::io::Write` pattern (many libraries provide both)

**Trade-offs**:
- Two similar methods to maintain
- Users must know which to use (demonstrated in docs)

### 6. Clear Naming

**Decision**: Use descriptive method names instead of collection-like names.

```rust
pub fn game_count(&self) -> usize  // Not `len()`
```

**Justification**:
- `game_count()` is self-documenting (not a collection)
- Prevents confusion with `Vec::len()`
- More readable for non-Rust developers

**Trade-offs**:
- Slightly longer names
- Individual methods for common metadata access

## API Design Patterns

### Builder Pattern for Options

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

### Progressive Disclosure

```rust
// Level 1: Simplest possible usage
let reader = ScidReader::open("database")?;
for game in reader.games() {
    println!("{}", game?);
}

// Level 2: With options
let options = PgnOptions::default();
for game in reader.games() {
    let game = game?;
    println!("{}", game.to_pgn_with_options(&options)?);
}

// Level 3: Full control with streaming
let options = PgnOptions::builder()
    .compact(true)
    .line_width(100)
    .build();
let file = File::create("output.pgn")?;
reader.write_pgn(file, options)?;
```

### Error Handling

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

## Memory Efficiency Considerations

```rust
// Small database (<1000 games): Use to_pgn()
let pgn_string = reader.to_pgn(options)?;  // Fine, allocates ~1MB
write!(output, "{}", pgn_string)?;

// Large database (>100,000 games): Use write_pgn()
let output = File::create("output.pgn")?;
reader.write_pgn(output, options)?;  // Streams, constant memory
```

## Acceptance Criteria

- [x] Complete API surface defined
- [x] All methods documented with examples
- [x] Design decisions justified
- [x] Trade-offs explained
- [x] Follows Rust API guidelines

## Implementation Notes

This task is design-only. No implementation required yet.

The API surface will be implemented in Task 7.2.1 (ScidReader::open()).

The following files will be created/modified:
1. `IMPLEMENTATION_TASK_7.1.2.md` - This design document

The actual implementation will happen in Task 7.2.x, where:
- Task 7.2.1: Implement ScidReader::open() and metadata accessors
- Task 7.2.2: Implement Game struct
- Task 7.3.1: Implement game() method
- Task 7.3.2: Implement games() iterator
- Task 7.4.1: Implement to_pgn()
- Task 7.4.2: Implement write_pgn()
