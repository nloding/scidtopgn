// Main move decoder - replicates SCID's decodeMove function
// From scidvspc/src/game.cpp decodeMove()

use crate::position::byte_stream::ScidByteStream;
use crate::position::{Color, PieceType, ScidMove, ScidPosition, Square};

/// Main move decoder - replicates SCID's decodeMove function
/// From scidvspc/src/game.cpp decodeMove()
pub fn decode_move(position: &ScidPosition, move_byte: u8) -> Result<ScidMove, String> {
    // Step 1: Extract piece number and move value
    // From SCID: pieceNum = (val >> 4)
    let piece_num = (move_byte >> 4) as usize;
    let move_value = move_byte & 0x0F;

    // Step 2: Get piece location from position
    // From SCID: sqList = pos->GetList(pos->GetToMove())
    //           sm->from = sqList[sm->pieceNum]
    let piece_list = position.piece_list(position.to_move);
    if piece_num >= 16 {
        return Err(format!("Invalid piece number: {}", piece_num));
    }
    let from_square = piece_list[piece_num];

    // Step 3: Get piece type from board
    // From SCID: sm->movingPiece = board[sm->from]
    let piece_type = position
        .piece_at(from_square)
        .ok_or("No piece at from square")?;

    // Step 4: Route to piece-specific decoder
    // From SCID: switch (piece_Type(sm->movingPiece))
    let mut scid_move = ScidMove {
        from: from_square,
        to: from_square, // Will be set by piece decoder
        moving_piece: piece_type,
        captured_piece: PieceType::Empty,
        promote: PieceType::Empty,
        piece_num: piece_num as u8,
    };

    match piece_type {
        PieceType::Pawn => decode_pawn(move_value, &mut scid_move, position.to_move)?,
        PieceType::Knight => decode_knight(move_value, &mut scid_move)?,
        PieceType::Rook => decode_rook(move_value, &mut scid_move)?,
        PieceType::Bishop => decode_bishop(move_value, &mut scid_move)?,
        PieceType::King => decode_king(move_value, &mut scid_move)?,
        PieceType::Queen => decode_queen(move_value, &mut scid_move)?,
        _ => return Err(format!("Invalid piece type: {:?}", piece_type)),
    }

    // Step 5: Set captured piece if target square occupied
    if let Some(captured) = position.piece_at(scid_move.to) {
        scid_move.captured_piece = captured;
    }

    Ok(scid_move)
}

/// Pawn move decoder - exact copy of SCID's decodePawn function
/// From scidvspc/src/game.cpp decodePawn()
pub fn decode_pawn(move_value: u8, scid_move: &mut ScidMove, to_move: Color) -> Result<(), String> {
    // SCID's exact arrays from game.cpp
    const TO_SQUARE_DIFF: [i8; 16] = [
        7, 8, 9, // 0-2: capture-left, forward, capture-right
        7, 8, 9, // 3-5: capture-left+Queen, forward+Queen, capture-right+Queen
        7, 8, 9, // 6-8: capture-left+Rook, forward+Rook, capture-right+Rook
        7, 8, 9, // 9-11: capture-left+Bishop, forward+Bishop, capture-right+Bishop
        7, 8, 9,  // 12-14: capture-left+Knight, forward+Knight, capture-right+Knight
        16, // 15: double pawn push (2 squares forward)
    ];

    const PROMO_PIECE_FROM_VAL: [PieceType; 16] = [
        PieceType::Empty,
        PieceType::Empty,
        PieceType::Empty, // 0-2
        PieceType::Queen,
        PieceType::Queen,
        PieceType::Queen, // 3-5
        PieceType::Rook,
        PieceType::Rook,
        PieceType::Rook, // 6-8
        PieceType::Bishop,
        PieceType::Bishop,
        PieceType::Bishop, // 9-11
        PieceType::Knight,
        PieceType::Knight,
        PieceType::Knight, // 12-14
        PieceType::Empty,  // 15
    ];

    if move_value >= 16 {
        return Err(format!("Invalid pawn move value: {}", move_value));
    }

    let square_diff = TO_SQUARE_DIFF[move_value as usize];

    // SCID's exact logic:
    // if (toMove == WHITE) {
    //     sm->to = sm->from + toSquareDiff[val];
    // } else {
    //     sm->to = sm->from - toSquareDiff[val];
    // }
    let target_square = match to_move {
        Color::White => scid_move.from.0 as i8 + square_diff,
        Color::Black => scid_move.from.0 as i8 - square_diff,
    };

    if target_square < 0 || target_square > 63 {
        return Err(format!("Target square out of bounds: {}", target_square));
    }

    scid_move.to = Square(target_square as u8);
    scid_move.promote = PROMO_PIECE_FROM_VAL[move_value as usize];

    Ok(())
}

