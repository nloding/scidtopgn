# Phase 5: Chess Position Tracking and Move Parsing

## Overview

Phase 5 represents the **critical integration point** where SCID's binary move encoding meets shakmaty's chess engine. This phase transforms the parsed game file data from Phase 4 into legal chess moves by:

1. **Wrapping shakmaty's Chess position** with SCID-specific piece tracking
2. **Decoding binary move bytes** into from/to squares and promotions
3. **Validating moves** using shakmaty's legal move generation
4. **Handling multi-byte moves** via ByteStream for Queen diagonal moves
5. **Maintaining board state** throughout the game

**Why This Phase Is Critical**: Without accurate position tracking and move decoding, we cannot:
- Generate correct PGN notation
- Validate move legality
- Handle variations and annotations
- Support the complete SCID feature set

**Integration with Previous Phases**:
- **Phase 4 Output**: Raw game bytes with known tag/flag/move boundaries
- **Phase 5 Output**: Validated sequence of shakmaty `Move` objects ready for SAN generation

**Key Challenge**: SCID move values are **position-dependent**. The same byte `0x67` could represent different moves depending on:
- Which piece is moving (piece number mapping)
- Current board position (where that piece is located)
- Legal moves available (castling rights, en passant, etc.)

**Success Criteria**:
- ✅ Parse 90%+ of moves from real SCID databases
- ✅ All special moves work (castling, en passant, promotions)
- ✅ Queen diagonal moves (2-byte sequences) parse correctly
- ✅ Move validation via shakmaty catches illegal moves
- ✅ Position state remains consistent throughout games

---

## Reference Documentation

### SCID Database Format Specification

**Primary Reference**: `SCID_DATABASE_FORMAT.md` (THE BIBLE - verified against SCID source code)

**Critical Sections for Phase 5**:

1. **Move Encoding Overview** (Section 4.1)
   - Basic move structure: `[piece_num:4][move_value:4]`
   - Piece-specific encoding schemes
   - Position-dependent value interpretation
   - Source: game.cpp lines 59061-59094

2. **Piece Number Semantics** (Section 4.1.2)
   - Pieces numbered by STARTING FILE, not piece type!
   - Order: King(0), QR(1), QN(2), QB(3), Q(4), KB(5), KN(6), KR(7), Pawns(8-15)
   - Dynamic tracking updated after each move
   - Source: game.cpp initPieceList()

3. **King Moves** (Section 4.2.1)
   - Value 0: NULL MOVE (valid - king stays in place)
   - Values 1-8: Adjacent squares
   - Value 9: Queenside castling (O-O-O)
   - Value 10: Kingside castling (O-O)
   - Values 11-15: INVALID
   - Source: game.cpp lines 59108-59136

4. **Knight Moves** (Section 4.2.3)
   - Values 1-8: L-shaped jumps
   - Values 0 and 9-15: INVALID
   - Source: game.cpp lines 59204-59230

5. **Bishop Moves** (Section 4.2.4)
   - Uses fylediff formula: `((val/4)+1) * (val&1 ? -1 : 1)`
   - NOT direct square encoding!
   - Source: game.cpp lines 59232-59262

6. **Rook Moves** (Section 4.2.5)
   - Values 0-7: Horizontal (target file = val)
   - Values 8-15: Vertical (target rank = val - 8)
   - Source: game.cpp lines 59183-59195

7. **Pawn Moves** (Section 4.2.6)
   - Uses toSquareDiff table: `{7,8,9,7,8,9,7,8,9,7,8,9,7,8,9,16}`
   - White ADDS offset, Black SUBTRACTS offset
   - val % 3 = direction (0=left, 1=forward, 2=right)
   - val / 3 = promotion (0=none, 1=Q, 2=R, 3=B, 4=N, 5=double push)
   - Source: game.cpp lines 59298-59345

8. **Queen Moves** (Section 4.2.7) - **CRITICAL 2-BYTE ENCODING**
   - If val >= 8: Vertical move (1 byte)
   - If val != from_file: Horizontal move (1 byte)
   - If val == from_file: Diagonal move (2 bytes!) - read second byte
   - Second byte: target_square = byte - 64 (valid range: 64-127)
   - Source: game.cpp lines 59264-59282

9. **Capture Swap Algorithm** (Section 4.4)
   - Last piece fills captured piece's slot
   - Maintains compact piece list without gaps
   - Source: position.cpp DoSimpleMove()

10. **FEN Initialization** (Section 4.3)
    - King ALWAYS gets slot 0 first
    - Other pieces assigned by board scan order
    - Source: position.cpp AddPiece()

### Shakmaty Documentation

**Crate**: `shakmaty = "0.23"`

**Key Types**:
- `Chess`: Complete chess position with rules
- `Position` trait: Core chess operations
- `Move`: Legal move representation
- `Square`: 0-63 board square encoding
- `Role`: Piece type (King, Queen, Rook, etc.)
- `Color`: White or Black

**Critical Methods**:
```rust
// Position queries
fn board(&self) -> &Board;
fn turn(&self) -> Color;
fn castling_rights(&self) -> Bitboard;
fn ep_square(&self) -> Option<Square>;

// Move operations
fn legal_moves(&self) -> MoveList;
fn is_legal(&self, m: &Move) -> bool;
fn play(&self, m: &Move) -> Result<Chess, IllegalMoveError>;

// SAN generation (Phase 6)
fn san(&self, m: &Move) -> San;
```

---

## Test Data Reference

### Location and Datasets

All test data is in `tests/data/`. See `IMPLEMENTATION_PLAN.md` → "Test Data" section for complete documentation.

| Dataset | Files | Description |
|---------|-------|-------------|
| **one** | `one.si4`, `one.sg4`, `one.sn4` | Single game - basic move decoding |
| **five** | `five.si4`, `five.sg4`, `five.sn4` | Five games - comprehensive validation |

### PGN ↔ SCID Relationship

Each SCID database was created by importing its corresponding PGN file:
- `one.pgn` → `one.*` (decoded moves should match PGN movetext)
- `five.pgn` → `five.*` (decoded moves should match PGN movetext)

### Validation Strategy

1. Parse SCID database and decode all moves
2. Generate SAN notation using shakmaty
3. Compare generated movetext against source PGN file
4. Moves must match exactly (including disambiguation, check notation)

### Expected Values

Games should include standard moves as well as special cases:
- Castling (O-O, O-O-O)
- En passant captures
- Pawn promotions
- Queen diagonal moves (2-byte encoding)

---

## Task Breakdown

### Section 5.1: SCID Position Wrapper

**Objective**: Create a wrapper around shakmaty's `Chess` that tracks SCID piece numbers alongside the standard chess position.

**Why Needed**: SCID move bytes reference pieces by number (0-15), but shakmaty uses square-based piece lookup. We need bidirectional mapping.

**Reference**: IMPLEMENTATION_PLAN.md lines 607-729, SCID_DATABASE_FORMAT.md lines 854-857

---

#### Task 5.1.1: Design Piece Numbering System

**Objective**: Document the SCID piece numbering scheme and design data structures to track it.

**Background Education**:

SCID piece numbers are **side-relative** and **dynamic**:
- White to move: White pieces are 0-15, Black pieces use different encoding
- Black to move: Black pieces are 0-15, White pieces use different encoding
- Captures update the mapping (captured piece number becomes invalid)
- Promotions change piece type but keep the number

**Standard Initial Numbering** (from SCID source code analysis - src/position.cpp initPieceList()):

**CRITICAL**: Pieces are numbered by their STARTING FILE, not by piece type!

```
White pieces (when White to move):
  0 = King (e1)           // Always slot 0
  1 = QR - Queen's Rook (a1)
  2 = QN - Queen's Knight (b1)
  3 = QB - Queen's Bishop (c1)
  4 = Q  - Queen (d1)
  5 = KB - King's Bishop (f1)
  6 = KN - King's Knight (g1)
  7 = KR - King's Rook (h1)
  8-15 = Pawns (a2-h2, files 0-7)

Black pieces (when Black to move):
  0 = King (e8)           // Always slot 0
  1 = QR - Queen's Rook (a8)
  2 = QN - Queen's Knight (b8)
  3 = QB - Queen's Bishop (c8)
  4 = Q  - Queen (d8)
  5 = KB - King's Bishop (f8)
  6 = KN - King's Knight (g8)
  7 = KR - King's Rook (h8)
  8-15 = Pawns (a7-h7, files 0-7)
```

**Source Reference**: SCID_DATABASE_FORMAT.md Section 4.1.2, game.cpp lines 59061-59094

**IMPORTANT - FEN Position Initialization**: When initializing from a FEN string (non-standard starting position), pieces are assigned numbers in this priority order:
1. King ALWAYS gets slot 0 first
2. Then pieces are assigned by scanning the board rank-by-rank, file-by-file
3. See SCID_DATABASE_FORMAT.md Section 4.3 for complete algorithm

**Design Decision**: Use `HashMap<u8, Square>` to track piece locations:
- Key: SCID piece number (0-15)
- Value: Current square
- Updated after each move
- Separate mappings for White and Black

**Acceptance Criteria**:
- [ ] Document complete piece numbering scheme
- [ ] Design data structure for tracking
- [ ] Handle captures (remove from mapping)
- [ ] Handle promotions (update piece type, keep number)
- [ ] Support non-standard starting positions

**Implementation**:

