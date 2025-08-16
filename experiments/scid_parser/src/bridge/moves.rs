/// Helper function to parse SCID promotion piece string to shakmaty Role
fn parse_promotion_piece_scid(promo: &str) -> Result<Role> {
    match promo {
        "q" | "Q" => Ok(Role::Queen),
        "r" | "R" => Ok(Role::Rook),
        "b" | "B" => Ok(Role::Bishop),
        "n" | "N" => Ok(Role::Knight),
        _ => Err(ScidError::conversion_error(format!("Unknown promotion piece: {}", promo))),
    }
}
/// Helper function to map SCID piece number to shakmaty Role
/// SCID piece_num values:
///   0-1: King
///   2-3: Queen
///   4-7: Rook
///   8-11: Bishop
///   12-15: Knight
///   16+: Pawn (if used)
fn scid_piece_num_to_role(piece_num: u8) -> Option<Role> {
    match piece_num {
        0 | 1 => Some(Role::King),
        2 | 3 => Some(Role::Queen),
        4..=7 => Some(Role::Rook),
        8..=11 => Some(Role::Bishop),
        12..=15 => Some(Role::Knight),
        _ => Some(Role::Pawn),
    }
}
/// Helper function to calculate target square from a starting square and a difference
/// Used for king, knight, and pawn moves (and others as needed)
fn calculate_target_square_scid(from: Square, diff: i32) -> Result<Square> {
    let idx = from as i32 + diff;
    if idx >= 0 && idx < 64 {
        Ok(Square::new(idx as u32))
    } else {
        Err(ScidError::conversion_error(format!("Target square out of bounds: {}", idx)))
    }
}
// ===============================
// Step 10: Basic SCID Move to Shakmaty Conversion
// This implementation covers basic conversion for normal moves, castling, and pawn promotion.
// It uses the MoveInterpretation field to select the conversion logic for each piece type.
// Future steps will add more robust error handling and edge case support.
// ===============================
// SCID Move to Shakmaty Move Conversion
//
// This module handles the conversion of SCID binary move data to shakmaty
// chess moves. SCID uses a compact binary encoding for moves that needs
// to be translated to shakmaty's strongly-typed move representation.

use shakmaty::{Move, Square, Role, Color, Chess, Position};
use crate::sg4::{DecodedMove, MoveInterpretation};
use crate::bridge::ScidToShakmaty;
use crate::error::{Result, ScidError};

/// Implementation of SCID to Shakmaty conversion for DecodedMove
/// 
/// This converts SCID's compact move encoding to shakmaty's strongly-typed
/// move representation. The conversion requires the current chess position
/// to determine piece locations and validate moves.
impl ScidToShakmaty for DecodedMove {
    type Output = Move;
    
    fn to_shakmaty(&self, position: &Chess) -> Result<Self::Output> {
        match &self.interpretation {
            MoveInterpretation::King { direction_code, .. } => {
                convert_king_move(*direction_code, self.piece_num, position)
            }
            MoveInterpretation::Queen { .. } => {
                convert_queen_move(self.move_value, self.piece_num, position)
            }
            MoveInterpretation::Rook { .. } => {
                convert_rook_move(self.move_value, self.piece_num, position)
            }
            MoveInterpretation::Bishop { .. } => {
                convert_bishop_move(self.move_value, self.piece_num, position)
            }
            MoveInterpretation::Knight { .. } => {
                convert_knight_move(self.move_value, self.piece_num, position)
            }
            MoveInterpretation::Pawn { promotion, .. } => {
                convert_pawn_move(self.move_value, self.piece_num, promotion.as_deref(), position)
            }
            MoveInterpretation::Unknown { reason } => {
                Err(ScidError::conversion_error(format!("Cannot convert unknown move: {}", reason)))
            }
        }
    }
}

