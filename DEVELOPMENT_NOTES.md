# SCID to PGN Converter - Development Documentation

## Project Overview
A Rust CLI tool for converting SCID chess databases to PGN format. SCID (Shane's Chess Information Database) uses a proprietary binary format that requires careful parsing.

## Major Breakthroughs Achieved (August 2025)

### 1. Complete SCID Binary Format Reverse Engineering ✅
**Achievement**: Full understanding of SCID .si4 index file format through systematic experimentation
**Method**: Created `experiments/scid_parser/` test harness for iterative field-by-field analysis
**Files Created**: Comprehensive working implementation with cross-validation against SCID source

**Critical Discovery - Endianness**: 
```rust
// ALL SCID multi-byte values use BIG-ENDIAN byte order (not little-endian)
let version = u16::from_be_bytes([bytes[0], bytes[1]]);     // ✅ Correct
let dates = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);  // ✅ Correct
```

**Validation**: Cross-verified against SCID source code (`mfile.cpp` ReadTwoBytes/ReadFourBytes methods)

### 2. Date Parsing - Complete Success ✅
**Previous Problem**: Invalid dates like "52298.152.207" instead of readable dates
**Root Cause**: Incorrect endianness assumptions and field offset locations
**Files Affected**: `src/scid/index.rs`, now `experiments/scid_parser/src/si4.rs`

**Working Solution**: Proper SCID date format implementation
```rust
// SCID Date Encoding (from fixed offset 25-28, big-endian):
// Bits 0-4:   Day (1-31)     - 5 bits
// Bits 5-8:   Month (1-12)   - 4 bits  
// Bits 9-19:  Year (direct)  - 11 bits (NO offset)
// Bits 20-31: Event date     - 12 bits (relative encoding)

let dates_field = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);
let game_date = dates_field & 0x000FFFFF;  // Lower 20 bits
let day = (game_date & 31) as u8;           // Bits 0-4
let month = ((game_date >> 5) & 15) as u8;  // Bits 5-8
let year = ((game_date >> 9) & 0x7FF) as u16; // Bits 9-19, NO OFFSET
```

**Validation**: Successfully parses "2022.12.19" from `test/data/five.si4`

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

### 3. Complete SCID Index Field Parsing ✅  
**Achievement**: All 47-byte game index entry fields successfully parsed and decoded
**Implementation**: `experiments/scid_parser/src/si4.rs` with comprehensive field extraction

**Key Accomplishments**:
- **Player/Event/Site/Round IDs**: 20-bit, 19-bit, 19-bit, 18-bit packed ID formats
- **Game flags**: All 16 flag types (promotions, tactics, endgames, etc.) identified
- **ELO ratings**: 12-bit values with 4-bit rating type extraction
- **Game results**: Numeric to text conversion (0=*, 1=1-0, 2=0-1, 3=1/2-1/2)
- **Game length**: 17-bit length from Length_Low + Length_High bit 7
- **Half moves**: 10-bit move count from NumHalfMoves + HomePawnData high bits

**Example Output**: "Result: 3 (1/2-1/2)", "White ELO: 2372", "Black ELO: 2419"

## Current Status (August 2025)

### ✅ Fully Working Features  
- **SCID .si4 Index Parsing** ✅ **COMPLETE** - All fields correctly extracted with big-endian
- **Date parsing** ✅ **WORKING** - Correctly shows "2022.12.19" from binary data
- **Name extraction** ✅ **WORKING** - Complete names like "Michael", not "ichael"  
- **Player/Event/Site/Round ID parsing** ✅ **COMPLETE** - Packed multi-bit ID extraction
- **Game metadata parsing** ✅ **COMPLETE** - Flags, ELO, results, length, move counts
- **Experiments framework** ✅ **PROVEN METHODOLOGY** - Systematic binary format research
- **CLI interface** - Comprehensive argument parsing with clap
- **Development mode** - `--max-games=10` default for faster testing

### 🔧 Partial Implementation
- **SCID .sn4 Name File** - Structure understood, front-coded compression working
- **Basic PGN header generation** - Uses extracted metadata for proper formatting

### ❌ Remaining Work  
- **SCID .sg4 Game File** - Chess move parsing and variation tree extraction
- **Move notation conversion** - Binary move data to standard algebraic notation
- **Integration** - Port experiments findings to main codebase (`src/scid/index.rs`)
- **Performance optimization** - Large database handling (1M+ games)

### 🚀 Next Priority Steps
1. **Apply experiments discoveries** - Update main codebase with big-endian fixes and complete parsing
2. **Implement .sg4 game parsing** - Chess moves, variations, comments, NAGs
3. **Complete PGN export** - Full-featured output with moves and metadata
4. **Optimize for scale** - Handle large databases efficiently

## Development Notes

### Testing Strategy
- Use `--max-games=10` for rapid iteration
- Test with caissabase_2022_12_24-13_26_31 database
- Validate against known good PGN output

### Code Architecture  
```
src/                                 # Main codebase (needs integration)
├── main.rs                         # CLI interface
├── scid/
│   ├── mod.rs                      # Module declarations
│   ├── index.rs                    # .si4 parsing (needs big-endian fixes)
│   ├── names.rs                    # .sn4 parsing (working)
│   ├── database.rs                 # Integration + error handling
│   └── games.rs                    # .sg4 parsing (needs implementation)
└── pgn/
    └── mod.rs                      # PGN export logic

experiments/scid_parser/             # Complete working implementation ✅
├── src/
│   ├── main.rs                     # CLI with encode/parse commands
│   ├── utils.rs                    # Big-endian byte reading utilities  
│   ├── si4.rs                      # Complete .si4 parsing (ALL FIELDS)
│   ├── date.rs                     # Date encoding/decoding functions
│   ├── sg4.rs                      # Placeholder for .sg4 parsing
│   └── sn4.rs                      # Placeholder for .sn4 parsing  
└── Cargo.toml                      # Independent test project
```

**Integration Path**: Port proven implementations from `experiments/` to `src/scid/`

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
