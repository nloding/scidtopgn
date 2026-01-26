# SCID Move Encoding Investigation

This document captures the findings from investigating why move decoding fails after the first few moves. The investigation compared the current implementation against the original SCID source code (`repomix-scidvspc.xml`) and the PGN specification.

## Executive Summary

The move decoding implementation has **5 fundamental encoding mismatches** with the original SCID format:

1. **King Move Encoding** - Uses different direction values and wrong castling codes (10/11 instead of 9/10)
2. **Knight Move Values** - Uses 0-7 instead of 1-8, wrong square index differences
3. **Bishop Encoding** - Completely different algorithm (uses direction+distance, should use target-file + direction-bit)
4. **Pawn Encoding** - Black capture directions are reversed (file deltas wrong for black)
5. **Capture Handling** - Missing swap algorithm causes piece number divergence after any capture

Additionally, there are **items needing verification or minor fixes**:

6. **Piece Numbering** - Standard position order needs verification; FEN parsing must use scan order (A8→H1) with King always #0
7. **Special Codes (PGN Annotations)** - King values 11-15 are PGN annotation markers, not chess moves. `END_GAME = 0x0F` (not 0x00). Byte `0x00` is a **null move**.
8. **Comment Encoding** - Comments use two-part encoding: `0x0C` marker in move data, actual text in separate section at end of game data.
9. **Promotion Handling** - Piece number must be preserved through promotion
10. **Castling Handling** - Both King and Rook List/ListPos must be updated
11. **PGN Tag Storage** - STR tags in Index+Name files; non-standard tags in Game file with common tag optimization (bytes 241-254)

**Verified Correct**:
- ✅ Rook Move Encoding
- ✅ Queen Move Encoding (including 2-byte diagonal moves)

---

## Architectural Overview: Two-Layer Decoding

SCID's move stream contains **two types of bytes**:

1. **Move Bytes** - Actual chess moves encoded as `[piece_num:4][move_value:4]`
2. **Annotation Bytes** - PGN metadata (NAGs, comments, variations) encoded using King piece (0) with values 11-15

These are handled at **different layers**:

```
┌─────────────────────────────────────────────────────────────┐
│                    Game Parser Layer                         │
│  - Reads byte stream                                        │
│  - Identifies annotation markers (0x0B-0x0F)                │
│  - Handles NAGs, comments, variations                       │
│  - Passes only actual move bytes to decoder                 │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼ (only move bytes)
┌─────────────────────────────────────────────────────────────┐
│                    Move Decoder Layer                        │
│  - Receives only valid move bytes                           │
│  - Decodes piece-specific move values                       │
│  - Returns from/to squares and promotion info               │
└─────────────────────────────────────────────────────────────┘
```

This architecture is **already correctly implemented** in our integration tests (`crates/core/tests/move_decoding.rs`, lines 55-110).

---

## 1. Piece Numbering Order

SCID uses **two different piece numbering schemes** depending on how the position is initialized:

1. **Standard Starting Position** - Fixed, hardcoded piece numbers
2. **Non-Standard Position (FEN)** - Sequential assignment based on parse order

### 1A. Standard Starting Position

**File**: `repomix-scidvspc.xml`, lines 77819-77837 (Position::StdStart function)

```cpp
AddToBoard(WK, E1);  List[WHITE][0] = E1;  ListPos[E1] = 0;   // King
AddToBoard(WR, A1);  List[WHITE][1] = A1;  ListPos[A1] = 1;   // Rook a1
AddToBoard(WN, B1);  List[WHITE][2] = B1;  ListPos[B1] = 2;   // Knight b1
AddToBoard(WB, C1);  List[WHITE][3] = C1;  ListPos[C1] = 3;   // Bishop c1
AddToBoard(WQ, D1);  List[WHITE][4] = D1;  ListPos[D1] = 4;   // Queen
AddToBoard(WB, F1);  List[WHITE][5] = F1;  ListPos[F1] = 5;   // Bishop f1
AddToBoard(WN, G1);  List[WHITE][6] = G1;  ListPos[G1] = 6;   // Knight g1
AddToBoard(WR, H1);  List[WHITE][7] = H1;  ListPos[H1] = 7;   // Rook h1
// Pawns: lines 77836-77837
AddToBoard(WP, A2+i); List[WHITE][i+8] = A2+i; ListPos[A2+i] = i+8;  // i=0..7
```

**SCID Piece Order** (standard starting position):
| Piece # | White | Black |
|---------|-------|-------|
| 0 | King (E1) | King (E8) |
| 1 | Rook (A1) | Rook (A8) |
| 2 | Knight (B1) | Knight (B8) |
| 3 | Bishop (C1) | Bishop (C8) |
| 4 | Queen (D1) | Queen (D8) |
| 5 | Bishop (F1) | Bishop (F8) |
| 6 | Knight (G1) | Knight (G8) |
| 7 | Rook (H1) | Rook (H8) |
| 8-15 | Pawns (A2-H2) | Pawns (A7-H7) |

**Key Pattern**: The order follows the board layout from queenside to kingside:
- `a1→b1→c1→d1→f1→g1→h1` (skipping e1 where king starts)
- Then pawns a2 through h2

### 1B. Non-Standard Position (FEN Parsing)

**File**: `repomix-scidvspc.xml`, lines 77877-77909 (Position::AddPiece function)

When a position is set from FEN, pieces are numbered **sequentially as they appear** in the FEN string:

```cpp
// Position::AddPiece():
//      Add a piece to the board and piecelist.
errorT
Position::AddPiece (pieceT p, squareT sq)
{
    colorT c = piece_Color(p);
    if (piece_Type(p) == KING) {
        // King is always at the start of the piecelist, so move the piece
        // already at location 0 if there is one:
        if (Count[c] > 0) {
            squareT oldsq = List[c][0];
            List[c][Count[c]] = oldsq;
            ListPos[oldsq] = Count[c];
        }
        List[c][0] = sq;
        ListPos[sq] = 0;
    } else {
        ListPos[sq] = Count[c];
        List[c][Count[c]] = sq;
    }
    Count[c]++;
    // ...
}
```

**FEN Parsing Rules**:
1. FEN scans **rank 8 to rank 1** (top to bottom), **file a to h** (left to right)
2. Each non-king piece gets the next available number (`Count[c]`)
3. **King ALWAYS gets piece number 0** - if another piece was already assigned 0, it gets bumped to the current count

**Example**: FEN `r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1`

The FEN parser encounters pieces in this order:
- Rank 8: r(a8), k(e8), r(h8) → Black
- Rank 7: p(a7), p(b7), ... p(h7) → Black
- Rank 2: P(a2), P(b2), ... P(h2) → White
- Rank 1: R(a1), K(e1), R(h1) → White

**Black piece numbering**:
| Encounter Order | Piece | Square | Initial # | Final # |
|-----------------|-------|--------|-----------|---------|
| 1 | Rook | A8 | 0 | 1 (bumped when King found) |
| 2 | King | E8 | 0 | 0 (King always 0) |
| 3 | Rook | H8 | 2 | 2 |
| 4-11 | Pawns | A7-H7 | 3-10 | 3-10 |

**White piece numbering**:
| Encounter Order | Piece | Square | Initial # | Final # |
|-----------------|-------|--------|-----------|---------|
| 1-8 | Pawns | A2-H2 | 0-7 | 1-8 (first pawn bumped when King found) |
| 9 | Rook | A1 | 8 | 9 (bumped when King found) |
| 10 | King | E1 | 0 | 0 (King always 0) |
| 11 | Rook | H1 | 10 | 10 |

**CRITICAL**: This means the **same position** can have **different piece numberings** depending on whether it was initialized via `StdStart()` or `ReadFromFEN()`.

For the standard starting position specifically, SCID uses the hardcoded `StdStart()` order, NOT the FEN sequential order.

### 1C. Current Implementation Status

**File**: `crates/core/src/parser/position.rs`

#### Standard Start (`standard_start()` function, lines 63-110)

The implementation has been updated to match SCID:

```rust
// White pieces (SCID standard order from repomix-scidvspc.xml:77819-77833)
mapping.white_pieces.insert(0, Square::E1); // King
mapping.white_pieces.insert(1, Square::A1); // Rook a1
mapping.white_pieces.insert(2, Square::B1); // Knight b1
mapping.white_pieces.insert(3, Square::C1); // Bishop c1
mapping.white_pieces.insert(4, Square::D1); // Queen
mapping.white_pieces.insert(5, Square::F1); // Bishop f1
mapping.white_pieces.insert(6, Square::G1); // Knight g1
mapping.white_pieces.insert(7, Square::H1); // Rook h1
// Pawns 8-15 = a2-h2
```

**Status**: ✅ **CORRECT** - Standard starting position piece numbering matches SCID.

#### FEN Parsing (`from_position()` function, lines 121+)

The implementation processes pieces by Role order (King, Queen, Rook, Bishop, Knight, Pawn), then assigns piece numbers sequentially with King forced to 0.

**Status**: ⚠️ **NEEDS VERIFICATION** - The implementation processes pieces by Role type rather than by FEN scanning order. Need to verify this produces the same numbering as SCID's `ReadFromFEN()` + `AddPiece()` behavior.

**Potential Issue**: If SCID assigns piece numbers strictly by FEN encounter order (rank 8→1, file a→h), but our implementation groups by piece type first, the numbering could differ for non-standard positions.

---

## 2. King Move Encoding

### 2A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59083-59109 (encodeKing)

```cpp
// encodeKing(): encoding of King moves.
static inline void
encodeKing (ByteBuffer * buf, simpleMoveT * sm)
{
    // Valid King difference-from-old-square values are:
    // -9, -8, -7, -1, 1, 7, 8, 9, and -2 and 2 for castling.
    // To convert this to a val in the range [1-10], we add 9 and
    // then look up the val[] table.
    // Coded values 1-8 are one-square moves; 9 and 10 are Castling.
    ASSERT(sm->pieceNum == 0);  // Kings MUST be piece Number zero.
    int diff = (int) sm->to - (int) sm->from;
    static const byte val[] = {
    /* -9 -8 -7 -6 -5 -4 -3 -2 -1  0  1   2  3  4  5  6  7  8  9 */
        1, 2, 3, 0, 0, 0, 0, 9, 4, 0, 5, 10, 0, 0, 0, 0, 6, 7, 8
    };
    // If target square is the from square, it is the null move, which
    // is represented as a king move to its own square and is encoded
    // as the byte value zero.
    if (sm->to == sm->from) {
        buf->PutByte (makeMoveByte (0, 0));
        return;
    }
    // Verify we have a valid King move:
    ASSERT(diff >= -9  &&  diff <= 9  &&  val[diff+9] != 0);
    buf->PutByte (makeMoveByte (0, val [diff + 9]));
}
```

**Key Insight**: The encoder uses `val[diff + 9]` to convert a square index difference to a move value. This lookup table maps:
- diff -9 → val 1 (Southwest)
- diff -8 → val 2 (South)
- diff -7 → val 3 (Southeast)
- diff -2 → val 9 (Queenside castle)
- diff -1 → val 4 (West)
- diff +1 → val 5 (East)
- diff +2 → val 10 (Kingside castle)
- diff +7 → val 6 (Northwest)
- diff +8 → val 7 (North)
- diff +9 → val 8 (Northeast)

**Critical Constraint**: `ASSERT(sm->pieceNum == 0)` - King MUST always be piece number 0.

### 2B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59111-59126 (decodeKing)

```cpp
// decodeKing(): decoding of King moves.
static inline errorT
decodeKing (byte val, simpleMoveT * sm)
{
    static const int sqdiff[] = {
        0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2
    };
    if (val == 0) {
        sm->to = sm->from;  // Null move
        return OK;
    }
    if (val < 1  ||  val > 10) { return ERROR_Decode; }  // ONLY values 1-10 valid!
    sm->to = sm->from + sqdiff[val];
    return OK;
}
```

The `sqdiff[]` array is the inverse of the encoding lookup - it maps move values back to square differences:

**SCID King Move Values** (values 0-10 ONLY):
| Value | sqdiff[val] | Square Arithmetic | Direction |
|-------|-------------|-------------------|-----------|
| 0 | 0 | to = from | Null move (king to same square) |
| 1 | -9 | to = from - 9 | Southwest (rank-1, file-1) |
| 2 | -8 | to = from - 8 | South (rank-1, same file) |
| 3 | -7 | to = from - 7 | Southeast (rank-1, file+1) |
| 4 | -1 | to = from - 1 | West (same rank, file-1) |
| 5 | +1 | to = from + 1 | East (same rank, file+1) |
| 6 | +7 | to = from + 7 | Northwest (rank+1, file-1) |
| 7 | +8 | to = from + 8 | North (rank+1, same file) |
| 8 | +9 | to = from + 9 | Northeast (rank+1, file+1) |
| 9 | -2 | to = from - 2 | **Queenside castle (O-O-O)** |
| 10 | +2 | to = from + 2 | **Kingside castle (O-O)** |

**Important**: Values 11-15 return `ERROR_Decode` - they are NOT valid king moves. These bytes (0x0B-0x0F) are PGN annotation markers handled at the game parser layer, not the move decoder layer.

### 2C. Understanding Square Index Arithmetic

SCID uses a linear square index where:
- `square_index = rank * 8 + file`
- Files: A=0, B=1, C=2, D=3, E=4, F=5, G=6, H=7
- Ranks: 1=0, 2=1, 3=2, 4=3, 5=4, 6=5, 7=6, 8=7

**Square Index Layout**:
```
A8=56  B8=57  C8=58  D8=59  E8=60  F8=61  G8=62  H8=63   (rank 8)
A7=48  B7=49  C7=50  D7=51  E7=52  F7=53  G7=54  H7=55   (rank 7)
A6=40  B6=41  C6=42  D6=43  E6=44  F6=45  G6=46  H6=47   (rank 6)
A5=32  B5=33  C5=34  D5=35  E5=36  F5=37  G5=38  H5=39   (rank 5)
A4=24  B4=25  C4=26  D4=27  E4=28  F4=29  G4=30  H4=31   (rank 4)
A3=16  B3=17  C3=18  D3=19  E3=20  F3=21  G3=22  H3=23   (rank 3)
A2=8   B2=9   C2=10  D2=11  E2=12  F2=13  G2=14  H2=15   (rank 2)
A1=0   B1=1   C1=2   D1=3   E1=4   F1=5   G1=6   H1=7    (rank 1)
```