/// Convert SCID king moves to shakmaty moves
/// 
/// Handles both normal king moves (direction_code 0-8) and castling (9-10)
fn convert_king_move(direction_code: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::King, position)?;
    
    match direction_code {
        // Castling moves
        9 => {
            // Queenside castling
            let (king_to, rook_from) = match from_square {
                square if square == Square::E1 => (Square::C1, Square::A1),
                square if square == Square::E8 => (Square::C8, Square::A8),
                _ => return Err(ScidError::conversion_error("King not on home rank for queenside castling")),
            };
            Ok(Move::Castle { king: from_square, rook: rook_from })
        }
        10 => {
            // Kingside castling  
            let (king_to, rook_from) = match from_square {
                square if square == Square::E1 => (Square::G1, Square::H1),
                square if square == Square::E8 => (Square::G8, Square::H8),
                _ => return Err(ScidError::conversion_error("King not on home rank for kingside castling")),
            };
            Ok(Move::Castle { king: from_square, rook: rook_from })
        }
        // Normal king moves using SCID's exact square difference table
        // SCID king move encoding from scidvspc/src/game.cpp decodeKing:
        // static const int sqdiff[] = { 0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2 };
        1..=10 => {
            // Use exact SCID mapping for consistency with position_tracker
            let square_diffs = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];
            let diff = square_diffs[direction_code as usize];
            
            let to_square = calculate_target_square_scid(from_square, diff)?;
            let capture = get_capture_at_square(to_square, position);
            
            Ok(Move::Normal {
                role: Role::King,
                from: from_square,
                to: to_square,
                capture,
                promotion: None,
            })
        }
        0 => {
            // Null move - this shouldn't happen in normal games
            Err(ScidError::conversion_error("King null move (direction 0) not supported"))
        }
        _ => Err(ScidError::conversion_error(format!("Invalid king direction code: {}", direction_code))),
    }
}

/// Convert SCID queen moves to shakmaty moves
fn convert_queen_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::Queen, position)?;
    
    let to_square = if move_value >= 8 {
        // Rook-like vertical move to rank (move_value - 8)
        let target_rank = move_value - 8;
        let square_index = from_square.file() as u8 + (target_rank * 8);
        if square_index < 64 {
            Square::new(square_index as u32)
        } else {
            return Err(ScidError::conversion_error(format!("Invalid queen target rank: {}", target_rank)));
        }
    } else {
        // Rook-like horizontal move to file or diagonal (needs position context)
        // For now, assume horizontal move to file
        let square_index = move_value + (from_square.rank() as u8 * 8);
        if square_index < 64 {
            Square::new(square_index as u32)
        } else {
            return Err(ScidError::conversion_error(format!("Invalid queen target file: {}", move_value)));
        }
    };
    
    let capture = get_capture_at_square(to_square, position);
    
    Ok(Move::Normal {
        role: Role::Queen,
        from: from_square,
        to: to_square,
        capture,
        promotion: None,
    })
}

/// Convert SCID rook moves to shakmaty moves
fn convert_rook_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::Rook, position)?;
    
    let to_square = if move_value >= 8 {
        // Vertical move to rank (move_value - 8)
        let target_rank = move_value - 8;
        let square_index = from_square.file() as u8 + (target_rank * 8);
        if square_index < 64 {
            Square::new(square_index as u32)
        } else {
            return Err(ScidError::conversion_error(format!("Invalid rook target rank: {}", target_rank)));
        }
    } else {
        // Horizontal move to file
        let square_index = move_value + (from_square.rank() as u8 * 8);
        if square_index < 64 {
            Square::new(square_index as u32)
        } else {
            return Err(ScidError::conversion_error(format!("Invalid rook target file: {}", move_value)));
        }
    };
    
    let capture = get_capture_at_square(to_square, position);
    
    Ok(Move::Normal {
        role: Role::Rook,
        from: from_square,
        to: to_square,
        capture,
        promotion: None,
    })
}

/// Convert SCID bishop moves to shakmaty moves
fn convert_bishop_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::Bishop, position)?;
    
    // For now, simplified bishop move conversion (target file from lower 3 bits)
    // Direction bit (bit 3) determines diagonal direction, but needs full implementation
    let target_file = move_value & 7;
    let square_index = target_file + (from_square.rank() as u8 * 8);
    let to_square = if square_index < 64 {
        Square::new(square_index as u32)
    } else {
        return Err(ScidError::conversion_error(format!("Invalid bishop target file: {}", target_file)));
    };
    
    let capture = get_capture_at_square(to_square, position);
    
    Ok(Move::Normal {
        role: Role::Bishop,
        from: from_square,
        to: to_square,
        capture,
        promotion: None,
    })
}

