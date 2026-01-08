//! SCID Move Decoders
//!
//! This module implements piece-specific decoders that convert SCID binary move
//! values into from/to squares and promotion information.
//!
//! # Move Encoding Overview
//!
//! SCID move bytes are structured as: `[piece_num:4][move_value:4]`
//! - Upper 4 bits: Piece number (0-15)
//! - Lower 4 bits: Move value (meaning depends on piece type)
//!
//! # Piece-Specific Encoding
//!
//! Each piece type has unique move value interpretations documented below.

use crate::error::{Result, ScidError};
use crate::parser::byte_stream::ByteStream;
use shakmaty::{Color, File, Rank, Role, Square};
use std::path::PathBuf;

/// Decoded SCID move information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedMove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
}

/// King move decoder
///
/// Move values:
/// - 0: Null move (error)
/// - 1-8: Adjacent squares (clockwise from NW)
///   - 1: up-left, 2: up, 3: up-right
///   - 4: left, 5: right
///   - 6: down-left, 7: down, 8: down-right
/// - 10: Kingside castle
/// - 11: Queenside castle
pub fn decode_king_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    if move_value == 0 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: "Null move not allowed".to_string(),
        });
    }

    let to = if (1..=8).contains(&move_value) {
        // Adjacent square moves
        // SCID direction mapping (from code analysis):
        // 1=NW(-1,+1), 2=N(0,+1), 3=NE(+1,+1)
        // 4=W(-1,0), 5=E(+1,0)
        // 6=SW(-1,-1), 7=S(0,-1), 8=SE(+1,-1)
        let (file_delta, rank_delta): (i8, i8) = match move_value {
            1 => (-1, 1),  // NW
            2 => (0, 1),   // N
            3 => (1, 1),   // NE
            4 => (-1, 0),  // W
            5 => (1, 0),   // E
            6 => (-1, -1), // SW
            7 => (0, -1),  // S
            8 => (1, -1),  // SE
            _ => unreachable!(),
        };

        let new_file = (from.file() as i8 + file_delta) as u32;
        let new_rank = (from.rank() as i8 + rank_delta) as u32;

        if new_file > 7 || new_rank > 7 {
            return Err(ScidError::ParseError {
                file: PathBuf::from("decoder"),
                offset: 0,
                message: format!("King move out of bounds from {:?}", from),
            });
        }

        Square::from_coords(File::new(new_file), Rank::new(new_rank))
    } else if move_value == 10 {
        // Kingside castle
        match color {
            Color::White => Square::G1,
            Color::Black => Square::G8,
        }
    } else if move_value == 11 {
        // Queenside castle
        match color {
            Color::White => Square::C1,
            Color::Black => Square::C8,
        }
    } else {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid king move value: {}", move_value),
        });
    };

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
    })
}

/// Knight move decoder
///
/// Move values 0-7 represent L-shaped jumps in clockwise order:
/// - 0: 2 up, 1 left
/// - 1: 2 up, 1 right
/// - 2: 1 up, 2 right
/// - 3: 1 down, 2 right
/// - 4: 2 down, 1 right
/// - 5: 2 down, 1 left
/// - 6: 1 down, 2 left
/// - 7: 1 up, 2 left
pub fn decode_knight_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    if move_value > 7 {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!("Invalid knight move value: {}", move_value),
        });
    }

    // Knight L-shaped offsets (file_delta, rank_delta)
    let (file_delta, rank_delta): (i8, i8) = match move_value {
        0 => (-1, 2),  // 2 up, 1 left
        1 => (1, 2),   // 2 up, 1 right
        2 => (2, 1),   // 1 up, 2 right
        3 => (2, -1),  // 1 down, 2 right
        4 => (1, -2),  // 2 down, 1 right
        5 => (-1, -2), // 2 down, 1 left
        6 => (-2, -1), // 1 down, 2 left
        7 => (-2, 1),  // 1 up, 2 left
        _ => unreachable!(),
    };

    let new_file = from.file() as i8 + file_delta;
    let new_rank = from.rank() as i8 + rank_delta;

    if !(0..=7).contains(&new_file) || !(0..=7).contains(&new_rank) {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!(
                "Knight move out of bounds from {:?} with value {}",
                from, move_value
            ),
        });
    }

    let to = Square::from_coords(File::new(new_file as u32), Rank::new(new_rank as u32));

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
    })
}

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

