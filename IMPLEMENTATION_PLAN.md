# SCID to PGN Converter - Implementation Plan

## Overview

This plan outlines the step-by-step implementation of a SCID database parser and PGN converter from scratch, following the proposed project structure and using the SCID database format specification as the technical reference.

## Guiding Principles

1. **Build incrementally** - Start simple, add complexity gradually
2. **Test continuously** - Validate each component before moving forward
3. **Follow the spec** - SCID_DATABASE_FORMAT.md is the source of truth
4. **Library-first** - Core functionality before CLI wrapper
5. **Leverage shakmaty** - Use battle-tested chess library for position/move logic
6. **Real data validation** - Test with actual SCID databases early and often

## Key Technologies

- **Rust** - Systems programming language for performance and safety
- **Shakmaty** - Comprehensive chess library handling position, moves, and SAN notation
- **Thiserror** - Ergonomic error handling
- **Clap** - Command-line argument parsing
- **Byteorder** - Big-endian binary parsing

By using shakmaty, we avoid reimplementing chess rules, move validation, and algebraic notation generation. This reduces implementation time by ~25% and eliminates an entire class of potential bugs.

---

## Phase 1: Foundation (Week 1)

### 1.1 Project Structure Setup

**Goal**: Create the workspace skeleton with all directories and basic configuration.

**Tasks**:
- Create workspace root `Cargo.toml` with resolver and workspace members
- Create `crates/core/` directory structure with all modules
- Create `crates/cli/` directory structure
- Set up workspace-level dependencies (thiserror, shakmaty, byteorder, etc.)
- Initialize git repository with .gitignore
- Create README.md with project description

**Deliverable**: Empty but compilable workspace with `cargo build` succeeding.

---

### 1.2 Core Types and Error Handling

**Goal**: Define fundamental types and error handling before parsing logic.

**Files to create**:
- `crates/core/src/error.rs` - Centralized error types
- `crates/core/src/types.rs` - Shared chess types

**Implementation**:

```rust
// error.rs - Use thiserror for clean error definitions
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
        message: String,
    },

    #[error("Invalid game index: {0}")]
    InvalidGameIndex(usize),
}

pub type Result<T> = std::result::Result<T, ScidError>;
```

```rust
// types.rs - Shared types (chess types come from shakmaty)
pub use shakmaty::{Color, Role, Square, Piece, Move as ChessMove};

#[derive(Debug, Clone)]
pub struct GameDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    Unknown,
}

impl GameResult {
    pub fn to_string(&self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unknown => "*",
        }
    }
}
```

**Testing**:
- Unit tests for error construction and display
- Unit tests for basic type operations

**Deliverable**: Compiled types and error handling ready for use.

---

## Phase 2: Index File Parser (Week 1-2)

### 2.1 SI4 Header Parsing

**Goal**: Read and validate .si4 file headers.

**File**: `crates/core/src/database/reader.rs`

**Implementation**:

```rust
const SI4_MAGIC: &[u8; 8] = b"Scid.si\0";
const SI4_HEADER_SIZE: usize = 182;

pub struct Si4Header {
    pub version: u16,
    pub base_type: u32,
    pub num_games: u32,
    pub auto_load: u32,
    pub description: String,
}

pub fn parse_si4_header(file: &mut File) -> Result<Si4Header> {
    let mut header_bytes = [0u8; SI4_HEADER_SIZE];
    file.read_exact(&mut header_bytes)?;

    // Validate magic
    if &header_bytes[0..8] != SI4_MAGIC {
        return Err(ScidError::InvalidFormat("Invalid SI4 magic".into()));
    }

    // Parse version (BIG-ENDIAN!)
    let version = u16::from_be_bytes([header_bytes[8], header_bytes[9]]);

    // Parse game count (24-bit big-endian)
    let num_games = u32::from_be_bytes([0, header_bytes[14], header_bytes[15], header_bytes[16]]);

    // Extract description (UTF-8, null-terminated)
    let description = String::from_utf8_lossy(&header_bytes[20..128])
        .trim_end_matches('\0')
        .to_string();

    Ok(Si4Header {
        version,
        base_type: u32::from_be_bytes([header_bytes[10], header_bytes[11], header_bytes[12], header_bytes[13]]),
        num_games,
        auto_load: u32::from_be_bytes([0, header_bytes[17], header_bytes[18], header_bytes[19]]),
        description,
    })
}
```

**Testing**:
- Test with real SCID database (e.g., `five.si4`)
- Verify version == 400
- Verify game count matches expected
- Test error handling with corrupted files

**Deliverable**: Validated header parsing with tests passing.

---

### 2.2 Game Index Entry Parsing

**Goal**: Parse 47-byte game index entries with all metadata fields.

**File**: `crates/core/src/database/index.rs`

**Critical Implementation Details**:
- All multi-byte values are **BIG-ENDIAN**
- Dates are at **fixed offset 25-28**
- IDs use packed bit fields (see spec for exact bit positions)
- Game result encoded in variation counts field

**Implementation**:

```rust
#[derive(Debug, Clone)]
pub struct GameIndexEntry {
    pub game_offset: u32,
    pub game_length: u32,
    pub white_id: u32,
    pub black_id: u32,
    pub event_id: u32,
    pub site_id: u32,
    pub round_id: u32,
    pub game_date: GameDate,
    pub event_date: Option<GameDate>,
    pub result: GameResult,
    pub white_elo: u16,
    pub black_elo: u16,
    pub eco_code: u16,
    pub half_moves: u16,
    pub flags: u16,
}

pub fn parse_game_index_entry(bytes: &[u8; 47]) -> Result<GameIndexEntry> {
    // Game offset and length (with 17-bit length extraction)
    let game_offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let length_low = u16::from_be_bytes([bytes[4], bytes[5]]);
    let length_high = bytes[6];
    let game_length = length_low as u32 | (((length_high & 0x80) as u32) << 9);

    // Player IDs (packed 20-bit values)
    let white_id = ((bytes[9] & 0xF0) as u32) << 12
                 | u16::from_be_bytes([bytes[10], bytes[11]]) as u32;
    let black_id = ((bytes[9] & 0x0F) as u32) << 16
                 | u16::from_be_bytes([bytes[12], bytes[13]]) as u32;

    // Event/Site/Round IDs (packed)
    let event_id = ((bytes[14] & 0xE0) as u32) << 11
                 | u16::from_be_bytes([bytes[15], bytes[16]]) as u32;
    let site_id = ((bytes[14] & 0x1C) as u32) << 14
                | u16::from_be_bytes([bytes[17], bytes[18]]) as u32;
    let round_id = ((bytes[14] & 0x03) as u32) << 16
                 | u16::from_be_bytes([bytes[19], bytes[20]]) as u32;

    // Dates field (CRITICAL: offset 25-28, big-endian)
    let dates_field = u32::from_be_bytes([bytes[25], bytes[26], bytes[27], bytes[28]]);
    let (game_date, event_date) = parse_dates_field(dates_field);

    // Result from variation counts
    let var_counts = u16::from_be_bytes([bytes[21], bytes[22]]);
    let result = match var_counts >> 12 {
        1 => GameResult::WhiteWins,
        2 => GameResult::BlackWins,
        3 => GameResult::Draw,
        _ => GameResult::Unknown,
    };

    // ELO ratings (12-bit values)
    let white_elo = u16::from_be_bytes([bytes[29], bytes[30]]) & 0x0FFF;
    let black_elo = u16::from_be_bytes([bytes[31], bytes[32]]) & 0x0FFF;

    // Other fields
    let eco_code = u16::from_be_bytes([bytes[23], bytes[24]]);
    let flags = u16::from_be_bytes([bytes[7], bytes[8]]);

    // Half-move count (10-bit split field)
    let half_moves_low = bytes[37] as u16;
    let half_moves_high = ((bytes[38] >> 6) & 0x03) as u16;
    let half_moves = half_moves_low | (half_moves_high << 8);

    Ok(GameIndexEntry {
        game_offset, game_length, white_id, black_id,
        event_id, site_id, round_id, game_date, event_date,
        result, white_elo, black_elo, eco_code, half_moves, flags,
    })
}

fn parse_dates_field(dates_field: u32) -> (GameDate, Option<GameDate>) {
    // Game date (lower 20 bits, absolute encoding)
    let game_date_raw = dates_field & 0x000FFFFF;
    let game_date = GameDate {
        day: (game_date_raw & 0x1F) as u8,
        month: ((game_date_raw >> 5) & 0x0F) as u8,
        year: ((game_date_raw >> 9) & 0x7FF) as u16,
    };

    // Event date (upper 12 bits, relative encoding)
    let event_data = (dates_field >> 20) & 0xFFF;
    let event_date = if event_data == 0 {
        None
    } else {
        let day = (event_data & 0x1F) as u8;
        let month = ((event_data >> 5) & 0x0F) as u8;
        let year_offset = ((event_data >> 9) & 0x7) as i16;

        if year_offset == 0 {
            None
        } else {
            let event_year = (game_date.year as i16 + year_offset - 4) as u16;
            Some(GameDate { day, month, year: event_year })
        }
    };

    (game_date, event_date)
}
```

**Testing**:
- Parse all entries from `five.si4`
- Verify dates match expected (e.g., 2022.12.19)
- Verify IDs are reasonable
- Verify ELO ratings are in valid range
- Test packed field extraction accuracy

**Deliverable**: Complete index entry parsing with comprehensive tests.

---

## Phase 3: Name File Parser (Week 2)

### 3.1 SN4 Header Parsing

**Goal**: Parse name file header to determine name counts and frequencies.

**File**: `crates/core/src/database/names.rs`

**Implementation**:

