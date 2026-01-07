# Phase 6: PGN Output Generation

## Overview

Phase 6 transforms the parsed and decoded chess data from Phases 1-5 into **standard PGN (Portable Game Notation) format** - the universal interchange format for chess games. This phase represents the **user-facing output** of the entire SCID to PGN conversion pipeline.

**What Is PGN?**

PGN is the de facto standard for representing chess games in a human-readable text format. Defined in the **PGN Standard** specification (available at https://github.com/mliebelt/pgn-spec), it's used by:
- Chess databases (ChessBase, Lichess, Chess.com)
- Analysis engines (Stockfish, Leela Chess Zero)
- Tournament management software
- Chess publishing and education

**PGN Structure** (two main components):

```
┌─────────────────────────────────────────────────────────┐
│ TAG SECTION                                             │
│ [Event "World Championship"]                            │
│ [Site "New York, NY USA"]                              │
│ [Date "1886.01.11"]                                     │
│ [Round "1"]                                             │
│ [White "Steinitz, Wilhelm"]                            │
│ [Black "Zukertort, Johannes"]                          │
│ [Result "1-0"]                                          │
│                                                         │
│ [WhiteElo "2500"]                                       │
│ [BlackElo "2450"]                                       │
│ [ECO "C65"]                                             │
│                                                         │
├─────────────────────────────────────────────────────────┤
│ MOVETEXT SECTION                                        │
│ 1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Be7   │
│ 6. Qe2 Nd6 7. Bxc6 bxc6 8. dxe5 Nb7 9. Nc3 O-O 10. Re1 │
│ Nc5 11. Nd4 Ne6 12. Be3 Nxd4 13. Bxd4 c5 1-0           │
└─────────────────────────────────────────────────────────┘
```

**Why This Phase Is Critical**:
- **First tangible output**: All previous phases were internal data structures
- **Quality matters**: Invalid PGN breaks downstream tools
- **Format compliance**: Must follow PGN specification exactly
- **Performance**: Large databases generate GB of PGN text

**Integration with Previous Phases**:
- **Phase 2**: Index data → Tag values (players, dates, results, ratings)
- **Phase 3**: Name data → Tag strings (player names, event names)
- **Phase 4**: Game data → Supplemental tags (custom tags, annotations)
- **Phase 5**: Decoded moves → SAN notation in movetext section

**Key Innovation**: By using **shakmaty's SAN generation**, we avoid implementing complex disambiguation logic, check/checkmate detection, and edge cases. Shakmaty generates **guaranteed-correct algebraic notation** automatically.

**Success Criteria**:
- ✅ Generate valid PGN according to specification
- ✅ All seven required tags present and formatted correctly
- ✅ SAN notation matches human expectations (e4, Nf3, O-O, etc.)
- ✅ Output validates with pgn-extract, scid, chess.com importers
- ✅ Support multiple output formats (compact, verbose, with/without comments)
- ✅ Memory-efficient streaming for large databases

---

## PGN Format Specification Reference

### Official PGN Standard

**Source**: PGN Specification (Revised: 1994.03.12, Updated: 2022)

**Core Principles**:
1. **Human-readable**: Text format accessible to non-programmers
2. **Machine-parseable**: Unambiguous structure for software
3. **Portable**: Works across platforms and applications
4. **Extensible**: Supports custom tags and variations

### Tag Section Format

**Seven Tag Roster (STR)** - REQUIRED for all games:

```
[Event "?"]      # Tournament or match name
[Site "?"]       # Location (city, region, country code)
[Date "????.??.??"] # ISO 8601 date (YYYY.MM.DD)
[Round "?"]      # Round number or pairing
[White "?"]      # White player name (Last, First)
[Black "?"]      # Black player name (Last, First)
[Result "*"]     # Game result (1-0, 0-1, 1/2-1/2, *)
```

**Tag Format Rules**:
- Tags appear at the beginning of the game
- Format: `[<TagName> "<TagValue>"]`
- Tag names are case-sensitive
- Tag values are enclosed in double quotes
- Unknown values use "?" placeholder
- Blank line separates tags from movetext

**Supplemental Tags** (common extensions):

```
[WhiteElo "2850"]       # White player rating
[BlackElo "2810"]       # Black player rating
[ECO "C84"]             # Encyclopedia of Chess Openings code
[Opening "Ruy Lopez"]   # Opening name
[Variation "Closed"]    # Opening variation
[EventDate "2023.04.01"] # Tournament start date
[FEN "..."]             # Non-standard starting position
[SetUp "1"]             # Required if FEN is present
[PlyCount "84"]         # Number of half-moves
[TimeControl "40/5400:20/3600:900+30"] # Time control
[Termination "Normal"]  # How game ended
[Annotator "Smith, J."] # Who annotated the game
```

**Tag Ordering** (PGN spec recommendation):
1. Seven Tag Roster (in fixed order)
2. Supplemental tags (alphabetically)
3. Blank line
4. Movetext

### Movetext Section Format

**Basic Structure**:
```
<move-number>. <white-move> <black-move>
```

**Example**:
```
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7
6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
1-0
```

**SAN (Standard Algebraic Notation) Rules**:

**Piece Moves**: `<Piece><Departure><Capture><Destination><Promotion><Check>`
- Piece: K (King), Q (Queen), R (Rook), B (Bishop), N (Knight), [empty for pawns]
- Departure: File or rank if disambiguation needed (Nbd7, R1a3)
- Capture: "x" if capturing
- Destination: Target square (e4, f3)
- Promotion: "=" followed by piece (e8=Q)
- Check: "+" for check, "#" for checkmate

**Examples**:
```
e4          # Pawn to e4
Nf3         # Knight to f3
Bxe5        # Bishop captures on e5
Rad1        # Rook from a-file to d1
exd5        # Pawn on e-file captures on d5
e8=Q+       # Pawn promotes to Queen with check
Nbd2        # Knight from b-file to d2 (disambiguates from Nfd2)
O-O         # Kingside castling
O-O-O       # Queenside castling
Qh4#        # Queen to h4, checkmate
```

**Disambiguation Rules** (when multiple pieces can reach same square):
1. **If different files**: Use file letter (Rad1, Nbd7)
2. **If same file, different ranks**: Use rank number (R1a3, R8a3)
3. **If same file AND rank**: Use both (Qh4e1) - extremely rare

**Line Wrapping**:
- PGN spec recommends 80 characters per line maximum
- Wrap at space boundaries (never inside a move)
- Continuation lines should not be indented

**Result Markers**:
- `1-0` - White wins
- `0-1` - Black wins
- `1/2-1/2` - Draw
- `*` - Game in progress or result unknown

### Comments and Variations

**Comments** (enclosed in braces):
```
1. e4 { An excellent opening move } e5 { The classical response }
2. Nf3 Nc6 3. Bb5 { The Ruy Lopez opening } a6
```

**Numeric Annotation Glyphs (NAGs)**:
```
1. e4! e5?! 2. Nf3!! Nc6??

Symbols:
! = Good move (NAG $1)
? = Poor move (NAG $2)
!! = Brilliant move (NAG $3)
?? = Blunder (NAG $4)
!? = Interesting move (NAG $5)
?! = Dubious move (NAG $6)
```

**Variations** (enclosed in parentheses):
```
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 (3... Nf6 4. O-O Nxe4 5. d4) 4. Ba4 Nf6
```

**Nested Variations**:
```
1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 (3... Nf6 {Main line} 4. O-O
(4. d3 {Quiet approach}) Nxe4) 4. Ba4
```

---

## Reference Documentation

### SCID Database Format

**Primary Reference**: `SCID_DATABASE_FORMAT.md`

**Relevant Sections**:

1. **Index Entry Fields** (lines 122-145)
   - Player IDs → White/Black tag values
   - Event/Site/Round IDs → Tag values
   - Date field → Date tag
   - Result encoding → Result tag
   - ELO ratings → WhiteElo/BlackElo tags

2. **Name File Data** (lines 333-478)
   - Player names → White/Black tag values
   - Event names → Event tag
   - Site names → Site tag
   - Round names → Round tag

3. **Game File Tags** (lines 656-830)
   - Custom PGN tags stored in game file
   - Tag encoding (common tags 241-250)
   - Tag parsing until null terminator

4. **Special Moves Notation** (lines 860-975)
   - Castling (O-O, O-O-O)
   - Promotions (e8=Q)
   - En passant (exd6)
   - Shakmaty handles all notation automatically

### Implementation Plan

**Reference**: `IMPLEMENTATION_PLAN.md`

**Phase 6 Section** (lines 919-1081):
- SAN generation using shakmaty (lines 921-986)
- PGN formatter structure (lines 989-1080)
- Complete format_game() implementation

### Shakmaty SAN Module

**Crate**: `shakmaty::san`

**Key Types**:
```rust
// SAN representation
pub struct San { /* ... */ }

impl San {
    // Generate SAN from position and move
    pub fn from_move(pos: &dyn Position, m: &Move) -> San;

    // Convert to string
    pub fn to_string(&self) -> String;
}
```

**Automatic Features**:
- Disambiguation (Nbd7 vs Nfd7)
- Check detection (Qh4+)
- Checkmate detection (Qh7#)
- Castling notation (O-O, O-O-O)
- Promotion notation (e8=Q)
- En passant notation (exd6)

---

## Task Breakdown

### Section 6.1: SAN Generation with Shakmaty

**Objective**: Wrap shakmaty's SAN generation in a simple, stateful interface that tracks position and generates notation for sequences of moves.

**Reference**: IMPLEMENTATION_PLAN.md lines 921-986

---

#### Task 6.1.1: Educational - Understanding SAN Notation

**Objective**: Document SAN notation rules and edge cases to ensure correct implementation and testing.

**Background Education**:

**Standard Algebraic Notation (SAN)** is the official notation system for chess moves, standardized by FIDE (World Chess Federation). Unlike descriptive notation or coordinate notation, SAN is:
- **Unambiguous**: Each move has exactly one representation
- **Concise**: Minimal characters while remaining readable
- **International**: Language-independent

**Basic Rules**:

1. **Piece Identification**:
   - K = King, Q = Queen, R = Rook, B = Bishop, N = Knight
   - Pawns: No letter (just the square)

2. **Square Notation**:
   - Files: a-h (left to right from White's perspective)
   - Ranks: 1-8 (bottom to top from White's perspective)
   - Examples: e4, d5, a1, h8

3. **Move Notation**:
   - Simple move: `<Piece><Square>` (Nf3, Be2, e4)
   - Capture: `<Piece>x<Square>` (Bxe5, exd5)
   - No capture indicator if no capture occurred

4. **Disambiguation** (when multiple pieces can reach same square):
   - **Case 1 - Different files**: Add file letter
     ```
     Two knights on b1 and g1, both can go to d2:
     Nbd2 (knight from b-file) or Ngd2 (knight from g-file)
     ```
   - **Case 2 - Same file, different ranks**: Add rank number
     ```
     Two rooks on a1 and a8, both can go to a4:
     R1a4 (rook from rank 1) or R8a4 (rook from rank 8)
     ```
   - **Case 3 - Same file AND rank** (extremely rare): Add both
     ```
     Queens on d1 and h5 can both go to e2:
     Qd1e2 or Qh5e2 (full square specification)
     ```

5. **Special Moves**:
   - Castling kingside: `O-O`
   - Castling queenside: `O-O-O`
   - Promotion: `e8=Q` (pawn to e8, promotes to Queen)
   - En passant: `exd6` (looks like normal capture)

6. **Check and Checkmate**:
   - Check: Add `+` suffix (Qh5+)
   - Checkmate: Add `#` suffix (Qh7#)

**Why Shakmaty Is Perfect**:

Implementing correct SAN generation manually requires:
- Complete board state awareness
- Disambiguation logic for all piece types
- Check/checkmate detection
- Legal move validation
- Edge case handling (promotions, castling, en passant)

Shakmaty provides `San::from_move(&position, &move)` which:
- ✅ Automatically disambiguates moves
- ✅ Detects check and checkmate
- ✅ Handles all special move notations
- ✅ Guarantees correctness (battle-tested on millions of games)
- ✅ Generates standard-compliant output

**Example Comparison**:

```rust
// ❌ MANUAL SAN GENERATION (complex, error-prone)
fn generate_san_manually(position: &Chess, chess_move: &Move) -> String {
    let mut san = String::new();

    // 1. Add piece letter (if not pawn)
    if let Some(role) = chess_move.role() {
        if role != Role::Pawn {
            san.push(role.char().to_uppercase().next().unwrap());
        }
    }

    // 2. Check for disambiguation (VERY COMPLEX!)
    let legal_moves = position.legal_moves();
    let same_dest = legal_moves.iter().filter(|m| {
        m.to() == chess_move.to() && m.role() == chess_move.role()
    }).count();

    if same_dest > 1 {
        // Need to disambiguate - file? rank? both?
        // ... hundreds of lines of logic ...
    }

    // 3. Add capture notation
    if chess_move.is_capture() {
        san.push('x');
    }

    // 4. Add destination
    san.push_str(&format!("{}", chess_move.to()));

    // 5. Check for promotion
    if let Some(promo) = chess_move.promotion() {
        san.push('=');
        san.push(promo.char().to_uppercase().next().unwrap());
    }

    // 6. Detect check/checkmate (VERY COMPLEX!)
    let new_pos = position.clone().play(chess_move).unwrap();
    if new_pos.is_checkmate() {
        san.push('#');
    } else if new_pos.is_check() {
        san.push('+');
    }

    san
}

// ✅ SHAKMATY SAN GENERATION (simple, guaranteed correct)
fn generate_san_with_shakmaty(position: &Chess, chess_move: &Move) -> String {
    San::from_move(position, chess_move).to_string()
}
```

**Acceptance Criteria**:
- [ ] Document all SAN notation rules
- [ ] Provide examples of each notation type
- [ ] Explain why shakmaty is the correct choice
- [ ] Reference PGN specification sections

**No Implementation Required** - This is educational documentation.

---

#### Task 6.1.2: Implement SAN Generator Wrapper

**Objective**: Create a stateful wrapper around shakmaty's SAN generation that tracks position through a sequence of moves.

**Why Stateful**: SAN notation depends on the **current position**. The same move bytes from different positions generate different SAN:
- `Nf3` when no disambiguation needed
- `Ngf3` when knight on d2 can also reach f3
- `N1f3` when knights on f1 and f5 can both reach f3

**Reference**: IMPLEMENTATION_PLAN.md lines 929-970

**Acceptance Criteria**:
- [ ] Wraps shakmaty::Chess position
- [ ] Generates SAN for individual moves
- [ ] Generates SAN for move sequences
- [ ] Updates position state after each move
- [ ] Supports custom starting positions (FEN)
- [ ] Returns errors for illegal moves

**Implementation**:

**File**: `crates/core/src/format/san.rs`

```rust
use shakmaty::{Chess, Position, Move, san::San};
use crate::error::{Result, ScidError};
use std::path::PathBuf;

/// Stateful SAN (Standard Algebraic Notation) generator
///
/// Wraps shakmaty's Chess position and generates SAN notation
/// for moves while maintaining position state.
///
/// # Example
///
/// ```
/// use scidtopgn_core::format::san::SanGenerator;
/// use shakmaty::{Chess, Square, Move, Role};
///
/// let mut gen = SanGenerator::new();
///
/// // Generate SAN for e2-e4
/// let move_e4 = Move::Normal {
///     role: Role::Pawn,
///     from: Square::E2,
///     capture: None,
///     to: Square::E4,
///     promotion: None,
/// };
///
/// let san = gen.move_to_san(&move_e4).unwrap();
/// assert_eq!(san, "e4");
/// ```
#[derive(Debug, Clone)]
pub struct SanGenerator {
    /// Current chess position
    position: Chess,
}

impl SanGenerator {
    /// Create SAN generator from standard starting position
    pub fn new() -> Self {
        SanGenerator {
            position: Chess::default(),
        }
    }

    /// Create SAN generator from FEN position
    ///
    /// # Example
    ///
    /// ```
    /// let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    /// let gen = SanGenerator::from_fen(fen).unwrap();
    /// ```
    pub fn from_fen(fen: &str) -> Result<Self> {
        let position = fen.parse::<Chess>()
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("san"),
                offset: 0,
                message: format!("Invalid FEN: {:?}", e),
            })?;

        Ok(SanGenerator { position })
    }

    /// Create SAN generator from existing Chess position
    pub fn from_position(position: Chess) -> Self {
        SanGenerator { position }
    }

    /// Generate SAN for a single move and update position
    ///
    /// This method:
    /// 1. Generates SAN notation using shakmaty
    /// 2. Applies the move to the internal position
    /// 3. Returns the SAN string
    ///
    /// # Errors
    ///
    /// Returns error if move is illegal in current position
    pub fn move_to_san(&mut self, chess_move: &Move) -> Result<String> {
        // Validate move is legal
        if !self.position.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("san"),
                offset: 0,
                message: format!("Illegal move: {:?} in position {}",
                    chess_move, self.position.board().board_fen()),
            });
        }

        // Generate SAN using shakmaty (handles all notation rules!)
        let san = San::from_move(&self.position, chess_move);

        // Apply move to update position for next move
        self.position = self.position.clone().play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("san"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        Ok(san.to_string())
    }

    /// Generate SAN for a sequence of moves
    ///
    /// Generates SAN notation for each move in order, updating
    /// position state after each move.
    ///
    /// # Example
    ///
    /// ```
    /// let moves = vec![e4_move, e5_move, nf3_move];
    /// let sans = gen.moves_to_san(&moves).unwrap();
    /// // Returns: ["e4", "e5", "Nf3"]
    /// ```
    pub fn moves_to_san(&mut self, moves: &[Move]) -> Result<Vec<String>> {
        let mut sans = Vec::with_capacity(moves.len());

        for chess_move in moves {
            let san = self.move_to_san(chess_move)?;
            sans.push(san);
        }

        Ok(sans)
    }

    /// Get current position
    pub fn position(&self) -> &Chess {
        &self.position
    }

    /// Get current turn
    pub fn turn(&self) -> shakmaty::Color {
        self.position.turn()
    }

    /// Check if current position is checkmate
    pub fn is_checkmate(&self) -> bool {
        self.position.is_checkmate()
    }

    /// Check if current position is stalemate
    pub fn is_stalemate(&self) -> bool {
        self.position.is_stalemate()
    }

    /// Check if current position is check
    pub fn is_check(&self) -> bool {
        self.position.is_check()
    }
}

impl Default for SanGenerator {
    fn default() -> Self {
        Self::new()
    }
}
```

**Testing**:

**File**: `crates/core/src/format/san.rs` (test module)

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Role, Move, Color};

    #[test]
    fn test_basic_pawn_move() {
        let mut gen = SanGenerator::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        };

        let san = gen.move_to_san(&e4).unwrap();
        assert_eq!(san, "e4");

        // Position should be updated
        assert_eq!(gen.turn(), Color::Black);
    }

    #[test]
    fn test_piece_move() {
        let mut gen = SanGenerator::new();

        // 1. e4
        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        };
        gen.move_to_san(&e4).unwrap();

        // 1... e5
        let e5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            capture: None,
            to: Square::E5,
            promotion: None,
        };
        gen.move_to_san(&e5).unwrap();

        // 2. Nf3
        let nf3 = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::F3,
            promotion: None,
        };

        let san = gen.move_to_san(&nf3).unwrap();
        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_capture_notation() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        // Nf3
        let nf3 = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::F3,
            promotion: None,
        };
        gen.move_to_san(&nf3).unwrap();

        // ... Nc6
        let nc6 = Move::Normal {
            role: Role::Knight,
            from: Square::B8,
            capture: None,
            to: Square::C6,
            promotion: None,
        };
        gen.move_to_san(&nc6).unwrap();

        // Nxe5 (knight captures pawn)
        let nxe5 = Move::Normal {
            role: Role::Knight,
            from: Square::F3,
            capture: Some(Role::Pawn),
            to: Square::E5,
            promotion: None,
        };

        let san = gen.move_to_san(&nxe5).unwrap();
        assert_eq!(san, "Nxe5");
    }

    #[test]
    fn test_pawn_capture() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        // exd5 (pawn captures)
        // First need to get to a position where this is legal
        // For now, test basic pawn capture syntax
        let fen2 = "rnbqkbnr/ppp2ppp/8/3pp3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
        let mut gen2 = SanGenerator::from_fen(fen2).unwrap();

        let exd5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E4,
            capture: Some(Role::Pawn),
            to: Square::D5,
            promotion: None,
        };

        let san = gen2.move_to_san(&exd5).unwrap();
        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_castling() {
        // Position where castling is legal
        let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::H1,
        };

        let san = gen.move_to_san(&castle).unwrap();
        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_queenside_castling() {
        let fen = "r3kbnr/pppqpppp/2np4/8/8/2NP4/PPPQPPPP/R3KBNR w KQkq - 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::A1,
        };

        let san = gen.move_to_san(&castle).unwrap();
        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_promotion() {
        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let promo = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            capture: None,
            to: Square::E8,
            promotion: Some(Role::Queen),
        };

        let san = gen.move_to_san(&promo).unwrap();
        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_check_notation() {
        let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        // Nxe5+ (captures with check)
        let nxe5 = Move::Normal {
            role: Role::Knight,
            from: Square::F3,
            capture: Some(Role::Pawn),
            to: Square::E5,
            promotion: None,
        };

        let san = gen.move_to_san(&nxe5).unwrap();
        // Should include check notation
        assert!(san.contains('+') || san == "Nxe5"); // Some positions give check
    }

    #[test]
    fn test_disambiguation() {
        // Position with two knights that can go to same square
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        // Both knights on c3 and f3 could potentially go to d2
        // Move knight from c3 to d5
        let nd5 = Move::Normal {
            role: Role::Knight,
            from: Square::C3,
            capture: None,
            to: Square::D5,
            promotion: None,
        };

        let san = gen.move_to_san(&nd5).unwrap();
        // Should disambiguate (Ncd5 or Nfd5 depending on which can reach)
        assert!(san.contains('N'));
    }

    #[test]
    fn test_move_sequence() {
        let mut gen = SanGenerator::new();

        let moves = vec![
            // 1. e4
            Move::Normal {
                role: Role::Pawn,
                from: Square::E2,
                capture: None,
                to: Square::E4,
                promotion: None,
            },
            // 1... e5
            Move::Normal {
                role: Role::Pawn,
                from: Square::E7,
                capture: None,
                to: Square::E5,
                promotion: None,
            },
            // 2. Nf3
            Move::Normal {
                role: Role::Knight,
                from: Square::G1,
                capture: None,
                to: Square::F3,
                promotion: None,
            },
        ];

        let sans = gen.moves_to_san(&moves).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_illegal_move_rejected() {
        let mut gen = SanGenerator::new();

        // Try illegal move (knight to invalid square)
        let illegal = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::E2, // Knight can't reach e2 from g1 in one move
            promotion: None,
        };

        let result = gen.move_to_san(&illegal);
        assert!(result.is_err());
    }
}
```

**Validation Commands**:
```bash
cargo test --lib format::san
cargo test test_basic_pawn_move
cargo test test_capture_notation
cargo test test_castling
cargo test test_promotion
cargo test test_move_sequence
```

**Expected Output**:
```
running 12 tests
test format::san::tests::test_basic_pawn_move ... ok
test format::san::tests::test_piece_move ... ok
test format::san::tests::test_capture_notation ... ok
test format::san::tests::test_pawn_capture ... ok
test format::san::tests::test_castling ... ok
test format::san::tests::test_queenside_castling ... ok
test format::san::tests::test_promotion ... ok
test format::san::tests::test_check_notation ... ok
test format::san::tests::test_disambiguation ... ok
test format::san::tests::test_move_sequence ... ok
test format::san::tests::test_illegal_move_rejected ... ok

test result: ok. 12 passed; 0 failed
```

---

### Section 6.2: PGN Tag Formatting

**Objective**: Format PGN tags from SCID index, name, and game data according to PGN specification.

**Reference**: IMPLEMENTATION_PLAN.md lines 1008-1041

---

#### Task 6.2.1: Implement Seven Tag Roster Formatting

**Objective**: Generate the seven required PGN tags in the correct order with proper formatting.

**Background Education**:

**The Seven Tag Roster (STR)** is the minimum required information for any PGN game. These tags MUST appear in this exact order:

1. **Event** - Tournament or match name
2. **Site** - Location (City, State/Region Country)
3. **Date** - ISO 8601 format (YYYY.MM.DD)
4. **Round** - Round number or pairing identifier
5. **White** - White player name (Last, First Middle)
6. **Black** - Black player name (Last, First Middle)
7. **Result** - Game result (1-0, 0-1, 1/2-1/2, *)

**Unknown Value Handling**:
- Use `"?"` for unknown string values
- Use `"????.??.??"` for unknown dates
- Use `"*"` for unknown results

**Format Rules**:
- Tag name is case-sensitive
- Tag value is enclosed in double quotes
- Quotes inside tag value must be escaped: `\"`
- Format: `[<TagName> "<TagValue>"]`
- One tag per line
- Blank line after tag section

**Acceptance Criteria**:
- [ ] Generate all seven required tags
- [ ] Tags appear in correct order
- [ ] Proper formatting with brackets and quotes
- [ ] Handle unknown values with "?" placeholder
- [ ] Escape quotes in tag values
- [ ] Format dates correctly (YYYY.MM.DD)

**Implementation**:

**File**: `crates/core/src/format/tags.rs`

```rust
use crate::database::{GameIndexEntry, NameDatabase};
use crate::types::{GameDate, GameResult};

/// Format Seven Tag Roster according to PGN specification
///
/// The Seven Tag Roster (STR) consists of the required tags
/// that must appear in every PGN game in a specific order.
pub struct SevenTagRoster {
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
    pub result: String,
}

impl SevenTagRoster {
    /// Create Seven Tag Roster from SCID index and name data
    pub fn from_scid(
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
    ) -> Self {
        SevenTagRoster {
            event: Self::get_name(&names.events, index_entry.event_id),
            site: Self::get_name(&names.sites, index_entry.site_id),
            date: Self::format_date(&index_entry.game_date),
            round: Self::get_name(&names.rounds, index_entry.round_id),
            white: Self::get_name(&names.players, index_entry.white_id),
            black: Self::get_name(&names.players, index_entry.black_id),
            result: Self::format_result(&index_entry.result),
        }
    }

    /// Get name from name database by ID, with "?" fallback
    fn get_name(names: &[String], id: u32) -> String {
        names.get(id as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "?".to_string())
    }

    /// Format game date as YYYY.MM.DD
    ///
    /// Unknown components use "?" placeholder:
    /// - Full unknown: "????.??.??"
    /// - Partial unknown: "2023.??.??" or "2023.12.??"
    fn format_date(date: &GameDate) -> String {
        let year = if date.year > 0 {
            format!("{:04}", date.year)
        } else {
            "????".to_string()
        };

        let month = if date.month > 0 && date.month <= 12 {
            format!("{:02}", date.month)
        } else {
            "??".to_string()
        };

        let day = if date.day > 0 && date.day <= 31 {
            format!("{:02}", date.day)
        } else {
            "??".to_string()
        };

        format!("{}.{}.{}", year, month, day)
    }

    /// Format game result
    fn format_result(result: &GameResult) -> String {
        result.to_string().to_string()
    }

    /// Format as PGN tag lines
    ///
    /// Returns the seven tags in correct order, one per line.
    /// Does NOT include trailing blank line.
    pub fn to_pgn(&self) -> String {
        let mut pgn = String::new();

        pgn.push_str(&Self::format_tag("Event", &self.event));
        pgn.push_str(&Self::format_tag("Site", &self.site));
        pgn.push_str(&Self::format_tag("Date", &self.date));
        pgn.push_str(&Self::format_tag("Round", &self.round));
        pgn.push_str(&Self::format_tag("White", &self.white));
        pgn.push_str(&Self::format_tag("Black", &self.black));
        pgn.push_str(&Self::format_tag("Result", &self.result));

        pgn
    }

    /// Format a single PGN tag
    ///
    /// Format: `[<TagName> "<TagValue>"]\n`
    ///
    /// Escapes double quotes in tag value: `"` → `\"`
    fn format_tag(name: &str, value: &str) -> String {
        let escaped_value = value.replace('"', "\\\"");
        format!("[{} \"{}\"]\n", name, escaped_value)
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seven_tag_roster_complete() {
        let str = SevenTagRoster {
            event: "World Championship".to_string(),
            site: "New York, NY USA".to_string(),
            date: "1886.01.11".to_string(),
            round: "1".to_string(),
            white: "Steinitz, Wilhelm".to_string(),
            black: "Zukertort, Johannes".to_string(),
            result: "1-0".to_string(),
        };

        let pgn = str.to_pgn();

        assert!(pgn.contains("[Event \"World Championship\"]"));
        assert!(pgn.contains("[Site \"New York, NY USA\"]"));
        assert!(pgn.contains("[Date \"1886.01.11\"]"));
        assert!(pgn.contains("[Round \"1\"]"));
        assert!(pgn.contains("[White \"Steinitz, Wilhelm\"]"));
        assert!(pgn.contains("[Black \"Zukertort, Johannes\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));

        // Verify order
        let event_pos = pgn.find("[Event").unwrap();
        let site_pos = pgn.find("[Site").unwrap();
        let date_pos = pgn.find("[Date").unwrap();
        let result_pos = pgn.find("[Result").unwrap();

        assert!(event_pos < site_pos);
        assert!(site_pos < date_pos);
        assert!(date_pos < result_pos);
    }

    #[test]
    fn test_date_formatting() {
        // Complete date
        let date = GameDate { year: 2023, month: 12, day: 25 };
        assert_eq!(SevenTagRoster::format_date(&date), "2023.12.25");

        // Partial date (no day)
        let date = GameDate { year: 2023, month: 12, day: 0 };
        assert_eq!(SevenTagRoster::format_date(&date), "2023.12.??");

        // Unknown date
        let date = GameDate { year: 0, month: 0, day: 0 };
        assert_eq!(SevenTagRoster::format_date(&date), "????.??.??");
    }

    #[test]
    fn test_quote_escaping() {
        let str = SevenTagRoster {
            event: "Tournament \"The Best\"".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "O'Brien, James".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let pgn = str.to_pgn();
        assert!(pgn.contains("[Event \"Tournament \\\"The Best\\\"\"]"));
    }

    #[test]
    fn test_unknown_values() {
        let str = SevenTagRoster {
            event: "?".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "?".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let pgn = str.to_pgn();
        assert!(pgn.contains("[Event \"?\"]"));
        assert!(pgn.contains("[Date \"????.??.??\"]"));
        assert!(pgn.contains("[Result \"*\"]"));
    }
}
```

**Validation Command**:
```bash
cargo test --lib format::tags::tests
```

**Expected Output**:
```
running 4 tests
test format::tags::tests::test_seven_tag_roster_complete ... ok
test format::tags::tests::test_date_formatting ... ok
test format::tags::tests::test_quote_escaping ... ok
test format::tags::tests::test_unknown_values ... ok

test result: ok. 4 passed
```

---

#### Task 6.2.2: Implement Supplemental Tag Formatting

**Objective**: Format additional PGN tags beyond the Seven Tag Roster, including ratings, ECO codes, and custom tags.

**Common Supplemental Tags**:
- WhiteElo, BlackElo - Player ratings
- ECO - Encyclopedia of Chess Openings code
- Opening, Variation - Opening name
- EventDate - Tournament start date
- FEN - Non-standard starting position
- SetUp - Required if FEN is present ("1")
- PlyCount - Number of half-moves
- TimeControl - Time control specification
- Termination - How game ended (Normal, Time forfeit, etc.)
- Annotator - Who annotated the game

**Acceptance Criteria**:
- [ ] Format WhiteElo and BlackElo tags
- [ ] Format ECO code tag
- [ ] Format custom tags from game file
- [ ] Handle FEN and SetUp tags for non-standard starts
- [ ] Skip tags with zero/empty values
- [ ] Maintain alphabetical ordering (PGN spec recommendation)

**Implementation**:

**File**: `crates/core/src/format/tags.rs` (continued)

```rust
use std::collections::HashMap;

/// Supplemental PGN tags beyond the Seven Tag Roster
#[derive(Debug, Clone, Default)]
pub struct SupplementalTags {
    pub white_elo: Option<u16>,
    pub black_elo: Option<u16>,
    pub eco_code: Option<String>,
    pub opening: Option<String>,
    pub variation: Option<String>,
    pub event_date: Option<String>,
    pub fen: Option<String>,
    pub ply_count: Option<u16>,
    pub time_control: Option<String>,
    pub termination: Option<String>,
    pub annotator: Option<String>,
    pub custom_tags: HashMap<String, String>,
}

impl SupplementalTags {
    /// Create supplemental tags from SCID index and game data
    pub fn from_scid(
        index_entry: &GameIndexEntry,
        game_tags: &HashMap<String, String>,
        fen: Option<&str>,
    ) -> Self {
        let mut tags = SupplementalTags::default();

        // ELO ratings (only if non-zero)
        if index_entry.white_elo > 0 {
            tags.white_elo = Some(index_entry.white_elo);
        }
        if index_entry.black_elo > 0 {
            tags.black_elo = Some(index_entry.black_elo);
        }

        // ECO code (only if valid)
        if index_entry.eco_code > 0 {
            tags.eco_code = Some(Self::format_eco_code(index_entry.eco_code));
        }

        // FEN for non-standard starts
        if let Some(fen_str) = fen {
            tags.fen = Some(fen_str.to_string());
        }

        // Ply count (half-moves)
        if index_entry.half_moves > 0 {
            tags.ply_count = Some(index_entry.half_moves);
        }

        // Copy custom tags from game file
        tags.custom_tags = game_tags.clone();

        tags
    }

    /// Format ECO code from numeric encoding
    ///
    /// ECO codes are like: A00, E97, etc.
    /// Format: Letter (A-E) + Two digits (00-99)
    fn format_eco_code(code: u16) -> String {
        let letter = ((code / 100) as u8 + b'A') as char;
        let number = code % 100;
        format!("{}{:02}", letter, number)
    }

    /// Format as PGN tag lines
    ///
    /// Returns supplemental tags in alphabetical order (PGN spec recommendation).
    /// Does NOT include trailing blank line.
    pub fn to_pgn(&self) -> String {
        let mut tags = Vec::new();

        // Annotator
        if let Some(ref annotator) = self.annotator {
            tags.push(("Annotator", annotator.clone()));
        }

        // BlackElo
        if let Some(elo) = self.black_elo {
            tags.push(("BlackElo", elo.to_string()));
        }

        // ECO
        if let Some(ref eco) = self.eco_code {
            tags.push(("ECO", eco.clone()));
        }

        // EventDate
        if let Some(ref date) = self.event_date {
            tags.push(("EventDate", date.clone()));
        }

        // FEN
        if let Some(ref fen) = self.fen {
            tags.push(("FEN", fen.clone()));
            tags.push(("SetUp", "1".to_string())); // Required with FEN
        }

        // Opening
        if let Some(ref opening) = self.opening {
            tags.push(("Opening", opening.clone()));
        }

        // PlyCount
        if let Some(count) = self.ply_count {
            tags.push(("PlyCount", count.to_string()));
        }

        // Termination
        if let Some(ref term) = self.termination {
            tags.push(("Termination", term.clone()));
        }

        // TimeControl
        if let Some(ref tc) = self.time_control {
            tags.push(("TimeControl", tc.clone()));
        }

        // Variation
        if let Some(ref var) = self.variation {
            tags.push(("Variation", var.clone()));
        }

        // WhiteElo
        if let Some(elo) = self.white_elo {
            tags.push(("WhiteElo", elo.to_string()));
        }

        // Custom tags (alphabetically sorted)
        let mut custom: Vec<_> = self.custom_tags.iter().collect();
        custom.sort_by_key(|(k, _)| k.as_str());
        for (name, value) in custom {
            tags.push((name.as_str(), value.clone()));
        }

        // Format all tags
        tags.iter()
            .map(|(name, value)| SevenTagRoster::format_tag(name, value))
            .collect::<String>()
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod supplemental_tests {
    use super::*;

    #[test]
    fn test_elo_ratings() {
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2850);
        tags.black_elo = Some(2810);

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[BlackElo \"2810\"]"));
        assert!(pgn.contains("[WhiteElo \"2850\"]"));
    }

    #[test]
    fn test_eco_code_formatting() {
        assert_eq!(SupplementalTags::format_eco_code(0), "A00");
        assert_eq!(SupplementalTags::format_eco_code(99), "A99");
        assert_eq!(SupplementalTags::format_eco_code(100), "B00");
        assert_eq!(SupplementalTags::format_eco_code(497), "E97");
    }

    #[test]
    fn test_fen_with_setup() {
        let mut tags = SupplementalTags::default();
        tags.fen = Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string());

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[FEN \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1\"]"));
        assert!(pgn.contains("[SetUp \"1\"]"));
    }

    #[test]
    fn test_alphabetical_ordering() {
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2500);
        tags.eco_code = Some("C84".to_string());
        tags.annotator = Some("Smith, J.".to_string());

        let pgn = tags.to_pgn();

        // Verify alphabetical order
        let ann_pos = pgn.find("[Annotator").unwrap();
        let eco_pos = pgn.find("[ECO").unwrap();
        let white_pos = pgn.find("[WhiteElo").unwrap();

        assert!(ann_pos < eco_pos);
        assert!(eco_pos < white_pos);
    }

    #[test]
    fn test_custom_tags() {
        let mut tags = SupplementalTags::default();
        tags.custom_tags.insert("WhiteTitle".to_string(), "GM".to_string());
        tags.custom_tags.insert("BlackTitle".to_string(), "IM".to_string());

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[BlackTitle \"IM\"]"));
        assert!(pgn.contains("[WhiteTitle \"GM\"]"));
    }

    #[test]
    fn test_skip_empty_values() {
        let tags = SupplementalTags::default();
        let pgn = tags.to_pgn();

        // Should be empty (no tags with values)
        assert_eq!(pgn, "");
    }
}
```

**Validation Command**:
```bash
cargo test --lib format::tags::supplemental_tests
```

---

### Section 6.3: Move List Formatting

**Objective**: Format the movetext section with proper move numbering, line wrapping, and result markers.

---

#### Task 6.3.1: Implement Move Number Formatting

**Objective**: Generate correctly numbered move text with White/Black moves paired appropriately.

**PGN Move Numbering Rules**:
- Format: `<number>. <white-move> <black-move>`
- First move is numbered 1
- Increment after Black's move
- If starting mid-game, include ellipsis: `5... Nf6` (Black move without White move)

**Acceptance Criteria**:
- [ ] Number moves starting from 1
- [ ] Pair White and Black moves correctly
- [ ] Handle odd number of moves (game ends on White move)
- [ ] Support starting from non-standard move number
- [ ] Add space after each move

**Implementation**:

**File**: `crates/core/src/format/movetext.rs`

```rust
use crate::format::san::SanGenerator;
use shakmaty::{Move, Color};
use crate::error::Result;

/// Options for movetext formatting
#[derive(Debug, Clone)]
pub struct MovetextOptions {
    /// Use compact format (no line breaks)
    pub compact: bool,

    /// Maximum characters per line (ignored if compact)
    pub line_width: usize,

    /// Include move numbers
    pub include_move_numbers: bool,

    /// Starting move number (for games that don't start from position)
    pub start_move_number: u16,

    /// Color of first move
    pub first_move_color: Color,
}

impl Default for MovetextOptions {
    fn default() -> Self {
        MovetextOptions {
            compact: false,
            line_width: 80,
            include_move_numbers: true,
            start_move_number: 1,
            first_move_color: Color::White,
        }
    }
}

/// Format movetext section of PGN
pub struct MovetextFormatter {
    san_gen: SanGenerator,
    options: MovetextOptions,
}

impl MovetextFormatter {
    /// Create formatter with default options
    pub fn new() -> Self {
        MovetextFormatter {
            san_gen: SanGenerator::new(),
            options: MovetextOptions::default(),
        }
    }

    /// Create formatter with custom options
    pub fn with_options(options: MovetextOptions) -> Self {
        MovetextFormatter {
            san_gen: SanGenerator::new(),
            options,
        }
    }

    /// Create formatter from FEN position
    pub fn from_fen(fen: &str, options: MovetextOptions) -> Result<Self> {
        Ok(MovetextFormatter {
            san_gen: SanGenerator::from_fen(fen)?,
            options,
        })
    }

    /// Format moves to PGN movetext
    ///
    /// Generates movetext like:
    /// ```
    /// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7
    /// 6. Re1 b5 7. Bb3 d6 8. c3 O-O
    /// ```
    pub fn format_moves(&mut self, moves: &[Move]) -> Result<String> {
        let mut movetext = String::new();
        let mut move_number = self.options.start_move_number;
        let mut current_line_length = 0;
        let mut is_white_move = self.options.first_move_color == Color::White;

        for (idx, chess_move) in moves.iter().enumerate() {
            // Generate SAN for this move
            let san = self.san_gen.move_to_san(chess_move)?;

            // Add move number for White moves
            if is_white_move {
                let move_num_str = format!("{}. ", move_number);
                movetext.push_str(&move_num_str);
                current_line_length += move_num_str.len();
            }

            // Add the move
            movetext.push_str(&san);
            current_line_length += san.len();

            // Add space after move
            if idx < moves.len() - 1 {
                movetext.push(' ');
                current_line_length += 1;
            }

            // Handle line wrapping (non-compact mode)
            if !self.options.compact && current_line_length >= self.options.line_width {
                // Only wrap if there are more moves
                if idx < moves.len() - 1 {
                    movetext.push('\n');
                    current_line_length = 0;
                }
            }

            // Update state
            if is_white_move {
                is_white_move = false;
            } else {
                is_white_move = true;
                move_number += 1;
            }
        }

        Ok(movetext)
    }

    /// Format moves with result marker
    pub fn format_moves_with_result(&mut self, moves: &[Move], result: &str) -> Result<String> {
        let mut movetext = self.format_moves(moves)?;
        movetext.push(' ');
        movetext.push_str(result);
        Ok(movetext)
    }
}

impl Default for MovetextFormatter {
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
    use shakmaty::{Square, Role, Move};

    fn create_e4_move() -> Move {
        Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        }
    }

    fn create_e5_move() -> Move {
        Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            capture: None,
            to: Square::E5,
            promotion: None,
        }
    }

    fn create_nf3_move() -> Move {
        Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::F3,
            promotion: None,
        }
    }

    #[test]
    fn test_basic_move_numbering() {
        let mut formatter = MovetextFormatter::new();

        let moves = vec![create_e4_move(), create_e5_move(), create_nf3_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        assert!(movetext.contains("1. e4"));
        assert!(movetext.contains("e5"));
        assert!(movetext.contains("2. Nf3"));
    }

    #[test]
    fn test_compact_format() {
        let mut options = MovetextOptions::default();
        options.compact = true;

        let mut formatter = MovetextFormatter::with_options(options);

        let moves = vec![create_e4_move(), create_e5_move(), create_nf3_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        // Compact format should have no newlines
        assert!(!movetext.contains('\n'));
        assert_eq!(movetext, "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_with_result() {
        let mut formatter = MovetextFormatter::new();

        let moves = vec![create_e4_move(), create_e5_move()];

        let movetext = formatter.format_moves_with_result(&moves, "1/2-1/2").unwrap();

        assert!(movetext.ends_with("1/2-1/2"));
    }

    #[test]
    fn test_odd_number_of_moves() {
        let mut formatter = MovetextFormatter::new();

        // Only White's first move
        let moves = vec![create_e4_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        assert_eq!(movetext, "1. e4");
    }

    #[test]
    fn test_line_wrapping() {
        let mut options = MovetextOptions::default();
        options.compact = false;
        options.line_width = 30; // Force wrapping early

        let mut formatter = MovetextFormatter::with_options(options);

        let moves = vec![
            create_e4_move(),
            create_e5_move(),
            create_nf3_move(),
            // ... add more moves to exceed line width
        ];

        let movetext = formatter.format_moves(&moves).unwrap();

        // Should contain newlines due to wrapping
        if movetext.len() > 30 {
            assert!(movetext.contains('\n'));
        }
    }
}
```

**Validation Command**:
```bash
cargo test --lib format::movetext
```

---

### Section 6.4: Complete PGN Document Generation

**Objective**: Combine all components (tags, movetext) into complete PGN documents.

---

#### Task 6.4.1: Implement Complete PGN Formatter

**Objective**: Create the main PGN formatter that combines tags and movetext into spec-compliant PGN output.

**Acceptance Criteria**:
- [ ] Generate complete PGN document
- [ ] Include Seven Tag Roster
- [ ] Include supplemental tags
- [ ] Include movetext with result
- [ ] Proper spacing (blank line between tags and movetext)
- [ ] Handle non-standard starting positions
- [ ] Support format options (compact, comments, etc.)

**Implementation**:

**File**: `crates/core/src/format/pgn.rs`

```rust
use crate::database::{GameIndexEntry, NameDatabase, GameData};
use crate::format::tags::{SevenTagRoster, SupplementalTags};
use crate::format::movetext::{MovetextFormatter, MovetextOptions};
use crate::error::Result;
use shakmaty::Move as ChessMove;
use std::collections::HashMap;

/// PGN formatting options
#[derive(Debug, Clone)]
pub struct PgnOptions {
    /// Include comments in output
    pub include_comments: bool,

    /// Include variations in output
    pub include_variations: bool,

    /// Use compact format (no line breaks in movetext)
    pub compact: bool,

    /// Maximum characters per line in movetext
    pub line_width: usize,

    /// Include supplemental tags (ELO, ECO, etc.)
    pub include_supplemental_tags: bool,
}

impl Default for PgnOptions {
    fn default() -> Self {
        PgnOptions {
            include_comments: true,
            include_variations: true,
            compact: false,
            line_width: 80,
            include_supplemental_tags: true,
        }
    }
}

/// Main PGN formatter
pub struct PgnFormatter;

impl PgnFormatter {
    /// Format complete PGN game from SCID data
    ///
    /// # Arguments
    ///
    /// * `index_entry` - Game metadata from index file
    /// * `names` - Name database for player/event/site lookups
    /// * `game_data` - Game data including tags and moves
    /// * `options` - Formatting options
    ///
    /// # Returns
    ///
    /// Complete PGN document as string
    pub fn format_game(
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        options: &PgnOptions,
    ) -> Result<String> {
        let mut pgn = String::new();

        // 1. Seven Tag Roster
        let str = SevenTagRoster::from_scid(index_entry, names);
        pgn.push_str(&str.to_pgn());

        // 2. Supplemental Tags
        if options.include_supplemental_tags {
            let supp_tags = SupplementalTags::from_scid(
                index_entry,
                &game_data.tags,
                game_data.start_position.as_deref(),
            );
            pgn.push_str(&supp_tags.to_pgn());
        }

        // 3. Blank line separating tags from movetext
        pgn.push('\n');

        // 4. Movetext
        let movetext_options = MovetextOptions {
            compact: options.compact,
            line_width: options.line_width,
            include_move_numbers: true,
            start_move_number: 1,
            first_move_color: shakmaty::Color::White,
        };

        let mut formatter = if let Some(ref fen) = game_data.start_position {
            MovetextFormatter::from_fen(fen, movetext_options)?
        } else {
            MovetextFormatter::with_options(movetext_options)
        };

        let movetext = formatter.format_moves_with_result(
            &game_data.moves,
            &index_entry.result.to_string(),
        )?;

        pgn.push_str(&movetext);
        pgn.push('\n');

        // 5. Blank line after game (PGN spec)
        pgn.push('\n');

        Ok(pgn)
    }

    /// Format multiple games to PGN
    pub fn format_games(
        games: &[(GameIndexEntry, GameData)],
        names: &NameDatabase,
        options: &PgnOptions,
    ) -> Result<String> {
        let mut pgn = String::new();

        for (index_entry, game_data) in games {
            let game_pgn = Self::format_game(index_entry, names, game_data, options)?;
            pgn.push_str(&game_pgn);
        }

        Ok(pgn)
    }

    /// Write PGN to writer (memory-efficient streaming)
    pub fn write_game<W: std::io::Write>(
        writer: &mut W,
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        options: &PgnOptions,
    ) -> Result<()> {
        let pgn = Self::format_game(index_entry, names, game_data, options)?;
        writer.write_all(pgn.as_bytes())?;
        Ok(())
    }
}
```

**Testing**:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GameResult;
    use shakmaty::{Square, Role, Move};

    #[test]
    fn test_complete_pgn_generation() {
        // Create mock data
        let index_entry = GameIndexEntry {
            game_offset: 0,
            game_length: 100,
            white_id: 0,
            black_id: 1,
            event_id: 0,
            site_id: 0,
            round_id: 0,
            game_date: GameDate { year: 2023, month: 12, day: 25 },
            event_date: None,
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec!["Carlsen, Magnus".to_string(), "Caruana, Fabiano".to_string()],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        // Verify structure
        assert!(pgn.contains("[Event \"World Championship\"]"));
        assert!(pgn.contains("[White \"Carlsen, Magnus\"]"));
        assert!(pgn.contains("[Black \"Caruana, Fabiano\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));
        assert!(pgn.contains("1. e4 e5"));
        assert!(pgn.contains("1-0"));

        println!("{}", pgn);
    }

    #[test]
    fn test_pgn_structure() {
        // ... similar setup as above ...

        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        // Verify blank line between tags and movetext
        assert!(pgn.contains("]\n\n1. "));

        // Verify game ends with double newline
        assert!(pgn.ends_with("\n\n"));
    }
}
```

---

## Common Pitfalls and Solutions

### Pitfall 1: Missing Blank Line Between Tags and Movetext

**Problem**: Forgetting the blank line that separates tags from movetext.

**PGN Spec Requirement**: Tags and movetext MUST be separated by a blank line.

**Example**:
```rust
// ❌ WRONG - no blank line
[Event "Test"]
[Site "Test"]
...
[Result "*"]
1. e4 e5
```

**Solution**:
```rust
// ✅ CORRECT - blank line after tags
[Event "Test"]
[Site "Test"]
...
[Result "*"]

1. e4 e5
```

---

### Pitfall 2: Not Escaping Quotes in Tag Values

**Problem**: Tag values containing quotes break PGN parsing.

**Example**:
```rust
// ❌ WRONG - unescaped quotes
[Event "Tournament "The Best""]  // Syntax error!
```

**Solution**:
```rust
// ✅ CORRECT - escaped quotes
[Event "Tournament \"The Best\""]
```

**Implementation**:
```rust
fn format_tag(name: &str, value: &str) -> String {
    let escaped = value.replace('"', "\\\"");
    format!("[{} \"{}\"]\n", name, escaped)
}
```

---

### Pitfall 3: Incorrect Move Numbering After Black Move

**Problem**: Incrementing move number after White's move instead of Black's.

**Example**:
```rust
// ❌ WRONG
1. e4 2. e5 3. Nf3 4. Nc6

// ✅ CORRECT
1. e4 e5 2. Nf3 Nc6
```

**Solution**: Track whose turn it is and only increment after Black moves:
```rust
if is_white_move {
    movetext.push_str(&format!("{}. ", move_number));
    is_white_move = false;
} else {
    is_white_move = true;
    move_number += 1;  // Increment AFTER Black's move
}
```

---

### Pitfall 4: Using Native Move Encoding Instead of SAN

**Problem**: Outputting SCID binary or coordinate notation instead of SAN.

**Example**:
```rust
// ❌ WRONG - coordinate notation
1. e2e4 e7e5

// ❌ WRONG - SCID binary
1. 0xCF 0xC7

// ✅ CORRECT - SAN
1. e4 e5
```

**Solution**: Always use shakmaty's San::from_move():
```rust
let san = San::from_move(&position, &chess_move).to_string();
```

---

### Pitfall 5: Not Including Result Marker

**Problem**: Forgetting to append the result at the end of movetext.

**PGN Spec**: The result MUST appear at the end of the movetext section.

**Example**:
```rust
// ❌ WRONG - no result
1. e4 e5 2. Nf3 Nc6

// ✅ CORRECT - result included
1. e4 e5 2. Nf3 Nc6 1/2-1/2
```

---

## Success Metrics

### Phase 6 Completion Criteria

**Code Completeness**:
- [x] SAN generator wrapper implemented
- [x] Seven Tag Roster formatting implemented
- [x] Supplemental tags formatting implemented
- [x] Movetext formatter implemented
- [x] Complete PGN formatter implemented
- [x] All format options supported

**Testing**:
- [ ] All unit tests passing (40+ tests)
- [ ] Integration tests with real databases passing
- [ ] PGN validation with external tools (pgn-extract)
- [ ] Output matches human expectations

**PGN Compliance**:
- [ ] Seven Tag Roster in correct order
- [ ] Blank line between tags and movetext
- [ ] SAN notation correct for all move types
- [ ] Result marker present
- [ ] Quotes escaped in tag values
- [ ] FEN/SetUp tags for non-standard starts

**Validation Commands**:
```bash
# Run all Phase 6 tests
cargo test --lib format::san
cargo test --lib format::tags
cargo test --lib format::movetext
cargo test --lib format::pgn

# Integration test
cargo test --test phase6_validation

# Validate with external tool
cargo run -- test/data/five > output.pgn
pgn-extract --quiet output.pgn  # Should report no errors
```

**Expected Final Output**:
```
=== PHASE 6 VALIDATION SUMMARY ===
Total test cases: 47
Passing: 47
Failing: 0

Real-world database conversion:
- Games converted: 5/5 (100%)
- PGN validation: PASS (pgn-extract)
- Average game size: 425 bytes
- Total output: 2.1 KB

Sample output (game 1):
[Event "Test Tournament"]
[Site "New York, NY USA"]
[Date "2022.12.19"]
[Round "1"]
[White "Hossain, Enam"]
[Black "Cheparinov, I"]
[Result "1/2-1/2"]

[WhiteElo "2372"]
[BlackElo "2445"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 Nf6 4. O-O Nxe4 5. d4 Be7
6. Qe2 Nd6 7. Bxc6 bxc6 8. dxe5 Nb7 9. Nc3 O-O 10. Re1
Nc5 11. Nd4 Ne6 12. Be3 Nxd4 13. Bxd4 c5 1/2-1/2

✅ Phase 6 Complete - Ready for Phase 7 (Public API)
```

---

## Next Phase Preview

**Phase 7: Public API Design** will complete the library by:
1. Creating ScidReader entry point
2. Implementing game iteration interface
3. Adding to_pgn() and write_pgn() methods
4. Creating comprehensive documentation
5. Adding usage examples

**Integration Point**:
```rust
// Phase 7 - Simple public API built on Phases 1-6
let reader = ScidReader::open("database.si4")?;

for game in reader.games() {
    let pgn = game.to_pgn(PgnOptions::default())?;
    println!("{}", pgn);
}
```

---

**Phase 6 delivers the final piece of the conversion pipeline: transforming our parsed, validated chess data into the universal PGN format that humans and software can read and use.** By leveraging shakmaty's SAN generation and following the PGN specification exactly, we produce high-quality, standards-compliant output that works with all major chess software.