/// Convert SCID knight moves to shakmaty moves
fn convert_knight_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::Knight, position)?;
    
    // Knight move lookup table with L-shaped square differences
    let square_diffs = match move_value {
        0 => 0,    // Null move (special case)
        1 => -17,  // Up 2, Left 1
        2 => -15,  // Up 2, Right 1
        3 => -10,  // Up 1, Left 2
        4 => -6,   // Up 1, Right 2
        5 => 6,    // Down 1, Left 2
        6 => 10,   // Down 1, Right 2
        7 => 15,   // Down 2, Left 1
        8 => 17,   // Down 2, Right 1
        _ => return Err(ScidError::conversion_error(format!("Invalid knight move value: {}", move_value))),
    };
    
    let to_square = calculate_target_square_scid(from_square, square_diffs)?;
    let capture = get_capture_at_square(to_square, position);
    
    Ok(Move::Normal {
        role: Role::Knight,
        from: from_square,
        to: to_square,
        capture,
        promotion: None,
    })
}

/// Convert SCID pawn moves to shakmaty moves
fn convert_pawn_move(move_value: u8, piece_num: u8, promotion: Option<&str>, position: &Chess) -> Result<Move> {
    let from_square = find_piece_square(piece_num, Role::Pawn, position)?;
    
    // Determine pawn color from current position
    let piece = position.board().piece_at(from_square)
        .ok_or_else(|| ScidError::conversion_error("No piece found at pawn from_square"))?;
    
    // Simplified pawn move calculation (forward move for now)
    let direction = match piece.color {
        Color::White => 8,  // White pawns move up (positive direction in shakmaty)
        Color::Black => -8, // Black pawns move down (negative direction in shakmaty)
    };
    
    let to_square = calculate_target_square_scid(from_square, direction)?;
    let capture = get_capture_at_square(to_square, position);
    
    // Handle promotion
    let promotion_role = if let Some(promo) = promotion {
        Some(parse_promotion_piece_scid(promo)?)
    } else {
        None
    };
    
    Ok(Move::Normal {
        role: Role::Pawn,
        from: from_square,
        to: to_square,
        capture,
        promotion: promotion_role,
    })
}

/// Helper function to find a piece's current square on the board
/// 
/// This is a simplified implementation that searches the board for the piece.
/// A full implementation would maintain piece tracking for efficiency.
fn find_piece_square(piece_num: u8, role: Role, position: &Chess) -> Result<Square> {
    let board = position.board();
    let current_color = position.turn();
    
    // Search the board for pieces of the specified role and color
    let mut found_pieces = Vec::new();
    
    for square in Square::ALL {
        if let Some(piece) = board.piece_at(square) {
            if piece.role == role && piece.color == current_color {
                found_pieces.push(square);
            }
        }
    }
    
    // For now, use piece_num to select from found pieces
    // This is simplified - a full implementation would maintain proper piece tracking
    if let Some(&square) = found_pieces.get(piece_num as usize % found_pieces.len()) {
        Ok(square)
    } else {
        Err(ScidError::conversion_error(format!("Could not find {:?} piece {} for color {:?}", 
            role, piece_num, current_color)))
    }
}