```rust
const SN4_MAGIC: &[u8; 8] = b"Scid.sn\0";

pub struct Sn4Header {
    pub timestamp: u32,
    pub num_players: u32,
    pub num_events: u32,
    pub num_sites: u32,
    pub num_rounds: u32,
    pub max_freq_players: u32,
    pub max_freq_events: u32,
    pub max_freq_sites: u32,
    pub max_freq_rounds: u32,
}

pub fn parse_sn4_header(file: &mut File) -> Result<Sn4Header> {
    let mut header_bytes = [0u8; 36];
    file.read_exact(&mut header_bytes)?;

    if &header_bytes[0..8] != SN4_MAGIC {
        return Err(ScidError::InvalidFormat("Invalid SN4 magic".into()));
    }

    // Parse 24-bit counts (big-endian)
    let num_players = u32::from_be_bytes([0, header_bytes[12], header_bytes[13], header_bytes[14]]);
    let num_events = u32::from_be_bytes([0, header_bytes[15], header_bytes[16], header_bytes[17]]);
    let num_sites = u32::from_be_bytes([0, header_bytes[18], header_bytes[19], header_bytes[20]]);
    let num_rounds = u32::from_be_bytes([0, header_bytes[21], header_bytes[22], header_bytes[23]]);

    // Parse max frequencies
    let max_freq_players = u32::from_be_bytes([0, header_bytes[24], header_bytes[25], header_bytes[26]]);
    let max_freq_events = u32::from_be_bytes([0, header_bytes[27], header_bytes[28], header_bytes[29]]);
    let max_freq_sites = u32::from_be_bytes([0, header_bytes[30], header_bytes[31], header_bytes[32]]);
    let max_freq_rounds = u32::from_be_bytes([0, header_bytes[33], header_bytes[34], header_bytes[35]]);

    Ok(Sn4Header {
        timestamp: u32::from_be_bytes([header_bytes[8], header_bytes[9], header_bytes[10], header_bytes[11]]),
        num_players, num_events, num_sites, num_rounds,
        max_freq_players, max_freq_events, max_freq_sites, max_freq_rounds,
    })
}
```

**Testing**:
- Verify header parsing with real .sn4 file
- Validate counts are reasonable
- Test error handling

**Deliverable**: SN4 header parsing working.

---

### 3.2 Front-Coding Decompression

**Goal**: Implement front-coding algorithm to decompress name strings.

**File**: `crates/core/src/database/names.rs`

**Implementation** (most complex parsing so far):

```rust
pub struct NameDatabase {
    pub players: Vec<String>,
    pub events: Vec<String>,
    pub sites: Vec<String>,
    pub rounds: Vec<String>,
}

fn read_variable_int(reader: &mut BufReader<File>, max_value: u32) -> Result<(u32, usize)> {
    if max_value < 256 {
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        Ok((buf[0] as u32, 1))
    } else if max_value < 65536 {
        let mut buf = [0u8; 2];
        reader.read_exact(&mut buf)?;
        Ok((u16::from_be_bytes(buf) as u32, 2))
    } else {
        let mut buf = [0u8; 3];
        reader.read_exact(&mut buf)?;
        Ok((u32::from_be_bytes([0, buf[0], buf[1], buf[2]]), 3))
    }
}

fn read_names_section(
    reader: &mut BufReader<File>,
    count: u32,
    max_frequency: u32,
) -> Result<Vec<String>> {
    let mut names = Vec::with_capacity(count as usize);
    let mut previous_name = String::new();

    for i in 0..count {
        // Read variable-length ID and frequency
        let (_name_id, _) = read_variable_int(reader, count)?;
        let (_frequency, _) = read_variable_int(reader, max_frequency)?;

        // Read total length
        let mut len_buf = [0u8; 1];
        reader.read_exact(&mut len_buf)?;
        let total_length = len_buf[0] as usize;

        // Read prefix length (not for first name)
        let prefix_length = if i == 0 {
            0
        } else {
            let mut prefix_buf = [0u8; 1];
            reader.read_exact(&mut prefix_buf)?;
            prefix_buf[0] as usize
        };

        // Read suffix
        let suffix_length = total_length - prefix_length;
        let mut suffix_bytes = vec![0u8; suffix_length];
        reader.read_exact(&mut suffix_bytes)?;

        // Reconstruct name using front-coding
        previous_name.truncate(prefix_length);
        previous_name.push_str(&String::from_utf8_lossy(&suffix_bytes));

        // Clean and store
        let clean_name = previous_name
            .chars()
            .filter(|&c| c >= ' ')
            .collect::<String>()
            .trim()
            .to_string();

        names.push(clean_name);
    }

    Ok(names)
}

pub fn parse_name_database(file: File) -> Result<NameDatabase> {
    let mut reader = BufReader::new(file);
    let header = parse_sn4_header_from_reader(&mut reader)?;

    // Read four sections in order
    let players = read_names_section(&mut reader, header.num_players, header.max_freq_players)?;
    let events = read_names_section(&mut reader, header.num_events, header.max_freq_events)?;
    let sites = read_names_section(&mut reader, header.num_sites, header.max_freq_sites)?;
    let rounds = read_names_section(&mut reader, header.num_rounds, header.max_freq_rounds)?;

    Ok(NameDatabase { players, events, sites, rounds })
}
```

**Testing**:
- Parse complete name database
- Verify player names match expected (e.g., "Hossain, Enam")
- Test front-coding reconstruction accuracy
- Test with various name lengths and special characters

**Deliverable**: Complete name database parsing.

---

## Phase 4: Game File Structure (Week 3)

### 4.1 Game Boundary Detection

**Goal**: Locate individual games within .sg4 file using index offsets.

**File**: `crates/core/src/database/games.rs`

**Implementation**:

```rust
pub struct GameData {
    pub tags: HashMap<String, String>,
    pub flags: u8,
    pub start_position: Option<String>, // FEN if non-standard
    pub moves: Vec<u8>, // Raw move bytes for now
}

pub fn read_game_data(
    sg4_file: &mut File,
    offset: u32,
    length: u32,
) -> Result<Vec<u8>> {
    sg4_file.seek(SeekFrom::Start(offset as u64))?;

    let mut game_bytes = vec![0u8; length as usize];
    sg4_file.read_exact(&mut game_bytes)?;

    Ok(game_bytes)
}
```

