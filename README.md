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