/// King move decoder - exact copy of SCID's decodeKing function
/// From scidvspc/src/game.cpp decodeKing()
pub fn decode_king(move_value: u8, scid_move: &mut ScidMove) -> Result<(), String> {
    // SCID's exact square difference array from game.cpp
    const SQUARE_DIFF: [i8; 11] = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];

    if move_value == 0 {
        // Null move - King stays in place
        scid_move.to = scid_move.from;
        return Ok(());
    }

    if move_value as usize >= SQUARE_DIFF.len() {
        return Err(format!("Invalid king move value: {}", move_value));
    }

    let square_diff = SQUARE_DIFF[move_value as usize];
    let target_square = scid_move.from.0 as i8 + square_diff;

    if target_square < 0 || target_square > 63 {
        return Err(format!(
            "King target square out of bounds: {}",
            target_square
        ));
    }

    scid_move.to = Square(target_square as u8);
    Ok(())
}

/// Knight move decoder - exact copy of SCID's decodeKnight function
/// From scidvspc/src/game.cpp decodeKnight()
pub fn decode_knight(move_value: u8, scid_move: &mut ScidMove) -> Result<(), String> {
    // SCID's exact square difference array from game.cpp
    const SQDIFF: [i8; 9] = [0, -17, -15, -10, -6, 6, 10, 15, 17];

    // SCID bounds checking - exact match
    if move_value < 1 || move_value > 8 {
        return Err(format!(
            "Invalid knight move value: {} (SCID valid: 1-8)",
            move_value
        ));
    }

    // SCID algorithm: sm->to = sm->from + sqdiff[val];
    let target_square = scid_move.from.0 as i8 + SQDIFF[move_value as usize];

    // SCID doesn't do bounds checking in decodeKnight - it relies on valid input
    // But we add basic bounds checking for safety
    if target_square < 0 || target_square > 63 {
        return Err(format!(
            "Knight target square out of bounds: {} (from square {}, diff {})",
            target_square, scid_move.from.0, SQDIFF[move_value as usize]
        ));
    }

    scid_move.to = Square(target_square as u8);
    Ok(())
}

/// Rook move decoder - exact copy of SCID's decodeRook function
/// From scidvspc/src/game.cpp decodeRook()
pub fn decode_rook(move_value: u8, scid_move: &mut ScidMove) -> Result<(), String> {
    if move_value > 15 {
        return Err(format!("Invalid rook move value: {}", move_value));
    }

    // SCID coordinate system: square = (rank << 3) | file
    let from_file = scid_move.from.0 & 0x7; // square_Fyle(from)
    let from_rank = (scid_move.from.0 >> 3) & 0x7; // square_Rank(from)

    // SCID algorithm exactly
    let target_square = if move_value >= 8 {
        // This is a move along a Fyle, to a different rank:
        // sm->to = square_Make (square_Fyle(sm->from), (val - 8));
        ((move_value - 8) << 3) | from_file
    } else {
        // sm->to = square_Make (val, square_Rank(sm->from));
        (from_rank << 3) | move_value
    };

    scid_move.to = Square(target_square);
    Ok(())
}

/// Bishop move decoder - exact copy of SCID's decodeBishop function
/// From scidvspc/src/game.cpp decodeBishop()
pub fn decode_bishop(move_value: u8, scid_move: &mut ScidMove) -> Result<(), String> {
    if move_value > 15 {
        return Err(format!("Invalid bishop move value: {}", move_value));
    }

    // SCID algorithm: byte fyle = (val & 7)
    let fyle = move_value & 7;
    let from_square = scid_move.from.0;
    let from_file = from_square & 7; // square_Fyle(from)

    // int fylediff = (int)fyle - (int)square_Fyle(sm->from)
    let fylediff = fyle as i8 - from_file as i8;

    // SCID algorithm exactly
    let target_square = if move_value >= 8 {
        // It is an up-left/down-right direction move.
        // sm->to = sm->from - 7 * fylediff;
        from_square as i8 - 7 * fylediff
    } else {
        // sm->to = sm->from + 9 * fylediff;
        from_square as i8 + 9 * fylediff
    };

    // SCID bounds checking: if (sm->to > H8) { return ERROR_Decode;}
    if target_square < 0 || target_square > 63 {
        return Err(format!(
            "Bishop target square out of bounds: {} (from square {}, diff {})",
            target_square, from_square, fylediff
        ));
    }

    scid_move.to = Square(target_square as u8);
    Ok(())
}