**Testing**:
- Read game slices from `five.sg4` using index offsets
- Verify boundary detection
- Ensure no overlap or gaps between games

**Deliverable**: Game data extraction working.

---

### 4.2 Tag and Flags Parsing

**Goal**: Parse PGN tags and game flags from game data.

**File**: `crates/core/src/database/games.rs`

**Implementation**:

```rust
const MAX_TAG_LEN: u8 = 240;
const COMMON_TAGS: [&str; 10] = [
    "WhiteTitle", "BlackTitle", "Annotator", "Opening",
    "Variation", "SubVariation", "WhiteElo", "BlackElo",
    "WhiteUSCF", "BlackUSCF",
];

pub fn parse_game_tags(bytes: &[u8]) -> Result<(HashMap<String, String>, u8, usize)> {
    let mut tags = HashMap::new();
    let mut pos = 0;

    loop {
        if pos >= bytes.len() {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: pos as u64,
                message: "Unexpected end of tags".into(),
            });
        }

        let byte = bytes[pos];
        pos += 1;

        if byte == 0 {
            // Null terminator - tags end
            break;
        }

        if byte == 255 {
            // Special 3-byte EventDate encoding
            pos += 3;
        } else if byte > MAX_TAG_LEN {
            // Common tag
            let tag_index = (byte - MAX_TAG_LEN - 1) as usize;
            let tag_name = COMMON_TAGS.get(tag_index)
                .ok_or_else(|| ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: pos as u64,
                    message: format!("Unknown common tag: {}", byte),
                })?;

            let value_len = bytes[pos] as usize;
            pos += 1;
            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(tag_name.to_string(), value);
        } else {
            // Regular tag
            let name_len = byte as usize;
            let name = String::from_utf8_lossy(&bytes[pos..pos + name_len]).to_string();
            pos += name_len;

            let value_len = bytes[pos] as usize;
            pos += 1;
            let value = String::from_utf8_lossy(&bytes[pos..pos + value_len]).to_string();
            pos += value_len;

            tags.insert(name, value);
        }
    }

    // Read flags byte
    let flags = bytes[pos];
    pos += 1;

    Ok((tags, flags, pos))
}
```

**Testing**:
- Parse tags from real game data
- Verify common vs. regular tag handling
- Test null terminator detection

**Deliverable**: Tag parsing working correctly.

---

## Phase 5: Chess Position Tracking with Shakmaty (Week 3-4)

### 5.1 Position Wrapper with SCID Piece Tracking

**Goal**: Wrap shakmaty's Chess position with SCID piece number tracking.

**File**: `crates/core/src/parser/position.rs`

**Implementation**:

```rust
use shakmaty::{Chess, Position, Square, Move, Role, Color};
use std::collections::HashMap;

/// Wrapper around shakmaty::Chess that tracks SCID piece numbers
pub struct ScidPosition {
    // Shakmaty handles all chess logic
    chess: Chess,

    // Map SCID piece numbers (0-15) to current squares
    // Updated as moves are made
    piece_locations: HashMap<u8, Square>,
}

impl ScidPosition {
    pub fn new() -> Self {
        let chess = Chess::default(); // Standard starting position
        let mut piece_locations = HashMap::new();

        // Initialize SCID piece number mappings for starting position
        // White: King=0, Queen=1, Ra1=2, Rh1=3, Bc1=4, Bf1=5, Nb1=6, Ng1=7, pawns=8-15
        // Black: Similar numbering for black pieces
        Self::init_piece_mappings(&mut piece_locations, &chess);

        ScidPosition {
            chess,
            piece_locations,
        }
    }

    fn init_piece_mappings(mappings: &mut HashMap<u8, Square>, chess: &Chess) {
        // Map initial piece numbers to squares based on SCID scheme
        // White pieces (0-15 when white to move)
        mappings.insert(0, Square::E1);  // White King
        mappings.insert(1, Square::D1);  // White Queen
        mappings.insert(2, Square::A1);  // White Rook a1
        mappings.insert(3, Square::H1);  // White Rook h1
        // ... continue for all pieces

        // Black pieces (0-15 when black to move)
        // Note: SCID piece numbers are relative to side to move
    }

    pub fn get_piece_square(&self, piece_num: u8) -> Option<Square> {
        self.piece_locations.get(&piece_num).copied()
    }

    pub fn board(&self) -> &shakmaty::Board {
        self.chess.board()
    }

    pub fn turn(&self) -> Color {
        self.chess.turn()
    }

    pub fn make_move(&mut self, chess_move: &Move) -> Result<()> {
        // Validate move is legal using shakmaty
        if !self.chess.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        // Apply move using shakmaty (handles all chess rules)
        self.chess = self.chess.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        // Update piece location tracking
        self.update_piece_locations(chess_move);

        Ok(())
    }

    fn update_piece_locations(&mut self, chess_move: &Move) {
        // Update internal mapping after move
        // Handle captures (remove captured piece)
        // Handle castling (move both king and rook)
        // Handle promotions (change piece type)
    }

    pub fn legal_moves(&self) -> Vec<Move> {
        // Use shakmaty's legal move generation
        let mut moves = Vec::new();
        self.chess.legal_moves(&mut moves);
        moves
    }

    pub fn find_move(&self, from: Square, to: Square, promotion: Option<Role>) -> Option<Move> {
        // Find legal move matching from/to squares
        // Used to convert SCID decoded moves to shakmaty Moves
        let legals = self.legal_moves();
        legals.into_iter().find(|m| {
            m.from() == Some(from) && m.to() == to &&
            m.promotion() == promotion
        })
    }
}
```