**Why the differences work**:
- Moving one rank up: +8 (next row of 8 squares)
- Moving one rank down: -8
- Moving one file right: +1
- Moving one file left: -1
- Diagonal up-right: +8 + 1 = +9
- Diagonal up-left: +8 - 1 = +7
- Diagonal down-right: -8 + 1 = -7
- Diagonal down-left: -8 - 1 = -9

**Example**: King on E1 (index 4):
| Move | Target | Calculation | Value |
|------|--------|-------------|-------|
| Ke1-f1 (East) | F1=5 | 5-4=+1 | 5 |
| Ke1-d1 (West) | D1=3 | 3-4=-1 | 4 |
| Ke1-e2 (North) | E2=12 | 12-4=+8 | 7 |
| Ke1-f2 (NE) | F2=13 | 13-4=+9 | 8 |
| Ke1-d2 (NW) | D2=11 | 11-4=+7 | 6 |
| O-O (Kingside) | G1=6 | 6-4=+2 | 10 |
| O-O-O (Queenside) | C1=2 | 2-4=-2 | 9 |

### 2D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 39-103

The current implementation has **two fundamental errors**:

#### Error 1: Direction Mapping Uses File/Rank Deltas Instead of Square Index Differences

```rust
// Current implementation (WRONG)
let (file_delta, rank_delta): (i8, i8) = match move_value {
    1 => (-1, 1),  // NW    <- SCID: value 1 = -9 = SW (down-left)
    2 => (0, 1),   // N     <- SCID: value 2 = -8 = S (down)
    3 => (1, 1),   // NE    <- SCID: value 3 = -7 = SE (down-right)
    4 => (-1, 0),  // W     <- SCID: value 4 = -1 = W (correct!)
    5 => (1, 0),   // E     <- SCID: value 5 = +1 = E (correct!)
    6 => (-1, -1), // SW    <- SCID: value 6 = +7 = NW (up-left)
    7 => (0, -1),  // S     <- SCID: value 7 = +8 = N (up)
    8 => (1, -1),  // SE    <- SCID: value 8 = +9 = NE (up-right)
    _ => unreachable!(),
};
```

**Problem**: The implementation interprets values 1-3 as "up" directions when SCID uses them for "down" directions, and values 6-8 as "down" when SCID uses them for "up".

**Comparison Table**:
| Value | Current Implementation | SCID Actual | Match? |
|-------|------------------------|-------------|--------|
| 1 | Northwest (up-left) | Southwest (down-left) | ❌ |
| 2 | North (up) | South (down) | ❌ |
| 3 | Northeast (up-right) | Southeast (down-right) | ❌ |
| 4 | West (left) | West (left) | ✅ |
| 5 | East (right) | East (right) | ✅ |
| 6 | Southwest (down-left) | Northwest (up-left) | ❌ |
| 7 | South (down) | North (up) | ❌ |
| 8 | Southeast (down-right) | Northeast (up-right) | ❌ |

#### Error 2: Wrong Castling Values

```rust
// Current implementation (WRONG)
} else if move_value == 10 {
    // Kingside castle <- CORRECT
} else if move_value == 11 {
    // Queenside castle <- WRONG! SCID uses 9 for queenside
}
```

**Problem**:
- Kingside castle: Value 10 ✅ (correct)
- Queenside castle: Value 11 ❌ (should be 9)
- Value 11 is `ENCODE_NAG` - an annotation marker that should never reach the decoder

#### Error 3: Null Move Handling

```rust
// Current implementation
if move_value == 0 {
    return Err(...)  // Treats null move as error
}
```

**Problem**: SCID treats value 0 as a valid null move (`sm->to = sm->from`). The decoder should return a move where from == to, not an error.

### 2E. Required Fix

The decoder should use SCID's square index arithmetic directly:

```rust
// Correct implementation approach
static SQDIFF: [i8; 11] = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];

pub fn decode_king_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    if move_value > 10 {
        return Err(...);  // Invalid - values 11-15 are annotation markers
    }

    if move_value == 0 {
        // Null move - king stays in place
        return Ok(DecodedMove { from, to: from, promotion: None });
    }

    // Calculate target square using SCID's index arithmetic
    let from_index = from as i8;  // shakmaty Square can convert to index
    let to_index = from_index + SQDIFF[move_value as usize];
    let to = Square::new(to_index as u32);

    Ok(DecodedMove { from, to, promotion: None })
}
```

### 2F. Impact Summary

| Issue | Severity | Impact |
|-------|----------|--------|
| Direction mapping inverted | High | 6 of 8 regular king moves decode to wrong square |
| Queenside castle wrong value | High | All queenside castles fail to decode |
| Null move treated as error | Medium | Null moves in analysis lines cause parse failure |

---

## 3. Knight Move Encoding

### 3A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59128-59147 (encodeKnight)

```cpp
// encodeKnight(): encoding Knight moves.
static inline void
encodeKnight (ByteBuffer * buf, simpleMoveT * sm)
{
    // Valid Knight difference-from-old-square values are:
    // -17, -15, -10, -6, 6, 10, 15, 17.
    // To convert this to a value in the range [1-8], we add 17 to
    // the difference and then look up the val[] table.
    int diff = (int) sm->to - (int) sm->from;
    static const byte val[] = {
    /* -17 -16 -15 -14 -13 -12 -11 -10 -9 -8 -7 -6 -5 -4 -3 -2 -1  0 */
        1,  0,  2,  0,  0,  0,  0,  3,  0, 0, 0, 4, 0, 0, 0, 0, 0, 0,
    /*  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 */
        0, 0, 0, 0, 0, 5, 0, 0, 0, 6, 0, 0, 0, 0, 7, 0, 8
    };
    // Verify we have a valid knight move:
    ASSERT (diff >= -17  &&  diff <= 17  &&  val[diff + 17] != 0);
    buf->PutByte (makeMoveByte (sm->pieceNum, val [diff + 17]));
}
```

**Key Insight**: The encoder uses `val[diff + 17]` to convert a square index difference to a move value. Only 8 of the 35 possible differences are valid knight moves:

| Square Diff | Array Index (diff+17) | Move Value |
|-------------|----------------------|------------|
| -17 | 0 | 1 |
| -15 | 2 | 2 |
| -10 | 7 | 3 |
| -6 | 11 | 4 |
| +6 | 23 | 5 |
| +10 | 27 | 6 |
| +15 | 32 | 7 |
| +17 | 34 | 8 |

All other array positions contain 0, which triggers the ASSERT failure for invalid moves.

### 3B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59149-59160 (decodeKnight)

```cpp
// decodeKnight(): decoding Knight moves.
static inline errorT
decodeKnight (byte val, simpleMoveT * sm)
{
    static const int sqdiff[] = {
        0, -17, -15, -10, -6, 6, 10, 15, 17
    };
    if (val < 1  ||  val > 8) { return ERROR_Decode; }
    sm->to = sm->from + sqdiff[val];
    return OK;
}
```

The `sqdiff[]` array is the inverse of the encoding lookup:

**SCID Knight Move Values** (values 1-8 ONLY):
| Value | sqdiff[val] | Square Arithmetic | Direction |
|-------|-------------|-------------------|-----------|
| 0 | 0 | INVALID | (placeholder in array) |
| 1 | -17 | to = from - 17 | 2 ranks down, 1 file left |
| 2 | -15 | to = from - 15 | 2 ranks down, 1 file right |
| 3 | -10 | to = from - 10 | 1 rank down, 2 files left |
| 4 | -6 | to = from - 6 | 1 rank down, 2 files right |
| 5 | +6 | to = from + 6 | 1 rank up, 2 files left |
| 6 | +10 | to = from + 10 | 1 rank up, 2 files right |
| 7 | +15 | to = from + 15 | 2 ranks up, 1 file left |
| 8 | +17 | to = from + 17 | 2 ranks up, 1 file right |

**Critical**: Value 0 returns `ERROR_Decode` - there is NO null move for knights!

### 3C. Understanding Knight Square Index Arithmetic

Using the same square index layout (rank * 8 + file):

```
A8=56  B8=57  C8=58  D8=59  E8=60  F8=61  G8=62  H8=63   (rank 8)
A7=48  B7=49  C7=50  D7=51  E7=52  F7=53  G7=54  H7=55   (rank 7)
A6=40  B6=41  C6=42  D6=43  E6=44  F6=45  G6=46  H6=47   (rank 6)
A5=32  B5=33  C5=34  D5=35  E5=36  F5=37  G5=38  H5=39   (rank 5)
A4=24  B4=25  C4=26  D4=27  E4=28  F4=29  G4=30  H4=31   (rank 4)
A3=16  B3=17  C3=18  D3=19  E3=20  F3=21  G3=22  H3=23   (rank 3)
A2=8   B2=9   C2=10  D2=11  E2=12  F2=13  G2=14  H2=15   (rank 2)
A1=0   B1=1   C1=2   D1=3   E1=4   F1=5   G1=6   H1=7    (rank 1)
```

**Why the knight differences work**:
- 2 ranks up = +16, 1 file right = +1 → total = +17
- 2 ranks up = +16, 1 file left = -1 → total = +15
- 1 rank up = +8, 2 files right = +2 → total = +10
- 1 rank up = +8, 2 files left = -2 → total = +6
- 1 rank down = -8, 2 files right = +2 → total = -6
- 1 rank down = -8, 2 files left = -2 → total = -10
- 2 ranks down = -16, 1 file right = +1 → total = -15
- 2 ranks down = -16, 1 file left = -1 → total = -17

**Example**: Knight on G1 (index 6):
| Move | Target | Calculation | Diff | Value |
|------|--------|-------------|------|-------|
| Ng1-f3 | F3=21 | 21-6=+15 | +15 | 7 |
| Ng1-h3 | H3=23 | 23-6=+17 | +17 | 8 |
| Ng1-e2 | E2=12 | 12-6=+6 | +6 | 5 |

**Example**: Knight on B1 (index 1):
| Move | Target | Calculation | Diff | Value |
|------|--------|-------------|------|-------|
| Nb1-c3 | C3=18 | 18-1=+17 | +17 | 8 |
| Nb1-a3 | A3=16 | 16-1=+15 | +15 | 7 |
| Nb1-d2 | D2=11 | 11-1=+10 | +10 | 6 |

### 3D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 105-159

The current implementation has **two fundamental errors**:

#### Error 1: Wrong Value Range (Off-by-One)

```rust
// Current implementation (WRONG)
pub fn decode_knight_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    if move_value > 7 {  // <- WRONG: should be < 1 || > 8
        return Err(...)
    }
```

**Problem**: SCID uses values 1-8, but the implementation accepts 0-7 and rejects 8.

#### Error 2: Wrong Direction Mapping

```rust
// Current implementation (WRONG)
let (file_delta, rank_delta): (i8, i8) = match move_value {
    0 => (-1, 2),  // 2 up, 1 left    <- INVALID in SCID!
    1 => (1, 2),   // 2 up, 1 right   <- SCID: 2 down, 1 left (-17)
    2 => (2, 1),   // 1 up, 2 right   <- SCID: 2 down, 1 right (-15)
    3 => (2, -1),  // 1 down, 2 right <- SCID: 1 down, 2 left (-10)
    4 => (1, -2),  // 2 down, 1 right <- SCID: 1 down, 2 right (-6)
    5 => (-1, -2), // 2 down, 1 left  <- SCID: 1 up, 2 left (+6)
    6 => (-2, -1), // 1 down, 2 left  <- SCID: 1 up, 2 right (+10)
    7 => (-2, 1),  // 1 up, 2 left    <- SCID: 2 up, 1 left (+15)
    _ => unreachable!(),  // value 8 unreachable! <- SCID: 2 up, 1 right (+17)
};
```

**Comparison Table**:
| Value | Current Implementation | SCID Actual | File Delta Match? | Rank Delta Match? |
|-------|------------------------|-------------|-------------------|-------------------|
| 0 | (-1, +2) = 2 up, 1 left | INVALID | N/A | N/A |
| 1 | (+1, +2) = 2 up, 1 right | (-1, -2) = 2 down, 1 left | ❌ | ❌ |
| 2 | (+2, +1) = 1 up, 2 right | (+1, -2) = 2 down, 1 right | ❌ | ❌ |
| 3 | (+2, -1) = 1 down, 2 right | (-2, -1) = 1 down, 2 left | ❌ | ✅ |
| 4 | (+1, -2) = 2 down, 1 right | (+2, -1) = 1 down, 2 right | ❌ | ❌ |
| 5 | (-1, -2) = 2 down, 1 left | (-2, +1) = 1 up, 2 left | ❌ | ❌ |
| 6 | (-2, -1) = 1 down, 2 left | (+2, +1) = 1 up, 2 right | ❌ | ❌ |
| 7 | (-2, +1) = 1 up, 2 left | (-1, +2) = 2 up, 1 left | ❌ | ❌ |
| 8 | (not handled) | (+1, +2) = 2 up, 1 right | N/A | N/A |

**Every single knight move decodes incorrectly!**

### 3E. Required Fix

The decoder should use SCID's square index arithmetic directly:

```rust
// Correct implementation approach
static SQDIFF: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];

pub fn decode_knight_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    if move_value < 1 || move_value > 8 {
        return Err(...);  // Invalid value
    }

    // Calculate target square using SCID's index arithmetic
    let from_index = from as i8;
    let to_index = from_index + SQDIFF[move_value as usize];

    // Bounds check (knight could jump off board from edge squares)
    if to_index < 0 || to_index > 63 {
        return Err(...);
    }

    let to = Square::new(to_index as u32);

    Ok(DecodedMove { from, to, promotion: None })
}
```

**Note**: Additional validation may be needed to ensure the knight doesn't "wrap around" the board (e.g., from H1 to A3 via +17 would be invalid). SCID relies on the encoder only producing valid moves, but a robust decoder should verify the file distance is ≤ 2.