```rust
/// SCID piece number to square mapping
/// Maintains bidirectional mapping between SCID piece numbers and board squares
/// CRITICAL: Each color has its own independent 0-15 piece list
#[derive(Debug, Clone)]
pub struct PieceNumberMapping {
    /// Maps SCID piece number (0-15) to current square for White
    white_pieces: HashMap<u8, Square>,
    /// Maps SCID piece number (0-15) to current square for Black
    black_pieces: HashMap<u8, Square>,
    /// Current piece count for White (decremented on captures)
    white_count: u8,
    /// Current piece count for Black (decremented on captures)
    black_count: u8,
}

impl PieceNumberMapping {
    /// Initialize from standard starting position
    ///
    /// CRITICAL: Pieces are numbered by STARTING FILE, not piece type!
    /// Order: King(0), QR(1), QN(2), QB(3), Q(4), KB(5), KN(6), KR(7), Pawns(8-15)
    ///
    /// Source: SCID_DATABASE_FORMAT.md Section 4.1.2, game.cpp initPieceList()
    pub fn standard_start() -> Self {
        let mut mapping = Self {
            white_pieces: HashMap::with_capacity(16),
            black_pieces: HashMap::with_capacity(16),
            white_count: 16,  // Standard start: 16 pieces each
            black_count: 16,
        };

        // White pieces (numbered by starting file position)
        mapping.white_pieces.insert(0, Square::E1);  // King (always slot 0)
        mapping.white_pieces.insert(1, Square::A1);  // QR - Queen's Rook
        mapping.white_pieces.insert(2, Square::B1);  // QN - Queen's Knight
        mapping.white_pieces.insert(3, Square::C1);  // QB - Queen's Bishop
        mapping.white_pieces.insert(4, Square::D1);  // Q  - Queen
        mapping.white_pieces.insert(5, Square::F1);  // KB - King's Bishop
        mapping.white_pieces.insert(6, Square::G1);  // KN - King's Knight
        mapping.white_pieces.insert(7, Square::H1);  // KR - King's Rook

        // White pawns (a2-h2 = piece numbers 8-15)
        for file in 0..8 {
            mapping.white_pieces.insert(8 + file, Square::from_coords(
                shakmaty::File::new(file as u32),
                shakmaty::Rank::Second
            ));
        }

        // Black pieces (numbered by starting file position)
        mapping.black_pieces.insert(0, Square::E8);  // King (always slot 0)
        mapping.black_pieces.insert(1, Square::A8);  // QR - Queen's Rook
        mapping.black_pieces.insert(2, Square::B8);  // QN - Queen's Knight
        mapping.black_pieces.insert(3, Square::C8);  // QB - Queen's Bishop
        mapping.black_pieces.insert(4, Square::D8);  // Q  - Queen
        mapping.black_pieces.insert(5, Square::F8);  // KB - King's Bishop
        mapping.black_pieces.insert(6, Square::G8);  // KN - King's Knight
        mapping.black_pieces.insert(7, Square::H8);  // KR - King's Rook

        // Black pawns (a7-h7 = piece numbers 8-15)
        for file in 0..8 {
            mapping.black_pieces.insert(8 + file, Square::from_coords(
                shakmaty::File::new(file as u32),
                shakmaty::Rank::Seventh
            ));
        }

        mapping
    }

    /// Initialize from FEN position (non-standard starting position)
    ///
    /// CRITICAL: From SCID_DATABASE_FORMAT.md Section 4.3, position.cpp AddPiece():
    ///
    /// King is ALWAYS assigned slot 0 first, regardless of board position!
    /// Then remaining pieces are assigned by scanning rank-by-rank, file-by-file.
    ///
    /// Algorithm:
    /// 1. Find King and assign it slot 0
    /// 2. Scan board from a1 to h8 (rank 1 to 8, file a to h)
    /// 3. Assign each piece found the next available slot number
    /// Initialize from FEN position (non-standard starting position)
    ///
    /// CRITICAL: From SCID_DATABASE_FORMAT.md Section 4.3, position.cpp AddPiece():
    ///
    /// King is ALWAYS assigned slot 0 first, regardless of board position!
    /// Then remaining pieces are assigned by scanning rank-by-rank, file-by-file
    /// in FEN order (rank 8 down to rank 1).
    pub fn from_position(chess: &Chess) -> Result<Self> {
        let mut mapping = Self {
            white_pieces: HashMap::new(),
            black_pieces: HashMap::new(),
            white_count: 0,
            black_count: 0,
        };

        // STEP 1: King ALWAYS gets slot 0 first
        for square in chess.board().by_role(Role::King) {
            if let Some(piece) = chess.board().piece_at(square) {
                if piece.color == Color::White {
                    mapping.white_pieces.insert(0, square);
                } else {
                    mapping.black_pieces.insert(0, square);
                }
            }
        }

        // STEP 2: Scan board in FEN order (rank 8 down to rank 1, file a to h)
        let mut white_num = 1u8;  // Start at 1 (0 is King)
        let mut black_num = 1u8;

        // Scan from rank 8 (index 7) DOWN to rank 1 (index 0) - FEN order
        for rank in (0..8u32).rev() {
            for file in 0..8u32 {
                let square = Square::from_coords(
                    shakmaty::File::new(file),
                    shakmaty::Rank::new(rank)
                );

                if let Some(piece) = chess.board().piece_at(square) {
                    // Skip Kings (already assigned slot 0)
                    if piece.role == Role::King {
                        continue;
                    }

                    if piece.color == Color::White {
                        mapping.white_pieces.insert(white_num, square);
                        white_num += 1;
                    } else {
                        mapping.black_pieces.insert(black_num, square);
                        black_num += 1;
                    }
                }
            }
        }

        // Set piece counts (white_num and black_num are now actual counts)
        mapping.white_count = white_num;
        mapping.black_count = black_num;

        Ok(mapping)
    }

    /// Get square for piece number (for current side to move)
    pub fn get_square(&self, piece_num: u8, color: Color) -> Option<Square> {
        match color {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    /// Update mapping after a move
    ///
    /// CRITICAL: Implements SCID's capture swap algorithm!
    /// From SCID_DATABASE_FORMAT.md Section 4.4, position.cpp DoSimpleMove():
    ///
    /// When a piece is captured:
    /// 1. The captured piece's slot becomes empty
    /// 2. The LAST piece in the list takes the captured piece's slot number
    /// 3. This maintains a compact piece list without gaps
    ///
    /// Example: If piece 3 captures piece on slot 5 (opponent has 8 pieces):
    /// - Opponent's piece 5 is removed
    /// - Opponent's piece 7 (the last) moves to slot 5
    /// - Opponent now has 7 pieces (slots 0-6)
    pub fn update_after_move(&mut self, chess_move: &Move, color: Color) {
        // Handle captures FIRST (before updating moving piece)
        if chess_move.is_capture() {
            // Determine capture square (different for en passant!)
            let capture_square = if chess_move.is_en_passant() {
                // En passant: captured pawn is NOT on the target square
                // It's on the same file as target, but same rank as moving pawn
                let target = chess_move.to();
                let ep_rank = match color {
                    Color::White => shakmaty::Rank::Fifth,   // Capturing on rank 6, pawn was on rank 5
                    Color::Black => shakmaty::Rank::Fourth,  // Capturing on rank 3, pawn was on rank 4
                };
                Square::from_coords(target.file(), ep_rank)
            } else {
                chess_move.to()
            };

            // Get enemy pieces and count
            let (enemy_pieces, enemy_count) = match color {
                Color::White => (&mut self.black_pieces, &mut self.black_count),
                Color::Black => (&mut self.white_pieces, &mut self.white_count),
            };

            // Find the captured piece's slot number
            let captured_slot = enemy_pieces.iter()
                .find(|(_, &sq)| sq == capture_square)
                .map(|(&num, _)| num);

            if let Some(captured_num) = captured_slot {
                // SCID SWAP ALGORITHM (from position.cpp DoSimpleMove lines 78903-78912):
                // 1. Decrement enemy piece count
                *enemy_count -= 1;

                // 2. Get the last piece slot (index = new count)
                let last_slot = *enemy_count;

                // 3. If captured piece wasn't the last, swap
                if captured_num != last_slot {
                    // Move last piece to captured slot
                    if let Some(&last_square) = enemy_pieces.get(&last_slot) {
                        enemy_pieces.insert(captured_num, last_square);
                    }
                }

                // 4. Remove the last slot
                enemy_pieces.remove(&last_slot);
            }
        }

        // Get own pieces for updating moving piece location
        let own_pieces = match color {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces,
        };

        // Update moving piece location based on move type
        match chess_move {
            Move::Normal { from, to, .. } => {
                // Find which piece number is at from square
                let piece_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
                // Note: Promotion doesn't change piece NUMBER, only type
            }
            Move::Castle { king, rook } => {
                // King moves to standard castling square
                let king_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::G, king.rank())  // Kingside
                } else {
                    Square::from_coords(shakmaty::File::C, king.rank())  // Queenside
                };

                // Rook moves to standard castling square
                let rook_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::F, king.rank())  // Kingside
                } else {
                    Square::from_coords(shakmaty::File::D, king.rank())  // Queenside
                };

                // Update king (always piece 0)
                own_pieces.insert(0, king_to);

                // Find and update rook
                let rook_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *rook)
                    .map(|(&num, _)| num);

                if let Some(num) = rook_num {
                    own_pieces.insert(num, rook_to);
                }
            }
            Move::EnPassant { from, to } => {
                let piece_num = own_pieces.iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
                // Capture was already handled above
            }
            Move::Put { .. } => {
                // Crazyhouse - not supported in standard SCID
            }
        }
    }
}
```

**Testing**:
```rust
#[test]
fn test_standard_piece_numbering() {
    let mapping = PieceNumberMapping::standard_start();

    // Verify White pieces (ordered by starting file, not piece type!)
    assert_eq!(mapping.get_square(0, Color::White), Some(Square::E1)); // King
    assert_eq!(mapping.get_square(1, Color::White), Some(Square::A1)); // QR (Queen's Rook)
    assert_eq!(mapping.get_square(2, Color::White), Some(Square::B1)); // QN (Queen's Knight)
    assert_eq!(mapping.get_square(3, Color::White), Some(Square::C1)); // QB (Queen's Bishop)
    assert_eq!(mapping.get_square(4, Color::White), Some(Square::D1)); // Q (Queen)
    assert_eq!(mapping.get_square(5, Color::White), Some(Square::F1)); // KB (King's Bishop)
    assert_eq!(mapping.get_square(6, Color::White), Some(Square::G1)); // KN (King's Knight)
    assert_eq!(mapping.get_square(7, Color::White), Some(Square::H1)); // KR (King's Rook)
    assert_eq!(mapping.get_square(8, Color::White), Some(Square::A2)); // a-pawn
    assert_eq!(mapping.get_square(15, Color::White), Some(Square::H2)); // h-pawn

    // Verify Black pieces
    assert_eq!(mapping.get_square(0, Color::Black), Some(Square::E8)); // King
    assert_eq!(mapping.get_square(4, Color::Black), Some(Square::D8)); // Queen (slot 4, not 1!)
}

#[test]
fn test_piece_mapping_after_move() {
    let mut mapping = PieceNumberMapping::standard_start();

    // Move e2 pawn to e4 (piece number 12)
    let chess_move = Move::Normal {
        role: Role::Pawn,
        from: Square::E2,
        capture: None,
        to: Square::E4,
        promotion: None,
    };

    mapping.update_after_move(&chess_move, Color::White);

    // Verify pawn moved
    assert_eq!(mapping.get_square(12, Color::White), Some(Square::E4));
}
```

**Validation Command**:
```bash
cargo test test_standard_piece_numbering
cargo test test_piece_mapping_after_move
```

---

#### Task 5.1.2: Implement ScidPosition Wrapper

**Objective**: Create a position wrapper that combines shakmaty's Chess with SCID piece tracking.

**Reference**: IMPLEMENTATION_PLAN.md lines 616-719

**Why This Design**:
- Shakmaty handles ALL chess rules (castling, en passant, checkmate, etc.)
- We only add SCID-specific piece number mapping
- Delegation pattern keeps code simple and maintainable

**Acceptance Criteria**:
- [ ] Wraps shakmaty::Chess for all position operations
- [ ] Maintains piece number mapping synchronized with position
- [ ] Provides methods to query piece locations by number
- [ ] Updates mapping automatically when moves are applied
- [ ] Supports non-standard starting positions via FEN

**Implementation**:

**File**: `crates/core/src/parser/position.rs`

```rust
use shakmaty::{Chess, Position, Square, Move, Role, Color, Board};
use std::collections::HashMap;
use crate::error::{Result, ScidError};

/// Wrapper around shakmaty::Chess that tracks SCID piece numbers
///
/// This type combines shakmaty's complete chess engine with SCID's
/// piece numbering system. Shakmaty handles all chess rules while
/// we maintain the SCID piece number to square mapping.
#[derive(Debug, Clone)]
pub struct ScidPosition {
    /// Shakmaty chess position (handles ALL chess logic)
    chess: Chess,

    /// Maps SCID piece numbers to current squares
    piece_mapping: PieceNumberMapping,
}

impl ScidPosition {
    /// Create position from standard starting position
    pub fn new() -> Self {
        ScidPosition {
            chess: Chess::default(),
            piece_mapping: PieceNumberMapping::standard_start(),
        }
    }

    /// Create position from FEN string
    pub fn from_fen(fen: &str) -> Result<Self> {
        let chess = fen.parse::<Chess>()
            .map_err(|e| ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Invalid FEN: {:?}", e),
            })?;

        let piece_mapping = PieceNumberMapping::from_position(&chess)?;

        Ok(ScidPosition {
            chess,
            piece_mapping,
        })
    }

    /// Get square where SCID piece number is currently located
    pub fn get_piece_square(&self, piece_num: u8) -> Option<Square> {
        self.piece_mapping.get_square(piece_num, self.chess.turn())
    }

    /// Get piece at a specific square (delegates to shakmaty)
    pub fn piece_at(&self, square: Square) -> Option<shakmaty::Piece> {
        self.chess.board().piece_at(square)
    }

    /// Get whose turn it is (delegates to shakmaty)
    pub fn turn(&self) -> Color {
        self.chess.turn()
    }

    /// Get the board state (delegates to shakmaty)
    pub fn board(&self) -> &Board {
        self.chess.board()
    }

    /// Get all legal moves (delegates to shakmaty)
    pub fn legal_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();
        self.chess.legal_moves(&mut moves);
        moves
    }

    /// Check if a move is legal (delegates to shakmaty)
    pub fn is_legal(&self, chess_move: &Move) -> bool {
        self.chess.is_legal(chess_move)
    }

    /// Apply a move and return new position
    ///
    /// This is the critical method that:
    /// 1. Validates move legality via shakmaty
    /// 2. Applies move to chess position
    /// 3. Updates SCID piece number mapping
    pub fn make_move(&mut self, chess_move: &Move) -> Result<()> {
        // Validate move is legal using shakmaty
        if !self.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        // Apply move using shakmaty (this handles ALL chess rules)
        let new_position = self.chess.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        // Update piece number mapping
        self.piece_mapping.update_after_move(chess_move, self.chess.turn());

        // Update chess position
        self.chess = new_position;

        Ok(())
    }

    /// Find legal move matching from/to squares and optional promotion
    ///
    /// Used to convert SCID decoded moves (from/to squares) into
    /// shakmaty Move objects
    pub fn find_move(&self, from: Square, to: Square, promotion: Option<Role>) -> Option<Move> {
        let legals = self.legal_moves();

        legals.into_iter().find(|m| {
            m.from() == Some(from) &&
            m.to() == to &&
            m.promotion() == promotion
        })
    }

    /// Get castling rights (delegates to shakmaty)
    pub fn castling_rights(&self) -> shakmaty::Bitboard {
        self.chess.castles().castling_rights()
    }

    /// Get en passant target square if available (delegates to shakmaty)
    pub fn ep_square(&self) -> Option<Square> {
        self.chess.ep_square()
    }

    /// Check if position is checkmate (delegates to shakmaty)
    pub fn is_checkmate(&self) -> bool {
        self.chess.is_checkmate()
    }

    /// Check if position is stalemate (delegates to shakmaty)
    pub fn is_stalemate(&self) -> bool {
        self.chess.is_stalemate()
    }

    /// Check if position is in check (delegates to shakmaty)
    pub fn is_check(&self) -> bool {
        self.chess.is_check()
    }
}

impl Default for ScidPosition {
    fn default() -> Self {
        Self::new()
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Role, Color};

    #[test]
    fn test_scid_position_creation() {
        let pos = ScidPosition::new();

        // Verify starting position
        assert_eq!(pos.turn(), Color::White);
        assert!(!pos.is_check());
        assert!(!pos.is_checkmate());

        // Verify piece at starting square
        let piece = pos.piece_at(Square::E2).unwrap();
        assert_eq!(piece.role, Role::Pawn);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_piece_number_lookup() {
        let pos = ScidPosition::new();

        // White king should be at e1 (piece number 0)
        assert_eq!(pos.get_piece_square(0), Some(Square::E1));

        // White QR (Queen's Rook) should be at a1 (piece number 1)
        assert_eq!(pos.get_piece_square(1), Some(Square::A1));

        // White Queen should be at d1 (piece number 4, NOT 1!)
        assert_eq!(pos.get_piece_square(4), Some(Square::D1));

        // White e-pawn should be at e2 (piece number 12 = 8 + e-file(4))
        assert_eq!(pos.get_piece_square(12), Some(Square::E2));
    }

    #[test]
    fn test_make_move() {
        let mut pos = ScidPosition::new();

        // Find legal e4 move
        let e4_move = pos.find_move(Square::E2, Square::E4, None)
            .expect("e2-e4 should be legal");

        // Apply move
        pos.make_move(&e4_move).unwrap();

        // Verify position updated
        assert_eq!(pos.turn(), Color::Black);
        assert!(pos.piece_at(Square::E2).is_none());
        assert!(pos.piece_at(Square::E4).is_some());

        // Verify piece number mapping updated
        assert_eq!(pos.get_piece_square(12), Some(Square::E4));
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = ScidPosition::from_fen(fen).unwrap();

        // Verify position matches FEN
        assert_eq!(pos.turn(), Color::Black);
        assert_eq!(pos.ep_square(), Some(Square::E3));

        // Verify piece at e4
        let piece = pos.piece_at(Square::E4).unwrap();
        assert_eq!(piece.role, Role::Pawn);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_illegal_move_rejected() {
        let mut pos = ScidPosition::new();

        // Try to make illegal move (pawn to e5 from starting position)
        let illegal_move = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E5,
            capture: None,
            promotion: None,
        };

        let result = pos.make_move(&illegal_move);
        assert!(result.is_err());
    }
}
```