**Testing**:
- Test starting position setup
- Test piece tracking updates correctly after moves
- Test legal move validation via shakmaty
- Test special moves (castling, en passant, promotions) handled by shakmaty

**Deliverable**: Position wrapper with SCID piece tracking leveraging shakmaty.

---

### 5.2 Move Decoder (SCID to Shakmaty)

**Goal**: Decode SCID binary move encoding to shakmaty Move types.

**File**: `crates/core/src/parser/decoder.rs`

**Implementation** (decodes SCID format, outputs shakmaty Moves):

```rust
use shakmaty::{Square, Move, Role};

pub struct ScidMoveDecoder {
    position: ScidPosition,
}

impl ScidMoveDecoder {
    pub fn new() -> Self {
        ScidMoveDecoder {
            position: ScidPosition::new(),
        }
    }

    pub fn decode_move(&mut self, move_byte: u8, stream: &mut ByteStream) -> Result<Move> {
        let piece_num = (move_byte >> 4) & 0x0F;
        let move_value = move_byte & 0x0F;

        // Get piece location from SCID tracking
        let from_square = self.position.get_piece_square(piece_num)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid piece number: {}", piece_num),
            })?;

        // Get piece at that square from shakmaty
        let piece = self.position.board().piece_at(from_square)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("No piece at square: {:?}", from_square),
            })?;

        // Decode SCID move value to target square
        let (to_square, promotion) = match piece.role {
            Role::King => self.decode_king_move(from_square, move_value)?,
            Role::Queen => self.decode_queen_move(from_square, move_value, stream)?,
            Role::Rook => self.decode_rook_move(from_square, move_value)?,
            Role::Bishop => self.decode_bishop_move(from_square, move_value)?,
            Role::Knight => self.decode_knight_move(from_square, move_value)?,
            Role::Pawn => self.decode_pawn_move(from_square, move_value)?,
        };

        // Use shakmaty to find the legal move matching our from/to/promotion
        let chess_move = self.position.find_move(from_square, to_square, promotion)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("No legal move from {:?} to {:?}", from_square, to_square),
            })?;

        // Apply move using shakmaty (validates and updates position)
        self.position.make_move(&chess_move)?;

        Ok(chess_move)
    }

    fn decode_king_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        let to = match move_value {
            0 => return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: "Null move".into(),
            }),
            1..=8 => {
                // Adjacent square move
                let diffs = [0, -9, -8, -7, -1, 1, 7, 8, 9];
                let offset = diffs[move_value as usize];
                self.offset_square(from, offset)?
            }
            10 => {
                // Kingside castle - shakmaty will recognize this from squares
                if self.position.turn() == Color::White {
                    Square::G1
                } else {
                    Square::G8
                }
            }
            11 => {
                // Queenside castle
                if self.position.turn() == Color::White {
                    Square::C1
                } else {
                    Square::C8
                }
            }
            _ => return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid king move value: {}", move_value),
            }),
        };
        Ok((to, None))
    }

    fn decode_pawn_move(&self, from: Square, move_value: u8) -> Result<(Square, Option<Role>)> {
        let forward = if self.position.turn() == Color::White { 8 } else { -8 };
        let (to, promotion) = match move_value {
            0 => (self.offset_square(from, forward - 1)?, None), // Capture left
            1 => (self.offset_square(from, forward)?, None),     // Move forward
            2 => (self.offset_square(from, forward + 1)?, None), // Capture right
            3..=5 => (self.offset_square(from, forward - 1)?, Some(Role::Queen)), // Promote capture left
            6..=8 => (self.offset_square(from, forward - 1)?, Some(Role::Rook)),
            9..=11 => (self.offset_square(from, forward - 1)?, Some(Role::Bishop)),
            12..=14 => (self.offset_square(from, forward - 1)?, Some(Role::Knight)),
            15 => (self.offset_square(from, forward * 2)?, None), // Double push
            _ => return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Invalid pawn move value: {}", move_value),
            }),
        };
        Ok((to, promotion))
    }

    fn decode_queen_move(&self, from: Square, move_value: u8, stream: &mut ByteStream)
        -> Result<(Square, Option<Role>)> {
        // CRITICAL: Queen diagonal moves are 2 bytes!
        let from_file = from.file() as u8;
        let from_rank = from.rank() as u8;

        let to = if move_value >= 8 {
            // Vertical move (1 byte)
            let target_rank = move_value - 8;
            Square::from_coords(shakmaty::File::new(from_file), shakmaty::Rank::new(target_rank))
        } else if move_value != from_file {
            // Horizontal move (1 byte)
            Square::from_coords(shakmaty::File::new(move_value), shakmaty::Rank::new(from_rank))
        } else {
            // Diagonal move (2 bytes) - read second byte from stream!
            let second_byte = stream.get_byte()?;
            if second_byte < 64 || second_byte > 127 {
                return Err(ScidError::ParseError {
                    file: PathBuf::from("sg4"),
                    offset: 0,
                    message: format!("Invalid queen diagonal byte: {}", second_byte),
                });
            }
            Square::new(second_byte - 64)
        };

        Ok((to, None))
    }

    // Implement decode_rook_move, decode_bishop_move, decode_knight_move similarly
    // All return (Square, Option<Role>) for target and optional promotion

    fn offset_square(&self, square: Square, offset: i8) -> Result<Square> {
        let idx = square.index() as i8 + offset;
        if idx < 0 || idx > 63 {
            return Err(ScidError::ParseError {
                file: PathBuf::from("sg4"),
                offset: 0,
                message: format!("Square offset out of bounds: {}", idx),
            });
        }
        Ok(Square::new(idx as u8))
    }
}
```

