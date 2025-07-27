# SCID to PGN Converter

A command-line tool written in Rust to convert SCID chess databases (.si4/.sg4/.sn4) to PGN format. This also serves as an experiment with vibe coding from an AI skeptic.

## Features

- Convert SCID databases to standard PGN format
- Support for game metadata (players, events, sites, dates, ratings)
- Optional inclusion of variations and comments
- Progress reporting for large databases
- Force overwrite protection

## Installation

```bash
cd scidtopgn
cargo build --release
```

The binary will be available at `target/release/scidtopgn`.

## Usage

```bash
# Convert a SCID database to PGN
scidtopgn /path/to/database

# Specify output file
scidtopgn /path/to/database -o output.pgn

# Include variations and comments
scidtopgn /path/to/database --variations --comments

# Limit number of games exported
scidtopgn /path/to/database --max-games 1000

# Force overwrite existing output file
scidtopgn /path/to/database --force
```

## Arguments

- `DATABASE`: Path to the SCID database (without extension - will look for .si4, .sg4, .sn4)
- `-o, --output FILE`: Output PGN file (if not specified, uses database name with .pgn extension)
- `-f, --force`: Force overwrite existing output file
- `-v, --variations`: Include variations in PGN output
- `-c, --comments`: Include comments in PGN output
- `--max-games N`: Maximum number of games to export (0 = all games)

## File Format Support

This tool supports SCID database format version 4, which consists of three files:

- `.si4`: Index file containing meta-information for each game
- `.sg4`: Game file containing actual moves, variations and comments  
- `.sn4`: Name file containing player names, tournament names, etc.

## Current Limitations

This is an initial implementation with the following limitations:

1. **Date parsing**: The SCID binary date format is not correctly parsed yet. Dates show as "????.??.??" for now.

2. **Move parsing**: The SCID move encoding is very complex and not fully implemented yet. Games will be exported with metadata but moves are currently placeholders.

3. **Name parsing**: The .sn4 name file parsing is simplified and uses placeholder names.

4. **Variations and comments**: While the structure is in place, full parsing of variations and comments from the .sg4 file is not yet implemented.

## Development Status

This project follows Rust best practices for CLI applications:

- Modular structure with separate modules for SCID parsing and PGN export
- Error handling using `std::io::Result`
- Command-line argument parsing with `clap`
- Proper project structure with `src/`, `Cargo.toml`, etc.

## Contributing

The main areas that need work:

1. **SCID move decoding**: Implement the complex move encoding used by SCID
2. **Name file parsing**: Properly parse the .sn4 name file format
3. **Variation support**: Parse and export chess variations
4. **Comment support**: Parse and export chess comments and annotations

## Architecture

```
src/
├── main.rs              # CLI entry point and argument parsing
├── scid/                # SCID database parsing
│   ├── mod.rs           # Module exports
│   ├── database.rs      # Main database coordination
│   ├── index.rs         # .si4 index file parsing
│   ├── names.rs         # .sn4 name file parsing
│   ├── games.rs         # .sg4 game file parsing
│   └── moves.rs         # Move encoding/decoding
└── pgn/                 # PGN export functionality
    ├── mod.rs           # Module exports
    └── exporter.rs      # PGN file generation
```

## License

MIT OR Apache-2.0