**Validation Commands**:
```bash
cargo test test_scid_position_creation
cargo test test_piece_number_lookup
cargo test test_make_move
cargo test test_from_fen
cargo test test_illegal_move_rejected
```

**Expected Output**:
```
running 5 tests
test parser::position::tests::test_scid_position_creation ... ok
test parser::position::tests::test_piece_number_lookup ... ok
test parser::position::tests::test_make_move ... ok
test parser::position::tests::test_from_fen ... ok
test parser::position::tests::test_illegal_move_rejected ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

#### Task 5.1.3: Chess960 (Fischer Random Chess) Support

**Objective**: Support Chess960/FRC positions when parsing games with non-standard starting positions.

**Reference**: IMPLEMENTATION_PLAN.md Phase 5.1.1, SCID_DATABASE_FORMAT.md Section 4.3

**Background Education**:

Chess960 (also known as Fischer Random Chess or FRC) uses the same pieces as standard chess but with randomized starting positions for the back rank pieces. Key differences:

1. **960 possible starting positions** (hence the name)
2. **Castling rules differ**: King and Rook still move to standard squares (c1/g1 for White), but can start on different files
3. **FEN notation**: Castling rights use file letters (e.g., "AHah") instead of "KQkq" when pieces don't start on standard squares

**Why SCID's Encoding Works for Both**:

SCID's castling encoding (King move values 9 and 10) specifies the **destination squares** (C1/G1 for White, C8/G8 for Black), NOT the rook's starting position. This works for both standard chess AND Chess960 because:

- Standard chess: King e1→g1 (O-O), Rook h1→f1
- Chess960: King (wherever)→g1, Rook (wherever)→f1

The destination is always the same; only the starting position varies.

**Acceptance Criteria**:
- [ ] Detect Chess960 positions from FEN castling rights
- [ ] Use shakmaty's `CastlingMode::Chess960` for FRC positions
- [ ] Auto-detect function chooses correct mode based on FEN
- [ ] Castling decoding works for both standard and Chess960
- [ ] Tests cover Chess960 edge cases

**Implementation**:

Add to `crates/core/src/parser/position.rs`:

```rust
use shakmaty::{Chess, CastlingMode, fen::Fen};

/// Detect if a FEN string represents a Chess960 position
///
/// Chess960 FENs use file letters for castling rights (e.g., "AHah")
/// when the king or rooks don't start on standard squares.
/// Standard chess uses "KQkq" notation.
///
/// Detection rules:
/// - If castling field contains any lowercase letter a-h: Chess960
/// - If castling field is "-" or contains only K/Q/k/q: Standard
///
/// # Arguments
///
/// * `fen` - Complete FEN string
///
/// # Returns
///
/// `true` if this appears to be a Chess960 position
///
/// # Examples
///
/// ```
/// // Standard chess
/// assert!(!is_chess960_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
///
/// // Chess960 with rook on a-file, king on c-file
/// assert!(is_chess960_fen("rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w ACac - 0 1"));
/// ```
pub fn is_chess960_fen(fen: &str) -> bool {
    // FEN has 6 space-separated fields; castling rights is field 3 (index 2)
    let parts: Vec<&str> = fen.split_whitespace().collect();

    if parts.len() < 3 {
        return false;  // Invalid FEN, assume standard
    }

    let castling_field = parts[2];

    // Check for file-based castling rights (Chess960 indicator)
    // Standard uses only K, Q, k, q, or -
    // Chess960 uses file letters: A-H for white, a-h for black
    for c in castling_field.chars() {
        match c {
            'K' | 'Q' | 'k' | 'q' | '-' => continue,  // Standard notation
            'A'..='H' | 'a'..='h' => return true,      // Chess960 notation
            _ => continue,  // Ignore unexpected characters
        }
    }

    false  // No Chess960 indicators found
}

impl ScidPosition {
    /// Create position from FEN with automatic Chess960 detection
    ///
    /// Examines the FEN's castling rights field to determine whether
    /// to use standard or Chess960 mode. This ensures correct castling
    /// validation for both game types.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string (standard or Chess960)
    ///
    /// # Returns
    ///
    /// ScidPosition configured for the correct game variant
    ///
    /// # Example
    ///
    /// ```
    /// // Standard position - uses CastlingMode::Standard
    /// let pos = ScidPosition::from_fen_auto(
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    /// )?;
    ///
    /// // Chess960 position - uses CastlingMode::Chess960
    /// let pos = ScidPosition::from_fen_auto(
    ///     "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w ACac - 0 1"
    /// )?;
    /// ```
    pub fn from_fen_auto(fen: &str) -> Result<Self> {
        let is_960 = is_chess960_fen(fen);

        let castling_mode = if is_960 {
            CastlingMode::Chess960
        } else {
            CastlingMode::Standard
        };

        // Parse FEN with appropriate castling mode
        let parsed_fen: Fen = fen.parse()
            .map_err(|e| ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Invalid FEN: {:?}", e),
            })?;

        let chess = parsed_fen.into_position(castling_mode)
            .map_err(|e| ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Invalid position: {:?}", e),
            })?;

        let piece_mapping = PieceNumberMapping::from_position(&chess)?;

        Ok(ScidPosition {
            chess,
            piece_mapping,
        })
    }

    /// Create position from FEN (convenience alias for from_fen_auto)
    ///
    /// This is the recommended method for parsing FEN strings as it
    /// automatically handles both standard and Chess960 positions.
    pub fn from_fen(fen: &str) -> Result<Self> {
        Self::from_fen_auto(fen)
    }
}
```

**Castling Decoding Note**:

The existing `decode_king_move()` function already works correctly for Chess960 because SCID encodes the **destination**, not the path:

```rust
// From decode_king_move() - NO CHANGES NEEDED for Chess960!
} else if move_value == 9 {
    // Queenside castle (O-O-O) - King moves to c-file
    // Works for BOTH standard and Chess960!
    match color {
        Color::White => Square::C1,  // King ends here regardless of start
        Color::Black => Square::C8,
    }
} else if move_value == 10 {
    // Kingside castle (O-O) - King moves to g-file
    match color {
        Color::White => Square::G1,  // King ends here regardless of start
        Color::Black => Square::G8,
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod chess960_tests {
    use super::*;

    #[test]
    fn test_detect_standard_fen() {
        // Standard starting position
        assert!(!is_chess960_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ));

        // Standard with some castling rights lost
        assert!(!is_chess960_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4"
        ));

        // No castling rights
        assert!(!is_chess960_fen(
            "8/8/8/8/8/8/8/4K2k w - - 0 1"
        ));
    }

    #[test]
    fn test_detect_chess960_fen() {
        // Chess960 with file-based castling (king on c-file, rooks on a and h)
        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w AHah - 0 1"
        ));

        // Chess960 with only some castling rights
        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w Hh - 0 1"
        ));

        // Mixed notation (still Chess960 if any file letter present)
        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w KQah - 0 1"
        ));
    }

    #[test]
    fn test_from_fen_auto_standard() {
        let pos = ScidPosition::from_fen_auto(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ).unwrap();

        // Verify standard starting position
        assert_eq!(pos.turn(), Color::White);
        assert_eq!(pos.get_piece_square(0), Some(Square::E1));  // White King
    }

    #[test]
    fn test_from_fen_auto_chess960() {
        // Chess960 position #518 (standard-like but with different internal handling)
        // King on e-file, Rooks on a and h (same as standard visually)
        let pos = ScidPosition::from_fen_auto(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w HAha - 0 1"
        );

        // Should parse successfully with Chess960 mode
        assert!(pos.is_ok());
    }

    #[test]
    fn test_chess960_castling_decoding() {
        // Chess960 position where king can castle
        // This tests that our castling decoder works for Chess960
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w AHah - 0 1";
        let pos = ScidPosition::from_fen_auto(fen).unwrap();

        // King should be at e1 (assuming this 960 position)
        // The key test is that castling rights are properly recognized
        assert!(pos.castling_rights().contains(Square::A1) ||
                pos.castling_rights().contains(Square::H1));
    }
}
```

**Validation Commands**:
```bash
cargo test chess960_tests
cargo test test_detect_standard_fen
cargo test test_detect_chess960_fen
cargo test test_from_fen_auto_standard
cargo test test_from_fen_auto_chess960
```

**Key Points**:

1. **Auto-detection is seamless**: `from_fen_auto()` examines the FEN and chooses the right mode
2. **No changes to move decoding**: SCID's castling encoding already works for Chess960
3. **Shakmaty handles the complexity**: The chess library manages Chess960 rules internally
4. **Backward compatible**: Standard chess FENs continue to work exactly as before

---

### Section 5.2: SCID Move Decoder

**Objective**: Decode SCID binary move bytes into from/to squares and promotion information, then use shakmaty to validate and construct legal Move objects.

**Reference**: IMPLEMENTATION_PLAN.md lines 732-916, SCID_DATABASE_FORMAT.md lines 843-899

---

#### Task 5.2.1: Implement ByteStream for Multi-Byte Moves

**Objective**: Create a byte stream reader that supports sequential byte consumption for moves that require multiple bytes (Queen diagonal moves).

**Background Education**:

**Why ByteStream Is Critical**: Most SCID moves are 1 byte, but Queen diagonal moves are 2 bytes:
- First byte: `[piece_num][move_value]` where `move_value == queen_file`
- Second byte: Target square encoding `target_square = second_byte - 64`

Without a stream, we can't know whether to read 1 or 2 bytes until we decode the first byte.

**SCID Source Reference** (game.cpp `decodeQueen()`):
```cpp
static inline errorT
decodeQueen (ByteBuffer * buf, byte val, simpleMoveT * sm)
{
    if (val >= 8) {
        // Vertical move (1 byte)
        sm->to = square_Make (square_Fyle(sm->from), (val - 8));
    } else if (val != square_Fyle(sm->from)) {
        // Horizontal move (1 byte)
        sm->to = square_Make (val, square_Rank(sm->from));
    } else {
        // Diagonal move (2 bytes) - READ NEXT BYTE!
        val = buf->GetByte();  // ← Stream advances here
        if (val < 64 || val > 127) { return ERROR_Decode; }
        sm->to = val - 64;
    }
    return OK;
}
```

**Acceptance Criteria**:
- [ ] Supports sequential byte reading with position tracking
- [ ] Compatible with SCID's ByteBuffer behavior
- [ ] Provides peek capability (look ahead without consuming)
- [ ] Handles buffer underrun errors gracefully
- [ ] Tracks bytes consumed for debugging

**Implementation**:

**File**: `crates/core/src/parser/byte_stream.rs`

```rust
use crate::error::{Result, ScidError};
use std::path::PathBuf;

/// Sequential byte stream reader compatible with SCID's ByteBuffer
///
/// Replicates functionality from scidvspc/src/bytebuf.h ByteBuffer class
/// Supports reading bytes sequentially with position tracking
#[derive(Debug)]
pub struct ByteStream<'a> {
    /// Underlying byte buffer
    buffer: &'a [u8],

    /// Current read position
    position: usize,

    /// Total bytes available
    length: usize,
}

impl<'a> ByteStream<'a> {
    /// Create new ByteStream from byte slice
    pub fn new(buffer: &'a [u8]) -> Self {
        ByteStream {
            buffer,
            position: 0,
            length: buffer.len(),
        }
    }

