# CRUSH.md - Development Guide for scidtopgn

## Build & Test Commands
```bash
# Build project
cargo build
cargo build --release

# Run CLI (development mode - first 10 games)
cargo run -- database_name
cargo run -- --max-games=0 database_name  # All games
cargo run -- -o output.pgn database_name

# Test commands
cargo test                                 # All tests
cargo test --lib                          # Unit tests only
cargo test --test integration_tests       # Integration tests
cargo test --test date_extraction_tests   # Date parsing tests
cargo test --test comprehensive_date_test # Comprehensive date tests
cargo test test_name                      # Single test by name

# Lint & format
cargo fmt
cargo clippy
```

## Code Style Guidelines
- **Imports**: Group std, external crates, then local modules with blank lines between
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Types**: Use explicit types for public APIs, prefer `&str` over `String` for parameters
- **Error Handling**: Use `Result<T, E>` for fallible operations, `expect()` with descriptive messages
- **Comments**: Document public APIs with `///`, use `//` for implementation details
- **Modules**: One module per file, use `mod.rs` for module organization
- **Binary Format**: All SCID multi-byte values use BIG-ENDIAN byte order (critical for parsing)
- **Test Data**: NEVER remove files from `test/data/` directory - required for validation