**Critical Notes**:
- SCID move bytes decode to from/to squares + promotion
- Shakmaty validates legality and constructs proper Move type
- Queen diagonal moves require ByteStream for 2-byte reads
- Shakmaty handles all chess rules (castling rights, en passant, etc.)

**Testing**:
- Test each piece type move decoding
- Test 2-byte Queen diagonal moves
- Test shakmaty validates moves correctly
- Test position updates via shakmaty
- Validate against known game sequences

**Deliverable**: Complete SCID decoder outputting shakmaty Moves.

---

## Phase 6: PGN Output with Shakmaty (Week 4)

### 6.1 Algebraic Notation Generation (Using Shakmaty)

**Goal**: Use shakmaty's built-in SAN generation.

**File**: `crates/core/src/format/pgn.rs`

**Implementation** (much simpler with shakmaty!):

```rust
use shakmaty::{Chess, Position, Move, san::San};

/// Helper to track position and generate SAN notation
pub struct SanGenerator {
    position: Chess,
}

impl SanGenerator {
    pub fn new() -> Self {
        SanGenerator {
            position: Chess::default(),
        }
    }

    pub fn from_position(position: Chess) -> Self {
        SanGenerator { position }
    }

    /// Generate SAN for a move and update position
    pub fn move_to_san(&mut self, chess_move: &Move) -> Result<String> {
        // Shakmaty generates SAN notation automatically!
        let san = San::from_move(&self.position, chess_move);

        // Apply move to update position for next move
        self.position = self.position.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("pgn"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        Ok(san.to_string())
    }

    /// Generate SAN for a sequence of moves
    pub fn moves_to_san(&mut self, moves: &[Move]) -> Result<Vec<String>> {
        moves.iter()
            .map(|m| self.move_to_san(m))
            .collect()
    }
}
```

**Key Benefits**:
- Shakmaty handles all SAN rules (disambiguation, check/checkmate symbols, etc.)
- No need to implement complex disambiguation logic
- Guaranteed correct SAN output
- Handles edge cases automatically

**Testing**:
- Test SAN generation for various move types
- Verify disambiguation in complex positions
- Test check/checkmate notation
- Validate output with PGN parsers

**Deliverable**: SAN generation using shakmaty.

---

### 6.2 PGN Formatter

**Goal**: Generate complete PGN output from parsed game data.

**File**: `crates/core/src/format/converter.rs`

**Implementation**:

```rust
#[derive(Debug, Clone)]
pub struct PgnOptions {
    pub include_comments: bool,
    pub include_variations: bool,
    pub compact: bool,
}

pub struct PgnFormatter;

impl PgnFormatter {
    pub fn format_game(
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        moves: &[ChessMove],
        options: &PgnOptions,
    ) -> String {
        let mut pgn = String::new();

        // Write seven tag roster
        pgn.push_str(&format!("[Event \"{}\"]\n", names.events.get(index_entry.event_id as usize).unwrap_or(&"?".to_string())));
        pgn.push_str(&format!("[Site \"{}\"]\n", names.sites.get(index_entry.site_id as usize).unwrap_or(&"?".to_string())));
        pgn.push_str(&format!("[Date \"{}.{:02}.{:02}\"]\n",
            index_entry.game_date.year,
            index_entry.game_date.month,
            index_entry.game_date.day));
        pgn.push_str(&format!("[Round \"{}\"]\n", names.rounds.get(index_entry.round_id as usize).unwrap_or(&"?".to_string())));
        pgn.push_str(&format!("[White \"{}\"]\n", names.players.get(index_entry.white_id as usize).unwrap_or(&"?".to_string())));
        pgn.push_str(&format!("[Black \"{}\"]\n", names.players.get(index_entry.black_id as usize).unwrap_or(&"?".to_string())));
        pgn.push_str(&format!("[Result \"{}\"]\n", index_entry.result.to_string()));

        // Optional tags
        if index_entry.white_elo > 0 {
            pgn.push_str(&format!("[WhiteElo \"{}\"]\n", index_entry.white_elo));
        }
        if index_entry.black_elo > 0 {
            pgn.push_str(&format!("[BlackElo \"{}\"]\n", index_entry.black_elo));
        }

        // Add custom tags from game data
        for (key, value) in &game_data.tags {
            pgn.push_str(&format!("[{} \"{}\"]\n", key, value));
        }

        pgn.push('\n');

        // Write moves using shakmaty
        let mut san_gen = SanGenerator::new();
        let mut move_num = 1;

        for (i, chess_move) in moves.iter().enumerate() {
            if i % 2 == 0 {
                pgn.push_str(&format!("{}. ", move_num));
            }

            // Shakmaty generates perfect SAN notation
            let san = san_gen.move_to_san(chess_move)?;
            pgn.push_str(&san);
            pgn.push(' ');

            if i % 2 == 1 {
                move_num += 1;
                if !options.compact && (i + 1) % 10 == 0 {
                    pgn.push('\n');
                }
            }
        }

        // Result
        pgn.push_str(&format!("{}\n\n", index_entry.result.to_string()));

        pgn
    }
}
```