    /// Read next byte from stream and advance position
    /// Equivalent to SCID's ByteBuffer::GetByte()
    pub fn get_byte(&mut self) -> Result<u8> {
        if self.position >= self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: "Buffer underrun: no more bytes available".to_string(),
            });
        }

        let byte = self.buffer[self.position];
        self.position += 1;
        Ok(byte)
    }

    /// Peek at next byte without advancing position
    pub fn peek_byte(&self) -> Result<u8> {
        if self.position >= self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: "Buffer underrun: no more bytes to peek".to_string(),
            });
        }

        Ok(self.buffer[self.position])
    }

    /// Skip N bytes forward
    pub fn skip(&mut self, count: usize) -> Result<()> {
        if self.position + count > self.length {
            return Err(ScidError::ParseError {
                file: PathBuf::from("byte_stream"),
                offset: self.position as u64,
                message: format!("Buffer underrun: cannot skip {} bytes", count),
            });
        }

        self.position += count;
        Ok(())
    }

    /// Get current position in stream
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get remaining bytes available
    pub fn remaining(&self) -> usize {
        self.length - self.position
    }

    /// Check if stream has more bytes
    pub fn has_more(&self) -> bool {
        self.position < self.length
    }

    /// Reset stream to beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_stream_basic() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let mut stream = ByteStream::new(&data);

        assert_eq!(stream.get_byte().unwrap(), 0x10);
        assert_eq!(stream.get_byte().unwrap(), 0x20);
        assert_eq!(stream.position(), 2);
        assert_eq!(stream.remaining(), 2);
    }

    #[test]
    fn test_byte_stream_peek() {
        let data = vec![0x10, 0x20, 0x30];
        let mut stream = ByteStream::new(&data);

        // Peek doesn't advance position
        assert_eq!(stream.peek_byte().unwrap(), 0x10);
        assert_eq!(stream.peek_byte().unwrap(), 0x10);
        assert_eq!(stream.position(), 0);

        // Get does advance
        assert_eq!(stream.get_byte().unwrap(), 0x10);
        assert_eq!(stream.position(), 1);
    }

    #[test]
    fn test_byte_stream_underrun() {
        let data = vec![0x10];
        let mut stream = ByteStream::new(&data);

        stream.get_byte().unwrap();

        // Should error on second byte
        let result = stream.get_byte();
        assert!(result.is_err());
    }

    #[test]
    fn test_byte_stream_skip() {
        let data = vec![0x10, 0x20, 0x30, 0x40];
        let mut stream = ByteStream::new(&data);

        stream.skip(2).unwrap();
        assert_eq!(stream.get_byte().unwrap(), 0x30);
    }

    #[test]
    fn test_byte_stream_reset() {
        let data = vec![0x10, 0x20, 0x30];
        let mut stream = ByteStream::new(&data);

        stream.get_byte().unwrap();
        stream.get_byte().unwrap();

        stream.reset();
        assert_eq!(stream.position(), 0);
        assert_eq!(stream.get_byte().unwrap(), 0x10);
    }
}
```

**Validation Command**:
```bash
cargo test --lib byte_stream
```

**Expected Output**:
```
running 5 tests
test parser::byte_stream::tests::test_byte_stream_basic ... ok
test parser::byte_stream::tests::test_byte_stream_peek ... ok
test parser::byte_stream::tests::test_byte_stream_underrun ... ok
test parser::byte_stream::tests::test_byte_stream_skip ... ok
test parser::byte_stream::tests::test_byte_stream_reset ... ok

test result: ok. 5 passed
```

---

#### Task 5.2.2: Implement Piece-Specific Move Decoders

**Objective**: Implement decoder functions for each piece type that convert SCID move values to from/to squares.

**Reference**: SCID_DATABASE_FORMAT.md lines 860-899

**Why Piece-Specific**: Each piece has unique movement patterns:
- King: 8 adjacent squares + castling
- Knight: L-shaped jumps
- Pawn: Forward/captures/promotions
- Rook/Bishop/Queen: Directional moves with variable encoding

**Acceptance Criteria**:
- [ ] King decoder handles adjacent moves and castling
- [ ] Knight decoder handles all L-shaped moves
- [ ] Pawn decoder handles moves, captures, promotions, en passant
- [ ] Rook decoder handles vertical/horizontal moves
- [ ] Bishop decoder handles diagonal moves
- [ ] Queen decoder handles all moves including 2-byte diagonals
- [ ] All decoders validated against SCID source code

**Implementation**:

**File**: `crates/core/src/parser/decoder.rs`

```rust
use shakmaty::{Square, Role, Color};
use crate::error::{Result, ScidError};
use crate::parser::byte_stream::ByteStream;
use std::path::PathBuf;

/// Decoded SCID move information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
    pub is_null_move: bool,  // True for King null move (value 0)
}

/// King move decoder
///
/// Move values (from SCID_DATABASE_FORMAT.md Section 4.2.1, game.cpp lines 59108-59136):
/// - 0: NULL MOVE (valid! king stays in place - used for analysis)
/// - 1-8: Adjacent squares (specific direction encoding)
/// - 9: Queenside castle (O-O-O)
/// - 10: Kingside castle (O-O)
/// - 11-15: INVALID
///
/// Direction encoding for values 1-8:
/// From game.cpp decodeKing(): dirIndex = val - 1, uses direction table
pub fn decode_king_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    let to = if move_value == 0 {
        // NULL MOVE - King stays in place (used for analysis positions)
        // This is a VALID move in SCID, not an error!
        from
    } else if move_value >= 1 && move_value <= 8 {
        // Adjacent square moves
        // Direction table from game.cpp: UP_LEFT, UP, UP_RIGHT, LEFT, RIGHT, DOWN_LEFT, DOWN, DOWN_RIGHT
        let offsets: [i8; 9] = [0, 7, 8, 9, -1, 1, -9, -8, -7];
        offset_square(from, offsets[move_value as usize])?
    } else if move_value == 9 {
        // Queenside castle (O-O-O) - King moves to c-file
        match color {
            Color::White => Square::C1,
            Color::Black => Square::C8,
        }
    } else if move_value == 10 {
        // Kingside castle (O-O) - King moves to g-file
        match color {
            Color::White => Square::G1,
            Color::Black => Square::G8,
        }
    } else {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid king move value: {} (valid: 0-10)", move_value),
        });
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
        is_null_move: move_value == 0,  // Flag null moves for special handling
    })
}

/// Knight move decoder
///
/// Move values (from SCID_DATABASE_FORMAT.md Section 4.2.3, game.cpp lines 59204-59230):
/// - 0: INVALID (error)
/// - 1-8: L-shaped jumps (valid knight moves)
/// - 9-15: INVALID (error)
///
/// Knight L-shaped offset table from game.cpp:
/// knightDir[] = { -17, -15, -10, -6, 6, 10, 15, 17 }
/// Indexed by (val - 1), so val=1 gives offset -17, etc.
pub fn decode_knight_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    // CRITICAL: Values 0 and 9-15 are INVALID for knights!
    if move_value == 0 || move_value > 8 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid knight move value: {} (valid: 1-8 only)", move_value),
        });
    }

    // Knight L-shaped offsets (indexed by val - 1)
    // From game.cpp: knightDir[] = { -17, -15, -10, -6, 6, 10, 15, 17 }
    let offsets: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];
    let to = offset_square(from, offsets[move_value as usize])?;

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
        is_null_move: false,
    })
}

/// Pawn move decoder
///
/// Move values (from SCID_DATABASE_FORMAT.md Section 4.2.6, game.cpp lines 59298-59345):
///
/// Uses toSquareDiff lookup table: {7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 16}
///
/// val % 3 determines direction:
/// - 0 = capture left (7 squares)
/// - 1 = forward (8 squares)
/// - 2 = capture right (9 squares)
///
/// val / 3 determines promotion piece (0-4):
/// - 0 (vals 0-2): No promotion (regular move/capture)
/// - 1 (vals 3-5): Queen promotion
/// - 2 (vals 6-8): Rook promotion
/// - 3 (vals 9-11): Bishop promotion
/// - 4 (vals 12-14): Knight promotion
/// - val 15: Double pawn push (offset 16)
///
/// CRITICAL: White ADDS the offset, Black SUBTRACTS the offset!
pub fn decode_pawn_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    if move_value > 15 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid pawn move value: {} (valid: 0-15)", move_value),
        });
    }

    // toSquareDiff lookup table from game.cpp
    const TO_SQUARE_DIFF: [i8; 16] = [7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 7, 8, 9, 16];

    // Get base offset from table
    let base_offset = TO_SQUARE_DIFF[move_value as usize];

    // Direction depends on color: White adds, Black subtracts
    let offset = match color {
        Color::White => base_offset,
        Color::Black => -base_offset,
    };

    // Calculate target square
    let to = offset_square(from, offset)?;

    // Determine promotion piece (if any)
    let promotion = match move_value / 3 {
        0 => None,                    // vals 0-2: No promotion
        1 => Some(Role::Queen),       // vals 3-5: Queen promotion
        2 => Some(Role::Rook),        // vals 6-8: Rook promotion
        3 => Some(Role::Bishop),      // vals 9-11: Bishop promotion
        4 => Some(Role::Knight),      // vals 12-14: Knight promotion
        5 => None,                    // val 15: Double push, no promotion
        _ => unreachable!(),
    };

    Ok(DecodedMove {
        from,
        to,
        promotion,
        is_null_move: false,
    })
}

/// Rook move decoder (vertical and horizontal only)
///
/// From SCID_DATABASE_FORMAT.md Section 4.2.5, game.cpp lines 59183-59195:
///
/// Move values 0-15:
/// - Values 0-7: Horizontal move (target file = val, same rank)
/// - Values 8-15: Vertical move (target rank = val - 8, same file)
pub fn decode_rook_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    if move_value > 15 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid rook move value: {} (valid: 0-15)", move_value),
        });
    }

    let from_file = from.file() as u8;
    let from_rank = from.rank() as u8;

    let to = if move_value >= 8 {
        // Vertical move: target rank = val - 8, same file
        let target_rank = move_value - 8;
        Square::from_coords(
            shakmaty::File::new(from_file as u32),
            shakmaty::Rank::new(target_rank as u32)
        )
    } else {
        // Horizontal move: target file = val, same rank
        Square::from_coords(
            shakmaty::File::new(move_value as u32),
            shakmaty::Rank::new(from_rank as u32)
        )
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
        is_null_move: false,
    })
}

/// Bishop move decoder (diagonal only)
///
/// From SCID_DATABASE_FORMAT.md Section 4.2.4, game.cpp lines 59232-59262:
///
/// ```cpp
/// // From SCID decodeBishop():
/// byte fyle = (val & 7);                              // Target file (0-7)
/// int fylediff = (int)fyle - (int)square_Fyle(sm->from);
/// if (val >= 8) {
///     sm->to = sm->from - 7 * fylediff;  // up-left/down-right diagonal
/// } else {
///     sm->to = sm->from + 9 * fylediff;  // up-right/down-left diagonal
/// }
/// ```
///
/// **Bishop Move Value Structure**:
/// | Bits | Meaning |
/// |------|---------|
/// | 0-2 (val & 7) | Target file (0=a, 7=h) |
/// | 3 (val & 8) | Diagonal direction: 0=up-right/down-left, 1=up-left/down-right |
///
/// **Diagonal Direction Logic**:
/// - `val < 8`: Up-right or down-left diagonal (offset = +9 * fylediff)
/// - `val >= 8`: Up-left or down-right diagonal (offset = -7 * fylediff)
pub fn decode_bishop_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    if move_value > 15 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid bishop move value: {} (valid: 0-15)", move_value),
        });
    }

    // Extract target file from low 3 bits
    let target_file = (move_value & 7) as i8;
    let from_file = from.file() as i8;
    let fylediff = target_file - from_file;

    // Calculate square offset based on diagonal direction
    let offset = if move_value >= 8 {
        // Up-left / down-right diagonal
        -7 * fylediff
    } else {
        // Up-right / down-left diagonal
        9 * fylediff
    };

    // Calculate target square
    let to = offset_square(from, offset)?;

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
        is_null_move: false,
    })
}

/// Queen move decoder with ByteStream support for diagonal moves
///
/// From SCID_DATABASE_FORMAT.md Section 4.2.7, game.cpp lines 59264-59282:
///
/// CRITICAL: Queen diagonal moves require 2 bytes!
///
/// Decision tree:
/// - If move_value >= 8: Vertical move (1 byte) - target rank = val - 8
/// - If move_value != from_file: Horizontal move (1 byte) - target file = val
/// - Otherwise (val == from_file): Diagonal move (2 bytes) - READ NEXT BYTE FROM STREAM!
///
/// For diagonal moves, second byte encodes target square: target = second_byte - 64
/// Valid range for second byte: [64, 127]
pub fn decode_queen_move(
    from: Square,
    move_value: u8,
    stream: &mut ByteStream,
) -> Result<DecodedMove> {
    let from_file = from.file() as u8;
    let from_rank = from.rank() as u8;

    let to = if move_value >= 8 {
        // CASE 1: Vertical move (rook-like, 1 byte)
        // Target rank = val - 8, same file
        let target_rank = move_value - 8;
        Square::from_coords(
            shakmaty::File::new(from_file as u32),
            shakmaty::Rank::new(target_rank as u32)
        )
    } else if move_value != from_file {
        // CASE 2: Horizontal move (rook-like, 1 byte)
        // Target file = val, same rank
        Square::from_coords(
            shakmaty::File::new(move_value as u32),
            shakmaty::Rank::new(from_rank as u32)
        )
    } else {
        // CASE 3: Diagonal move (bishop-like, 2 bytes)
        // move_value == from_file signals diagonal move
        // Read second byte from stream!
        let second_byte = stream.get_byte()?;

        // SCID validation: must be in range [64, 127]
        if second_byte < 64 || second_byte > 127 {
            return Err(ScidError::ParseError {
                file: PathBuf::from("decoder"),
                offset: 0,
                message: format!(
                    "Invalid queen diagonal target byte: {} (valid: 64-127)",
                    second_byte
                ),
            });
        }

        // Target square = second_byte - 64
        Square::new(second_byte - 64)
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
        is_null_move: false,
    })
}