### 3F. Impact Summary

| Issue | Severity | Impact |
|-------|----------|--------|
| Off-by-one value range | High | Value 8 rejected, value 0 accepted incorrectly |
| All direction mappings wrong | Critical | 100% of knight moves decode to wrong square |
| Value 0 accepted | Medium | Invalid moves may be processed instead of rejected |

---

## 4. Bishop Move Encoding

### 4A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59197-59213 (encodeBishop)

```cpp
// encodeBishop(): encoding Bishop moves.
static inline void
encodeBishop (ByteBuffer * buf, simpleMoveT * sm)
{
    // We encode a Bishop move as the Fyle moved to, plus
    // a one-bit flag to indicate if the direction was
    // up-right/down-left or vice versa.
    ASSERT (sm->to <= H8  &&  sm->from <= H8);
    byte val;
    val = square_Fyle(sm->to);
    int rankdiff = (int)square_Rank(sm->to) - (int)square_Rank(sm->from);
    int fylediff = (int)square_Fyle(sm->to) - (int)square_Fyle(sm->from);
    // If (rankdiff * fylediff) is negative, it's up-left/down-right:
    if (rankdiff * fylediff < 0) { val += 8; }
    buf->PutByte (makeMoveByte (sm->pieceNum, val));
}
```

**Key Insight**: Bishop moves are encoded using just 4 bits:
- **Bits 0-2 (val & 7)**: Target file (0-7 = a-h)
- **Bit 3 (val & 8)**: Diagonal direction flag

**Diagonal Direction Logic**:
The product `rankdiff * fylediff` determines the diagonal type:
- **Positive product** (same sign): Up-right or down-left diagonal → `val < 8`
- **Negative product** (opposite signs): Up-left or down-right diagonal → `val >= 8`

| From → To | rankdiff | fylediff | Product | Direction | val |
|-----------|----------|----------|---------|-----------|-----|
| D4 → F6 | +2 | +2 | +4 (positive) | Up-right | 5 (file F) |
| D4 → B2 | -2 | -2 | +4 (positive) | Down-left | 1 (file B) |
| D4 → B6 | +2 | -2 | -4 (negative) | Up-left | 9 (file B + 8) |
| D4 → F2 | -2 | +2 | -4 (negative) | Down-right | 13 (file F + 8) |

### 4B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59215-59230 (decodeBishop)

```cpp
// decodeBishop(): decoding Bishop moves.
static inline errorT
decodeBishop (byte val, simpleMoveT * sm)
{
    byte fyle = (val & 7);
    int fylediff = (int)fyle - (int)square_Fyle(sm->from);
    if (val >= 8) {
        // It is an up-left/down-right direction move.
        sm->to = sm->from - 7 * fylediff;
    } else {
        // It is an up-right/down-left direction move.
        sm->to = sm->from + 9 * fylediff;
    }
    if (sm->to > H8) { return ERROR_Decode;}
    return OK;
}
```

**Decoding Algorithm**:

1. Extract target file: `fyle = val & 7`
2. Calculate file difference: `fylediff = fyle - from_file`
3. Apply diagonal formula based on direction bit:
   - `val < 8` (up-right/down-left): `to = from + 9 * fylediff`
   - `val >= 8` (up-left/down-right): `to = from - 7 * fylediff`

### 4C. Understanding Bishop Square Index Arithmetic

The formulas `+9` and `-7` come from the square index math:

**Up-right diagonal** (rank+1, file+1): Each step = +8 (rank) + 1 (file) = **+9**
**Down-left diagonal** (rank-1, file-1): Each step = -8 (rank) - 1 (file) = **-9** = +9 * (-1)

**Up-left diagonal** (rank+1, file-1): Each step = +8 (rank) - 1 (file) = **+7**
**Down-right diagonal** (rank-1, file+1): Each step = -8 (rank) + 1 (file) = **-7** = -7 * (+1)

The formula uses `fylediff` (which can be positive or negative) to handle both directions on each diagonal:

| Diagonal Type | fylediff | Formula | Calculation |
|---------------|----------|---------|-------------|
| Up-right | +N | from + 9*N | Moves N files right, N ranks up |
| Down-left | -N | from + 9*(-N) | Moves N files left, N ranks down |
| Up-left | -N | from - 7*(-N) = from + 7*N | Moves N files left, N ranks up |
| Down-right | +N | from - 7*N | Moves N files right, N ranks down |

**Example**: Bishop on D4 (index 27, file 3):

```
Square Index Layout:
A8=56  B8=57  C8=58  D8=59  E8=60  F8=61  G8=62  H8=63
A7=48  B7=49  C7=50  D7=51  E7=52  F7=53  G7=54  H7=55
A6=40  B6=41  C6=42  D6=43  E6=44  F6=45  G6=46  H6=47
A5=32  B5=33  C5=34  D5=35  E5=36  F5=37  G5=38  H5=39
A4=24  B4=25  C4=26  D4=27  E4=28  F4=29  G4=30  H4=31  <- Bishop on D4 (27)
A3=16  B3=17  C3=18  D3=19  E3=20  F3=21  G3=22  H3=23
A2=8   B2=9   C2=10  D2=11  E2=12  F2=13  G2=14  H2=15
A1=0   B1=1   C1=2   D1=3   E1=4   F1=5   G1=6   H1=7
```

| Move | Target | fylediff | val | Formula | Calculation |
|------|--------|----------|-----|---------|-------------|
| Bd4-f6 | F6=45 | 5-3=+2 | 5 | 27 + 9*2 | = 45 ✓ |
| Bd4-g7 | G7=54 | 6-3=+3 | 6 | 27 + 9*3 | = 54 ✓ |
| Bd4-a1 | A1=0 | 0-3=-3 | 0 | 27 + 9*(-3) | = 0 ✓ |
| Bd4-b6 | B6=41 | 1-3=-2 | 9 (1+8) | 27 - 7*(-2) | = 41 ✓ |
| Bd4-a7 | A7=48 | 0-3=-3 | 8 (0+8) | 27 - 7*(-3) | = 48 ✓ |
| Bd4-f2 | F2=13 | 5-3=+2 | 13 (5+8) | 27 - 7*2 | = 13 ✓ |
| Bd4-h8 | H8=63 | 7-3=+4 | 15 (7+8) | 27 - 7*4 | = -1 ❌ (invalid - out of bounds) |

Note: The last example shows an invalid move (D4-H8 is not on the same diagonal). SCID relies on the encoder only producing valid moves; the decoder just checks `sm->to > H8`.

### 4D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 189-275

The current implementation uses a **completely different algorithm**:

```rust
// Current implementation (COMPLETELY WRONG)
pub fn decode_bishop_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    // Calculate all diagonal squares from 'from' and pick by index
    let mut diagonals: Vec<Square> = Vec::new();

    // Up-left diagonal
    let mut f = from.file() as i8 - 1;
    let mut r = from.rank() as i8 + 1;
    while f >= 0 && r <= 7 {
        diagonals.push(...);
        f -= 1;
        r += 1;
    }

    // Up-right diagonal
    // ... (similar enumeration)

    // Down-left diagonal
    // ... (similar enumeration)

    // Down-right diagonal
    // ... (similar enumeration)

    // Sort diagonals to get consistent ordering
    diagonals.sort_by_key(|s| u32::from(*s));

    let to = diagonals[move_value as usize];  // <- WRONG!
```

#### Fundamental Algorithm Mismatch

| Aspect | Current Implementation | SCID Actual |
|--------|------------------------|-------------|
| Encoding basis | Index into sorted diagonal list | Target file + direction bit |
| Value meaning | Arbitrary index (0 to N-1) | Bits 0-2: file, Bit 3: direction |
| Calculation | Enumerate, sort, index | Direct arithmetic formula |
| Value range | 0 to (diagonal_count - 1) | 0-15 (file 0-7 + direction bit) |

#### Why This Fails Completely

**SCID's encoding**: `val = target_file + (8 if up-left/down-right else 0)`

For a bishop on D4:
- Move to F6: SCID encodes as `val = 5` (file F = 5, up-right diagonal)
- Move to B6: SCID encodes as `val = 9` (file B = 1, + 8 for up-left diagonal)

**Current implementation**: Would enumerate all 13 diagonal squares from D4, sort them by index (A1=0, B2=9, C3=18, E5=36, F6=45, G7=54, H8=63, A7=48, B6=41, C5=34, E3=20, F2=13, G1=6), and use move_value as index.

The sorted order would be: A1, G1, B2, F2, C3, E3, D4(skip), C5, E5, B6, F6, A7, G7, H8

So `move_value = 5` would decode to **F6** in SCID but to **E3** in current implementation!

### 4E. Required Fix

The decoder should use SCID's direct arithmetic:

```rust
// Correct implementation approach
pub fn decode_bishop_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    // Extract target file from lower 3 bits
    let target_file = (move_value & 7) as i8;
    let from_file = from.file() as i8;
    let from_index = from as i8;

    // Calculate file difference
    let fylediff = target_file - from_file;

    // Calculate target square based on diagonal direction
    let to_index = if move_value >= 8 {
        // Up-left/down-right diagonal
        from_index - 7 * fylediff
    } else {
        // Up-right/down-left diagonal
        from_index + 9 * fylediff
    };

    // Bounds check
    if to_index < 0 || to_index > 63 {
        return Err(...);
    }

    let to = Square::new(to_index as u32);

    Ok(DecodedMove { from, to, promotion: None })
}
```

**Note**: Additional validation should verify the move is actually diagonal (rank difference equals absolute file difference).

### 4F. Impact Summary

| Issue | Severity | Impact |
|-------|----------|--------|
| Completely wrong algorithm | Critical | 100% of bishop moves decode incorrectly |
| Wrong interpretation of move_value | Critical | Value treated as index, not file+direction |
| Enumeration approach | Fundamental | Cannot produce correct results |

---

## 5. PGN Annotation Markers (NOT King Moves)

### Understanding the Architecture

SCID cleverly reuses the King piece number (0) with move values 11-15 for **PGN annotation markers**. This works because:

1. Actual king moves only need values 0-10
2. Values 11-15 are unused for chess moves
3. Byte `0x0B` through `0x0F` (piece 0, values 11-15) become annotation markers

**These are NOT chess moves** - they are metadata for PGN export.

### SCID Source Code

**File**: `repomix-scidvspc.xml`, lines 59349-59358

```cpp
// Special-move tokens:
// Since king-move values 1-10 are taken for actual King moves, only
// 11-15 (and zero) are available for non-move information.
#define ENCODE_NAG          11
#define ENCODE_COMMENT      12
#define ENCODE_START_MARKER 13
#define ENCODE_END_MARKER   14
#define ENCODE_END_GAME     15
```

### PGN Standard Reference