/// Queen move decoder - legacy single-byte version (DEPRECATED)
/// From scidvspc/src/game.cpp decodeQueen()
///
/// ⚠️  LIMITATION: This function only supports 1-byte Queen moves (rook-like).
/// For complete Queen move support including diagonal moves, use decode_queen_with_stream().
///
/// DEPRECATION NOTE: Queen diagonal moves require 2-byte encoding and stream access.
/// This function is kept for backward compatibility but will fail on diagonal moves.
pub fn decode_queen(move_value: u8, scid_move: &mut ScidMove) -> Result<(), String> {
    // SCID coordinate system
    let from_file = scid_move.from.0 & 0x7; // square_Fyle(from)
    let from_rank = (scid_move.from.0 >> 3) & 0x7; // square_Rank(from)

    if move_value >= 8 {
        // ✅ CASE 1: Rook-vertical move (FULLY SUPPORTED)
        // SCID: sm->to = square_Make(square_Fyle(sm->from), (val - 8))
        let target_rank = move_value - 8;
        if target_rank > 7 {
            return Err(format!("Invalid queen target rank: {}", target_rank));
        }
        let target_square = (target_rank << 3) | from_file;
        scid_move.to = Square(target_square);
    } else if move_value != from_file {
        // ✅ CASE 2: Rook-horizontal move (FULLY SUPPORTED)
        // SCID: sm->to = square_Make(val, square_Rank(sm->from))
        if move_value > 7 {
            return Err(format!("Invalid queen target file: {}", move_value));
        }
        let target_square = (from_rank << 3) | move_value;
        scid_move.to = Square(target_square);
    } else {
        // ⚠️  CASE 3: Diagonal move (NOT SUPPORTED IN LEGACY VERSION)
        // Queen diagonal moves require 2-byte encoding and stream access.
        // Use decode_queen_with_stream() for complete functionality.
        return Err(
            "Queen diagonal moves require stream access - use decode_queen_with_stream() instead"
                .to_string(),
        );
    }

    Ok(())
}

/// Main move decoder with stream support for multi-byte moves
/// Based on SCID's decodeMove() but with ByteBuffer-compatible streaming
pub fn decode_move_with_stream(
    position: &ScidPosition,
    stream: &mut ScidByteStream,
) -> Result<ScidMove, String> {
    // Step 1: Read first move byte from stream
    let move_byte = stream
        .get_byte()
        .map_err(|e| format!("Failed to read move byte: {}", e))?;

    // Step 2: Extract piece number and move value (same as before)
    let piece_num = (move_byte >> 4) as usize;
    let move_value = move_byte & 0x0F;

    // Step 3: Get piece location and type (same as existing decode_move)
    let piece_list = position.piece_list(position.to_move);
    if piece_num >= 16 {
        return Err(format!("Invalid piece number: {}", piece_num));
    }
    let from_square = piece_list[piece_num];

    let piece_type = position
        .piece_at(from_square)
        .ok_or("No piece at from square")?;

    // Step 4: Create move structure
    let mut scid_move = ScidMove {
        from: from_square,
        to: from_square,
        moving_piece: piece_type,
        captured_piece: PieceType::Empty,
        promote: PieceType::Empty,
        piece_num: piece_num as u8,
    };

    // Step 5: Route to piece-specific decoder (UPDATED FOR STREAM)
    match piece_type {
        PieceType::Pawn => decode_pawn(move_value, &mut scid_move, position.to_move)?,
        PieceType::Knight => decode_knight(move_value, &mut scid_move)?,
        PieceType::Rook => decode_rook(move_value, &mut scid_move)?,
        PieceType::Bishop => decode_bishop(move_value, &mut scid_move)?,
        PieceType::King => decode_king(move_value, &mut scid_move)?,
        // 🔥 KEY CHANGE: Use stream-aware Queen decoder
        PieceType::Queen => decode_queen_with_stream(move_value, &mut scid_move, stream)?,
        _ => return Err(format!("Invalid piece type: {:?}", piece_type)),
    }

    // Step 6: Set captured piece if target square occupied
    if let Some(captured) = position.piece_at(scid_move.to) {
        scid_move.captured_piece = captured;
    }

    Ok(scid_move)
}