/// Bishop move decoder
///
/// Move values 0-15 encode diagonal destinations.
/// The encoding maps to the bishop's potential target squares.
pub fn decode_bishop_move(from: Square, move_value: u8) -> Result<DecodedMove> {
    // Bishop moves are encoded as diagonal index
    // Value encodes which diagonal square to move to
    //
    // From SCID source: bishop uses values 0-13 for diagonal squares
    // Each diagonal has at most 7 squares the bishop can reach
    // The encoding is: find all reachable diagonal squares, index by value

    // Calculate all diagonal squares from 'from' and pick by index
    let mut diagonals: Vec<Square> = Vec::new();

    // Up-left diagonal
    let mut f = from.file() as i8 - 1;
    let mut r = from.rank() as i8 + 1;
    while f >= 0 && r <= 7 {
        diagonals.push(Square::from_coords(
            File::new(f as u32),
            Rank::new(r as u32),
        ));
        f -= 1;
        r += 1;
    }

    // Up-right diagonal
    f = from.file() as i8 + 1;
    r = from.rank() as i8 + 1;
    while f <= 7 && r <= 7 {
        diagonals.push(Square::from_coords(
            File::new(f as u32),
            Rank::new(r as u32),
        ));
        f += 1;
        r += 1;
    }

    // Down-left diagonal
    f = from.file() as i8 - 1;
    r = from.rank() as i8 - 1;
    while f >= 0 && r >= 0 {
        diagonals.push(Square::from_coords(
            File::new(f as u32),
            Rank::new(r as u32),
        ));
        f -= 1;
        r -= 1;
    }

    // Down-right diagonal
    f = from.file() as i8 + 1;
    r = from.rank() as i8 - 1;
    while f <= 7 && r >= 0 {
        diagonals.push(Square::from_coords(
            File::new(f as u32),
            Rank::new(r as u32),
        ));
        f += 1;
        r -= 1;
    }

    // Sort diagonals to get consistent ordering
    diagonals.sort_by_key(|s| u32::from(*s));

    if (move_value as usize) >= diagonals.len() {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!(
                "Invalid bishop move value {} from {:?} (max: {})",
                move_value,
                from,
                diagonals.len().saturating_sub(1)
            ),
        });
    }

    let to = diagonals[move_value as usize];

    Ok(DecodedMove {
        from,
        to,
        promotion: None,
    })
}