**Testing**:
- Generate PGN for complete games
- Verify format matches PGN specification
- Test with various options
- Validate with PGN parsers/validators

**Deliverable**: Complete PGN generation.

---

## Phase 7: Public API Design (Week 5)

### 7.1 ScidReader Implementation

**Goal**: Create the main library entry point.

**File**: `crates/core/src/lib.rs`

**Implementation**:

```rust
pub struct ScidReader {
    si4_header: Si4Header,
    index_entries: Vec<GameIndexEntry>,
    names: NameDatabase,
    sg4_file: File,
}

impl ScidReader {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let base_path = path.as_ref();

        // Validate and open all three files
        let mut si4_file = File::open(base_path.with_extension("si4"))?;
        let sn4_file = File::open(base_path.with_extension("sn4"))?;
        let sg4_file = File::open(base_path.with_extension("sg4"))?;

        // Parse index header
        let si4_header = parse_si4_header(&mut si4_file)?;

        // Parse all index entries
        let mut index_entries = Vec::with_capacity(si4_header.num_games as usize);
        for _ in 0..si4_header.num_games {
            let mut entry_bytes = [0u8; 47];
            si4_file.read_exact(&mut entry_bytes)?;
            index_entries.push(parse_game_index_entry(&entry_bytes)?);
        }

        // Parse name database
        let names = parse_name_database(sn4_file)?;

        Ok(ScidReader {
            si4_header,
            index_entries,
            names,
            sg4_file,
        })
    }

    pub fn games(&self) -> impl Iterator<Item = Result<Game>> + '_ {
        (0..self.index_entries.len()).map(move |i| self.get_game(i))
    }

    pub fn get_game(&self, index: usize) -> Result<Game> {
        let entry = self.index_entries.get(index)
            .ok_or(ScidError::InvalidGameIndex(index))?;

        // Read game data from SG4
        let game_bytes = read_game_data(
            &mut self.sg4_file.try_clone()?,
            entry.game_offset,
            entry.game_length,
        )?;

        // Parse game
        let (tags, flags, move_start) = parse_game_tags(&game_bytes)?;

        // Decode moves
        let mut decoder = MoveDecoder::new();
        let moves = decoder.decode_all_moves(&game_bytes[move_start..])?;

        Ok(Game {
            index: *entry,
            tags,
            moves,
        })
    }

    pub fn to_pgn(&self, options: PgnOptions) -> Result<String> {
        let mut pgn = String::new();

        for game in self.games() {
            let game = game?;
            let game_pgn = PgnFormatter::format_game(
                &game.index,
                &self.names,
                &game.tags,
                &game.moves,
                &options,
            );
            pgn.push_str(&game_pgn);
        }

        Ok(pgn)
    }

    pub fn write_pgn(&self, mut writer: impl std::io::Write, options: PgnOptions) -> Result<()> {
        for game in self.games() {
            let game = game?;
            let game_pgn = PgnFormatter::format_game(
                &game.index,
                &self.names,
                &game.tags,
                &game.moves,
                &options,
            );
            writer.write_all(game_pgn.as_bytes())?;
        }
        Ok(())
    }

    pub fn metadata(&self) -> &Si4Header {
        &self.si4_header
    }
}
```

**Testing**:
- Test opening various SCID databases
- Test game iteration
- Test PGN generation
- Integration tests with real databases

**Deliverable**: Complete public API.

---

### 7.2 Prelude Module

**Goal**: Create convenient re-exports for library users.

**File**: `crates/core/src/prelude.rs`

**Implementation**:

```rust
pub use crate::{ScidReader, ScidError, Result};
pub use crate::database::{Game, GameIndexEntry};
pub use crate::format::{PgnOptions, PgnFormatter};
pub use crate::types::GameResult;

// Re-export common shakmaty types
pub use shakmaty::{Color, Role, Square, Move, Chess, Position};
```

**Deliverable**: Ergonomic imports for library users.

---

## Phase 8: CLI Tool (Week 5-6)

### 8.1 CLI Argument Parsing

**Goal**: Create user-friendly command-line interface.

**File**: `crates/cli/src/args.rs`

**Implementation**:

```rust
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "scidtopgn")]
#[command(about = "Convert SCID chess databases to PGN format")]
pub struct Args {
    /// Path to SCID database (without extension)
    #[arg(value_name = "DATABASE")]
    pub input: String,

    /// Output file (stdout if not specified)
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<String>,

    /// Include comments in output
    #[arg(long, default_value = "true")]
    pub comments: bool,

    /// Include variations in output
    #[arg(long, default_value = "true")]
    pub variations: bool,

    /// Compact output (no line breaks)
    #[arg(long)]
    pub compact: bool,

    /// Convert only specific game range (e.g., "1-100")
    #[arg(short, long, value_name = "RANGE")]
    pub range: Option<String>,
}
```

