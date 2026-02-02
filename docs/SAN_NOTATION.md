# SAN Notation Education

## Standard Algebraic Notation (SAN)

**Background**: SAN is the official notation system for chess moves, standardized by FIDE (World Chess Federation).

**Key Characteristics**:
- **Unambiguous**: Each move has exactly one representation
- **Concise**: Minimal characters while remaining readable
- **International**: Language-independent

---

## Basic Rules

### 1. Piece Identification

| Piece | Notation |
|-------|----------|
| King | K |
| Queen | Q |
| Rook | R |
| Bishop | B |
| Knight | N |

**Pawns**: No letter (just the square)

### 2. Square Notation

- **Files**: a-h (left to right from White's perspective)
- **Ranks**: 1-8 (bottom to top from White's perspective)

**Examples**:
- e4, d5, a1, h8

### 3. Move Notation

**Simple move**: `<Piece><Square>`

**Examples**: Nf3, Be2, e4

**Capture**: `<Piece>x<Square>`

**Examples**: Bxe5, exd5

**No capture indicator**: If no capture occurred, no 'x' appears

### 4. Disambiguation

When multiple pieces can reach the same square:

**Case 1 - Different files**: Add file letter

```
Two knights on b1 and g1, both can go to d2:
Nbd2 (knight from b-file) or Ngd2 (knight from g-file)
```

**Case 2 - Same file, different ranks**: Add rank number

```
Two rooks on a1 and a8, both can go to a4:
R1a4 (rook from rank 1) or R8a4 (rook from rank 8)
```

**Case 3 - Same file AND rank** (extremely rare): Add both

```
Queens on d1 and h5 can both go to e2:
Qd1e2 or Qh5e2 (full square specification)
```

### 5. Special Moves

| Move Type | Notation |
|-----------|----------|
| Castling kingside | O-O |
| Castling queenside | O-O-O |
| Promotion | e8=Q (pawn to e8, promotes to Queen) |
| En passant | exd6 (looks like normal capture) |

### 6. Check and Checkmate

- **Check**: Add `+` suffix (Qh5+)
- **Checkmate**: Add `#` suffix (Qh7#)

---

## Why Shakmaty Is Perfect

Implementing correct SAN generation manually requires:

- Complete board state awareness
- Disambiguation logic for all piece types
- Check/checkmate detection
- Legal move validation
- Edge case handling (promotions, castling, en passant)

**Shakmaty provides `San::from_move(&position, &move)` which**:

✅ Automatically disambiguates moves
✅ Detects check and checkmate
✅ Handles all special move notations
✅ Guarantees correctness (battle-tested on millions of games)
✅ Generates standard-compliant output

---

## Examples Comparison

### ❌ MANUAL SAN GENERATION (complex, error-prone)

```rust
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
```

### ✅ SHAKMATY SAN GENERATION (simple, guaranteed correct)

```rust
fn generate_san_with_shakmaty(position: &Chess, chess_move: &Move) -> String {
    San::from_move(position, chess_move).to_string()
}
```

---

## PGN Specification Reference

According to the PGN specification (Section 1.2):

- SAN is defined as "Standard Algebraic Notation"
- All moves must be in SAN format
- Check and checkmate must be indicated with `+` and `#`
- Castling uses `O-O` and `O-O-O`
- Promotion uses `<square>=<promoted piece>`

---

## Edge Cases to Consider

1. **Multiple promotions in one move**: Only one promotion per move
2. **Castling through check**: Legal move, but SAN doesn't indicate path
3. **En passant capture**: Pawn captures piece behind it
4. **Ambiguous moves**: Shakmaty handles all disambiguation cases
5. **Empty squares**: No notation for passing or skipping moves
6. **King moves**: `K` prefix required even in complex positions

---

## Verification Checklist

To ensure correct SAN generation:

- [x] Piece identification uses correct letters
- [x] Pawn moves don't have piece letters
- [x] Captures use 'x' notation
- [x] Disambiguation adds file or rank as needed
- [x] Castling uses O-O/O-O-O
- [x] Promotion uses = notation
- [x] En passant works correctly
- [x] Check indicated with +
- [x] Checkmate indicated with #
- [x] All moves standard-compliant