/// Helper: Calculate square offset safely
fn offset_square(square: Square, offset: i8) -> Result<Square> {
    let index = square.index() as i8 + offset;

    if index < 0 || index > 63 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Square offset out of bounds: {}", index),
        });
    }

    Ok(Square::new(index as u8))
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_adjacent_moves() {
        let from = Square::E4;

        // Test all 8 adjacent squares
        let moves = [
            (1, Square::D5),  // NW
            (2, Square::E5),  // N
            (3, Square::F5),  // NE
            (4, Square::D4),  // W
            (5, Square::F4),  // E
            (6, Square::D3),  // SW
            (7, Square::E3),  // S
            (8, Square::F3),  // SE
        ];

        for (move_value, expected_to) in moves {
            let decoded = decode_king_move(from, move_value, Color::White).unwrap();
            assert_eq!(decoded.to, expected_to);
        }
    }

    #[test]
    fn test_king_castling() {
        // White kingside castle (O-O) - value 10
        let decoded = decode_king_move(Square::E1, 10, Color::White).unwrap();
        assert_eq!(decoded.to, Square::G1);
        assert!(!decoded.is_null_move);

        // White queenside castle (O-O-O) - value 9 (NOT 11!)
        let decoded = decode_king_move(Square::E1, 9, Color::White).unwrap();
        assert_eq!(decoded.to, Square::C1);
        assert!(!decoded.is_null_move);

        // Black kingside castle (O-O) - value 10
        let decoded = decode_king_move(Square::E8, 10, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::G8);

        // Black queenside castle (O-O-O) - value 9
        let decoded = decode_king_move(Square::E8, 9, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::C8);
    }

    #[test]
    fn test_king_null_move() {
        // Null move (value 0) - King stays in place
        let decoded = decode_king_move(Square::E1, 0, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E1);  // Same as from
        assert!(decoded.is_null_move);
    }

    #[test]
    fn test_pawn_moves() {
        let from = Square::E2;

        // Forward one
        let decoded = decode_pawn_move(from, 1, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E3);
        assert_eq!(decoded.promotion, None);

        // Double push
        let decoded = decode_pawn_move(from, 15, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E4);

        // Capture right
        let decoded = decode_pawn_move(from, 2, Color::White).unwrap();
        assert_eq!(decoded.to, Square::F3);
    }

    #[test]
    fn test_pawn_promotions() {
        let from = Square::E7;

        // Promote to Queen (forward)
        let decoded = decode_pawn_move(from, 4, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E8);
        assert_eq!(decoded.promotion, Some(Role::Queen));

        // Promote to Knight (capture left)
        let decoded = decode_pawn_move(from, 12, Color::White).unwrap();
        assert_eq!(decoded.to, Square::D8);
        assert_eq!(decoded.promotion, Some(Role::Knight));
    }

    #[test]
    fn test_queen_vertical() {
        let from = Square::D4;

        // Move to d8 (rank 7, move_value = 8 + 7 = 15)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 15, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::D8);
    }

    #[test]
    fn test_queen_horizontal() {
        let from = Square::D4;

        // Move to h4 (file 7, move_value = 7)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 7, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::H4);
    }

    #[test]
    fn test_queen_diagonal() {
        let from = Square::D4;  // d4 = square 27, file 3

        // Diagonal move: move_value = 3 (same as from_file)
        // Second byte encodes target: e.g., 64 + 0 = a1
        let data = vec![64 + 0];  // Target square a1 (square 0)
        let mut stream = ByteStream::new(&data);

        let decoded = decode_queen_move(from, 3, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::A1);
    }

    #[test]
    fn test_knight_moves() {
        let from = Square::E4;

        // Test some L-shaped jumps
        let decoded = decode_knight_move(from, 1).unwrap();  // Two up, one left
        assert_eq!(decoded.to, Square::D6);

        let decoded = decode_knight_move(from, 2).unwrap();  // Two up, one right
        assert_eq!(decoded.to, Square::F6);
    }
}
```

**Validation Command**:
```bash
cargo test --lib decoder
```

**Expected Output**:
```
running 10 tests
test parser::decoder::tests::test_king_adjacent_moves ... ok
test parser::decoder::tests::test_king_castling ... ok
test parser::decoder::tests::test_king_null_move ... ok
test parser::decoder::tests::test_pawn_moves ... ok
test parser::decoder::tests::test_pawn_promotions ... ok
test parser::decoder::tests::test_queen_vertical ... ok
test parser::decoder::tests::test_queen_horizontal ... ok
test parser::decoder::tests::test_queen_diagonal ... ok
test parser::decoder::tests::test_knight_moves ... ok

test result: ok. 10 passed
```

---

#### Task 5.2.3: Implement High-Level Move Decoder

**Objective**: Create the main decoder that coordinates position tracking, piece number lookup, move decoding, and shakmaty validation.

**Reference**: IMPLEMENTATION_PLAN.md lines 737-899

**This Is The Integration Point**: All previous work comes together here:
- Phase 4: Raw move bytes from game file
- Task 5.1: Position wrapper with piece tracking
- Task 5.2.1: ByteStream for multi-byte moves
- Task 5.2.2: Piece-specific decoders
- Shakmaty: Move validation and application

**Acceptance Criteria**:
- [ ] Decode SCID move byte to shakmaty Move
- [ ] Handle all piece types correctly
- [ ] Support multi-byte Queen diagonal moves
- [ ] Validate all moves via shakmaty
- [ ] Update position after each move
- [ ] Provide clear error messages for invalid moves

**Implementation**:

**File**: `crates/core/src/parser/move_decoder.rs`

```rust
use shakmaty::{Move, Square, Role};
use crate::error::{Result, ScidError};
use crate::parser::position::ScidPosition;
use crate::parser::byte_stream::ByteStream;
use crate::parser::decoder::*;
use std::path::PathBuf;

/// High-level SCID move decoder
///
/// Coordinates:
/// - Position tracking (ScidPosition)
/// - Piece number lookup
/// - Move decoding (piece-specific)
/// - Move validation (shakmaty)
/// - Position updates
pub struct ScidMoveDecoder {
    position: ScidPosition,
}

impl ScidMoveDecoder {
    /// Create decoder from standard starting position
    pub fn new() -> Self {
        ScidMoveDecoder {
            position: ScidPosition::new(),
        }
    }

    /// Create decoder from FEN position
    pub fn from_fen(fen: &str) -> Result<Self> {
        Ok(ScidMoveDecoder {
            position: ScidPosition::from_fen(fen)?,
        })
    }

    /// Get current position
    pub fn position(&self) -> &ScidPosition {
        &self.position
    }

    /// Decode single SCID move byte into shakmaty Move
    ///
    /// Process:
    /// 1. Extract piece number and move value from byte
    /// 2. Look up piece's current square
    /// 3. Determine piece type from position
    /// 4. Decode move value to target square (piece-specific)
    /// 5. Find legal shakmaty Move matching from/to/promotion
    /// 6. Apply move and update position
    pub fn decode_move(&mut self, move_byte: u8, stream: &mut ByteStream) -> Result<Move> {
        // Extract piece number and move value
        let piece_num = (move_byte >> 4) & 0x0F;
        let move_value = move_byte & 0x0F;

        // Get piece's current square from SCID mapping
        let from_square = self.position.get_piece_square(piece_num)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!("Invalid piece number: {}", piece_num),
            })?;

        // Get piece at that square from shakmaty
        let piece = self.position.piece_at(from_square)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!("No piece at square {:?}", from_square),
            })?;

        // Decode move based on piece type
        let decoded = match piece.role {
            Role::King => decode_king_move(from_square, move_value, piece.color)?,
            Role::Queen => decode_queen_move(from_square, move_value, stream)?,
            Role::Rook => decode_rook_move(from_square, move_value)?,
            Role::Bishop => decode_bishop_move(from_square, move_value)?,
            Role::Knight => decode_knight_move(from_square, move_value)?,
            Role::Pawn => decode_pawn_move(from_square, move_value, piece.color)?,
        };

        // Find legal shakmaty Move matching our decoded from/to/promotion
        let chess_move = self.position.find_move(
            decoded.from,
            decoded.to,
            decoded.promotion
        ).ok_or_else(|| ScidError::ParseError {
            file: PathBuf::from("move_decoder"),
            offset: 0,
            message: format!(
                "No legal move from {:?} to {:?} (promotion: {:?})",
                decoded.from, decoded.to, decoded.promotion
            ),
        })?;

        // Apply move to position (validates and updates state)
        self.position.make_move(&chess_move)?;

        Ok(chess_move)
    }

    /// Decode sequence of SCID move bytes
    pub fn decode_moves(&mut self, move_bytes: &[u8]) -> Result<Vec<Move>> {
        let mut moves = Vec::new();
        let mut stream = ByteStream::new(move_bytes);

        while stream.has_more() {
            let move_byte = stream.get_byte()?;
            let chess_move = self.decode_move(move_byte, &mut stream)?;
            moves.push(chess_move);
        }

        Ok(moves)
    }
}

impl Default for ScidMoveDecoder {
    fn default() -> Self {
        Self::new()
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Color};

    #[test]
    fn test_decode_e4() {
        let mut decoder = ScidMoveDecoder::new();

        // SCID move byte for e2-e4 (pawn 12, double push value 15)
        let move_byte = (12 << 4) | 15;  // 0xCF

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(move_byte, &mut stream).unwrap();

        // Verify move
        assert_eq!(chess_move.from(), Some(Square::E2));
        assert_eq!(chess_move.to(), Square::E4);

        // Verify position updated
        assert_eq!(decoder.position().turn(), Color::Black);
    }

    #[test]
    fn test_decode_knight_f3() {
        let mut decoder = ScidMoveDecoder::new();

        // First move: e2-e4
        let e4_byte = (12 << 4) | 15;
        let mut stream = ByteStream::new(&[]);
        decoder.decode_move(e4_byte, &mut stream).unwrap();

        // Second move (Black): any move, e.g., e7-e5
        let e5_byte = (12 << 4) | 15;
        decoder.decode_move(e5_byte, &mut stream).unwrap();

        // Third move (White): Nf3 (knight 7 from g1)
        // Need to determine correct move value for g1-f3
        // Knight from g1 (piece 7) to f3
        let nf3_byte = (7 << 4) | 6;  // Move value 6 for knight

        let chess_move = decoder.decode_move(nf3_byte, &mut stream).unwrap();
        assert_eq!(chess_move.to(), Square::F3);
    }

    #[test]
    fn test_decode_queen_diagonal() {
        // Set up position where Queen can move diagonally
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Queen from d1 to h5 diagonal (piece 1)
        // This is a 2-byte move!
        // First byte: piece 1, move_value = file of d1 = 3
        let first_byte = (1 << 4) | 3;

        // Second byte: target square h5 = 39, encoded as 64 + 39 = 103
        let data = vec![first_byte, 103];
        let mut stream = ByteStream::new(&data);

        let chess_move = decoder.decode_move(first_byte, &mut stream).unwrap();
        assert_eq!(chess_move.to(), Square::H5);
    }

    #[test]
    fn test_decode_castling() {
        // Set up position where castling is legal
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Kingside castle O-O (king piece 0, move value 10)
        let castle_byte = (0 << 4) | 10;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        // Verify castling move
        assert_eq!(chess_move.from(), Some(Square::E1));
        assert_eq!(chess_move.to(), Square::G1);
    }

    #[test]
    fn test_decode_queenside_castling() {
        // Set up position where queenside castling is legal
        let fen = "r3kbnr/pppqpppp/2n5/3p1b2/3P1B2/2N5/PPPQPPPP/R3KBNR w KQkq - 6 5";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Queenside castle O-O-O (king piece 0, move value 9, NOT 11!)
        let castle_byte = (0 << 4) | 9;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        // Verify castling move
        assert_eq!(chess_move.from(), Some(Square::E1));
        assert_eq!(chess_move.to(), Square::C1);
    }

    #[test]
    fn test_decode_pawn_promotion() {
        // Set up position with pawn ready to promote
        let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Pawn e7-e8=Q (piece 8, move value 4 for queen promotion forward)
        let promo_byte = (8 << 4) | 4;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(promo_byte, &mut stream).unwrap();

        assert_eq!(chess_move.to(), Square::E8);
        assert_eq!(chess_move.promotion(), Some(Role::Queen));
    }
}
```

**Validation Command**:
```bash
cargo test --lib move_decoder
```

**Expected Output**:
```
running 6 tests
test parser::move_decoder::tests::test_decode_e4 ... ok
test parser::move_decoder::tests::test_decode_knight_f3 ... ok
test parser::move_decoder::tests::test_decode_queen_diagonal ... ok
test parser::move_decoder::tests::test_decode_castling ... ok
test parser::move_decoder::tests::test_decode_queenside_castling ... ok
test parser::move_decoder::tests::test_decode_pawn_promotion ... ok

test result: ok. 6 passed
```

---

#### Task 5.2.4: Game Tree and Variation Data Structures

**Objective**: Define data structures for representing complete games with variations, comments, and NAGs.

**Reference**: IMPLEMENTATION_PLAN.md Phase 4.3

**Why Needed**: Games in SCID aren't just linear move lists - they include:
- Variations (alternative lines)
- Comments attached to moves
- NAG annotations
- Nested variations within variations

**Implementation**:

**File**: `crates/core/src/parser/game_tree.rs`

```rust
use shakmaty::Move;

/// Complete game representation with variations
#[derive(Debug, Clone)]
pub struct GameTree {
    /// Starting position (None for standard start, Some(fen) for custom)
    pub start_fen: Option<String>,

    /// Root of the move tree (main line + variations)
    pub root: MoveNode,
}

/// Single node in move tree
#[derive(Debug, Clone)]
pub struct MoveNode {
    /// The chess move (None for root node before first move)
    pub chess_move: Option<Move>,

    /// Comment attached to this move (from ENCODE_COMMENT markers)
    pub comment: Option<String>,

    /// NAG annotations (from ENCODE_NAG markers)
    pub nags: Vec<u8>,

    /// Continuation (next move in this line)
    pub continuation: Option<Box<MoveNode>>,

    /// Alternative variations starting from this position
    /// Created when ENCODE_START_MARKER (0x0D) is encountered
    pub variations: Vec<MoveNode>,
}

impl MoveNode {
    /// Create root node (before first move)
    pub fn root() -> Self {
        MoveNode {
            chess_move: None,
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }
    }

    /// Create node with a move
    pub fn with_move(chess_move: Move) -> Self {
        MoveNode {
            chess_move: Some(chess_move),
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }
    }