/// Pawn move decoder
///
/// Move values:
/// - 0: Capture left (toward a-file)
/// - 1: Move forward one square
/// - 2: Capture right (toward h-file)
/// - 3-5: Queen promotion (left capture, forward, right capture)
/// - 6-8: Rook promotion
/// - 9-11: Bishop promotion
/// - 12-14: Knight promotion
/// - 15: Double push
pub fn decode_pawn_move(from: Square, move_value: u8, color: Color) -> Result<DecodedMove> {
    let forward = match color {
        Color::White => 1i8,
        Color::Black => -1i8,
    };

    let from_file = from.file() as i8;
    let from_rank = from.rank() as i8;

    let (file_delta, rank_delta, promotion) = match move_value {
        0 => (-1, forward, None),                // Capture left
        1 => (0, forward, None),                 // Forward one
        2 => (1, forward, None),                 // Capture right
        3 => (-1, forward, Some(Role::Queen)),   // Queen promo (capture left)
        4 => (0, forward, Some(Role::Queen)),    // Queen promo (forward)
        5 => (1, forward, Some(Role::Queen)),    // Queen promo (capture right)
        6 => (-1, forward, Some(Role::Rook)),    // Rook promo (capture left)
        7 => (0, forward, Some(Role::Rook)),     // Rook promo (forward)
        8 => (1, forward, Some(Role::Rook)),     // Rook promo (capture right)
        9 => (-1, forward, Some(Role::Bishop)),  // Bishop promo (capture left)
        10 => (0, forward, Some(Role::Bishop)),  // Bishop promo (forward)
        11 => (1, forward, Some(Role::Bishop)),  // Bishop promo (capture right)
        12 => (-1, forward, Some(Role::Knight)), // Knight promo (capture left)
        13 => (0, forward, Some(Role::Knight)),  // Knight promo (forward)
        14 => (1, forward, Some(Role::Knight)),  // Knight promo (capture right)
        15 => (0, forward * 2, None),            // Double push
        _ => {
            return Err(ScidError::ParseError {
                file: PathBuf::from("decoder"),
                offset: 0,
                message: format!("Invalid pawn move value: {}", move_value),
            })
        }
    };

    let new_file = from_file + file_delta;
    let new_rank = from_rank + rank_delta;

    if !(0..=7).contains(&new_file) || !(0..=7).contains(&new_rank) {
        return Err(ScidError::ParseError {
            file: PathBuf::from("decoder"),
            offset: 0,
            message: format!(
                "Pawn move out of bounds from {:?} with value {}",
                from, move_value
            ),
        });
    }

    let to = Square::from_coords(File::new(new_file as u32), Rank::new(new_rank as u32));

    Ok(DecodedMove {
        from,
        to,
        promotion,
    })
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_king_adjacent_moves() {
        // Test from e4 - all directions available
        let from = Square::E4;

        let cases = [
            (1, Square::D5), // NW
            (2, Square::E5), // N
            (3, Square::F5), // NE
            (4, Square::D4), // W
            (5, Square::F4), // E
            (6, Square::D3), // SW
            (7, Square::E3), // S
            (8, Square::F3), // SE
        ];

        for (move_value, expected_to) in cases {
            let decoded = decode_king_move(from, move_value, Color::White).unwrap();
            assert_eq!(
                decoded.to, expected_to,
                "King move value {} from {:?}",
                move_value, from
            );
        }
    }

    #[test]
    fn test_king_castling() {
        // White kingside castle
        let decoded = decode_king_move(Square::E1, 10, Color::White).unwrap();
        assert_eq!(decoded.to, Square::G1);

        // White queenside castle
        let decoded = decode_king_move(Square::E1, 11, Color::White).unwrap();
        assert_eq!(decoded.to, Square::C1);

        // Black kingside castle
        let decoded = decode_king_move(Square::E8, 10, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::G8);

        // Black queenside castle
        let decoded = decode_king_move(Square::E8, 11, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::C8);
    }

    #[test]
    fn test_knight_moves() {
        let from = Square::E4;

        // Test a few knight moves
        let decoded = decode_knight_move(from, 0).unwrap(); // 2 up, 1 left
        assert_eq!(decoded.to, Square::D6);

        let decoded = decode_knight_move(from, 1).unwrap(); // 2 up, 1 right
        assert_eq!(decoded.to, Square::F6);

        let decoded = decode_knight_move(from, 2).unwrap(); // 1 up, 2 right
        assert_eq!(decoded.to, Square::G5);

        let decoded = decode_knight_move(from, 4).unwrap(); // 2 down, 1 right
        assert_eq!(decoded.to, Square::F2);
    }

    #[test]
    fn test_rook_moves() {
        let from = Square::D4;

        // Horizontal to a-file
        let decoded = decode_rook_move(from, 0).unwrap();
        assert_eq!(decoded.to, Square::A4);

        // Horizontal to h-file
        let decoded = decode_rook_move(from, 7).unwrap();
        assert_eq!(decoded.to, Square::H4);

        // Vertical to rank 1
        let decoded = decode_rook_move(from, 8).unwrap();
        assert_eq!(decoded.to, Square::D1);

        // Vertical to rank 8
        let decoded = decode_rook_move(from, 15).unwrap();
        assert_eq!(decoded.to, Square::D8);
    }

    #[test]
    fn test_pawn_basic_moves() {
        let from = Square::E2;

        // Forward one
        let decoded = decode_pawn_move(from, 1, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E3);
        assert_eq!(decoded.promotion, None);

        // Double push
        let decoded = decode_pawn_move(from, 15, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E4);
        assert_eq!(decoded.promotion, None);

        // Capture right
        let decoded = decode_pawn_move(from, 2, Color::White).unwrap();
        assert_eq!(decoded.to, Square::F3);

        // Capture left
        let decoded = decode_pawn_move(from, 0, Color::White).unwrap();
        assert_eq!(decoded.to, Square::D3);
    }

    #[test]
    fn test_pawn_black_moves() {
        let from = Square::E7;

        // Forward one (Black moves down)
        let decoded = decode_pawn_move(from, 1, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::E6);

        // Double push
        let decoded = decode_pawn_move(from, 15, Color::Black).unwrap();
        assert_eq!(decoded.to, Square::E5);
    }

    #[test]
    fn test_pawn_promotions() {
        let from = Square::E7;

        // Queen promotion forward
        let decoded = decode_pawn_move(from, 4, Color::White).unwrap();
        assert_eq!(decoded.to, Square::E8);
        assert_eq!(decoded.promotion, Some(Role::Queen));

        // Knight promotion capture left
        let decoded = decode_pawn_move(from, 12, Color::White).unwrap();
        assert_eq!(decoded.to, Square::D8);
        assert_eq!(decoded.promotion, Some(Role::Knight));

        // Rook promotion capture right
        let decoded = decode_pawn_move(from, 8, Color::White).unwrap();
        assert_eq!(decoded.to, Square::F8);
        assert_eq!(decoded.promotion, Some(Role::Rook));
    }

    #[test]
    fn test_queen_vertical() {
        let from = Square::D4;

        // Move to d8 (rank 7, move_value = 8 + 7 = 15)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 15, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::D8);

        // Move to d1 (rank 0, move_value = 8 + 0 = 8)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 8, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::D1);
    }

    #[test]
    fn test_queen_horizontal() {
        let from = Square::D4;

        // Move to a4 (file 0, move_value = 0)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 0, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::A4);

        // Move to h4 (file 7, move_value = 7)
        let mut stream = ByteStream::new(&[]);
        let decoded = decode_queen_move(from, 7, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::H4);
    }

    #[test]
    fn test_queen_diagonal() {
        let from = Square::D4; // d4, file 3

        // Diagonal move: move_value = 3 (same as from_file)
        // Second byte encodes target: 64 + square_index
        // Target a1 = square 0, so second_byte = 64
        let data = vec![64]; // Target a1
        let mut stream = ByteStream::new(&data);

        let decoded = decode_queen_move(from, 3, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::A1);

        // Target h8 = square 63, so second_byte = 64 + 63 = 127
        let data = vec![127]; // Target h8
        let mut stream = ByteStream::new(&data);

        let decoded = decode_queen_move(from, 3, &mut stream).unwrap();
        assert_eq!(decoded.to, Square::H8);
    }

    #[test]
    fn test_queen_diagonal_invalid() {
        let from = Square::D4;

        // Invalid second byte (< 64)
        let data = vec![50];
        let mut stream = ByteStream::new(&data);

        let result = decode_queen_move(from, 3, &mut stream);
        assert!(result.is_err());

        // Invalid second byte (> 127)
        let data = vec![128];
        let mut stream = ByteStream::new(&data);

        let result = decode_queen_move(from, 3, &mut stream);
        assert!(result.is_err());
    }

    #[test]
    fn test_bishop_moves() {
        let from = Square::D4;

        // Bishop from d4 can reach 13 diagonal squares
        // Test first diagonal target
        let decoded = decode_bishop_move(from, 0).unwrap();
        // First target depends on sorted order
        assert!(decoded.to != from);

        // The decoded square should be on a diagonal
        let file_diff = (decoded.to.file() as i8 - from.file() as i8).abs();
        let rank_diff = (decoded.to.rank() as i8 - from.rank() as i8).abs();
        assert_eq!(file_diff, rank_diff, "Bishop move must be diagonal");
    }
}
