use crate::error::{Result, ScidError};
use crate::parser::byte_stream::ByteStream;
use shakmaty::{Color, Role, Square};
use std::path::PathBuf;

/// Decoded SCID move information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
    pub is_null_move: bool,
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
        is_null_move: move_value == 0,
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
            message: format!(
                "Invalid knight move value: {} (valid: 1-8 only)",
                move_value
            ),
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
        0 => None,
        1 => Some(Role::Queen),
        2 => Some(Role::Rook),
        3 => Some(Role::Bishop),
        4 => Some(Role::Knight),
        5 => None,
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
            shakmaty::Rank::new(target_rank as u32),
        )
    } else {
        // Horizontal move: target file = val, same rank
        Square::from_coords(
            shakmaty::File::new(move_value as u32),
            shakmaty::Rank::new(from_rank as u32),
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
            shakmaty::Rank::new(target_rank as u32),
        )
    } else if move_value != from_file {
        // CASE 2: Horizontal move (rook-like, 1 byte)
        // Target file = val, same rank
        Square::from_coords(
            shakmaty::File::new(move_value as u32),
            shakmaty::Rank::new(from_rank as u32),
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
        Square::new((second_byte - 64) as u32)
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
    square
        .offset(offset as i32)
        .ok_or_else(|| ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Square offset out of bounds"),
        })
}
