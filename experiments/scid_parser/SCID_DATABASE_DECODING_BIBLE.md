# The SCID Database Game Decoding Bible

## Introduction

This document provides an exhaustive, step-by-step explanation of how SCID (Shane's Chess Information Database) decodes chess games from its proprietary database format. It is based on a deep analysis of the SCID source code, especially `scidvspc/src/game.cpp`, and is intended for programmers who have no prior knowledge of SCID or its data structures. Every aspect of game decoding, piece identification, move determination, and special move handling is covered in detail.

---

## 1. SCID Database Format Overview

SCID stores chess games in a compact binary format, optimized for speed and space. Each game record contains metadata (event, players, date, etc.) and a sequence of encoded moves. The move encoding is designed to minimize space while allowing fast decoding and validation.

- **Game Metadata**: Stored as tags (Event, Site, Date, Round, White, Black, Result, etc.)
- **Move List**: Encoded as a sequence of bytes, each representing a move in the game
- **Position State**: Maintained throughout parsing to track the board after each move

---

## 2. Core Data Structures

### 2.1. `Game` Class
- Maintains the current position (`CurrentPos`), move list, and metadata
- Handles move decoding, application, and navigation (forward/backward, variations)

### 2.2. `Position` Class
- Represents the current board state
- Provides piece lists, board array, and methods for move application/undo
- Key methods: `DoSimpleMove`, `UndoSimpleMove`, `GetList`, `GetBoard`, `GetToMove`

### 2.3. Piece Lists and Lookup Arrays
- **List[][]**: For each side, an array of piece locations (squares)
- **ListPos[64]**: For each square, the index in List[][] of the piece on that square
- Enables fast lookup: square → piece index, piece index → square

---

## 3. Move Encoding Format

Each move is encoded as one or more bytes, depending on the piece and move type. The encoding is designed to be as compact as possible:

- **First Byte**: Encodes piece type, piece index, and move-specific value
    - High nibble (4 bits): Piece index (pieceNum)
    - Low nibble (4 bits): Move value (meaning depends on piece type)
- **Additional Bytes**: Used for promotions, castling, en passant, or other special moves

---

## 4. Decoding a Move: Step-by-Step

### 4.1. Identify the Piece Moving

- The high nibble of the move byte (`val >> 4`) gives the piece index (`pieceNum`)
- The current position (`CurrentPos`) maintains a piece list for the side to move
- Use `CurrentPos->GetList(CurrentPos->GetToMove())` to get the piece list
- The moving piece's square is `sqList[pieceNum]`
- The piece type is determined by looking up the board array: `CurrentPos->GetBoard()[from]`

### 4.2. Determine the Move Destination

- The low nibble of the move byte (`val & 0xF`) is the move value
- The meaning of the move value depends on the piece type:
    - **Pawn**: Encodes direction, captures, promotions, en passant
    - **Knight**: Encodes jump pattern
    - **Bishop**: Encodes diagonal movement (see below)
    - **Rook**: Encodes rank/file movement
    - **Queen**: Encodes combined rook/bishop movement
    - **King**: Encodes normal move or castling
- Piece-specific decoder functions interpret the move value using the current position

#### Example: Bishop Move Decoding
```cpp
static errorT decodeBishop(byte val, simpleMoveT * sm) {
    byte fyle = (val & 7);
    int fylediff = (int)fyle - (int)square_Fyle(sm->from);
    if (val >= 8) {
        sm->to = sm->from - 7 * fylediff;  // up-left/down-right
    } else {
        sm->to = sm->from + 9 * fylediff;  // up-right/down-left
    }
}
```
- The move value encodes the target file; the decoder calculates the destination square algebraically

### 4.3. Special Moves

#### 4.3.1. Castling
- King moves with a special move value indicate castling
- The decoder recognizes castling by the move value and applies both king and rook moves
- Position is updated accordingly

#### 4.3.2. En Passant
- Pawn moves with a special flag indicate en passant
- The decoder checks the position for en passant eligibility
- The captured pawn is removed from the board

#### 4.3.3. Promotion
- Pawn moves with a promotion flag trigger a second byte encoding the promoted piece
- The decoder replaces the pawn with the promoted piece on the destination square

---

## 5. Applying the Move and Updating Position

- After decoding, the move is applied to the current position using `CurrentPos->DoSimpleMove(sm)`
- The position's piece lists and board array are updated
- The ply count and side to move are updated
- If the move is part of a variation, the position is saved/restored as needed

---

## 6. Handling Variations and Navigation

- SCID supports complex variation trees
- Moves can have multiple child variations, each with its own position state
- The `Game` class provides methods to move forward, backward, into/out of variations
- Position is restored as needed when navigating

---

## 7. Move Validation and SAN Generation

- After each move, SCID validates legality using the current position
- SAN (Standard Algebraic Notation) is generated using the updated position
- Special moves (castling, en passant, promotion) are reflected in SAN

---

## 8. Summary of Key Algorithms

### 8.1. Piece Lookup
- `pieceNum` → `sqList[pieceNum]` → `from` square
- `from` square → `board[from]` → piece type

### 8.2. Move Decoding
- Piece-specific decoder functions interpret move value using position context
- Bishop, rook, queen: algebraic calculation based on current square and move value
- Pawn: direction, capture, promotion, en passant flags
- King: normal move or castling

### 8.3. Position Update
- `DoSimpleMove(sm)` applies move, updates piece lists, board, en passant, castling rights
- Undo/redo supported for navigation and variations

---

## 9. Example: Full Move Decoding Walkthrough

Suppose the next move byte is `0x4B`:
- High nibble: `0x4` → pieceNum = 4
- Low nibble: `0xB` → move value = 11
- Get side to move (e.g., White)
- Get piece list for White: `sqList = CurrentPos->GetList(WHITE)`
- `from = sqList[4]` (the 5th piece in White's list)
- `piece = board[from]` (e.g., Bishop)
- Call `decodeBishop(11, sm)` to determine destination square
- Apply move with `DoSimpleMove(sm)`
- Update position, piece lists, ply count, side to move

---

## 10. Edge Cases and Error Handling

- Illegal moves are detected during decoding and application
- If a move cannot be decoded (e.g., invalid pieceNum), an error is returned
- Position state is restored if navigation moves into/out of variations

---

## 11. References and Further Reading

- SCID Source Code: `scidvspc/src/game.cpp`, `position.cpp`, `move.cpp`
- SCID Database Format Documentation: [SCID_DATABASE_FORMAT.md]
- PGN Standard: [https://www.chessclub.com/help/PGN-spec]

---

## 12. Conclusion

SCID's game decoding is a position-centric, piece-list-driven process. Every move is interpreted in the context of the current position, using compact encoding and piece-specific algorithms. Special moves are handled with dedicated flags and logic. The position is updated after each move, supporting full navigation and variation trees. This document provides all the details needed for a programmer to implement or understand SCID's game decoding from scratch.
