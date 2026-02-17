# SCID to PGN Converter

A Rust library and CLI tool for parsing SCID (Shane's Chess Information Database) files and converting them to standard PGN (Portable Game Notation) format.

## Features

- Parse SCID database files (.si4, .sn4, .sg4)
- Extract game metadata (players, dates, ratings, results, ECO codes, FIDE IDs)
- Decode chess moves with full position tracking
- Generate standard PGN output
- Both library and CLI interfaces
- Full roundtrip support (SCID → PGN → SCID)

## Installation

```bash
# Build release version
cargo build --release

# The binary will be at target/release/scidtopgn
```

## CLI Usage

```bash
# Convert a SCID database to PGN (outputs to stdout)
scidtopgn /path/to/database

# Write to a file instead
scidtopgn /path/to/database -o output.pgn

# Show game count only
scidtopgn /path/to/database --count

# Verbose output
scidtopgn /path/to/database -v
```

**Note**: Specify the database path without the file extensions (e.g., use `mydb` not `mydb.si4`).

### Example

```bash
$ scidtopgn tests/data/one
[Event "47th ch-Bangahbandhu 2022"]
[Site "Dhaka BAN"]
[Date "2022.12.19"]
[Round "5.6"]
[White "Hossain, Enam"]
[Black "Murshed, N"]
[Result "1/2-1/2"]
[WhiteTitle "GM"]
[BlackTitle "GM"]
[WhiteElo "2372"]
[BlackElo "2419"]
[ECO "B36f"]
[Opening "Sicilian"]
[Variation "accelerated fianchetto, Gurgenidze variation"]
[EventDate "2022.12.15"]
[WhiteFideId "10200649"]
[BlackFideId "10200010"]

1. e4 c5 2. Nf3 Nc6 3. d4 cxd4 4. Nxd4 g6 5. c4 Nf6 6. Nc3 Nxd4 7. Qxd4 d6 8. f3
Bg7 9. Be3 O-O 10. Qd2 Be6 11. Rc1 Qa5 12. b3 Rfc8 13. Bd3 Nd7 14. Nb5 Qxd2+
15. Kxd2 Nc5 16. Be2 a6 17. Nd4 Bd7 18. Rhd1 Ne6 19. Nxe6
1/2-1/2
```

## Library Usage

```rust
use scidtopgn::Database;

// Open a SCID database
let mut db = Database::open("path/to/database")?;

// Iterate over games
for game in db.games() {
    let game = game?;
    println!("{}", game.to_pgn());
}

// Or get a specific game by index
let game = db.get_game(0)?;
println!("White: {}", game.white);
println!("Black: {}", game.black);
```

## Architecture

This project uses a Cargo workspace with two crates:

- `scidtopgn` - Core library for SCID parsing and PGN generation
- `scidtopgn-cli` - Command-line interface

## Building

```bash
# Build all crates
cargo build

# Build release version (optimized)
cargo build --release

# Run all tests (138 tests)
cargo test --all

# Run CLI directly
cargo run -p scidtopgn-cli -- /path/to/database
```

## PGN Tags Supported

| Tag | Description |
|-----|-------------|
| Event | Tournament/event name |
| Site | Location |
| Date | Game date (YYYY.MM.DD) |
| Round | Round number |
| White | White player name |
| Black | Black player name |
| Result | Game result (1-0, 0-1, 1/2-1/2, *) |
| WhiteTitle | White's title (GM, IM, etc.) |
| BlackTitle | Black's title |
| WhiteElo | White's rating |
| BlackElo | Black's rating |
| ECO | Opening code |
| Opening | Opening name |
| Variation | Opening variation |
| EventDate | Event start date |
| WhiteFideId | White's FIDE ID |
| BlackFideId | Black's FIDE ID |

## Dependencies

- [shakmaty](https://github.com/niklasf/shakmaty) - Chess move generation and validation
- [thiserror](https://github.com/niklasf/thiserror) - Error handling
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing

## SCID Format Reference

See `SCID_DATABASE_FORMAT.md` for detailed documentation of the SCID file format.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