/// Queen move decoder with ByteBuffer-compatible stream access - COMPLETE IMPLEMENTATION ✅
///
/// EXACT REPLICATION of scidvspc/src/game.cpp decodeQueen() function with full 2-byte support.
/// This function implements complete Queen move decoding including diagonal moves that require
/// reading additional bytes from the stream.
///
/// **Supported Move Types**:
/// - ✅ Rook-vertical moves (1 byte): `move_value >= 8`
/// - ✅ Rook-horizontal moves (1 byte): `move_value != from_file && move_value < 8`
/// - ✅ Diagonal moves (2 bytes): `move_value == from_file` (NEW IMPLEMENTATION)
///
/// **Stream Usage**:
/// - 1-byte moves: Stream position unchanged
/// - 2-byte moves: Stream advances by 1 additional byte
///
/// **SCID Algorithm Compliance**:
/// - Trigger condition: `val == square_Fyle(sm->from)`
/// - Target encoding: `target_square = (second_byte - 64)`
/// - Validation range: second_byte ∈ [64, 127]
///
/// **Example Usage**:
/// ```rust
/// let move_bytes = [0x13, 0x6D]; // Queen diagonal: D4 -> F6
/// let mut stream = ScidByteStream::new(&move_bytes);
/// let first_byte = stream.get_byte().unwrap(); // 0x13
/// let move_value = first_byte & 0x0F;          // 0x03
/// decode_queen_with_stream(move_value, &mut scid_move, &mut stream).unwrap();
/// assert_eq!(stream.position(), 1); // Second byte consumed
/// ```
pub fn decode_queen_with_stream(
    move_value: u8,
    scid_move: &mut ScidMove,
    stream: &mut ScidByteStream,
) -> Result<(), String> {
    // SCID coordinate system
    let from_file = scid_move.from.0 & 0x7; // square_Fyle(from)
    let from_rank = (scid_move.from.0 >> 3) & 0x7; // square_Rank(from)

    if move_value >= 8 {
        // ✅ CASE 1: Rook-vertical move (ALREADY WORKING)
        // SCID: sm->to = square_Make (square_Fyle(sm->from), (val - 8))
        let target_rank = move_value - 8;
        if target_rank > 7 {
            return Err(format!("Invalid queen target rank: {}", target_rank));
        }
        let target_square = (target_rank << 3) | from_file;
        scid_move.to = Square(target_square);
    } else if move_value != from_file {
        // ✅ CASE 2: Rook-horizontal move (ALREADY WORKING)
        // SCID: sm->to = square_Make (val, square_Rank(sm->from))
        if move_value > 7 {
            return Err(format!("Invalid queen target file: {}", move_value));
        }
        let target_square = (from_rank << 3) | move_value;
        scid_move.to = Square(target_square);
    } else {
        // 🔥 CASE 3: Diagonal move (NEW IMPLEMENTATION)
        // SCID: val = buf->GetByte(); sm->to = val - 64;

        let second_byte = stream
            .get_byte()
            .map_err(|e| format!("Failed to read second byte for Queen diagonal move: {}", e))?;

        // SCID validation: if (val < 64 || val > 127) { return ERROR_Decode; }
        if second_byte < 64 || second_byte > 127 {
            return Err(format!(
                "Invalid Queen diagonal target byte: {} (valid range: 64-127)",
                second_byte
            ));
        }

        // SCID target calculation: sm->to = val - 64
        let target_square = second_byte - 64;
        if target_square > 63 {
            return Err(format!(
                "Queen diagonal target square out of bounds: {}",
                target_square
            ));
        }

        scid_move.to = Square(target_square);

        // Additional validation: Verify it's actually a diagonal move
        let to_file = target_square & 0x7;
        let to_rank = (target_square >> 3) & 0x7;

        let file_distance = (to_file as i8 - from_file as i8).abs();
        let rank_distance = (to_rank as i8 - from_rank as i8).abs();

        if file_distance != rank_distance || file_distance == 0 {
            return Err(format!(
                "Invalid Queen diagonal move geometry: from {}{} to {}{}",
                char::from(b'a' + from_file),
                from_rank + 1,
                char::from(b'a' + to_file),
                to_rank + 1
            ));
        }
    }

    Ok(())
}