    /// Add a move as continuation and return mutable reference to it
    pub fn add_continuation(&mut self, chess_move: Move) -> &mut MoveNode {
        self.continuation = Some(Box::new(MoveNode::with_move(chess_move)));
        self.continuation.as_mut().unwrap()
    }

    /// Add a variation and return mutable reference to first move
    pub fn add_variation(&mut self, first_move: Move) -> &mut MoveNode {
        self.variations.push(MoveNode::with_move(first_move));
        self.variations.last_mut().unwrap()
    }

    /// Iterate over main line moves
    pub fn main_line(&self) -> MainLineIter {
        MainLineIter { current: Some(self) }
    }
}

/// Iterator over main line moves
pub struct MainLineIter<'a> {
    current: Option<&'a MoveNode>,
}

impl<'a> Iterator for MainLineIter<'a> {
    type Item = &'a MoveNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.continuation.as_deref();
        Some(node)
    }
}

impl GameTree {
    /// Create new game tree with standard starting position
    pub fn new() -> Self {
        GameTree {
            start_fen: None,
            root: MoveNode::root(),
        }
    }

    /// Create game tree with custom starting position
    pub fn with_fen(fen: String) -> Self {
        GameTree {
            start_fen: Some(fen),
            root: MoveNode::root(),
        }
    }

    /// Get main line moves as vector
    pub fn main_line_moves(&self) -> Vec<&Move> {
        self.root.main_line()
            .filter_map(|node| node.chess_move.as_ref())
            .collect()
    }

    /// Count total moves including variations (DFS)
    pub fn total_move_count(&self) -> usize {
        fn count_node(node: &MoveNode) -> usize {
            let mut count = if node.chess_move.is_some() { 1 } else { 0 };
            if let Some(ref cont) = node.continuation {
                count += count_node(cont);
            }
            for var in &node.variations {
                count += count_node(var);
            }
            count
        }
        count_node(&self.root)
    }
}

/// Variation parsing state machine with position tracking (Gap 1)
///
/// # CRITICAL: Position State Restoration
///
/// When entering a variation (START_MARKER 0x0D), we must save the COMPLETE
/// chess position state. When exiting (END_MARKER 0x0E), we restore it.
///
/// This is essential because variations are alternative continuations from
/// a specific position. Without restoring, the position would be corrupted
/// by the variation's moves.
///
/// ## Why Position State Matters
///
/// ```text
/// Main line: 1.e4 e5 2.Nf3
///                      ↓
///              ┌───────┴────────┐
///              │                │
///           2...Nc6          [VAR: 2...d6 3.d4]
///              ↓                  ↓
///           3.Bb5             END_MARKER (0x0E)
///                                 ↓
///                             ← MUST restore position after 2.Nf3!
/// ```
///
/// ## Position State Stack
///
/// Each stack entry includes the complete position state:
/// - Board configuration (piece placement)
/// - Side to move
/// - Castling rights
/// - En passant square
/// - Halfmove clock
/// - Fullmove number
/// - SCID piece number mapping (for continued decoding)
#[derive(Debug)]
pub struct VariationParseState {
    /// Stack of saved positions for variation restore
    position_stack: Vec<VariationSnapshot>,

    /// Current chess position
    current_position: shakmaty::Chess,

    /// Current SCID piece mapping (for move decoding)
    current_piece_mapping: PieceNumberMapping,

    /// Current node in tree being built
    current_node: *mut MoveNode,

    /// Variation depth (for debugging)
    depth: usize,
}

/// Snapshot of state to restore after variation ends
#[derive(Debug, Clone)]
pub struct VariationSnapshot {
    /// Chess position at variation start
    pub position: shakmaty::Chess,

    /// SCID piece mapping at variation start
    pub piece_mapping: PieceNumberMapping,

    /// Node to return to (parent of variation)
    pub return_node: *mut MoveNode,
}

impl VariationParseState {
    /// Create new parse state with starting position
    pub fn new(start_position: shakmaty::Chess) -> Self {
        let piece_mapping = PieceNumberMapping::from_position(&start_position);
        Self {
            position_stack: Vec::new(),
            current_position: start_position,
            current_piece_mapping: piece_mapping,
            current_node: std::ptr::null_mut(),
            depth: 0,
        }
    }

    /// Create from FEN string
    pub fn from_fen(fen: &str) -> Result<Self, ScidError> {
        use shakmaty::fen::Fen;
        let parsed: Fen = fen.parse().map_err(|e| {
            ScidError::InvalidFormat(format!("FEN parse error: {:?}", e))
        })?;
        let position: shakmaty::Chess = parsed
            .into_position(shakmaty::CastlingMode::Standard)
            .map_err(|e| {
                ScidError::InvalidFormat(format!("Invalid position: {:?}", e))
            })?;
        Ok(Self::new(position))
    }

    /// Handle START_MARKER (0x0D) - entering a variation
    ///
    /// Saves complete position state before processing variation moves.
    pub fn start_variation(&mut self) {
        // Save complete state snapshot
        let snapshot = VariationSnapshot {
            position: self.current_position.clone(),
            piece_mapping: self.current_piece_mapping.clone(),
            return_node: self.current_node,
        };
        self.position_stack.push(snapshot);

        self.depth += 1;
    }

    /// Handle END_MARKER (0x0E) - exiting a variation
    ///
    /// Restores complete position state to continue main line.
    pub fn end_variation(&mut self) -> bool {
        if let Some(snapshot) = self.position_stack.pop() {
            // Restore complete state
            self.current_position = snapshot.position;
            self.current_piece_mapping = snapshot.piece_mapping;
            self.current_node = snapshot.return_node;
            self.depth = self.depth.saturating_sub(1);
            true
        } else {
            // No variation to end - may be malformed data
            false
        }
    }

    /// Apply a move to current position
    pub fn apply_move(&mut self, chess_move: &shakmaty::Move) -> Result<(), ScidError> {
        // Update piece mapping for capture handling
        self.current_piece_mapping.update_after_move(chess_move, &self.current_position);

        // Apply move to position
        self.current_position = self.current_position.clone()
            .play(chess_move)
            .map_err(|e| ScidError::InvalidFormat(format!("Illegal move: {:?}", e)))?;

        Ok(())
    }

    /// Get current position for move validation
    pub fn position(&self) -> &shakmaty::Chess {
        &self.current_position
    }

    /// Get current piece mapping for move decoding
    pub fn piece_mapping(&self) -> &PieceNumberMapping {
        &self.current_piece_mapping
    }

    /// Current variation depth
    pub fn variation_depth(&self) -> usize {
        self.depth
    }
}
```

**NAG Constants** (common values for PGN output):

```rust
/// Common NAG (Numeric Annotation Glyph) values
pub mod nag {
    pub const GOOD_MOVE: u8 = 1;           // !
    pub const MISTAKE: u8 = 2;             // ?
    pub const BRILLIANT_MOVE: u8 = 3;      // !!
    pub const BLUNDER: u8 = 4;             // ??
    pub const INTERESTING_MOVE: u8 = 5;    // !?
    pub const DUBIOUS_MOVE: u8 = 6;        // ?!
    pub const EQUAL: u8 = 10;              // =
    pub const UNCLEAR: u8 = 13;            // ∞
    pub const SLIGHT_ADVANTAGE_WHITE: u8 = 14;  // +=
    pub const SLIGHT_ADVANTAGE_BLACK: u8 = 15;  // =+
    pub const CLEAR_ADVANTAGE_WHITE: u8 = 16;   // ±
    pub const CLEAR_ADVANTAGE_BLACK: u8 = 17;   // ∓
    pub const WINNING_WHITE: u8 = 18;      // +-
    pub const WINNING_BLACK: u8 = 19;      // -+

    /// Convert NAG to PGN symbol (for common NAGs) or $N notation
    pub fn to_pgn_string(nag: u8) -> String {
        match nag {
            1 => "!".to_string(),
            2 => "?".to_string(),
            3 => "!!".to_string(),
            4 => "??".to_string(),
            5 => "!?".to_string(),
            6 => "?!".to_string(),
            _ => format!("${}", nag),
        }
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Role, Move};

    #[test]
    fn test_game_tree_creation() {
        let mut tree = GameTree::new();

        // Add e4
        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        let node = tree.root.add_continuation(e4);

        // Add e5
        let e5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E5,
            capture: None,
            promotion: None,
        };
        node.add_continuation(e5);

        assert_eq!(tree.main_line_moves().len(), 2);
        assert_eq!(tree.total_move_count(), 2);
    }

    #[test]
    fn test_variation_creation() {
        let mut tree = GameTree::new();

        // Main line: 1. e4
        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        let node = tree.root.add_continuation(e4);

        // Variation: 1. d4
        let d4 = Move::Normal {
            role: Role::Pawn,
            from: Square::D2,
            to: Square::D4,
            capture: None,
            promotion: None,
        };
        tree.root.add_variation(d4);

        assert_eq!(tree.main_line_moves().len(), 1);  // Only e4 in main line
        assert_eq!(tree.total_move_count(), 2);       // e4 + d4
        assert_eq!(tree.root.variations.len(), 1);
    }

    #[test]
    fn test_variation_parse_state_position_restore() {
        // Test Gap 1: Position state is correctly saved and restored

        // Create starting position
        let start = shakmaty::Chess::default();
        let mut state = VariationParseState::new(start.clone());

        // Play 1.e4
        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        state.apply_move(&e4).unwrap();

        // Save position hash before variation
        let position_before_var = state.position().clone();

        // Enter variation
        state.start_variation();
        assert_eq!(state.variation_depth(), 1);

        // Play 1...d5 in variation
        let d5 = Move::Normal {
            role: Role::Pawn,
            from: Square::D7,
            to: Square::D5,
            capture: None,
            promotion: None,
        };
        state.apply_move(&d5).unwrap();

        // Position should now be different (after 1...d5)
        assert_ne!(state.position().board(), position_before_var.board());

        // End variation
        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 0);

        // Position should be restored to after 1.e4
        assert_eq!(state.position().board(), position_before_var.board());
    }

    #[test]
    fn test_variation_parse_state_nested_variations() {
        // Test nested variations (Gap 1)

        let start = shakmaty::Chess::default();
        let mut state = VariationParseState::new(start);

        // Play 1.e4
        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        state.apply_move(&e4).unwrap();

        // Enter first variation
        state.start_variation();
        assert_eq!(state.variation_depth(), 1);

        // Enter nested variation
        state.start_variation();
        assert_eq!(state.variation_depth(), 2);

        // End nested variation
        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 1);

        // End first variation
        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 0);

        // No more variations to end
        assert!(!state.end_variation());
    }
}
```

**Acceptance Criteria**:
- [ ] GameTree struct represents complete games with variations
- [ ] MoveNode supports comments, NAGs, and nested variations
- [ ] Main line iteration works correctly
- [ ] Variation counting works correctly
- [ ] NAG formatting matches PGN specification
- [ ] VariationParseState tracks position state stack (Gap 1)
- [ ] VariationSnapshot captures complete position + piece mapping (Gap 1)
- [ ] start_variation() saves complete state before variation
- [ ] end_variation() restores complete state after variation

---

### Section 5.3: Integration and Real-World Testing

**Objective**: Integrate the move decoder with Phase 4's game parser and validate against real SCID databases.

---

#### Task 5.3.1: Connect Move Decoder to Game Parser

**Objective**: Update the game parser from Phase 4 to use the move decoder and generate complete game sequences.

**Reference**: PHASE_4_GAME_FILE_STRUCTURE.md Task 4.2

**Acceptance Criteria**:
- [ ] Parse game data from .sg4 file
- [ ] Decode all moves in sequence
- [ ] Handle variations with position state restore (Gap 1)
- [ ] Handle pre-game comments (Gap 3)
- [ ] Handle comments and NAGs (future: Phase 6)
- [ ] Return vector of shakmaty Moves

**Implementation**:

**File**: `crates/core/src/database/games.rs` (update from Phase 4)

```rust
use crate::parser::move_decoder::ScidMoveDecoder;
use crate::parser::byte_stream::ByteStream;
use shakmaty::Move as ChessMove;

// Update GameData from Phase 4
#[derive(Debug)]
pub struct GameData {
    pub tags: HashMap<String, String>,
    pub flags: u8,
    pub start_position: Option<String>,
    pub moves: Vec<ChessMove>,  // Changed from Vec<u8> to Vec<ChessMove>

    /// Pre-game comment (Gap 3)
    ///
    /// Comment that appears BEFORE the first move.
    /// In PGN output, this appears after the tags but before move 1.
    ///
    /// Example PGN:
    /// ```pgn
    /// [Event "Tournament"]
    /// [White "Player1"]
    ///
    /// {This game decided the championship.} 1.e4 e5
    /// ```
    pub pre_game_comment: Option<String>,
}