**Testing**:
- Test argument parsing
- Test help output
- Test invalid arguments

**Deliverable**: CLI argument parsing.

---

### 8.2 Main CLI Logic

**Goal**: Wire everything together in the CLI tool.

**File**: `crates/cli/src/main.rs`

**Implementation**:

```rust
use clap::Parser;
use scidtopgn_core::prelude::*;
use std::fs::File;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Open SCID database
    eprintln!("Opening database: {}", args.input);
    let reader = ScidReader::open(&args.input)?;

    eprintln!("Database: {}", reader.metadata().description);
    eprintln!("Games: {}", reader.metadata().num_games);

    // Configure PGN options
    let options = PgnOptions {
        include_comments: args.comments,
        include_variations: args.variations,
        compact: args.compact,
    };

    // Write output
    match args.output {
        Some(path) => {
            eprintln!("Writing to file: {}", path);
            let file = File::create(path)?;
            reader.write_pgn(file, options)?;
        }
        None => {
            let stdout = io::stdout();
            let handle = stdout.lock();
            reader.write_pgn(handle, options)?;
        }
    }

    eprintln!("Conversion complete!");
    Ok(())
}
```

**Testing**:
- End-to-end CLI tests
- Test with various arguments
- Test error handling and user messages

**Deliverable**: Working CLI tool.

---

## Phase 9: Testing & Validation (Week 6)

### 9.1 Unit Tests

**Coverage**:
- All parsing functions
- All type conversions
- All edge cases

**Deliverable**: High unit test coverage.

---

### 9.2 Integration Tests

**File**: `crates/core/tests/integration.rs`

**Tests**:
- Complete database parsing
- PGN generation validation
- Performance benchmarks

**Deliverable**: Comprehensive integration tests.

---

### 9.3 Real-World Validation

**Goal**: Test with actual large databases.

**Tests**:
- Parse 100,000+ game database
- Verify PGN output with chess tools
- Performance profiling
- Memory usage analysis

**Deliverable**: Production-ready library.

---

## Phase 10: Documentation & Polish (Week 7)

### 10.1 API Documentation

**Tasks**:
- Document all public APIs with rustdoc
- Create examples in `examples/` directory
- Write comprehensive README
- Add usage examples

**Deliverable**: Complete documentation.

---

### 10.2 Performance Optimization

**Tasks**:
- Profile parsing performance
- Optimize hot paths
- Add benchmarks
- Memory optimization

**Deliverable**: Optimized library.

---

## Success Criteria

### Minimum Viable Product (MVP)
- ✅ Parse .si4, .sn4, .sg4 files correctly
- ✅ Extract game metadata (players, dates, results)
- ✅ Decode chess moves to algebraic notation
- ✅ Generate valid PGN output
- ✅ Working CLI tool

### Full Feature Set
- ✅ Support all SCID move encodings
- ✅ Handle variations and comments
- ✅ Support annotations (NAGs)
- ✅ Memory-efficient streaming for large databases
- ✅ Comprehensive error handling
- ✅ Full test coverage

### Production Ready
- ✅ Performance: Parse 1M+ games efficiently
- ✅ Reliability: Handle corrupted data gracefully
- ✅ Usability: Clear error messages and documentation
- ✅ Compatibility: Works with all SCID database versions

---

## Risk Mitigation

### Known Challenges

1. **Queen Diagonal Moves (2-byte encoding)**
   - **Risk**: Complex streaming parser required
   - **Mitigation**: Implement ByteStream early, test thoroughly

2. **Position Tracking Complexity**
   - **Risk**: Bugs in board state tracking
   - **Mitigation**: Extensive unit tests, validate with known games

3. **Performance with Large Databases**
   - **Risk**: Memory/speed issues with millions of games
   - **Mitigation**: Profile early, use streaming APIs, memory mapping

4. **SCID Format Variations**
   - **Risk**: Edge cases not in spec
   - **Mitigation**: Test with diverse databases, graceful error handling

---

## Timeline Summary (Updated with Shakmaty)

- **Week 1**: Foundation + Index Parser
- **Week 2**: Name Parser + Game Structure
- **Week 3**: SCID Position Wrapper + Move Decoder (simplified with shakmaty)
- **Week 4**: PGN Output (using shakmaty SAN) + Public API
- **Week 5**: CLI Tool + Integration Testing
- **Week 6**: Real-World Validation + Documentation + Polish

**Total**: ~5-6 weeks for production-ready implementation

**Time Savings from Shakmaty**:
- ~1-2 weeks saved on chess position logic
- ~1 week saved on SAN generation and disambiguation
- Fewer bugs to fix (battle-tested library)
- More time for SCID-specific parsing and optimization

---

## Next Steps

1. Set up workspace structure
2. Implement core types and errors
3. Begin SI4 header parsing with real test data
4. Iterate through phases systematically
5. Test continuously with real SCID databases

This plan provides a clear roadmap from empty workspace to production-ready SCID parser with comprehensive testing and documentation.