/// Helper function to detect captures at a target square
fn get_capture_at_square(square: Square, position: &Chess) -> Option<Role> {
    if let Some(piece) = position.board().piece_at(square) {
        // Only count as capture if it's opponent's piece
        if piece.color != position.turn() {
            Some(piece.role)
        } else {
            None
        }
    } else {
        None
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Chess, Position};
    use crate::sg4::{DecodedMove, MoveInterpretation};
    
    /// Test basic king move conversion
    #[test]
    fn test_king_move_conversion() {
        let chess = Chess::default();
        
        // Test direction 5 (east) which should be valid from E1 to F1
        let decoded_move = DecodedMove {
            piece_num: 0,
            move_value: 5,
            raw_byte: 0x15,
            interpretation: MoveInterpretation::King { 
                direction_code: 5, 
                description: "King move east".to_string() 
            },
        };
        
        let result = decoded_move.to_shakmaty(&chess);
        if let Err(ref error) = result {
            println!("King move error: {:?}", error);
            println!("From square E1 = {}", Square::E1 as u8);
            println!("To square would be: {} + {} = {}", Square::E1 as u8, -8, (Square::E1 as i8) + (-8));
        }
        assert!(result.is_ok(), "King move conversion should succeed");
        
        if let Ok(shakmaty_move) = result {
            if let Move::Normal { role, .. } = shakmaty_move {
                assert_eq!(role, Role::King);
            } else {
                panic!("Expected normal king move, got: {:?}", shakmaty_move);
            }
        }
    }
    
    /// Test castling move conversion
    #[test]
    fn test_castling_conversion() {
        let chess = Chess::default();
        
        // Test kingside castling
        let kingside_move = DecodedMove {
            piece_num: 0,
            move_value: 10,
            raw_byte: 0x1A,
            interpretation: MoveInterpretation::King { 
                direction_code: 10, 
                description: "Kingside castling".to_string() 
            },
        };
        
        let result = kingside_move.to_shakmaty(&chess);
        assert!(result.is_ok(), "Kingside castling conversion should succeed");
        
        if let Ok(shakmaty_move) = result {
            if let Move::Castle { king, rook } = shakmaty_move {
                assert_eq!(king, Square::E1);
                assert_eq!(rook, Square::H1);
            } else {
                panic!("Expected castling move, got: {:?}", shakmaty_move);
            }
        }
    }
    
    /// Test queen move conversion
    #[test]
    fn test_queen_move_conversion() {
        let chess = Chess::default();
        
        let decoded_move = DecodedMove {
            piece_num: 0,
            move_value: 12, // Move to rank 4 (12 - 8 = 4)
            raw_byte: 0x2C,
            interpretation: MoveInterpretation::Queen { 
                move_type: "vertical to rank 4".to_string(),
                description: "Queen to rank 4".to_string() 
            },
        };
        
        let result = decoded_move.to_shakmaty(&chess);
        assert!(result.is_ok(), "Queen move conversion should succeed");
        
        if let Ok(shakmaty_move) = result {
            if let Move::Normal { role, .. } = shakmaty_move {
                assert_eq!(role, Role::Queen);
            } else {
                panic!("Expected normal queen move, got: {:?}", shakmaty_move);
            }
        }
    }
    
    /// Test error handling for unknown moves
    #[test]
    fn test_unknown_move_error() {
        let chess = Chess::default();
        
        let decoded_move = DecodedMove {
            piece_num: 0,
            move_value: 0,
            raw_byte: 0xFF,
            interpretation: MoveInterpretation::Unknown { reason: "test error".to_string() },
        };
        
        let result = decoded_move.to_shakmaty(&chess);
        assert!(result.is_err(), "Unknown move should return error");
        
        if let Err(error) = result {
            assert!(format!("{}", error).contains("test error"));
        }
    }
    
    /// Test helper function for finding pieces
    #[test]
    fn test_find_piece_square() {
        let chess = Chess::default();
        
        // Try to find the white king (should be on e1)
        let result = find_piece_square(0, Role::King, &chess);
        assert!(result.is_ok(), "Should find white king");
        
        if let Ok(square) = result {
            assert_eq!(square, Square::E1);
        }
    }
    
    /// Test square calculation helper
    #[test]
    fn test_calculate_target_square() {
        // Let's debug the square numbering first
        println!("E2 = {}, E3 = {}", Square::E2 as u8, Square::E3 as u8);
        println!("E4 = {}, F4 = {}", Square::E4 as u8, Square::F4 as u8);
        
        // Test moving one square forward (in shakmaty, square numbers increase up the board)
        // E2 is square 12, E3 is square 20, so difference is +8, not -8
        let result = calculate_target_square_scid(Square::E2, 8);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Square::E3);
        
        // Test moving one square right
        let result = calculate_target_square_scid(Square::E4, 1);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Square::F4);
        
        // Test out of bounds
        let result = calculate_target_square_scid(Square::A1, -1);
        assert!(result.is_err(), "Should fail for out of bounds move");
    }
}

/// Public function for converting SCID moves to shakmaty moves with position context
/// 
/// This function is used by the optimized position tracker and other components
/// that need to convert SCID moves using the current chess position.
pub fn convert_scid_to_shakmaty(scid_move: &crate::sg4::DecodedMove, position: &Chess) -> Result<Move> {
    // Use the existing ScidToShakmaty trait implementation
    scid_move.to_shakmaty(position)
}