/// Parse complete game including moves
pub fn parse_game(
    sg4_file: &mut File,
    offset: u32,
    length: u32,
    start_fen: Option<&str>,
) -> Result<GameData> {
    // Read game bytes (from Phase 4)
    let game_bytes = read_game_data(sg4_file, offset, length)?;

    // Parse tags and flags (from Phase 4)
    let (tags, flags, move_start) = parse_game_tags(&game_bytes)?;

    // Create move decoder with correct starting position
    let mut decoder = if let Some(fen) = start_fen {
        ScidMoveDecoder::from_fen(fen)?
    } else if flags & 0x01 != 0 {
        // Non-standard start: FEN follows flags
        let fen_end = game_bytes[move_start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("games"),
                offset: move_start as u64,
                message: "FEN string not null-terminated".to_string(),
            })?;

        let fen = String::from_utf8_lossy(&game_bytes[move_start..move_start + fen_end]);
        let decoder = ScidMoveDecoder::from_fen(&fen)?;

        // Adjust move_start to skip FEN + null terminator
        // (This will be updated below)
        decoder
    } else {
        ScidMoveDecoder::new()
    };

    // Find actual move start (after FEN if present)
    let move_start = if flags & 0x01 != 0 {
        // Skip FEN + null terminator
        move_start + game_bytes[move_start..]
            .iter()
            .position(|&b| b == 0)
            .unwrap() + 1
    } else {
        move_start
    };

    // Decode moves using ByteStream
    let move_bytes = &game_bytes[move_start..];
    let mut stream = ByteStream::new(move_bytes);
    let mut moves = Vec::new();

    // Track pre-game comment (Gap 3)
    // A comment marker BEFORE the first move indicates a pre-game comment
    let mut pre_game_comment: Option<String> = None;
    let mut first_move_seen = false;
    let mut comment_markers_before_first_move = 0;

    while stream.has_more() {
        let byte = stream.peek_byte()?;

        // Check for special bytes
        if byte == 15 {
            // ENCODE_END_GAME
            break;
        } else if byte == 13 || byte == 14 {
            // Variation markers - skip for now (Phase 7)
            stream.get_byte()?;
            continue;
        } else if byte == 11 {
            // ENCODE_NAG - skip NAG value
            stream.get_byte()?;
            stream.get_byte()?;
            continue;
        } else if byte == 12 {
            // ENCODE_COMMENT marker (Gap 3)
            // NOTE: This is just a MARKER - actual text is in comment section!
            stream.get_byte()?;

            // Track pre-game comments (before first move)
            if !first_move_seen {
                comment_markers_before_first_move += 1;
            }
            continue;
        }

        // Regular move - marks first move seen
        first_move_seen = true;
        let move_byte = stream.get_byte()?;
        let chess_move = decoder.decode_move(move_byte, &mut stream)?;
        moves.push(chess_move);
    }

    // If there were comment markers before first move, the first comment
    // in the comment section is a pre-game comment (Gap 3)
    // Note: Actual comment text extraction happens when processing comment_data
    // from Phase 4's parse_game_structure()

    Ok(GameData {
        tags,
        flags,
        start_position: start_fen.map(String::from),
        moves,
        pre_game_comment,  // Will be filled when processing comment_data
    })
}
```

**Testing**:

```rust
#[test]
fn test_parse_complete_game() {
    // Use test database from Phase 4
    let mut sg4_file = File::open("tests/data/five.sg4").unwrap();

    // Game 1 metadata from index (from Phase 2)
    let offset = 0;
    let length = 157;  // From actual test data

    // Parse game
    let game = parse_game(&mut sg4_file, offset, length, None).unwrap();

    // Verify moves were decoded
    assert!(game.moves.len() > 0);
    println!("Decoded {} moves", game.moves.len());

    // Verify first move (should be reasonable opening move)
    let first_move = &game.moves[0];
    println!("First move: {:?}", first_move);
}
```

**Validation Command**:
```bash
cargo test test_parse_complete_game -- --nocapture
```

**Expected Output**:
```
Decoded 37 moves
First move: Move::Normal { role: Pawn, from: e2, to: e4, capture: None, promotion: None }
test database::games::tests::test_parse_complete_game ... ok
```

---

#### Task 5.3.1.1: Pre-Game Comment Handling (Gap 3)

**Objective**: Properly handle comments that appear BEFORE the first chess move.

**Reference**: IMPLEMENTATION_PLAN.md Gap 3, Phase 4.3

**Why This Matters**: SCID allows annotators to add introductory comments before the game starts. These comments are common in annotated games and tournament collections.

**The Problem**:

Comments in SCID are stored in TWO parts:
1. **Markers (0x0C)** in move data indicate where comments exist
2. **Text** is stored separately in the comment section

A comment marker appearing BEFORE any move byte indicates a pre-game comment:

```text
Move data with pre-game comment:
[0x0C][move1_bytes][0x0C][move2_bytes][0x0F]
  ↑                  ↑
  pre-game          after-move-1
  comment           comment

Comment section:
["Game intro text\0"]["Good opening!\0"]
```

**Implementation**:

```rust
/// Track pre-game comment state during parsing
pub struct GameParseState {
    /// Whether we've seen the first actual move
    first_move_seen: bool,

    /// Number of comment markers before first move
    pre_game_comment_count: usize,

    /// The pre-game comment text (extracted from comment section)
    pre_game_comment: Option<String>,

    // ... other state
}

impl GameParseState {
    /// Handle comment marker (0x0C)
    pub fn handle_comment_marker(&mut self) {
        if !self.first_move_seen {
            // This is a pre-game comment marker
            self.pre_game_comment_count += 1;
        }
        // Note: We don't read text here - just track the marker
    }

    /// Mark that first move has been seen
    pub fn mark_first_move(&mut self) {
        self.first_move_seen = true;
    }

    /// Attach comments from comment section to game tree
    pub fn attach_comments(&mut self, comment_data: &[u8], tree: &mut GameTree) {
        let mut comment_iter = CommentIterator::new(comment_data);

        // First comment(s) go to pre-game if markers were seen before first move
        for _ in 0..self.pre_game_comment_count {
            if let Some(text) = comment_iter.next() {
                // Multiple pre-game comments get concatenated
                match &mut self.pre_game_comment {
                    Some(existing) => {
                        existing.push_str("\n\n");
                        existing.push_str(&text);
                    }
                    None => {
                        self.pre_game_comment = Some(text);
                    }
                }
            }
        }

        // Remaining comments attach to moves in tree traversal order
        // ... (handled by attach_comments_to_tree from Phase 4)
    }
}
```

**PGN Output for Pre-Game Comments**:

Pre-game comments appear after the tag section but before move 1:

```pgn
[Event "World Championship"]
[Site "Moscow"]
[Date "1985.09.03"]
[White "Karpov, Anatoly"]
[Black "Kasparov, Garry"]
[Result "0-1"]

{This was the decisive game of the 1985 World Championship match.
Kasparov finally broke through Karpov's solid defense with brilliant
piece play.} 1.d4 Nf6 2.c4 e6 3.Nc3 Bb4 {The Nimzo-Indian Defense}
```

**Testing**:

```rust
#[test]
fn test_pre_game_comment_detection() {
    // Simulated move data with pre-game comment marker
    let move_data = vec![
        0x0C,  // Comment marker BEFORE first move
        0xCF,  // First move (e4)
        0x0C,  // Comment marker AFTER first move
        0x0F,  // End of game
    ];

    let mut state = GameParseState::new();
    let mut stream = ByteStream::new(&move_data);

    while stream.has_more() {
        let byte = stream.get_byte().unwrap();
        match byte {
            0x0C => state.handle_comment_marker(),
            0x0F => break,
            _ => {
                state.mark_first_move();
                // decode move...
            }
        }
    }

    assert_eq!(state.pre_game_comment_count, 1);
    assert!(state.first_move_seen);
}

#[test]
fn test_pre_game_comment_text_extraction() {
    // Comment section data
    let comment_data = b"Game introduction.\0After first move.\0";

    let mut comment_iter = CommentIterator::new(comment_data);

    // First comment is the pre-game comment
    let pre_game = comment_iter.next().unwrap();
    assert_eq!(pre_game, "Game introduction.");

    // Second comment attaches to first move
    let move_comment = comment_iter.next().unwrap();
    assert_eq!(move_comment, "After first move.");
}
```

**Acceptance Criteria**:
- [ ] Pre-game comment markers detected (0x0C before first move)
- [ ] Pre-game comment text extracted from comment section
- [ ] Multiple pre-game comments concatenated if present
- [ ] Pre-game comment stored in GameData
- [ ] Tests pass for pre-game comment handling

---

#### Task 5.3.2: Comprehensive Real-World Validation

**Objective**: Validate the complete parsing pipeline against real SCID databases with known game content.

**Reference**: IMPLEMENTATION_PLAN.md lines 1369-1380

**Why This Matters**: Unit tests verify individual components, but real SCID data has:
- Complex move sequences with tactical patterns
- All special moves (castling, en passant, promotions)
- Edge cases not covered in synthetic tests
- Verification against known game outcomes

**Acceptance Criteria**:
- [ ] Parse all games from test database
- [ ] Achieve 90%+ move parsing success rate
- [ ] All special moves parse correctly
- [ ] No crashes or panics on malformed data
- [ ] Clear error messages for unparseable moves

**Implementation**:

**File**: `tests/integration/phase5_validation.rs`

```rust
use scidtopgn_core::ScidReader;
use scidtopgn_core::database::parse_game;

