# SCID to PGN Converter - Development Documentation

## Project Overview
A Rust CLI tool for converting SCID chess databases to PGN format. SCID (Shane's Chess Information Database) uses a proprietary binary format that requires careful parsing.

## Major Issues Solved (July 2025)

### 1. Date Parsing Bug - "52298.152.207" Issue
**Problem**: Invalid dates like "52298.152.207" instead of readable dates
**Root Cause**: Incorrect bit-field extraction from SCID's packed date encoding
**Files Affected**: `src/scid/index.rs`

**Solution**: Proper SCID date format implementation
```rust
// SCID Date Encoding (20-bit packed field):
// Bits 0-4:   Day (1-31)     - 5 bits
// Bits 5-8:   Month (1-12)   - 4 bits  
// Bits 9-19:  Year - 1900    - 11 bits

let date_value = dates & 0x000FFFFF; // Extract lower 20 bits
let day = (date_value & 31) as u8;           // Bits 0-4
let month = ((date_value >> 5) & 15) as u8;  // Bits 5-8
let year = (date_value >> 9) as u16;         // Bits 9-19
```

**Validation**: Now correctly shows dates like "1791.12.24"

### 2. Name Extraction Bug - "ichael" vs "Michael" Issue  
**Problem**: Names extracted partially - "Michael" became "ichael", "Patrick" became "atrick"
**Root Cause**: Misunderstanding of SCID's front-coded string compression format
**Files Affected**: `src/scid/names.rs`, `src/scid/database.rs`

**Solution**: Proper SCID .sn4 format parsing based on official source code
- Implemented correct 44-byte header parsing
- Fixed front-coded string decompression 
- Added proper control character cleaning
- Used variable-length encoding for IDs and frequencies

**Key Discovery**: SCID strings are NOT prefix-compressed as initially assumed, but stored with length byte + string data + control character cleaning needed.

**Validation**: Now extracts complete names: "Michael", "Patrick", "'t Hart, Joost TE"

### 3. Error Type Integration
**Problem**: Type mismatch between `Box<dyn std::error::Error>` and `io::Error`
**Solution**: Error conversion with `.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))`

## SCID File Format Documentation

### File Structure
- **basename.si4**: Index file with game metadata (dates, player/event IDs, file offsets)
- **basename.sn4**: Name database (player names, event names, site names, round names)  
- **basename.sg4**: Game data (chess moves, annotations, variations)

### .sn4 Name File Format
```
Header (44 bytes):
- Magic: "Scid.sn\0" (8 bytes)
- Version: 2 bytes
- Timestamp: 4 bytes
- Num names per type: 4 × 3 bytes (PLAYER, EVENT, SITE, ROUND)
- Max ID per type: 4 × 3 bytes  
- Flags: 1 byte
- Reserved: 3 bytes

Data Section:
- Names in order: PLAYER(0), EVENT(1), SITE(2), ROUND(3)
- Each entry: variable-length ID + frequency + front-coded string
- String format: length_byte + string_data
```

### Variable-Length Encoding
```
If first_byte < 128: single byte value
If first_byte >= 128: two bytes, value = (first_byte & 0x7F) | (second_byte << 7)
```

## Current Status

### ✅ Working Features
- Date parsing (shows "1791.12.24" instead of garbage)
- Name extraction (complete names like "Michael", not "ichael")
- Basic PGN header generation
- CLI interface with proper argument parsing
- Development mode (--max-games=10 default for faster testing)

### ❌ Known Issues
- Game data parsing from .sg4 files ("failed to fill whole buffer" error)
- Move notation conversion not implemented
- Limited to first 10 games by default (development setting)

### 🔄 Next Steps
1. Fix .sg4 game data reading
2. Implement chess move parsing and PGN notation conversion
3. Add proper error handling for malformed game data
4. Performance optimization for large databases

## Development Notes

### Testing Strategy
- Use `--max-games=10` for rapid iteration
- Test with caissabase_2022_12_24-13_26_31 database
- Validate against known good PGN output

### Code Architecture
```
src/
├── main.rs           # CLI interface + comprehensive fix documentation
├── scid/
│   ├── mod.rs        # Module declarations
│   ├── index.rs      # .si4 parsing + date fix implementation
│   ├── names.rs      # .sn4 parsing + name extraction fix
│   ├── database.rs   # Integration + error type handling
│   └── games.rs      # .sg4 parsing (needs work)
└── pgn/
    └── mod.rs        # PGN export logic
```

### Key Technical References
- SCID source code: https://github.com/benini/scid/blob/master/src/namebase.cpp
- SCID namebase.h for constants and structures
- PGN specification for output format

## Usage Examples

```bash
# Development mode (first 10 games only)
cargo run -- scid/caissabase_2022_12_24-13_26_31

# Convert all games  
cargo run -- --max-games=0 scid/caissabase_2022_12_24-13_26_31

# Specify output file
cargo run -- -o output.pgn scid/caissabase_2022_12_24-13_26_31

# Force overwrite existing file
cargo run -- -f -o existing.pgn scid/caissabase_2022_12_24-13_26_31
```

## Debugging Commands

```bash
# Build and test
cargo build
./target/debug/scidtopgn -o test.pgn scid/caissabase_2022_12_24-13_26_31

# Check parsing output
head -20 test.pgn  # Should show proper dates and names
```

## Git Commit Strategy

Use descriptive commits that preserve knowledge:
```bash
git commit -m "Fix SCID name parsing: implement proper front-coded string decompression

- Solves 'ichael' vs 'Michael' partial name extraction issue  
- Implements official SCID .sn4 binary format parsing
- Based on SCID namebase.cpp source code analysis
- Correctly handles variable-length encoding and front-coding
- Names now extract completely: 'Michael', 'Patrick', etc."
```

## Knowledge Preservation

This documentation ensures that all technical discoveries, implementation details, and debugging insights are preserved for future development, even if the code is moved or the conversation context is lost.