These markers correspond to PGN features as defined in the PGN specification (https://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm):

#### NAG (Numeric Annotation Glyph) - Section 8.2.4

NAGs are language-independent move annotations. In PGN they appear as `$n` where n is 0-255:

| NAG Value | Symbol | Meaning |
|-----------|--------|---------|
| $1 | ! | Good move |
| $2 | ? | Poor move |
| $3 | !! | Very good move |
| $4 | ?? | Very poor move |
| $5 | !? | Speculative move |
| $6 | ?! | Questionable move |
| $10 | = | Equal position |
| $14 | += | White has slight advantage |
| $15 | =+ | Black has slight advantage |
| ... | ... | (values 1-255 defined) |

**SCID Encoding**: When byte `0x0B` is encountered, the **next byte** contains the NAG value (0-255).

#### Comments

PGN supports two comment types:
- **Brace comments**: `{This is a comment}` - text between curly braces
- **Line comments**: `; comment to end of line`

**SCID Encoding**: Comments use a **two-part encoding**:

1. **In move data**: Byte `0x0C` (ENCODE_COMMENT) marks that a comment exists at this position. It does NOT contain the comment text.
2. **After move data**: All comment texts are stored as null-terminated strings in a separate section at the end of the game data.

**Game Data Structure** (from `Game::Encode`, lines 59721-59749):
```cpp
// 1. Non-STR PGN tags
err = encodeTags (buf, TagList, NumTags);

// 2. Flags byte (NonStandardStart, Promotions, UnderPromos)
buf->PutByte (flags);

// 3. Start FEN (if NonStandardStart)
if (NonStandardStart) {
    buf->PutTerminatedString (tempStr);  // FEN string
}

// 4. Move data (includes 0x0C markers for comments, but NOT the text)
err = encodeVariation (buf, FirstMove->next, &varCount, &nagCount, 0);

// 5. Comments section (null-terminated strings in tree traversal order)
err = encodeComments (buf, FirstMove, &commentCount);
```

**Decoding comments**: When decoding, `0x0C` markers just flag moves as having comments. After decoding all moves, `decodeComments()` reads the comment strings from the end of the game data and assigns them to the marked moves in tree traversal order.

#### Variations (RAV - Recursive Annotation Variation)

PGN supports alternative move sequences in parentheses:
```pgn
1. e4 e5 2. Nf3 (2. Bc4 Nc6 3. Qh5) 2... Nc6
```

**SCID Encoding**:
- `0x0D` (ENCODE_START_MARKER) = Start of variation `(`
- `0x0E` (ENCODE_END_MARKER) = End of variation `)`

Variations can be nested.

#### End of Game

**SCID Encoding**: `0x0F` (ENCODE_END_GAME) marks the end of move data.

### Byte-Level Summary

| Byte | Piece | Value | Meaning | Following Data |
|------|-------|-------|---------|----------------|
| `0x00` | King (0) | 0 | **NULL MOVE** (king stays in place) | None - this is a valid move! |
| `0x0B` | King (0) | 11 | NAG marker | 1 byte: NAG value (0-255) |
| `0x0C` | King (0) | 12 | Comment marker | None in move data! Text is in comments section at end |
| `0x0D` | King (0) | 13 | Start variation | None (push position to stack) |
| `0x0E` | King (0) | 14 | End variation | None (pop position from stack) |
| `0x0F` | King (0) | 15 | End of game | None (stop processing) |

**Important**: Byte `0x00` is NOT an end marker - it's a **null move** (used in chess analysis where a player "passes"). The decodeKing function handles this:

```cpp
if (val == 0) {
    sm->to = sm->from;  // Null move - king stays in place
    return OK;
}
```

A null move is identified by `from == to` and the moving piece being a King.

### Current Implementation - Game Parser Layer

**File**: `crates/core/tests/move_decoding.rs`, lines 55-110

The integration tests have a **bug** in the constant definitions:

```rust
mod special_bytes {
    pub const END_GAME: u8 = 0x00;      // BUG: This is NULL MOVE, not end of game!
    pub const START_VAR: u8 = 0x0D;     // Correct
    pub const END_VAR: u8 = 0x0E;       // Correct
    pub const NAG: u8 = 0x0B;           // Correct
    pub const COMMENT: u8 = 0x0C;       // Correct
    pub const END_GAME_ALT: u8 = 0x0F;  // This is actually the REAL end of game!
}
```

The fix should be:
```rust
mod special_bytes {
    pub const NULL_MOVE: u8 = 0x00;     // Null move (king stays in place)
    pub const NAG: u8 = 0x0B;           // NAG marker
    pub const COMMENT: u8 = 0x0C;       // Comment marker
    pub const START_VAR: u8 = 0x0D;     // Start variation
    pub const END_VAR: u8 = 0x0E;       // End variation
    pub const END_GAME: u8 = 0x0F;      // End of game
}
```

The game parsing logic is otherwise correct:

```rust
while stream.has_more() {
    let byte = stream.get_byte()?;

    match byte {
        special_bytes::END_GAME | special_bytes::END_GAME_ALT => {
            // End of game - stop processing
            break;
        }
        special_bytes::START_VAR => {
            // Start of variation - track depth, skip variation moves
            variation_depth += 1;
            continue;
        }
        special_bytes::END_VAR => {
            // End of variation
            if variation_depth > 0 {
                variation_depth -= 1;
            }
            continue;
        }
        special_bytes::NAG => {
            // NAG - skip the NAG value byte
            if stream.has_more() {
                stream.get_byte()?;  // Consume NAG value
            }
            continue;
        }
        special_bytes::COMMENT => {
            // Comment marker - just note that a comment exists here
            // The actual comment text is in a separate section after move data
            // For now, we skip it (or could track for later retrieval)
            continue;
        }
        _ => {
            // Regular move byte - pass to decoder
            if variation_depth == 0 {
                match decoder.decode_move(byte, &mut stream) {
                    Ok(chess_move) => moves.push(chess_move),
                    Err(e) => { /* handle error */ }
                }
            }
        }
    }
}
```

### Current Implementation - Move Decoder Layer

**File**: `crates/core/src/parser/decoder.rs`, lines 78-96

The decoder incorrectly handles value 11:

```rust
} else if move_value == 10 {
    // Kingside castle - CORRECT
} else if move_value == 11 {
    // Queenside castle - WRONG!
    // Value 11 is ENCODE_NAG, should never reach decoder
}
```

### Impact and Fix

**The decoder is correct to reject values 11-15** - those bytes should be intercepted by the game parser layer before reaching the decoder.

The bug is:
- Value 11 shouldn't be interpreted as queenside castle
- **Queenside castle is value 9** (not 11)
- **Kingside castle is value 10** (this is correct)

The fix for the decoder is simply:
```rust
9 => queenside castle (O-O-O)   // Square diff -2
10 => kingside castle (O-O)      // Square diff +2
// Values 11-15 return error (handled at game parser layer)
```

---

## 6. Rook Move Encoding

### 6A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59162-59181 (encodeRook)

```cpp
// encodeRook(): encoding rook moves.
static inline void
encodeRook (ByteBuffer * buf, simpleMoveT * sm)
{
    // Valid Rook moves are to same rank, OR to same fyle.
    // We encode the 8 squares on the same rank 0-8, and the 8
    // squares on the same fyle 9-15. This means that for any particular
    // rook move, two of the values in the range [0-15] will be
    // meaningless, as they will represent the from-square.
    ASSERT (sm->from <= H8  &&  sm->to <= H8);
    byte val;
    // Check if the two squares share the same rank:
    if (square_Rank(sm->from) == square_Rank(sm->to)) {
        val = square_Fyle(sm->to);
    } else {
        val = 8 + square_Rank(sm->to);
    }
    buf->PutByte (makeMoveByte (sm->pieceNum, val));
}
```

**Key Insight**: Rook moves use a simple 4-bit encoding:
- **Values 0-7**: Horizontal move (same rank) to target file (a=0, b=1, ... h=7)
- **Values 8-15**: Vertical move (same file) to target rank (1=8, 2=9, ... 8=15)

**Note from SCID comment**: Two values in 0-15 are "meaningless" for any given rook position - the values that would encode moving to the from-square itself. The encoder never produces these, and the decoder doesn't explicitly reject them.

### 6B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59183-59195 (decodeRook)

```cpp
// decodeRook(): decoding Rook moves.
static inline errorT
decodeRook (byte val, simpleMoveT * sm)
{
    if (val >= 8) {
        // This is a move along a Fyle, to a different rank:
        sm->to = square_Make (square_Fyle(sm->from), (val - 8));
    } else {
        sm->to = square_Make (val, square_Rank(sm->from));
    }
    return OK;
}
```

**Decoding Algorithm**:
1. If `val >= 8`: Vertical move - keep same file, target rank = `val - 8`
2. If `val < 8`: Horizontal move - target file = `val`, keep same rank

**SCID Rook Move Values**:
| Value | Type | Target |
|-------|------|--------|
| 0 | Horizontal | File A (same rank) |
| 1 | Horizontal | File B (same rank) |
| 2 | Horizontal | File C (same rank) |
| 3 | Horizontal | File D (same rank) |
| 4 | Horizontal | File E (same rank) |
| 5 | Horizontal | File F (same rank) |
| 6 | Horizontal | File G (same rank) |
| 7 | Horizontal | File H (same rank) |
| 8 | Vertical | Rank 1 (same file) |
| 9 | Vertical | Rank 2 (same file) |
| 10 | Vertical | Rank 3 (same file) |
| 11 | Vertical | Rank 4 (same file) |
| 12 | Vertical | Rank 5 (same file) |
| 13 | Vertical | Rank 6 (same file) |
| 14 | Vertical | Rank 7 (same file) |
| 15 | Vertical | Rank 8 (same file) |

### 6C. Understanding Rook Encoding

The `square_Make(file, rank)` function creates a square index: `rank * 8 + file`.

**Example**: Rook on D4 (index 27, file 3, rank 3):

```
Square Index Layout:
A8=56  B8=57  C8=58  D8=59  E8=60  F8=61  G8=62  H8=63   (rank 7)
A7=48  B7=49  C7=50  D7=51  E7=52  F7=53  G7=54  H7=55   (rank 6)
A6=40  B6=41  C6=42  D6=43  E6=44  F6=45  G6=46  H6=47   (rank 5)
A5=32  B5=33  C5=34  D5=35  E5=36  F5=37  G5=38  H5=39   (rank 4)
A4=24  B4=25  C4=26  D4=27  E4=28  F4=29  G4=30  H4=31   (rank 3) <- Rook on D4
A3=16  B3=17  C3=18  D3=19  E3=20  F3=21  G3=22  H3=23   (rank 2)
A2=8   B2=9   C2=10  D2=11  E2=12  F2=13  G2=14  H2=15   (rank 1)
A1=0   B1=1   C1=2   D1=3   E1=4   F1=5   G1=6   H1=7    (rank 0)
```

| Move | Target | val | Type | SCID Calculation |
|------|--------|-----|------|------------------|
| Rd4-a4 | A4=24 | 0 | Horizontal | square_Make(0, 3) = 3*8+0 = 24 ✓ |
| Rd4-h4 | H4=31 | 7 | Horizontal | square_Make(7, 3) = 3*8+7 = 31 ✓ |
| Rd4-d1 | D1=3 | 8 | Vertical | square_Make(3, 0) = 0*8+3 = 3 ✓ |
| Rd4-d8 | D8=59 | 15 | Vertical | square_Make(3, 7) = 7*8+3 = 59 ✓ |
| Rd4-f4 | F4=29 | 5 | Horizontal | square_Make(5, 3) = 3*8+5 = 29 ✓ |
| Rd4-d6 | D6=43 | 13 | Vertical | square_Make(3, 5) = 5*8+3 = 43 ✓ |

**Note**: Value 3 (horizontal to D-file) and value 11 (vertical to rank 4) would encode "move to self" for a rook on D4, but the encoder never produces these values.

### 6D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 161-187

```rust
/// Rook move decoder
///
/// Move values:
/// - 0-7: Horizontal move to file a-h
/// - 8-15: Vertical move to rank 1-8
pub fn decode_rook_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    let to = if move_value < 8 {
        // Horizontal move to specific file
        Square::from_coords(File::new(move_value as u32), from.rank())
    } else if move_value < 16 {
        // Vertical move to specific rank
        let target_rank = move_value - 8;
        Square::from_coords(from.file(), Rank::new(target_rank as u32))
    } else {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid rook move value: {}", move_value),
        });
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
    })
}
```

### 6E. Verification

**Comparison Table**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| Values 0-7 meaning | Horizontal to file 0-7 | Horizontal to file 0-7 | ✅ |
| Values 8-15 meaning | Vertical to rank 0-7 | Vertical to rank 0-7 | ✅ |
| Horizontal calculation | `Square::from_coords(File::new(val), from.rank())` | `square_Make(val, from_rank)` | ✅ |
| Vertical calculation | `Square::from_coords(from.file(), Rank::new(val-8))` | `square_Make(from_file, val-8)` | ✅ |
| Value > 15 handling | Returns error | No explicit check (relies on encoder) | ✅ (safer) |

**Verification with Examples** (Rook on D4):

| Move | val | Current Implementation | SCID | Match? |
|------|-----|------------------------|------|--------|
| Rd4-a4 | 0 | from_coords(File(0), Rank(3)) = A4 | square_Make(0, 3) = A4 | ✅ |
| Rd4-h4 | 7 | from_coords(File(7), Rank(3)) = H4 | square_Make(7, 3) = H4 | ✅ |
| Rd4-d1 | 8 | from_coords(File(3), Rank(0)) = D1 | square_Make(3, 0) = D1 | ✅ |
| Rd4-d8 | 15 | from_coords(File(3), Rank(7)) = D8 | square_Make(3, 7) = D8 | ✅ |

### 6F. Status Summary

| Aspect | Status |
|--------|--------|
| Algorithm | ✅ Correct |
| Value interpretation | ✅ Correct |
| Horizontal moves (0-7) | ✅ Correct |
| Vertical moves (8-15) | ✅ Correct |
| Edge case handling | ✅ Correct (rejects val > 15) |

**Status**: ✅ **VERIFIED CORRECT** - Rook encoding matches SCID exactly.

---

## 7. Queen Move Encoding

### 7A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59232-59262 (encodeQueen)

```cpp
// encodeQueen(): encoding Queen moves.
static inline void
encodeQueen (ByteBuffer * buf, simpleMoveT * sm)
{
    // We cannot fit all Queen moves in one byte, so Rooklike moves
    // are in one byte (encoded the same way as Rook moves),
    // while diagonal moves are in two bytes.
    ASSERT (sm->to <= H8  &&  sm->from <= H8);
    byte val;
    if (square_Rank(sm->from) == square_Rank(sm->to)) {
        // Rook-horizontal move:
        val = square_Fyle(sm->to);
        buf->PutByte (makeMoveByte (sm->pieceNum, val));
    } else if (square_Fyle(sm->from) == square_Fyle(sm->to)) {
        // Rook-vertical move:
        val = 8 + square_Rank(sm->to);
        buf->PutByte (makeMoveByte (sm->pieceNum, val));
    } else {
        // Diagonal move:
        ASSERT (dirIsDiagonal [sqDir [sm->from][sm->to]]);
        // First, we put a rook-horizontal move to the from square (which
        // is illegal of course) to indicate it is NOT a rooklike move:
        val = square_Fyle(sm->from);
        buf->PutByte (makeMoveByte (sm->pieceNum, val));
        // Now we put the to-square in the next byte. We add a 64 to it
        // to make sure that it cannot clash with the Special tokens (which
        // are in the range 0 to 15, since they are special King moves).
        buf->PutByte (sm->to + 64);
    }
}
```

**Key Insight**: Queen moves use a clever variable-length encoding:
- **Rook-like moves (horizontal/vertical)**: 1 byte, encoded same as Rook
- **Diagonal moves**: 2 bytes, using an "impossible" move as a signal

**Encoding Logic**:
| Move Type | Bytes | First Byte | Second Byte |
|-----------|-------|------------|-------------|
| Horizontal (same rank) | 1 | target_file (0-7) | - |
| Vertical (same file) | 1 | 8 + target_rank (8-15) | - |
| Diagonal | 2 | from_file (0-7) | target_square + 64 (64-127) |

**The Diagonal Trick**: For diagonal moves, the first byte is set to `from_file`. Since a horizontal move to the same file would be a "move to self" (illegal), this signals that a second byte follows containing the actual target square.

### 7B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59264-59282 (decodeQueen)

```cpp
// decodeQueen(): decoding Queen moves.
static inline errorT
decodeQueen (ByteBuffer * buf, byte val, simpleMoveT * sm)
{
    if (val >= 8) {
        // Rook-vertical move:
        sm->to = square_Make (square_Fyle(sm->from), (val - 8));
    } else if (val != square_Fyle(sm->from)) {
        // Rook-horizontal move:
        sm->to = square_Make (val, square_Rank(sm->from));
    } else {
        // Diagonal move: coded in TWO bytes.
        val = buf->GetByte();
        if (val < 64  ||  val > 127) { return ERROR_Decode; }
        sm->to = val - 64;
    }
    return OK;
}
```

**Decoding Algorithm**:
1. If `val >= 8`: Vertical move (1 byte) - keep same file, target rank = `val - 8`
2. Else if `val != from_file`: Horizontal move (1 byte) - target file = `val`, keep same rank
3. Else (`val == from_file`): Diagonal move (2 bytes) - read second byte, validate 64-127, target = `second_byte - 64`

### 7C. Understanding Queen Encoding

**Example**: Queen on D4 (index 27, file 3, rank 3):

```
Square Index Layout:
A8=56  B8=57  C8=58  D8=59  E8=60  F8=61  G8=62  H8=63   (rank 7)
A7=48  B7=49  C7=50  D7=51  E7=52  F7=53  G7=54  H7=55   (rank 6)
A6=40  B6=41  C6=42  D6=43  E6=44  F6=45  G6=46  H6=47   (rank 5)
A5=32  B5=33  C5=34  D5=35  E5=36  F5=37  G5=38  H5=39   (rank 4)
A4=24  B4=25  C4=26  D4=27  E4=28  F4=29  G4=30  H4=31   (rank 3) <- Queen on D4
A3=16  B3=17  C3=18  D3=19  E3=20  F3=21  G3=22  H3=23   (rank 2)
A2=8   B2=9   C2=10  D2=11  E2=12  F2=13  G2=14  H2=15   (rank 1)
A1=0   B1=1   C1=2   D1=3   E1=4   F1=5   G1=6   H1=7    (rank 0)
```

| Move | Target | Type | Byte 1 | Byte 2 | Decoding |
|------|--------|------|--------|--------|----------|
| Qd4-a4 | A4=24 | Horizontal | 0 | - | val=0, 0≠3, horizontal to file 0 |
| Qd4-h4 | H4=31 | Horizontal | 7 | - | val=7, 7≠3, horizontal to file 7 |
| Qd4-d1 | D1=3 | Vertical | 8 | - | val=8≥8, vertical to rank 0 |
| Qd4-d8 | D8=59 | Vertical | 15 | - | val=15≥8, vertical to rank 7 |
| Qd4-f6 | F6=45 | Diagonal | 3 | 109 (45+64) | val=3=from_file, read 109, target=45 |
| Qd4-a7 | A7=48 | Diagonal | 3 | 112 (48+64) | val=3=from_file, read 112, target=48 |
| Qd4-g1 | G1=6 | Diagonal | 3 | 70 (6+64) | val=3=from_file, read 70, target=6 |

**Why +64 for diagonal targets?**
Adding 64 ensures the second byte is always in range 64-127 (since square indices are 0-63). This prevents collision with:
- Special tokens (0x0B-0x0F = 11-15) which are King piece + values 11-15
- Any other low byte values that might be misinterpreted

### 7D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 346-393

```rust
/// Queen move decoder with ByteStream support for diagonal moves
///
/// CRITICAL: Queen diagonal moves require 2 bytes!
///
/// Move values:
/// - 0-7: Horizontal move to file a-h (1 byte)
/// - 8-15: Vertical move to rank 1-8 (1 byte)
/// - If move_value == from_file: Diagonal move (2 bytes!)
///   - Second byte: target square + 64
pub fn decode_queen_move(
    from: Square,
    move_value: u8,
    stream: &mut ByteStream,
) -> Result<DecodedMove> {
    let from_file = from.file() as u8;

    let to = if move_value >= 8 {
        // Vertical move (rook-like, 1 byte)
        let target_rank = move_value - 8;
        Square::from_coords(from.file(), Rank::new(target_rank as u32))
    } else if move_value != from_file {
        // Horizontal move (rook-like, 1 byte)
        Square::from_coords(File::new(move_value as u32), from.rank())
    } else {
        // Diagonal move (bishop-like, 2 bytes)
        // Read second byte from stream!
        let second_byte = stream.get_byte()?;

        // SCID validation: must be in range [64, 127]
        if !(64..=127).contains(&second_byte) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("decoder"),
                offset: 0,
                message: format!("Invalid queen diagonal target byte: {}", second_byte),
            });
        }

        // Target square = second_byte - 64
        let target_index = (second_byte - 64) as u32;
        Square::new(target_index)
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
    })
}
```

### 7E. Verification

**Comparison Table**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| Vertical detection | `move_value >= 8` | `val >= 8` | ✅ |
| Vertical calculation | `from.file(), Rank::new(val-8)` | `square_Make(from_file, val-8)` | ✅ |
| Horizontal detection | `move_value != from_file` | `val != from_file` | ✅ |
| Horizontal calculation | `File::new(val), from.rank()` | `square_Make(val, from_rank)` | ✅ |
| Diagonal detection | `move_value == from_file` | `val == from_file` | ✅ |
| Diagonal: read 2nd byte | `stream.get_byte()` | `buf->GetByte()` | ✅ |
| Diagonal: validation | `64..=127` | `< 64 \|\| > 127` | ✅ |
| Diagonal: target calc | `second_byte - 64` | `val - 64` | ✅ |

**Verification with Examples** (Queen on D4, from_file=3):

| Move | val | 2nd byte | Current Implementation | SCID | Match? |
|------|-----|----------|------------------------|------|--------|
| Qd4-a4 | 0 | - | 0≠3 → horizontal, File(0) | horizontal to file 0 | ✅ |
| Qd4-h4 | 7 | - | 7≠3 → horizontal, File(7) | horizontal to file 7 | ✅ |
| Qd4-d1 | 8 | - | 8≥8 → vertical, Rank(0) | vertical to rank 0 | ✅ |
| Qd4-d8 | 15 | - | 15≥8 → vertical, Rank(7) | vertical to rank 7 | ✅ |
| Qd4-f6 | 3 | 109 | 3=3 → diagonal, 109-64=45=F6 | diagonal to 45 | ✅ |
| Qd4-a7 | 3 | 112 | 3=3 → diagonal, 112-64=48=A7 | diagonal to 48 | ✅ |

### 7F. Status Summary

| Aspect | Status |
|--------|--------|
| Algorithm | ✅ Correct |
| Vertical moves (val ≥ 8) | ✅ Correct |
| Horizontal moves (val ≠ from_file) | ✅ Correct |
| Diagonal detection (val = from_file) | ✅ Correct |
| 2-byte diagonal handling | ✅ Correct |
| Second byte validation (64-127) | ✅ Correct |
| ByteStream integration | ✅ Correct |

**Status**: ✅ **VERIFIED CORRECT** - Queen encoding matches SCID exactly.

---

## 8. Pawn Encoding (WRONG for Black!)

### 8A. SCID Encoding Function

**File**: `repomix-scidvspc.xml`, lines 59284-59321

```cpp
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// encodePawn(): encoding Pawn moves.
//
static inline void
encodePawn (ByteBuffer * buf, simpleMoveT * sm)
{
    // Pawn moves require a promotion encoding.
    // The pawn moves are:
    // 0 = capture-left,
    // 1 = forward,
    // 2 = capture-right (all no promotion);
    //    3/4/5 = 0/1/2 with Queen promo;
    //    6/7/8 = 0/1/2 with Rook promo;
    //  9/10/11 = 0/1/2 with Bishop promo;
    // 12/13/14 = 0/1/2 with Knight promo;
    // 15 = forward TWO squares.
    byte val;
    int diff = (int)(sm->to) - (int)(sm->from);
    if (diff < 0) { diff = -diff; }
    if (diff == 16) { // Move forward two squares
        val = 15;
        ASSERT (sm->promote == EMPTY);
    } else {
        if (diff == 7) { val = 0; }
        else if (diff == 8) { val = 1; }
        else {  // diff is 9:
            ASSERT (diff == 9);
            val = 2;
        }
        if (sm->promote != EMPTY) {
            // Handle promotions.
            // sm->promote must be Queen=2,Rook=3, Bishop=4 or Knight=5.
            // We add 3 for Queen, 6 for Rook, 9 for Bishop, 12 for Knight.
            ASSERT (sm->promote >= QUEEN  &&  sm->promote <= KNIGHT);
            val += 3 * ((sm->promote) - 1);
        }
    }
    buf->PutByte (makeMoveByte (sm->pieceNum, val));
}
```

### 8B. SCID Decoding Function

**File**: `repomix-scidvspc.xml`, lines 59323-59348

```cpp
//~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// decodePawn(): decoding Pawn moves.
//
static inline errorT
decodePawn (byte val, simpleMoveT * sm, colorT toMove)
{
    static const int
    toSquareDiff [16] = {
        7,8,9, 7,8,9, 7,8,9, 7,8,9, 7,8,9, 16
    };
    static const pieceT
    promoPieceFromVal [16] = {
        EMPTY,EMPTY,EMPTY,
        QUEEN,QUEEN,QUEEN,
        ROOK,ROOK,ROOK,
        BISHOP,BISHOP,BISHOP,
        KNIGHT,KNIGHT,KNIGHT,
        EMPTY
    };
    if (toMove == WHITE) {
        sm->to = sm->from + toSquareDiff[val];
    } else {
        sm->to = sm->from - toSquareDiff[val];
    }
    sm->promote = promoPieceFromVal[val];
    return OK;
}
```

### 8C. Understanding Pawn Encoding

**Key Insight**: SCID uses **absolute square index differences**, not file/rank deltas.

**Square Index Layout**:
```
    a   b   c   d   e   f   g   h
8: 56  57  58  59  60  61  62  63
7: 48  49  50  51  52  53  54  55
6: 40  41  42  43  44  45  46  47
5: 32  33  34  35  36  37  38  39
4: 24  25  26  27  28  29  30  31
3: 16  17  18  19  20  21  22  23
2:  8   9  10  11  12  13  14  15
1:  0   1   2   3   4   5   6   7
```

**Square Difference Lookup Table**:
```cpp
toSquareDiff[16] = {7,8,9, 7,8,9, 7,8,9, 7,8,9, 7,8,9, 16}
```

| val | Pattern       | Diff | Meaning                          |
|-----|---------------|------|----------------------------------|
| 0   | capture-left  | 7    | No promotion, capture toward a-file (from White view) |
| 1   | forward       | 8    | No promotion, forward one        |
| 2   | capture-right | 9    | No promotion, capture toward h-file (from White view) |
| 3   | capture-left  | 7    | Queen promotion, capture         |
| 4   | forward       | 8    | Queen promotion, forward         |
| 5   | capture-right | 9    | Queen promotion, capture         |
| 6   | capture-left  | 7    | Rook promotion, capture          |
| 7   | forward       | 8    | Rook promotion, forward          |
| 8   | capture-right | 9    | Rook promotion, capture          |
| 9   | capture-left  | 7    | Bishop promotion, capture        |
| 10  | forward       | 8    | Bishop promotion, forward        |
| 11  | capture-right | 9    | Bishop promotion, capture        |
| 12  | capture-left  | 7    | Knight promotion, capture        |
| 13  | forward       | 8    | Knight promotion, forward        |
| 14  | capture-right | 9    | Knight promotion, capture        |
| 15  | double push   | 16   | No promotion, forward two        |

**The Color-Dependent Calculation**:

```cpp
if (toMove == WHITE) {
    sm->to = sm->from + toSquareDiff[val];  // ADD for White
} else {
    sm->to = sm->from - toSquareDiff[val];  // SUBTRACT for Black
}
```

**Critical**: The same `val` produces **opposite file effects** for White vs Black!

**White Pawn Example (e2, index=12)**:
| val | diff | Calculation | Result | Target |
|-----|------|-------------|--------|--------|
| 0   | 7    | 12 + 7      | 19     | d3 (toward a-file) |
| 1   | 8    | 12 + 8      | 20     | e3 (forward) |
| 2   | 9    | 12 + 9      | 21     | f3 (toward h-file) |
| 15  | 16   | 12 + 16     | 28     | e4 (double push) |

**Black Pawn Example (e7, index=52)**:
| val | diff | Calculation | Result | Target |
|-----|------|-------------|--------|--------|
| 0   | 7    | 52 - 7      | 45     | f6 (toward h-file) |
| 1   | 8    | 52 - 8      | 44     | e6 (forward) |
| 2   | 9    | 52 - 9      | 43     | d6 (toward a-file) |
| 15  | 16   | 52 - 16     | 36     | e5 (double push) |

**Understanding the Terminology**:
- "capture-left" means "capture to your left from your perspective"
- White sits at rank 1, so White's "left" = toward a-file (lower file index)
- Black sits at rank 8, so Black's "left" = toward h-file (higher file index)
- The SAME val value achieves this through ADD vs SUBTRACT!

### 8D. Current Implementation Status

**File**: `crates/core/src/parser/decoder.rs`, lines 288-344

```rust
pub fn decode_pawn_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    let forward = match color {
        Color::White => 1i8,
        Color::Black => -1i8,
    };

    let from_file = from.file() as i8;
    let from_rank = from.rank() as i8;

    let (file_delta, rank_delta, promotion) = match move_value {
        0 => (-1, forward, None),                // Capture left  <- WRONG FOR BLACK!
        1 => (0, forward, None),                 // Forward one
        2 => (1, forward, None),                 // Capture right <- WRONG FOR BLACK!
        3 => (-1, forward, Some(Role::Queen)),   // Queen promo   <- WRONG FOR BLACK!
        4 => (0, forward, Some(Role::Queen)),
        5 => (1, forward, Some(Role::Queen)),    //               <- WRONG FOR BLACK!
        6 => (-1, forward, Some(Role::Rook)),    //               <- WRONG FOR BLACK!
        7 => (0, forward, Some(Role::Rook)),
        8 => (1, forward, Some(Role::Rook)),     //               <- WRONG FOR BLACK!
        9 => (-1, forward, Some(Role::Bishop)),  //               <- WRONG FOR BLACK!
        10 => (0, forward, Some(Role::Bishop)),
        11 => (1, forward, Some(Role::Bishop)),  //               <- WRONG FOR BLACK!
        12 => (-1, forward, Some(Role::Knight)), //               <- WRONG FOR BLACK!
        13 => (0, forward, Some(Role::Knight)),
        14 => (1, forward, Some(Role::Knight)),  //               <- WRONG FOR BLACK!
        15 => (0, forward * 2, None),            // Double push - CORRECT
        // ...
    };

    let new_file = from_file + file_delta;
    let new_rank = from_rank + rank_delta;
    // ...
}
```

**Problem**: Uses fixed file deltas (`-1` for "left", `+1` for "right") regardless of color.

### 8E. Verification

**Comparison Table**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| Forward move detection | val 1, 4, 7, 10, 13 | same | ✅ |
| Forward calculation | `from_rank + forward` | `from +/- 8` | ✅ |
| Double push detection | val 15 | val 15 | ✅ |
| Double push calculation | `from_rank + 2*forward` | `from +/- 16` | ✅ |
| Capture-left for White | file_delta = -1 | from + 7 | ✅ |
| Capture-left for Black | file_delta = -1 | from - 7 | ❌ |
| Capture-right for White | file_delta = +1 | from + 9 | ✅ |
| Capture-right for Black | file_delta = +1 | from - 9 | ❌ |
| Promotion detection | val / 3 | promoPieceFromVal[val] | ✅ |
| Promotion pieces | Queen/Rook/Bishop/Knight | QUEEN/ROOK/BISHOP/KNIGHT | ✅ |

**White Pawn Verification (e2, val=0 capture-left)**:
| Aspect | Current | SCID | Match? |
|--------|---------|------|--------|
| From square | E2 | E2 | ✅ |
| file_delta | -1 | N/A | - |
| Calculation | E2: file 4-1=3, rank 1+1=2 | 12 + 7 = 19 | - |
| Target | D3 (file 3, rank 2) | D3 (index 19) | ✅ |

**Black Pawn Verification (e7, val=0 capture-left)**:
| Aspect | Current | SCID | Match? |
|--------|---------|------|--------|
| From square | E7 | E7 | ✅ |
| file_delta | -1 | N/A | - |
| Calculation | E7: file 4-1=3, rank 6-1=5 | 52 - 7 = 45 | - |
| Target | **D6** (file 3, rank 5) | **F6** (index 45) | ❌ **WRONG!** |

**Black Pawn Verification (e7, val=2 capture-right)**:
| Aspect | Current | SCID | Match? |
|--------|---------|------|--------|
| From square | E7 | E7 | ✅ |
| file_delta | +1 | N/A | - |
| Calculation | E7: file 4+1=5, rank 6-1=5 | 52 - 9 = 43 | - |
| Target | **F6** (file 5, rank 5) | **D6** (index 43) | ❌ **WRONG!** |

### 8F. Status Summary

| Aspect | Status |
|--------|--------|
| Move value mapping | ✅ Correct |
| Double push (val 15) | ✅ Correct |
| Forward moves (val 1,4,7,10,13) | ✅ Correct |
| White captures (val 0,2,3,5,6,8,9,11,12,14) | ✅ Correct |
| Black captures (val 0,2,3,5,6,8,9,11,12,14) | ❌ Wrong |
| Promotion piece mapping | ✅ Correct |
| Rank direction | ✅ Correct |
| File direction (color-dependent) | ❌ Wrong for Black |

**Required Fix**: For Black, reverse the file deltas for captures:

```rust
// Correct approach - color-dependent capture directions
let capture_left_file_delta = match color {
    Color::White => -1,  // toward a-file (lower file index)
    Color::Black => 1,   // toward h-file (higher file index)
};

let (file_delta, rank_delta, promotion) = match move_value {
    0 => (capture_left_file_delta, forward, None),       // Capture left
    1 => (0, forward, None),                              // Forward one
    2 => (-capture_left_file_delta, forward, None),      // Capture right
    3 => (capture_left_file_delta, forward, Some(Role::Queen)),
    4 => (0, forward, Some(Role::Queen)),
    5 => (-capture_left_file_delta, forward, Some(Role::Queen)),
    // ... etc for all promotion types
    15 => (0, forward * 2, None),                         // Double push
    // ...
};
```

**Alternative (cleaner) approach - use square index arithmetic like SCID**:

```rust
// More direct translation of SCID approach
const TO_SQUARE_DIFF: [i8; 16] = [7,8,9, 7,8,9, 7,8,9, 7,8,9, 7,8,9, 16];
const PROMO_PIECE: [Option<Role>; 16] = [
    None,None,None,
    Some(Role::Queen),Some(Role::Queen),Some(Role::Queen),
    Some(Role::Rook),Some(Role::Rook),Some(Role::Rook),
    Some(Role::Bishop),Some(Role::Bishop),Some(Role::Bishop),
    Some(Role::Knight),Some(Role::Knight),Some(Role::Knight),
    None,
];

let from_index = from.rank() as i8 * 8 + from.file() as i8;
let diff = TO_SQUARE_DIFF[move_value as usize];
let to_index = match color {
    Color::White => from_index + diff,
    Color::Black => from_index - diff,
};
let to = Square::new(to_index as u32);
let promotion = PROMO_PIECE[move_value as usize];
```

**Status**: ❌ **WRONG** - Black pawn captures use incorrect file direction.

---

## 9. Piece List Management & Position Updates

This section documents the complete SCID piece list management system, which is critical for correctly decoding move bytes. Move bytes encode **piece numbers**, so the piece list must be kept in sync with SCID's algorithms.

### 9A. SCID Data Structures

**File**: `repomix-scidvspc.xml`, Position class

```cpp
// Core data structures for piece tracking
squareT  List[2][16];     // List[color][piece_num] = square
byte     ListPos[64];     // ListPos[square] = piece_num (reverse lookup)
byte     Count[2];        // Count[color] = number of active pieces (0-16)
pieceT   Board[66];       // Board[square] = piece type at that square
byte     Material[16];    // Material[piece_type] = count of that piece type
```

**simpleMoveT Structure** (lines 65833-65848):
```cpp
struct simpleMoveT {
    byte     pieceNum;        // Piece number (0-15) of moving piece
    pieceT   movingPiece;     // Type of moving piece
    squareT  from;            // Source square
    squareT  to;              // Destination square
    byte     capturedNum;     // Piece number of captured piece (for undo)
    pieceT   capturedPiece;   // Type of captured piece
    pieceT   promote;         // Promotion piece type (or EMPTY)
    squareT  capturedSquare;  // Square of captured piece (different from 'to' for en passant)
    byte     castleFlags;     // Pre-move castling rights (for undo)
    squareT  epSquare;        // Pre-move en passant square (for undo)
    ushort   oldHalfMoveClock;// Pre-move half-move clock (for undo)
    int      score;           // Used for move ordering
};
```

### 9B. Standard Position Initialization

**File**: `repomix-scidvspc.xml`, lines 77808-77838 (Position::StdStart)

```cpp
void Position::StdStart (void)
{
    Clear();
    Material[WK] = Material[BK] = 1;
    Material[WQ] = Material[BQ] = 1;
    Material[WR] = Material[BR] = 2;
    Material[WB] = Material[BB] = 2;
    Material[WN] = Material[BN] = 2;
    Material[WP] = Material[BP] = 8;
    Count[WHITE] = Count[BLACK] = 16;

    // Hardcoded piece numbering (SAME for White and Black):
    AddToBoard(WK, E1);  List[WHITE][0] = E1;  ListPos[E1] = 0;   // 0: King
    AddToBoard(BK, E8);  List[BLACK][0] = E8;  ListPos[E8] = 0;
    AddToBoard(WR, A1);  List[WHITE][1] = A1;  ListPos[A1] = 1;   // 1: Queen's Rook
    AddToBoard(BR, A8);  List[BLACK][1] = A8;  ListPos[A8] = 1;
    AddToBoard(WN, B1);  List[WHITE][2] = B1;  ListPos[B1] = 2;   // 2: Queen's Knight
    AddToBoard(BN, B8);  List[BLACK][2] = B8;  ListPos[B8] = 2;
    AddToBoard(WB, C1);  List[WHITE][3] = C1;  ListPos[C1] = 3;   // 3: Queen's Bishop
    AddToBoard(BB, C8);  List[BLACK][3] = C8;  ListPos[C8] = 3;
    AddToBoard(WQ, D1);  List[WHITE][4] = D1;  ListPos[D1] = 4;   // 4: Queen
    AddToBoard(BQ, D8);  List[BLACK][4] = D8;  ListPos[D8] = 4;
    AddToBoard(WB, F1);  List[WHITE][5] = F1;  ListPos[F1] = 5;   // 5: King's Bishop
    AddToBoard(BB, F8);  List[BLACK][5] = F8;  ListPos[F8] = 5;
    AddToBoard(WN, G1);  List[WHITE][6] = G1;  ListPos[G1] = 6;   // 6: King's Knight
    AddToBoard(BN, G8);  List[BLACK][6] = G8;  ListPos[G8] = 6;
    AddToBoard(WR, H1);  List[WHITE][7] = H1;  ListPos[H1] = 7;   // 7: King's Rook
    AddToBoard(BR, H8);  List[BLACK][7] = H8;  ListPos[H8] = 7;
    for (uint i=0; i < 8; i++) {
        AddToBoard(WP, A2+i); List[WHITE][i+8] = A2+i; ListPos[A2+i] = i+8;  // 8-15: Pawns a-h
        AddToBoard(BP, A7+i); List[BLACK][i+8] = A7+i; ListPos[A7+i] = i+8;
    }
    // ... castling, EP, etc.
}
```

**Standard Position Piece Numbers**:
| Piece # | White Piece | White Square | Black Piece | Black Square |
|---------|-------------|--------------|-------------|--------------|
| 0 | King | E1 | King | E8 |
| 1 | Queen's Rook | A1 | Queen's Rook | A8 |
| 2 | Queen's Knight | B1 | Queen's Knight | B8 |
| 3 | Queen's Bishop | C1 | Queen's Bishop | C8 |
| 4 | Queen | D1 | Queen | D8 |
| 5 | King's Bishop | F1 | King's Bishop | F8 |
| 6 | King's Knight | G1 | King's Knight | G8 |
| 7 | King's Rook | H1 | King's Rook | H8 |
| 8 | a-pawn | A2 | a-pawn | A7 |
| 9 | b-pawn | B2 | b-pawn | B7 |
| 10 | c-pawn | C2 | c-pawn | C7 |
| 11 | d-pawn | D2 | d-pawn | D7 |
| 12 | e-pawn | E2 | e-pawn | E7 |
| 13 | f-pawn | F2 | f-pawn | F7 |
| 14 | g-pawn | G2 | g-pawn | G7 |
| 15 | h-pawn | H2 | h-pawn | H7 |

### 9C. FEN Position Initialization

**File**: `repomix-scidvspc.xml`, lines 77883-77908 (Position::AddPiece)

```cpp
errorT Position::AddPiece (pieceT p, squareT sq)
{
    ASSERT (p != EMPTY);
    colorT c = piece_Color(p);
    if (Count[c] == 16) { return ERROR_PieceCount; }
    ASSERT(Count[c] <= 15);

    if (piece_Type(p) == KING) {
        // King is ALWAYS piece #0 - if we already have pieces, swap!
        if (Material[p] > 0) { return ERROR_PieceCount; }  // Already have a king
        if (Count[c] > 0) {
            // Move the piece at slot 0 to the end
            squareT oldsq = List[c][0];
            List[c][Count[c]] = oldsq;
            ListPos[oldsq] = Count[c];
        }
        List[c][0] = sq;
        ListPos[sq] = 0;
    } else {
        // Non-king pieces get sequential numbers
        ListPos[sq] = Count[c];
        List[c][Count[c]] = sq;
    }
    Count[c]++;
    Material[p]++;
    AddToBoard (p, sq);
    return OK;
}
```

**File**: `repomix-scidvspc.xml`, lines 79797-79841 (Position::ReadFromFEN)

```cpp
errorT Position::ReadFromFEN (const char * str)
{
    // FEN square mapping: index 0 = A8, 1 = B8, ... 63 = H1
    static squareT fenSqToRealSquare [64];
    // ... initialization ...
    for (int sq=0; sq < 64; sq++) {
        fenSqToRealSquare [sq] = (squareT)((7 - (sq)/8) * 8 + ((sq) % 8));
    }

    Clear ();
    while (count < 64) {
        // ... parse FEN character ...
        pieceT p = pieceFromByte [(byte) *s];
        if (p == EMPTY) { return ERROR_InvalidFEN; }
        if (AddPiece (p, fenSqToRealSquare[count]) != OK) {
            return ERROR_InvalidFEN;
        }
        count++;
        s++;
    }
    // ...
}
```

**FEN Scan Order**: A8 → B8 → ... → H8 → A7 → ... → H1 (rank 8 to rank 1, file a to file h)

**FEN Piece Numbering Rules**:
1. **King is ALWAYS piece #0** - When the King is encountered, if other pieces have already been added, the piece at slot 0 is moved to the end, and King takes slot 0.
2. **Other pieces get sequential numbers** in FEN scan order.
3. FEN positions may have different piece numbering than standard positions!

**Example**: FEN `"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1"` (after 1. e4)

Black piece scan order (A8 → H8, then A7 → H7):
- A8: Rook → piece #0 (but will be swapped when King found)
- B8: Knight → piece #1
- C8: Bishop → piece #2
- D8: Queen → piece #3
- E8: King → **KING found!** Rook at slot 0 moves to slot 4, King becomes #0
- F8: Bishop → piece #5
- G8: Knight → piece #6
- H8: Rook → piece #7
- A7-H7: Pawns → pieces #8-15

**Result for Black after FEN parsing**:
| Piece # | Piece | Square |
|---------|-------|--------|
| 0 | King | E8 |
| 1 | Knight | B8 |
| 2 | Bishop | C8 |
| 3 | Queen | D8 |
| 4 | Rook | A8 (was #0, swapped) |
| 5 | Bishop | F8 |
| 6 | Knight | G8 |
| 7 | Rook | H8 |
| 8-15 | Pawns | A7-H7 |

**⚠️ IMPORTANT**: This is DIFFERENT from standard position numbering! Standard has Rook at #1, FEN has it at #4.

### 9D. DoSimpleMove - Complete Move Making

**File**: `repomix-scidvspc.xml`, lines 78868-78965 (Position::DoSimpleMove)

```cpp
void Position::DoSimpleMove (simpleMoveT * sm)
{
    squareT from = sm->from;
    squareT to = sm->to;
    pieceT p = Board[from];
    pieceT ptype = piece_Type(p);
    colorT enemy = color_Flip(ToMove);

    // ============ STEP 1: Record piece number ============
    sm->pieceNum = ListPos[from];
    sm->capturedPiece = Board[to];
    sm->capturedSquare = to;
    sm->castleFlags = Castling;
    sm->epSquare = EPTarget;
    sm->oldHalfMoveClock = HalfMoveClock;
    HalfMoveClock++;
    PlyCounter++;

    // ============ STEP 2: Handle null move ============
    if (isNullMove(sm)) {
        ToMove = enemy;
        EPTarget = NULL_SQUARE;
        return;
    }

    // ============ STEP 3: Handle en passant ============
    if (ptype == PAWN && sm->capturedPiece == EMPTY
            && square_Fyle(from) != square_Fyle(to)) {
        pieceT enemyPawn = piece_Make(enemy, PAWN);
        sm->capturedSquare = (ToMove == WHITE ? (to - 8) : (to + 8));
        sm->capturedPiece = enemyPawn;
    }

    // ============ STEP 4: Handle captures (SWAP ALGORITHM) ============
    if (sm->capturedPiece != EMPTY) {
        sm->capturedNum = ListPos[sm->capturedSquare];
        Count[enemy]--;
        // SWAP: Move last piece into captured slot
        ListPos[List[enemy][Count[enemy]]] = sm->capturedNum;
        List[enemy][sm->capturedNum] = List[enemy][Count[enemy]];
        Material[sm->capturedPiece]--;
        HalfMoveClock = 0;
        RemoveFromBoard (sm->capturedPiece, sm->capturedSquare);
    }

    // ============ STEP 5: Handle promotion ============
    if (sm->promote != EMPTY) {
        Material[p]--;
        RemoveFromBoard (p, from);
        p = piece_Make(ToMove, sm->promote);
        Material[p]++;
        AddToBoard (p, from);
        // NOTE: Piece number does NOT change! List/ListPos stay the same.
    }

    // ============ STEP 6: Make the actual move ============
    List[ToMove][sm->pieceNum] = to;
    ListPos[to] = sm->pieceNum;
    RemoveFromBoard (p, from);
    AddToBoard (p, to);

    // ============ STEP 7: Handle castling ============
    if (ptype == KING && square_Fyle(from) == E_FYLE &&
            (square_Fyle(to) == C_FYLE || square_Fyle(to) == G_FYLE)) {
        squareT rookfrom, rookto;
        pieceT rook = piece_Make (ToMove, ROOK);
        if (square_Fyle(to) == C_FYLE) {
            rookfrom = to - 2;   // A1/A8
            rookto = to + 1;     // D1/D8
        } else {
            rookfrom = to + 1;   // H1/H8
            rookto = to - 1;     // F1/F8
        }
        ListPos[rookto] = ListPos[rookfrom];
        List[ToMove][ListPos[rookto]] = rookto;
        RemoveFromBoard (rook, rookfrom);
        AddToBoard (rook, rookto);
    }

    // ============ STEP 8: Update castling rights, EP square, etc. ============
    // ... (omitted for brevity)

    ToMove = enemy;
}
```

### 9E. UndoSimpleMove - Complete Move Unmaking

**File**: `repomix-scidvspc.xml`, lines 78992-79060 (Position::UndoSimpleMove)

```cpp
void Position::UndoSimpleMove (simpleMoveT * m)
{
    squareT from = m->from;
    squareT to = m->to;
    pieceT p = Board[to];
    EPTarget = m->epSquare;
    Castling = m->castleFlags;
    HalfMoveClock = m->oldHalfMoveClock;
    PlyCounter--;
    ToMove = color_Flip(ToMove);
    m->pieceNum = ListPos[to];

    // ============ Handle null move ============
    if (isNullMove(m)) {
        return;
    }

    // ============ Handle capture undo (REVERSE SWAP) ============
    if (m->capturedPiece != EMPTY) {
        colorT c = color_Flip(ToMove);
        // Reverse the swap: put the piece that was swapped back to the end
        ListPos[List[c][m->capturedNum]] = Count[c];
        ListPos[m->capturedSquare] = m->capturedNum;
        List[c][Count[c]] = List[c][m->capturedNum];
        List[c][m->capturedNum] = m->capturedSquare;
        Material[m->capturedPiece]++;
        Count[c]++;
    }

    // ============ Handle promotion undo ============
    if (m->promote != EMPTY) {
        Material[p]--;
        RemoveFromBoard (p, to);
        p = piece_Make(ToMove, PAWN);
        Material[p]++;
        AddToBoard (p, to);
    }

    // ============ Unmake the move ============
    List[ToMove][m->pieceNum] = from;
    ListPos[from] = m->pieceNum;
    RemoveFromBoard (p, to);
    AddToBoard (p, from);
    if (m->capturedPiece != EMPTY) {
        AddToBoard (m->capturedPiece, m->capturedSquare);
    }

    // ============ Handle castling undo ============
    if ((piece_Type(p) == KING) && square_Fyle(from) == E_FYLE
            && (square_Fyle(to) == C_FYLE || square_Fyle(to) == G_FYLE)) {
        squareT rookfrom, rookto;
        pieceT rook = (ToMove == WHITE? WR : BR);
        if (square_Fyle(to) == C_FYLE) {
            rookfrom = to - 2;   rookto = to + 1;
        } else {
            rookfrom = to + 1;   rookto = to - 1;
        }
        ListPos[rookfrom] = ListPos[rookto];
        List[ToMove][ListPos[rookfrom]] = rookfrom;
        RemoveFromBoard (rook, rookto);
        AddToBoard (rook, rookfrom);
    }
}
```

### 9F. Understanding the Capture Swap Algorithm

**Why Swap?**: SCID maintains piece numbers as a contiguous array (0 to Count-1). When a piece is captured, a "hole" would appear. SCID fills the hole by moving the LAST piece into the captured slot.

**Capture Swap Step-by-Step**:

```
Before capture (Black has 16 pieces):
  List[BLACK][0]  = e8 (King)        ListPos[e8]  = 0
  List[BLACK][1]  = a8 (Rook)        ListPos[a8]  = 1
  ...
  List[BLACK][11] = d5 (d-pawn)      ListPos[d5]  = 11  <- TO BE CAPTURED
  ...
  List[BLACK][15] = h7 (h-pawn)      ListPos[h7]  = 15  <- LAST PIECE
  Count[BLACK] = 16

Capture d-pawn (piece #11):
  capturedNum = ListPos[d5] = 11
  Count[BLACK]-- → Count[BLACK] = 15

  // SWAP: h-pawn takes d-pawn's slot
  ListPos[List[BLACK][15]] = capturedNum    → ListPos[h7] = 11
  List[BLACK][11] = List[BLACK][15]         → List[BLACK][11] = h7

After capture:
  List[BLACK][0]  = e8 (King)        ListPos[e8]  = 0
  List[BLACK][1]  = a8 (Rook)        ListPos[a8]  = 1
  ...
  List[BLACK][11] = h7 (h-pawn)      ListPos[h7]  = 11  <- WAS #15, NOW #11!
  ...
  List[BLACK][14] = g7 (g-pawn)      ListPos[g7]  = 14  <- NOW THE LAST PIECE
  // List[BLACK][15] is now garbage (Count is 15)
  Count[BLACK] = 15
```

**Critical**: The h-pawn's piece number changed from 15 to 11! Future move bytes for the h-pawn will use piece number 11.

### 9G. Understanding Promotion

**Key Insight**: Promotion does NOT change the piece number!

```cpp
if (sm->promote != EMPTY) {
    Material[p]--;                      // Decrement pawn count
    RemoveFromBoard (p, from);          // Remove pawn from board
    p = piece_Make(ToMove, sm->promote); // Create new piece type
    Material[p]++;                      // Increment promoted piece count
    AddToBoard (p, from);               // Add promoted piece to board
}
// Then the normal move code runs:
List[ToMove][sm->pieceNum] = to;        // SAME piece number!
ListPos[to] = sm->pieceNum;             // SAME piece number!
```

A pawn that was piece #12 becomes a Queen still as piece #12. The piece TYPE changes (in Board and Material), but the piece NUMBER stays the same.

### 9H. Understanding Castling

**Key Insight**: Castling updates both the King AND the Rook in List/ListPos.

```cpp
// King already moved in Step 6

// Rook move:
ListPos[rookto] = ListPos[rookfrom];           // Rook keeps its piece number
List[ToMove][ListPos[rookto]] = rookto;        // Update List with new square
RemoveFromBoard (rook, rookfrom);
AddToBoard (rook, rookto);
```

The Rook's piece number doesn't change, but its square in List updates, and ListPos moves from old square to new square.

### 9I. Null Moves

**File**: `repomix-scidvspc.xml`, lines 65849-65853

```cpp
inline bool isNullMove (simpleMoveT * sm)
{
    return (sm->from == sm->to && sm->from != NULL_SQUARE
              && piece_Type(sm->movingPiece) == KING);
}
```

A null move is when the King "moves" to its own square. It only changes the side to move and clears the EP square.

### 9J. Current Implementation Status

**File**: `crates/core/src/parser/position.rs`

```rust
// Current capture handling (WRONG):
if chess_move.is_capture() {
    let capture_square = /* ... */;
    opponent_pieces.retain(|_, &mut sq| sq != capture_square);  // Just removes!
}

// Current promotion handling (INCOMPLETE):
// Does not explicitly preserve piece number through promotion

// Current castling handling:
// Updates king and rook squares but may not handle ListPos correctly
```

### 9K. Verification

**Comparison Table - Capture Handling**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| Get captured piece number | By square search | `ListPos[capturedSquare]` | ⚠️ Equivalent |
| Store captured piece number | Not stored | `sm->capturedNum` | ❌ Missing |
| Decrement Count | Implicit (HashMap size) | `Count[enemy]--` | ⚠️ Implicit |
| Swap last piece to slot | Not done | Yes | ❌ Missing |
| Update swapped piece's ListPos | Not done | Yes | ❌ Missing |
| Result | Piece removed with hole | Contiguous array | ❌ Different |

**Comparison Table - Promotion Handling**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| Piece number preserved | Needs verification | Yes | ⚠️ Verify |
| Material tracking | Not tracked | `Material[p]--/++` | N/A |

**Comparison Table - Castling Handling**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| King piece number preserved | Yes | Yes | ✅ |
| Rook piece number preserved | Yes | Yes | ✅ |
| ListPos updated for rook | Needs verification | Yes | ⚠️ Verify |

**Comparison Table - FEN Initialization**:
| Aspect | Current Implementation | SCID Actual | Match? |
|--------|------------------------|-------------|--------|
| King always piece #0 | Needs verification | Yes (via swap) | ⚠️ Verify |
| Other pieces sequential | Needs verification | Yes (scan order) | ⚠️ Verify |
| Scan order | Unknown | A8→H8, A7→H7, ... H1 | ⚠️ Verify |

### 9L. Status Summary

| Aspect | Status |
|--------|--------|
| **Standard Position Init** | |
| Piece numbering order | ⚠️ Verify matches SCID hardcoded order |
| List/ListPos setup | ⚠️ Verify both directions maintained |
| **FEN Position Init** | |
| Scan order (A8→H1) | ⚠️ Verify |
| King swap to slot 0 | ⚠️ Verify |
| Sequential numbering | ⚠️ Verify |
| **Capture Handling** | |
| Swap algorithm | ❌ Missing - just removes |
| Last piece renumbering | ❌ Missing |
| capturedNum storage | ❌ Missing (needed for undo) |
| **Promotion Handling** | |
| Piece number preserved | ⚠️ Verify |
| **Castling Handling** | |
| Rook ListPos update | ⚠️ Verify |
| **Null Moves** | |
| Detection (King from==to) | ⚠️ Verify |

### 9M. Required Implementation

**1. Data Structure Changes**:
```rust
pub struct ScidPieceMapping {
    // Per-color piece tracking
    white_pieces: [Option<Square>; 16],  // piece_num -> square (None if captured)
    black_pieces: [Option<Square>; 16],
    white_count: u8,                      // Number of active white pieces
    black_count: u8,                      // Number of active black pieces

    // Reverse lookup (optional but helps)
    // square_to_piece: HashMap<Square, (Color, u8)>,  // square -> (color, piece_num)
}
```

**2. Capture Handling**:
```rust
fn handle_capture(&mut self, captured_square: Square, opponent_color: Color) {
    let pieces = match opponent_color {
        Color::White => &mut self.white_pieces,
        Color::Black => &mut self.black_pieces,
    };
    let count = match opponent_color {
        Color::White => &mut self.white_count,
        Color::Black => &mut self.black_count,
    };

    // Find captured piece's number
    let captured_num = (0..*count)
        .find(|&i| pieces[i as usize] == Some(captured_square))
        .expect("Captured piece not found");

    // Decrement count
    *count -= 1;

    // Swap: move last piece to captured slot (if not already last)
    if captured_num != *count {
        pieces[captured_num as usize] = pieces[*count as usize];
    }

    // Clear last slot
    pieces[*count as usize] = None;
}
```

**3. FEN Initialization**:
```rust
fn init_from_fen(&mut self, fen: &str) {
    // Clear all
    self.white_pieces = [None; 16];
    self.black_pieces = [None; 16];
    self.white_count = 0;
    self.black_count = 0;

    // Parse FEN in order: A8, B8, ..., H8, A7, ..., H1
    for (fen_idx, piece) in fen_pieces.iter().enumerate() {
        let square = fen_index_to_square(fen_idx);  // (7 - fen_idx/8) * 8 + (fen_idx % 8)
        let color = piece.color();

        if piece.is_king() {
            // King is always slot 0 - swap if needed
            let pieces = self.pieces_mut(color);
            let count = self.count_mut(color);
            if *count > 0 {
                // Move piece at slot 0 to end
                pieces[*count as usize] = pieces[0];
            }
            pieces[0] = Some(square);
        } else {
            // Non-king gets sequential number
            let pieces = self.pieces_mut(color);
            let count = self.count_mut(color);
            pieces[*count as usize] = Some(square);
        }
        *self.count_mut(color) += 1;
    }
}
```

**Status**: ❌ **WRONG** - Multiple issues with piece list management:
- Capture handling missing swap algorithm
- FEN initialization order/algorithm needs verification
- Standard position initialization order needs verification

---

## Summary of Required Fixes

| Component | Status | Fix Required |
|-----------|--------|--------------|
| **Piece Numbering** | | |
| Standard Position | ⚠️ Verify | Verify matches SCID hardcoded order (K=0, QR=1, QN=2, QB=3, Q=4, KB=5, KN=6, KR=7, Pawns=8-15) |
| FEN Position | ⚠️ Verify | King must be #0 (swap if needed), others sequential in scan order (A8→H8, A7→H7, ...H1) |
| **Move Decoding** | | |
| King Moves | ❌ Wrong | Use square index differences, fix castling to 9/10 (not 10/11) |
| Knight Moves | ❌ Wrong | Use values 1-8 with correct square differences |
| Bishop Moves | ❌ Wrong | Implement target-file + direction-bit algorithm |
| Rook Moves | ✅ Correct | No changes needed |
| Queen Moves | ✅ Correct | No changes needed |
| Pawn Moves | ❌ Wrong | Reverse file delta for black captures (val 0,2 and promotions 3,5,6,8,9,11,12,14) |
| **Position Updates** | | |
| Capture Handling | ❌ Wrong | Implement swap algorithm: last piece moves to captured slot, decrement Count |
| Promotion Handling | ⚠️ Verify | Piece number must be preserved (pawn→Queen still same piece #) |
| Castling Handling | ⚠️ Verify | Both King and Rook List/ListPos must be updated |
| Null Moves | ⚠️ Verify | King from==to means null move (just change side to move) |
| **Special Markers** | | |
| PGN Annotations | ⚠️ Partial | Fix `END_GAME` constant (0x0F not 0x00), handle null moves (0x00) |
| Comment Encoding | ⚠️ Partial | Comments are NOT inline - `0x0C` is just a marker, text is at end of game data |

---

## Why First Move Works

The first move `1. e4` (byte `0xCF`) works because:
- Piece number 12 (upper nibble `0xC`) → e-pawn in both implementations
- Move value 15 (lower nibble `0xF`) → double push in both implementations

But subsequent moves fail because:
- Knight/Bishop/King moves use wrong encoding algorithms (square index vs file/rank deltas)
- Black pawn captures use wrong file direction
- Capture handling is WRONG: missing swap algorithm causes piece number remapping to diverge after any capture

---

## 10. PGN Tag Storage

SCID stores PGN tags in **multiple locations** across the database files, with different encoding for standard vs non-standard tags.

### 10A. Seven Tag Roster (STR) - Standard PGN Tags

The PGN specification defines seven mandatory tags called the "Seven Tag Roster":
1. Event
2. Site
3. Date
4. Round
5. White
6. Black
7. Result

**SCID stores these in TWO files:**

#### Index File (.si4) - IndexEntry Structure

**File**: `repomix-scidvspc.xml`, lines 61129-61177 (IndexEntry::Read)

```cpp
errorT IndexEntry::Read (MFile * fp, versionT version)
{
    // Game file offset and length
    Offset = fp->ReadFourBytes ();
    Length_Low = fp->ReadTwoBytes ();
    Length_High = fp->ReadOneByte();
    Flags = fp->ReadTwoBytes ();

    // White and Black player name IDs (reference to Name file)
    WhiteBlack_High = fp->ReadOneByte ();
    WhiteID_Low = fp->ReadTwoBytes ();
    BlackID_Low = fp->ReadTwoBytes ();

    // Event, Site and Round name IDs (reference to Name file)
    EventSiteRnd_High = fp->ReadOneByte ();
    EventID_Low = fp->ReadTwoBytes ();
    SiteID_Low = fp->ReadTwoBytes ();
    RoundID_Low = fp->ReadTwoBytes ();

    VarCounts = fp->ReadTwoBytes();
    EcoCode = fp->ReadTwoBytes ();

    // Date and EventDate stored in four bytes
    Dates = fp->ReadFourBytes();

    // ELO ratings
    WhiteElo = fp->ReadTwoBytes ();
    BlackElo = fp->ReadTwoBytes ();

    FinalMatSig = fp->ReadFourBytes ();
    NumHalfMoves = fp->ReadOneByte ();

    // HomePawnData (9 bytes)
    // ...
}
```

**IndexEntry Fields**:
| Field | Size | Description |
|-------|------|-------------|
| Offset | 4 bytes | Offset in .sg4 game file |
| Length | 3 bytes | Length of game data in .sg4 |
| Flags | 2 bytes | Game flags |
| WhiteID | 20 bits | ID in Name file (NAME_PLAYER) |
| BlackID | 20 bits | ID in Name file (NAME_PLAYER) |
| EventID | 19 bits | ID in Name file (NAME_EVENT) |
| SiteID | 19 bits | ID in Name file (NAME_SITE) |
| RoundID | 19 bits | ID in Name file (NAME_ROUND) |
| VarCounts | 2 bytes | Variation/NAG/Comment counts |
| EcoCode | 2 bytes | ECO classification |
| Dates | 4 bytes | Date + EventDate encoded |
| WhiteElo | 2 bytes | White player rating |
| BlackElo | 2 bytes | Black player rating |
| FinalMatSig | 4 bytes | Final material signature |
| NumHalfMoves | 10 bits | Number of half-moves |
| HomePawnData | 9 bytes | Pawn structure signature |

#### Name File (.sn4) - String Table

The Name file stores actual string values for player names, event names, site names, and round strings. IndexEntry stores **IDs** that reference entries in this file.

**File**: `repomix-scidvspc.xml`, lines 58993-58999 (LoadStandardTags)

```cpp
Game::LoadStandardTags (IndexEntry * ie, NameBase * nb)
{
    SetEventStr (ie->GetEventName (nb));
    SetSiteStr (ie->GetSiteName (nb));
    SetWhiteStr (ie->GetWhiteName (nb));
    SetBlackStr (ie->GetBlackName (nb));
    // Round and Date are handled separately
}
```

**Name Types**:
| Type | Constant | Description |
|------|----------|-------------|
| NAME_PLAYER | 0 | Player names (White, Black) |
| NAME_EVENT | 1 | Event names |
| NAME_SITE | 2 | Site names |
| NAME_ROUND | 3 | Round strings |

### 10B. Non-Standard PGN Tags

Non-standard tags are stored in the **Game file (.sg4)** at the start of each game's data block.

**File**: `repomix-scidvspc.xml`, lines 59554-59585 (encodeTags)

```cpp
static errorT encodeTags (ByteBuffer * buf, tagT * tagList, uint numTags)
{
    uint length;
    for (uint i=0; i < numTags; i++) {
        char * tag = tagList[i].tag;
        uint tagnum = 1;
        const char ** common = commonTags;

        // Check if this is a "common" tag (single-byte encoding)
        while (*common != NULL) {
            if (strEqual (tag, *common)) {
                buf->PutByte ((byte) MAX_TAG_LEN + tagnum);  // 241+
                break;
            } else {
                common++;
                tagnum++;
            }
        }

        if (*common == NULL) {
            // Not a common tag - store length + tag name
            length = strLength (tag);
            buf->PutByte ((byte) length);
            buf->PutFixedString (tag, length);
        }

        // Store tag value (always length-prefixed)
        length = strLength (tagList[i].value);
        buf->PutByte ((byte) length);
        buf->PutFixedString (tagList[i].value, length);
    }
    buf->PutByte (0);  // Null terminator
    return buf->Status();
}
```

### 10C. Common Tags Optimization

**File**: `repomix-scidvspc.xml`, lines 59529-59552

```cpp
// Common tags are encoded in one byte, as a value over 240.
// This means that the maximum length of a non-common tag is 240
// bytes, and the maximum number of common tags is 15.
const char * commonTags [255 - MAX_TAG_LEN] =
{
    // 241, 242: Country
    "WhiteCountry", "BlackCountry",
    // 243: Annotator
    "Annotator",
    // 244: PlyCount
    "PlyCount",
    // 245: EventDate (plain text encoding)
    "EventDate",
    // 246, 247: Opening, Variation
    "Opening", "Variation",
    // 248-250: Setup and Source
    "Setup", "Source", "SetUp",
    // 252-254: spare for future use
    NULL, NULL, NULL, NULL,
    // 255: Reserved for compact EventDate encoding
    NULL
};
```

**Common Tag Byte Values**:
| Byte | Tag Name |
|------|----------|
| 241 | WhiteCountry |
| 242 | BlackCountry |
| 243 | Annotator |
| 244 | PlyCount |
| 245 | EventDate |
| 246 | Opening |
| 247 | Variation |
| 248 | Setup |
| 249 | Source |
| 250 | SetUp |
| 255 | EventDate (binary 3-byte encoding) |

### 10D. Tag Decoding

**File**: `repomix-scidvspc.xml`, lines 59590-59626 (DecodeTags)

```cpp
errorT Game::DecodeTags (ByteBuffer * buf, bool storeTags)
{
    byte b;
    char tag [255];
    char value [255];
    b = buf->GetByte ();

    while (b != 0 && buf->Status() == OK) {
        if (b == 255) {
            // Special binary 3-byte encoding of EventDate
            dateT date = 0;
            b = buf->GetByte(); date = (date << 8) | b;
            b = buf->GetByte(); date = (date << 8) | b;
            b = buf->GetByte(); date = (date << 8) | b;
            SetEventDate (date);
        } else if (b > MAX_TAG_LEN) {
            // Common tag (byte 241-254) - name not stored
            char * ctag = (char *) commonTags[b - MAX_TAG_LEN - 1];
            b = buf->GetByte ();
            buf->GetFixedString (value, b);
            value[b] = '\0';
            if (storeTags) { AddPgnTag (ctag, value); }
        } else {
            // Non-common tag - name is stored
            buf->GetFixedString (tag, b);  // b = tag name length
            tag[b] = '\0';
            b = buf->GetByte ();           // value length
            buf->GetFixedString (value, b);
            value[b] = '\0';
            if (storeTags) { AddPgnTag (tag, value); }
        }
        b = buf->GetByte();
    }
    return buf->Status();
}
```

### 10E. Tag Encoding Format Summary

**Non-standard tag encoding in .sg4 game data**:

```
┌─────────────────────────────────────────────────────────────┐
│                    Tag Section                               │
├─────────────────────────────────────────────────────────────┤
│ For each tag:                                                │
│   IF common tag (WhiteCountry, Annotator, etc.):            │
│     [1 byte: 241-254 = tag ID]                              │
│     [1 byte: value length]                                  │
│     [N bytes: value string]                                 │
│   ELSE IF EventDate binary:                                 │
│     [1 byte: 255]                                           │
│     [3 bytes: date encoded]                                 │
│   ELSE (non-common tag):                                    │
│     [1 byte: tag name length (1-240)]                       │
│     [N bytes: tag name string]                              │
│     [1 byte: value length]                                  │
│     [N bytes: value string]                                 │
├─────────────────────────────────────────────────────────────┤
│ [1 byte: 0x00 = end of tags]                                │
└─────────────────────────────────────────────────────────────┘
```

### 10F. Complete Game Data Structure

**File**: `repomix-scidvspc.xml`, lines 59721-59749 (Game::Encode)

```cpp
errorT Game::Encode (ByteBuffer * buf, IndexEntry * ie)
{
    buf->Empty();

    // 1. Non-STR PGN tags (WhiteElo, BlackElo, ECO, Annotator, etc.)
    err = encodeTags (buf, TagList, NumTags);

    // 2. Flags byte
    byte flags = 0;
    if (NonStandardStart) { flags += 1; }
    if (PromotionsFlag)   { flags += 2; }
    if (UnderPromosFlag)  { flags += 4; }
    buf->PutByte (flags);

    // 3. Start FEN (if non-standard start)
    if (NonStandardStart) {
        char tempStr [256];
        StartPos->PrintFEN (tempStr, FEN_ALL_FIELDS);
        buf->PutTerminatedString (tempStr);
    }

    // 4. Move data (with annotation markers)
    err = encodeVariation (buf, FirstMove->next, &varCount, &nagCount, 0);

    // 5. Comments section (null-terminated strings)
    err = encodeComments (buf, FirstMove, &commentCount);

    // 6. Update IndexEntry with computed fields
    // ...
}
```

**Complete .sg4 Game Data Layout**:
```
┌─────────────────────────────────────────────────────────────┐
│ 1. Tags Section (non-STR tags, null-terminated)             │
├─────────────────────────────────────────────────────────────┤
│ 2. Flags Byte                                               │
│    Bit 0: NonStandardStart                                  │
│    Bit 1: PromotionsFlag                                    │
│    Bit 2: UnderPromosFlag                                   │
├─────────────────────────────────────────────────────────────┤
│ 3. Start FEN (if NonStandardStart, null-terminated)         │
├─────────────────────────────────────────────────────────────┤
│ 4. Move Data                                                │
│    - Move bytes [piece_num:4][move_value:4]                 │
│    - Annotation markers (0x0B-0x0F)                         │
│    - Ends with 0x0F (END_GAME)                              │
├─────────────────────────────────────────────────────────────┤
│ 5. Comments Section                                         │
│    - Null-terminated strings in tree traversal order        │
│    - One string per 0x0C marker in move data                │
└─────────────────────────────────────────────────────────────┘
```

### 10G. Current Implementation Status

**Status**: ⚠️ **Needs Verification**

The current implementation needs to be verified against this tag encoding scheme. Key areas to check:
- Index file parsing for STR tags (WhiteID, BlackID, EventID, SiteID, RoundID)
- Name file parsing for string lookup
- Non-standard tag decoding from game data
- Common tag optimization (byte values 241-254)

---

## References

- **SCID Source Code**: `repomix-scidvspc.xml` (Scid vs. PC source)
- **PGN Standard**: https://www.saremba.de/chessgml/standards/pgn/pgn-complete.htm
  - Section 8.2.4: Numeric Annotation Glyphs (NAGs)
  - Section 8.2.5: Comments
  - Section 8.2.6: Recursive Annotation Variations (RAV)