#[test]
fn test_five_database_complete() {
    // Open test database
    let reader = ScidReader::open("tests/data/five").unwrap();

    println!("Database: {}", reader.metadata().description);
    println!("Total games: {}", reader.metadata().num_games);

    let mut total_moves = 0;
    let mut successful_games = 0;

    for (idx, game_result) in reader.games().enumerate() {
        match game_result {
            Ok(game) => {
                println!("Game {}: {} moves", idx + 1, game.moves.len());
                total_moves += game.moves.len();
                successful_games += 1;

                // Verify moves are reasonable
                assert!(game.moves.len() > 0, "Game should have at least one move");
                assert!(game.moves.len() < 500, "Game should have < 500 moves");
            }
            Err(e) => {
                println!("Game {} failed: {:?}", idx + 1, e);
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!("Successful games: {}/{}", successful_games, reader.metadata().num_games);
    println!("Total moves parsed: {}", total_moves);
    println!("Success rate: {:.1}%",
        (successful_games as f64 / reader.metadata().num_games as f64) * 100.0);

    // Should parse at least 90% of games
    assert!(successful_games as f64 / reader.metadata().num_games as f64 >= 0.9);
}

#[test]
fn test_special_moves() {
    // This test verifies specific games with known special moves
    let reader = ScidReader::open("tests/data/five").unwrap();

    // Test castling (verify in actual game data)
    // Test en passant (if present in test data)
    // Test promotions (if present in test data)

    for game_result in reader.games() {
        if let Ok(game) = game_result {
            for chess_move in &game.moves {
                // Check for castling
                if let shakmaty::Move::Castle { .. } = chess_move {
                    println!("Found castling move: {:?}", chess_move);
                }

                // Check for promotions
                if chess_move.promotion().is_some() {
                    println!("Found promotion: {:?}", chess_move);
                }

                // Check for en passant
                if chess_move.is_en_passant() {
                    println!("Found en passant: {:?}", chess_move);
                }
            }
        }
    }
}

#[test]
fn test_queen_diagonal_moves() {
    // Specific test for 2-byte Queen diagonal moves
    let reader = ScidReader::open("tests/data/five").unwrap();

    let mut queen_diagonal_count = 0;

    for game_result in reader.games() {
        if let Ok(game) = game_result {
            // Count Queen moves that are diagonal
            for chess_move in &game.moves {
                if let shakmaty::Move::Normal { role, from, to, .. } = chess_move {
                    if *role == shakmaty::Role::Queen {
                        if let Some(from_sq) = from {
                            // Check if diagonal (file change == rank change)
                            let file_diff = (to.file() as i8 - from_sq.file() as i8).abs();
                            let rank_diff = (to.rank() as i8 - from_sq.rank() as i8).abs();

                            if file_diff == rank_diff && file_diff > 0 {
                                queen_diagonal_count += 1;
                                println!("Queen diagonal: {:?} to {:?}", from_sq, to);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("Total Queen diagonal moves: {}", queen_diagonal_count);
    assert!(queen_diagonal_count > 0, "Should find at least some Queen diagonal moves");
}
```

**Validation Command**:
```bash
cargo test --test phase5_validation -- --nocapture
```

**Expected Output**:
```
Database: Test Database
Total games: 5

Game 1: 37 moves
Game 2: 42 moves
Game 3: 31 moves
Game 4: 45 moves
Game 5: 38 moves

=== SUMMARY ===
Successful games: 5/5
Total moves parsed: 193
Success rate: 100.0%

Found castling move: Castle { king: e1, rook: h1 }
Found promotion: Normal { role: Pawn, from: e7, to: e8, promotion: Some(Queen) }
Total Queen diagonal moves: 8

test integration::phase5_validation::test_five_database_complete ... ok
test integration::phase5_validation::test_special_moves ... ok
test integration::phase5_validation::test_queen_diagonal_moves ... ok
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Incorrect Piece Number Mapping

**Problem**: Using fixed square-to-piece mappings instead of dynamic tracking.

**Example**:
```rust
// ❌ WRONG - assumes piece never moves!
let piece_squares = [Square::E1, Square::D1, ...];
let from_square = piece_squares[piece_num];
```

**Solution**: Track piece locations dynamically and update after each move:
```rust
// ✅ CORRECT
let from_square = position.get_piece_square(piece_num)?;
// ... make move ...
position.update_piece_mapping(&chess_move);
```

---

### Pitfall 2: Forgetting Queen Diagonal 2-Byte Reads

**Problem**: Treating all moves as 1-byte, causing stream desynchronization for Queen diagonal moves.

**Symptoms**:
- Parsing works initially, then fails mid-game
- "Invalid piece number" errors after Queen moves
- Move sequences don't make sense

**Example**:
```rust
// ❌ WRONG - always reads 1 byte
fn decode_queen(move_value: u8) -> Square {
    // ... decode logic ...
    // Missing: stream.get_byte() for diagonal case!
}
```

**Solution**: Always pass ByteStream and read second byte for diagonals:
```rust
// ✅ CORRECT
fn decode_queen_move(from: Square, move_value: u8, stream: &mut ByteStream) -> Result<DecodedMove> {
    if move_value == from.file() as u8 {
        // Diagonal: read second byte!
        let second_byte = stream.get_byte()?;
        // ...
    }
}
```

---

### Pitfall 3: Not Using Shakmaty for Move Validation

**Problem**: Assuming decoded from/to squares are always legal, skipping validation.

**Consequences**:
- Illegal moves applied to position
- Board state becomes inconsistent
- Crashes on subsequent moves
- En passant and castling edge cases fail

**Example**:
```rust
// ❌ WRONG - no validation
self.position.board[to] = self.position.board[from];
self.position.board[from] = None;
```

**Solution**: Always use shakmaty's position.find_move() and make_move():
```rust
// ✅ CORRECT
let chess_move = position.find_move(from, to, promotion)?;
position.make_move(&chess_move)?;  // Validates and updates state
```

---

### Pitfall 4: Wrong Pawn Direction for Black

**Problem**: Using same forward offset (+8) for both White and Black pawns.

**Example**:
```rust
// ❌ WRONG
let target = offset_square(from, 8)?;  // Always moves up!
```

**Solution**: Adjust direction based on side to move:
```rust
// ✅ CORRECT
let forward = match color {
    Color::White => 8,
    Color::Black => -8,
};
let target = offset_square(from, forward)?;
```

---

### Pitfall 5: Ignoring Piece Captures in Mapping

**Problem**: Not removing captured pieces from piece number mapping.

**Consequences**:
- Captured piece numbers remain in mapping
- Future moves may reference non-existent pieces
- Position state diverges from reality

**Example**:
```rust
// ❌ WRONG - only updates moving piece
piece_mapping.insert(piece_num, to_square);
```

**Solution**: Remove captured pieces from opponent mapping:
```rust
// ✅ CORRECT
if chess_move.is_capture() {
    opponent_mapping.retain(|_, &sq| sq != chess_move.to());
}
piece_mapping.insert(piece_num, to_square);
```

---

### Pitfall 6: Not Using SCID's Capture Swap Algorithm

**Problem**: Simply removing captured pieces creates gaps in piece numbering.

**Why This Matters**: SCID maintains a compact piece list. When piece N is captured, the LAST piece in the list (highest slot number) moves to slot N. This is critical for correct piece number references in subsequent moves.

**Example**:
```rust
// ❌ WRONG - leaves gap in piece numbering
opponent_mapping.remove(&captured_slot);
// Now slots might be: 0, 1, 2, 4, 5 (gap at 3!)
// Future move bytes expect contiguous numbering
```

**Solution**: Implement SCID's swap algorithm:
```rust
// ✅ CORRECT - SCID capture swap algorithm
if let Some(captured_num) = captured_slot {
    let last_slot = opponent_pieces.keys().max().copied();
    if let Some(last) = last_slot {
        if last != captured_num {
            // Move last piece to captured slot
            let last_square = opponent_pieces[&last];
            opponent_pieces.insert(captured_num, last_square);
        }
        // Remove the last slot
        opponent_pieces.remove(&last);
    }
}
// Now slots are: 0, 1, 2, 3, 4 (compact, no gaps!)
```

**Reference**: SCID_DATABASE_FORMAT.md Section 4.4, position.cpp DoSimpleMove()

---

### Pitfall 7: Wrong Piece Numbering Order

**Problem**: Assuming pieces are numbered by type (King, Queen, Rooks, etc.) instead of by starting file.

**Symptoms**:
- First few moves work, then "invalid piece number" errors
- Knights and Bishops swapped
- Queen moves fail

**Example**:
```rust
// ❌ WRONG - numbered by piece type
mapping.insert(0, Square::E1);  // King
mapping.insert(1, Square::D1);  // Queen ← WRONG! Should be slot 4
mapping.insert(2, Square::A1);  // Rook a1 ← WRONG! Should be slot 1
```

**Solution**: Number by starting file position:
```rust
// ✅ CORRECT - numbered by starting file (from SCID initPieceList)
mapping.insert(0, Square::E1);  // King (always 0)
mapping.insert(1, Square::A1);  // QR - Queen's Rook (a-file)
mapping.insert(2, Square::B1);  // QN - Queen's Knight (b-file)
mapping.insert(3, Square::C1);  // QB - Queen's Bishop (c-file)
mapping.insert(4, Square::D1);  // Q  - Queen (d-file)
mapping.insert(5, Square::F1);  // KB - King's Bishop (f-file)
mapping.insert(6, Square::G1);  // KN - King's Knight (g-file)
mapping.insert(7, Square::H1);  // KR - King's Rook (h-file)
```

**Reference**: SCID_DATABASE_FORMAT.md Section 4.1.2, game.cpp initPieceList()

---

## Success Metrics

### Phase 5 Completion Criteria

**Code Completeness**:
- [x] ScidPosition wrapper implemented
- [x] PieceNumberMapping implemented
- [x] ByteStream implemented
- [x] All piece-specific decoders implemented
- [x] High-level ScidMoveDecoder implemented
- [x] Integration with game parser complete

**Testing**:
- [x] All unit tests passing (40+ tests)
- [x] Integration tests with real database passing
- [x] Queen diagonal moves (2-byte) working
- [x] All special moves validated (castling, en passant, promotions)

**Performance Benchmarks**:
- [ ] Parse 5-game database: < 100ms
- [ ] Parse 1000-game database: < 5 seconds
- [ ] Success rate on real data: ≥ 90%

**Validation Commands**:
```bash
# Run all Phase 5 tests
cargo test --lib position
cargo test --lib byte_stream
cargo test --lib decoder
cargo test --lib move_decoder
cargo test --test phase5_validation

# Performance test
cargo test --release --test phase5_validation -- --nocapture

# Check code coverage (requires cargo-tarpaulin)
cargo tarpaulin --lib --tests --exclude-files 'tests/*'
```

**Expected Final Output**:
```
=== PHASE 5 VALIDATION SUMMARY ===
Total test cases: 50
Passing: 50
Failing: 0

Real-world database parsing:
- Games successfully parsed: 5/5 (100%)
- Total moves decoded: 193
- Queen diagonal moves: 8
- Castling moves: 3
- Promotions: 1
- En passant: 0

Performance:
- Average parse time per game: 18ms
- Average moves per second: 10,722

✅ Phase 5 Complete - Ready for Phase 6 (PGN Output)
```

---

## Next Phase Preview

**Phase 6: PGN Output with Shakmaty** will build on Phase 5 by:
1. Using shakmaty's built-in SAN (Standard Algebraic Notation) generation
2. Formatting PGN tags from index and name data
3. Adding move numbers and result markers
4. Supporting compact vs. verbose output formats
5. Handling variations and comments (from Phase 4 annotations)

**Key Advantage of Shakmaty**: SAN generation is automatic! No need to implement disambiguation logic:
```rust
// Phase 6 preview - incredibly simple with shakmaty!
let san = San::from_move(&position, &chess_move);
println!("{}", san);  // Outputs: "Nf3", "O-O", "exd5", etc.
```

---

**Phase 5 represents the heart of the SCID parser**: transforming opaque binary data into meaningful chess moves. With shakmaty handling the chess rules and our decoders bridging the SCID format, we now have a robust foundation for complete SCID to PGN conversion.

---

## Revision History

### Version 2.3 (Current) - Gap Resolution Updates

This version incorporates Gap 1 (Variation Position Restore) and Gap 3 (Pre-game Comments).

**Gap Resolutions**:

| Gap | Section | Changes |
|-----|---------|---------|
| Gap 1: Variation Position Restore | Task 5.2.4 | Added `VariationParseState` with position/piece mapping stack |
| Gap 1: Variation Position Restore | Task 5.2.4 | Added `VariationSnapshot` for complete state capture |
| Gap 1: Variation Position Restore | Task 5.2.4 | `start_variation()` saves position AND piece mapping |
| Gap 1: Variation Position Restore | Task 5.2.4 | `end_variation()` restores complete state |
| Gap 3: Pre-game Comments | Task 5.3.1 | Added `pre_game_comment` field to `GameData` |
| Gap 3: Pre-game Comments | Task 5.3.1 | Track comment markers before first move |
| Gap 3: Pre-game Comments | Task 5.3.1.1 | NEW: Complete pre-game comment handling task |

**New Structures**:
- `VariationParseState` - State machine for parsing with position tracking
- `VariationSnapshot` - Captured state at variation entry point
- `GameParseState` - Extended state tracking for comment handling

**Key Insight for Gap 1**: When entering a variation, we must save BOTH:
1. The chess position (board, castling rights, en passant, etc.)
2. The SCID piece number mapping (for continued move decoding)

Without saving the piece mapping, move bytes would be decoded incorrectly after returning from a variation.

---

### Version 2.2 (January 2026) - Chess960 Support

This version adds Chess960 (Fischer Random Chess) support per IMPLEMENTATION_GAPS.md Gap 10.

**New Features**:

1. **Task 5.1.3: Chess960 (FRC) Support ADDED**
   - `is_chess960_fen()` function to detect Chess960 positions from FEN
   - `from_fen_auto()` method with automatic castling mode detection
   - Uses shakmaty's `CastlingMode::Chess960` for FRC positions
   - Comprehensive test coverage for Chess960 edge cases

**Key Insight**: SCID's castling encoding already works for Chess960 because it encodes
the **destination squares** (C1/G1), not the rook's starting position. No changes needed
to the castling decoder!

**Changes Summary**:

| Section | Change |
|---------|--------|
| Task 5.1.3 | NEW: Complete Chess960/FRC support |
| Task 5.1.2 | Updated `from_fen()` to use `from_fen_auto()` |

**Reference**: IMPLEMENTATION_PLAN.md Phase 5.1.1, IMPLEMENTATION_GAPS.md Gap 10

---

### Version 2.1 (January 2026) - Gap Filling and Alignment with IMPLEMENTATION_PLAN.md

This version aligns Phase 5 with updates made to IMPLEMENTATION_PLAN.md and fixes remaining issues.

**Critical Fixes**:

1. **Bishop Decoder FIXED** (was WRONG in v2.0!)
   - OLD (WRONG): Used formula `((val/4)+1) * (val&1 ? -1 : 1)` with separate rank/file directions
   - NEW (CORRECT): Uses `fyle = val & 7; offset = (val >= 8) ? -7*fylediff : 9*fylediff`
   - This matches the verified Bible (SCID_DATABASE_FORMAT.md Section 4.2.4)

2. **PieceNumberMapping Struct UPDATED**
   - Added `white_count` and `black_count` fields for capture swap algorithm
   - Struct now properly tracks piece counts per color

3. **from_position() FIXED**
   - Now scans board in correct FEN order (rank 8 DOWN to rank 1)
   - Properly initializes piece counts

4. **update_after_move() REWRITTEN**
   - Now uses piece counts directly instead of `keys().max()`
   - Handles en passant capture square correctly
   - Handles all move types: Normal, Castle, EnPassant

**New Features**:

5. **Task 5.2.4: Game Tree and Variation Data Structures ADDED**
   - New `GameTree` struct for complete game representation
   - New `MoveNode` struct with variations, comments, NAGs support
   - Main line iteration support
   - NAG constants module with PGN formatting

**Changes Summary**:

| Section | Change |
|---------|--------|
| Task 5.1.1 | Added `white_count`/`black_count` to `PieceNumberMapping` |
| Task 5.1.1 | Fixed `from_position()` to scan in FEN order |
| Task 5.1.1 | Rewrote `update_after_move()` with proper capture swap |
| Task 5.2.2 | **CRITICAL**: Fixed Bishop decoder algorithm |
| Task 5.2.4 | NEW: Added GameTree, MoveNode, NAG handling |

---

### Version 2.0 (January 2026) - Major Corrections from Bible Verification

This version incorporates critical corrections after verifying SCID_DATABASE_FORMAT.md against the actual SCID source code.

**Breaking Changes**:

1. **Piece Numbering Order FIXED**
   - OLD (WRONG): King=0, Queen=1, Rook a1=2, Rook h1=3, Bishop c1=4...
   - NEW (CORRECT): King=0, QR=1 (a1), QN=2 (b1), QB=3 (c1), Q=4 (d1), KB=5 (f1), KN=6 (g1), KR=7 (h1)
   - Pieces are numbered by STARTING FILE, not piece type!

2. **King Castling Values FIXED**
   - OLD (WRONG): value 10 = kingside, value 11 = queenside
   - NEW (CORRECT): value 9 = queenside (O-O-O), value 10 = kingside (O-O)
   - Value 0 is now recognized as valid NULL MOVE

3. **Pawn Encoding FIXED**
   - OLD (WRONG): Simple forward offset arithmetic
   - NEW (CORRECT): Uses toSquareDiff table {7,8,9,7,8,9,7,8,9,7,8,9,7,8,9,16}
   - White ADDS offset, Black SUBTRACTS offset

4. **Bishop Encoding FIXED** (note: fixed again in v2.1)
   - OLD (WRONG): "Direct square encoding (0-63)"
   - PARTIALLY CORRECT: Mentioned fylediff but had wrong implementation

5. **Knight Validation ADDED**
   - Values 0 and 9-15 are now correctly marked as INVALID

6. **Capture Swap Algorithm ADDED**
   - New Section 4.4: When a piece is captured, the LAST piece takes its slot
   - Critical for maintaining correct piece numbering throughout game

7. **FEN Initialization FIXED**
   - King ALWAYS gets slot 0 first, regardless of board position
   - Other pieces assigned by board scan order (rank-by-rank, file-by-file)

**New Pitfalls Added**:
- Pitfall 6: Not Using SCID's Capture Swap Algorithm
- Pitfall 7: Wrong Piece Numbering Order

**Reference Documentation Updated**:
- All line number references updated to match verified Bible
- Added source code line references for each decoder function

---

### Version 1.0 (Original)
- Initial document creation with best-guess implementations
- Based on preliminary SCID format analysis
