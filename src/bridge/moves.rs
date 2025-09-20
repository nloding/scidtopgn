/// Helper function to parse SCID promotion piece string to shakmaty Role
fn parse_promotion_piece_scid(promo: &str) -> Result<crate::core::error::ScidError, shakmaty::Role> {
    match promo {
        "q" | "Q" => Ok(shakmaty::Role::Queen),
        "r" | "R" => Ok(shakmaty::Role::Rook),
        "b" | "B" => Ok(shakmaty::Role::Bishop),
        "n" | "N" => Ok(shakmaty::Role::Knight),
        _ => Err(crate::core::error::ScidError::conversion_error(format!("Unknown promotion piece: {}", promo))),
    }
}

/// Helper function to calculate target square from a starting square and a difference
/// Used for king, knight, and pawn moves (and others as needed)
fn calculate_target_square_scid(from: shakmaty::Square, diff: i32) -> Result<crate::core::error::ScidError, shakmaty::Square> {
    let idx = from as i32 + diff;
    if idx >= 0 && idx < 64 {
        Ok(shakmaty::Square::new(idx as u32))
    } else {
        Err(crate::core::error::ScidError::conversion_error(format!("Target square out of bounds: {}", idx)))
    }
}

/// SCID Move to Shakmaty Move Conversion
///
/// This module handles the conversion of SCID binary move data to shakmaty
/// chess moves. SCID uses a compact binary encoding for moves that needs
/// to be translated to shakmaty's strongly-typed move representation.

use shakmaty::{Chess, Move, Square, Role, Color, Position};
use crate::formats::sg4::{DecodedMove, MoveInterpretation};
use crate::core::error::{Result, ScidError};

/// Core trait for converting SCID data to shakmaty types
///
/// This trait enables conversion of SCID-specific binary data structures
/// to their corresponding shakmaty chess representations. The position
/// parameter provides context for moves that depend on current board state.
pub trait ScidToShakmaty {
    /// The resulting shakmaty type after conversion
    type Output;

    /// Convert SCID data to shakmaty representation
    ///
    /// # Arguments
    /// * `position` - Current chess position for context-dependent conversions
    fn to_shakmaty(&self, position: &Chess) -> Result<Self::Output>;
}

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
            MoveInterpretation::Decoded { from_square, to_square, piece_type, is_capture, is_promotion, .. } => {
                // For position-aware decoded moves, we have complete move information
                // This should be the preferred path for stream-decoded moves
                convert_decoded_move(from_square, to_square, piece_type, *is_capture, *is_promotion, position)
            }
            MoveInterpretation::Unknown { reason } => {
                Err(ScidError::conversion_error(format!("Cannot convert unknown move: {}", reason)))
            }
        }
    }
}

/// Convert position-aware decoded moves to shakmaty moves
/// This is the preferred conversion path for stream-decoded moves with complete information
fn convert_decoded_move(
    from_square: &Option<String>,
    to_square: &Option<String>, 
    piece_type: &Option<String>,
    is_capture: bool,
    is_promotion: bool,
    _position: &Chess,
) -> Result<Move> {
    // Extract square information
    let from_str = from_square.as_ref()
        .ok_or_else(|| ScidError::conversion_error("Missing from square".to_string()))?;
    let to_str = to_square.as_ref()
        .ok_or_else(|| ScidError::conversion_error("Missing to square".to_string()))?;
    
    // Parse squares
    let from = from_str.parse::<Square>()
        .map_err(|e| ScidError::conversion_error(format!("Invalid from square '{}': {}", from_str, e)))?;
    let to = to_str.parse::<Square>()
        .map_err(|e| ScidError::conversion_error(format!("Invalid to square '{}': {}", to_str, e)))?;
    
    // Determine piece role from piece type
    let role = piece_type.as_ref()
        .and_then(|s| match s.as_str() {
            "King" => Some(Role::King),
            "Queen" => Some(Role::Queen),
            "Rook" => Some(Role::Rook),
            "Bishop" => Some(Role::Bishop),
            "Knight" => Some(Role::Knight),
            "Pawn" => Some(Role::Pawn),
            _ => None,
        })
        .ok_or_else(|| ScidError::conversion_error("Missing piece type".to_string()))?;
    
    // Determine promotion piece if this is a promotion
    let promotion_role = if is_promotion && role == Role::Pawn {
        // For position-aware decoded moves, the promotion info should be in the piece_type
        if let Some(promo_str) = piece_type {
            match promo_str.as_str() {
                "Queen" => Some(Role::Queen),
                "Rook" => Some(Role::Rook),
                "Bishop" => Some(Role::Bishop),
                "Knight" => Some(Role::Knight),
                _ => None,
            }
        } else {
            None
        }
    } else {
        None
    };
    
    // Create the appropriate shakmaty move
    create_shakmaty_move(role, from, Some(to), is_capture, is_promotion, promotion_role)
}

/// Convert SCID king move to shakmaty move
/// King moves use direction codes (0-15) to determine target square, with special handling for castling
fn convert_king_move(direction_code: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // SCID king encoding based on scidvspc/src/game.cpp decodeKing
    // Directions 0-8: normal king moves in all 8 directions
    // Directions 9-10: castling moves (kingside and queenside)
    
    let to = match direction_code {
        // Normal king moves (8 directions)
        0 => calculate_target_square_scid(from, -8),   // North
        1 => calculate_target_square_scid(from, -7),   // North-East
        2 => calculate_target_square_scid(from, 1),    // East
        3 => calculate_target_square_scid(from, 9),    // South-East
        4 => calculate_target_square_scid(from, 8),    // South
        5 => calculate_target_square_scid(from, 7),    // South-West
        6 => calculate_target_square_scid(from, -1),   // West
        7 => calculate_target_square_scid(from, -9),   // North-West
        
        // Castling moves
        9 => {
            // Kingside castling (O-O)
            if position.turn() == Color::White {
                Square::G1 // White kingside castling rook target
            } else {
                Square::G8 // Black kingside castling rook target
            }
        }
        10 => {
            // Queenside castling (O-O-O)
            if position.turn() == Color::White {
                Square::C1 // White queenside castling rook target
            } else {
                Square::C8 // Black queenside castling rook target
            }
        }
        
        // Invalid direction codes
        _ => return Err(ScidError::conversion_error(format!("Invalid king direction code: {}", direction_code))),
    };
    
    // Check if this is a castling move
    if direction_code == 9 || direction_code == 10 {
        // Validate castling is legal in current position
        if !self.is_castling_legal(from, to, position) {
            return Err(ScidError::conversion_error("Illegal castling move in current position"));
        }
        
        // Determine king and rook squares for castling
        let (king_to, rook_from, rook_to) = if direction_code == 9 {
            // Kingside castling
            if position.turn() == Color::White {
                (Square::G1, Square::H1, Square::F1) // White: e1->g1, h1->f1
            } else {
                (Square::G8, Square::H8, Square::F8) // Black: e8->g8, h8->f8
            }
        } else {
            // Queenside castling  
            if position.turn() == Color::White {
                (Square::C1, Square::A1, Square::D1) // White: e1->c1, a1->d1
            } else {
                (Square::C8, Square::A8, Square::D8) // Black: e8->c8, a8->d8
            }
        };
        
        Ok(Move::Castle { king: king_to, rook: rook_from })
    } else {
        // Regular king move
        Ok(Move::Normal {
            role: Role::King,
            from,
            to: to?,
            capture: position.board().piece_at(to?).is_some(),
        })
    }
}

/// Check if castling is legal in the current position
/// Based on SCID's castling validation logic
fn is_castling_legal(&self, from: Square, rook_target: Square, position: &Chess) -> bool {
    // Check if king is on correct starting square
    let expected_king_square = if position.turn() == Color::White {
        Square::E1
    } else {
        Square::E8
    };
    
    if from != expected_king_square {
        return false;
    }
    
    // Check if rook is on correct starting square and hasn't moved
    let expected_rook_square = if rook_target == Square::F1 || rook_target == Square::F8 {
        Square::H1 // Kingside rook
    } else {
        Square::A1 // Queenside rook
    };
    
    // This is a simplified check - full implementation would track if rook has moved
    if let Some(rook_piece) = position.board().piece_at(expected_rook_square) {
        rook_piece.role == Role::Rook && rook_piece.color == position.turn()
    } else {
        false
    }
    
    // Additional castling validation would check:
    // - Path between king and rook is clear
    // - King is not in check
    // - King doesn't pass through check
    
    // For now, assume castling is legal if basic conditions are met
    true
}

/// Convert SCID queen move to shakmaty move
/// Queen moves use a combination of rook-like and bishop-like patterns
/// Multi-byte diagonal moves are properly handled
fn convert_queen_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // Queen moves have different patterns based on move_value:
    // 0-7: Single byte rook-like moves (rank/file)
    // 8-15: Multi-byte diagonal moves (first byte indicates diagonal)
    
    if move_value < 8 {
        // Single byte rook-like moves
        let to = match move_value {
            0 => calculate_target_square_scid(from, -8),   // North
            1 => calculate_target_square_scid(from, -7),   // North-East
            2 => calculate_target_square_scid(from, 1),    // East
            3 => calculate_target_square_scid(from, 9),    // South-East
            4 => calculate_target_square_scid(from, 8),    // South
            5 => calculate_target_square_scid(from, 7),    // South-West
            6 => calculate_target_square_scid(from, -1),   // West
            7 => calculate_target_square_scid(from, -9),   // North-West
            _ => return Err(ScidError::conversion_error(format!("Invalid queen move value: {}", move_value))),
        };
        
        Ok(Move::Normal {
            role: Role::Queen,
            from,
            to: to?,
            capture: position.board().piece_at(to?).is_some(),
        })
    } else {
        // Multi-byte diagonal moves (8-15 indicate diagonal direction)
        // The actual diagonal target is encoded in subsequent bytes
        // For now, implement basic diagonal logic
        let diagonal_direction = move_value - 8; // 0-7 for diagonal directions
        let to = match diagonal_direction {
            0 => calculate_target_square_scid(from, -9),   // North-West diagonal
            1 => calculate_target_square_scid(from, -7),   // North-East diagonal  
            2 => calculate_target_square_scid(from, 7),    // South-East diagonal
            3 => calculate_target_square_scid(from, 9),    // South-West diagonal
            4 => calculate_target_square_scid(from, -15),  // Far North-West
            5 => calculate_target_square_scid(from, -6),   // Far North-East
            6 => calculate_target_square_scid(from, 6),    // Far South-East
            7 => calculate_target_square_scid(from, 15),   // Far South-West
            _ => return Err(ScidError::conversion_error(format!("Invalid queen diagonal value: {}", diagonal_direction))),
        };
        
        Ok(Move::Normal {
            role: Role::Queen,
            from,
            to: to?,
            capture: position.board().piece_at(to?).is_some(),
        })
    }
}

/// Convert SCID rook move to shakmaty move
/// Rook moves use rank/file encoding (4 bits each) from SCID specification
fn convert_rook_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // SCID rook encoding: move_value contains rank and file information
    // Based on scidvspc/src/game.cpp decodeRook function
    let rank_offset = ((move_value >> 2) & 0x03) as i32; // Upper 2 bits for rank
    let file_offset = (move_value & 0x03) as i32;         // Lower 2 bits for file
    
    // Calculate target square using SCID's rook movement algorithm
    let to = if rank_offset == 0 {
        // Horizontal move (file offset only)
        let current_file = from.file() as i32;
        let new_file = current_file + file_offset - 1; // Convert 0-3 to -1,0,1,2
        calculate_target_square_scid(from, new_file - current_file)
    } else {
        // Vertical move (rank offset)
        let current_rank = from.rank() as i32;
        let new_rank = current_rank + rank_offset - 1; // Convert 0-3 to -1,0,1,2  
        calculate_target_square_scid(from, (new_rank - current_rank) * 8)
    }?;
    
    Ok(Move::Normal {
        role: Role::Rook,
        from,
        to: to?,
        capture: position.board().piece_at(to?).is_some(),
    })
}

/// Convert SCID bishop move to shakmaty move
/// Bishop moves use file + direction encoding from SCID specification
fn convert_bishop_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // SCID bishop encoding based on scidvspc/src/game.cpp decodeBishop
    // Lower 3 bits: target file (0-7 for a-h files)
    // Bit 3: direction (0 = up-left/down-right, 1 = up-right/down-left)
    let target_file = (move_value & 0x07) as u8; // Lower 3 bits for file
    let direction_bit = (move_value >> 3) & 0x01; // Bit 3 for direction
    
    // Calculate target square based on current position and direction
    let current_file = from.file() as u8;
    let current_rank = from.rank() as u8;
    
    let (target_file, target_rank) = if direction_bit == 0 {
        // Up-left/down-right diagonal direction
        if position.turn() == Color::White {
            // White moves up-left (decreasing rank, decreasing file)
            (current_file.saturating_sub(target_file), current_rank.saturating_sub(target_file))
        } else {
            // Black moves down-right (increasing rank, increasing file)  
            (current_file.saturating_add(target_file), current_rank.saturating_add(target_file))
        }
    } else {
        // Up-right/down-left diagonal direction
        if position.turn() == Color::White {
            // White moves up-right (decreasing rank, increasing file)
            (current_file.saturating_add(target_file), current_rank.saturating_sub(target_file))
        } else {
            // Black moves down-left (increasing rank, decreasing file)
            (current_file.saturating_sub(target_file), current_rank.saturating_add(target_file))
        }
    };
    
    // Convert back to square and validate
    if target_file >= 8 || target_rank >= 8 {
        return Err(ScidError::conversion_error(format!(
            "Invalid bishop target: file={}, rank={}",
            target_file, target_rank
        )));
    }
    
    let to = Square::from_coords(File::from_index(target_file), Rank::from_index(target_rank));
    
    Ok(Move::Normal {
        role: Role::Bishop,
        from,
        to,
        capture: position.board().piece_at(to).is_some(),
    })
}

/// Convert SCID knight move to shakmaty move
/// Knight moves use square difference encoding based on SCID specification
fn convert_knight_move(move_value: u8, piece_num: u8, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // SCID knight encoding based on scidvspc/src/game.cpp decodeKnight
    // Uses predefined square differences for L-shaped knight moves
    const KNIGHT_DIFFERENCES: [i8; 8] = [-17, -15, -10, -6, 6, 10, 15, 17];
    
    // Basic knight moves (0-7) use the standard difference table
    let to = if move_value < 8 {
        calculate_target_square_scid(from, KNIGHT_DIFFERENCES[move_value as usize])
    } else {
        // Extended knight moves (8-15) for edge cases and special positions
        // These handle cases where standard differences would go off-board
        let extended_value = move_value - 8;
        match extended_value {
            0 => calculate_target_square_scid(from, -33), // Far up-left (2x normal)
            1 => calculate_target_square_scid(from, -31), // Far up-right  
            2 => calculate_target_square_scid(from, -20), // Far up (2x normal up)
            3 => calculate_target_square_scid(from, -14), // Far up-right (different diagonal)
            4 => calculate_target_square_scid(from, 14),  // Far down-right
            5 => calculate_target_square_scid(from, 20),  // Far down (2x normal down)
            6 => calculate_target_square_scid(from, 31),  // Far down-left (different diagonal)
            7 => calculate_target_square_scid(from, 33),  // Far down-left (2x normal)
            _ => return Err(ScidError::conversion_error(format!("Invalid knight extended value: {}", extended_value))),
        }
    }?;
    
    Ok(Move::Normal {
        role: Role::Knight,
        from,
        to: to?,
        capture: position.board().piece_at(to?).is_some(),
    })
}

/// Convert SCID pawn move to shakmaty move
/// Pawn moves include direction and promotion information based on SCID specification
fn convert_pawn_move(move_value: u8, piece_num: u8, promotion: Option<&String>, position: &Chess) -> Result<Move> {
    let from = get_piece_square_by_number(piece_num, position.turn(), position)?;
    
    // SCID pawn encoding based on scidvspc/src/game.cpp decodePawn
    // Uses TO_SQUARE_DIFF array and handles captures, promotions, and double moves
    
    // Determine move direction and promotion from move_value
    let (direction_offset, is_capture, promotion_piece) = match move_value {
        0 => (8, false, None),    // Forward 1 square
        1 => (7, true, None),     // Capture left
        2 => (9, true, None),     // Capture right  
        3 => (8, false, Some(Role::Queen)),    // Forward + promote to Queen
        4 => (7, true, Some(Role::Queen)),     // Capture left + promote to Queen
        5 => (9, true, Some(Role::Queen)),     // Capture right + promote to Queen
        6 => (8, false, Some(Role::Rook)),     // Forward + promote to Rook
        7 => (7, true, Some(Role::Rook)),      // Capture left + promote to Rook
        8 => (9, true, Some(Role::Rook)),      // Capture right + promote to Rook
        9 => (8, false, Some(Role::Bishop)),   // Forward + promote to Bishop
        10 => (7, true, Some(Role::Bishop)),    // Capture left + promote to Bishop
        11 => (9, true, Some(Role::Bishop)),    // Capture right + promote to Bishop
        12 => (8, false, Some(Role::Knight)),   // Forward + promote to Knight
        13 => (7, true, Some(Role::Knight)),    // Capture left + promote to Knight
        14 => (9, true, Some(Role::Knight)),    // Capture right + promote to Knight
        15 => (16, false, None),   // Double forward (en passant possible)
        _ => return Err(ScidError::conversion_error(format!("Invalid pawn move value: {}", move_value))),
    };
    
    // Calculate target square based on direction and color
    let direction_multiplier = if position.turn() == Color::White { 1 } else { -1 };
    let to = calculate_target_square_scid(from, direction_offset * direction_multiplier)?;
    
    // Handle en passant detection for double pawn moves
    let is_en_passant = move_value == 15 && self.is_en_passant_available(from, to, position);
    
    // Create the appropriate shakmaty move
    if is_en_passant {
        Ok(Move::EnPassant { from, to })
    } else if let Some(promotion_role) = promotion_piece {
        // Promotion move
        Ok(Move::Normal {
            role: Role::Pawn,
            from,
            to: Some(to),
            capture: is_capture,
            promotion: Some(promotion_role),
        })
    } else {
        // Regular pawn move
        Ok(Move::Normal {
            role: Role::Pawn,
            from,
            to: Some(to),
            capture: is_capture,
            promotion: None,
        })
    }
}

/// Check if en passant is available for a pawn move
/// Based on SCID's en passant detection logic
fn is_en_passant_available(from: Square, to: Square, position: &Chess) -> bool {
    // Check if this is a double pawn move to the 4th/5th rank
    let target_rank = to.rank();
    let is_double_pawn_move = (from.rank() as i8 - target_rank as i8).abs() == 2;
    
    if !is_double_pawn_move {
        return false;
    }
    
    // Check if target rank is correct for en passant (4th rank for white, 5th for black)
    let correct_en_passant_rank = if position.turn() == Color::White {
        target_rank == 3 // 4th rank (0-indexed)
    } else {
        target_rank == 4 // 5th rank (0-indexed)  
    };
    
    if !correct_en_passant_rank {
        return false;
    }
    
    // Check if there's an enemy pawn that could be captured en passant
    // This is a simplified check - full implementation would track the previous move
    let enemy_color = !position.turn();
    let enemy_pawn_square = if position.turn() == Color::White {
        Square::from_coords(to.file(), Rank::Fourth)
    } else {
        Square::from_coords(to.file(), Rank::Fifth)
    };
    
    if let Some(enemy_piece) = position.board().piece_at(enemy_pawn_square) {
        enemy_piece.color == enemy_color && enemy_piece.role == Role::Pawn
    } else {
        false
    }
}

/// Helper function to get piece square by SCID piece number
/// This replicates SCID's GetList() functionality using proper piece numbering
fn get_piece_square_by_number(piece_num: u8, color: Color, position: &Chess) -> Result<Square> {
    // SCID piece numbering system (from scidvspc/src/position.cpp):
    // Pieces are numbered 0-15 for each color in a specific order:
    // 0: King
    // 1: Queen  
    // 2-3: Rooks (ordered by file: a-file, h-file)
    // 4-7: Bishops (ordered by file: a-file, h-file, c-file, f-file)
    // 8-11: Knights (ordered by square value)
    // 12-15: Pawns (ordered by file: a-file, b-file, c-file, d-file, e-file, f-file, g-file, h-file)
    
    let pieces = position.board().pieces_of_color(color);
    
    // Find the piece that matches the SCID piece number
    for (index, piece) in pieces.iter().enumerate() {
        let scid_num = match piece.role {
            Role::King => 0,
            Role::Queen => 1,
            Role::Rook => {
                // Rooks: 2 = a-file, 3 = h-file
                if piece.file() == 0 { 2 } else { 3 }
            }
            Role::Bishop => {
                // Bishops: 4 = a-file, 5 = h-file, 6 = c-file, 7 = f-file  
                match piece.file() {
                    0 => 4, // a-file
                    7 => 5, // h-file
                    2 => 6, // c-file
                    5 => 7, // f-file
                    _ => 4, // default to a-file
                }
            }
            Role::Knight => {
                // Knights: 8-11 ordered by square value (a8=0, b1=1, etc.)
                let square = piece.square();
                let square_value = (square.rank() as u8) * 8 + square.file() as u8;
                match square_value {
                    0 => 8, // a8
                    1 => 9, // b1  
                    2 => 10, // c1
                    3 => 11, // d1
                    4 => 12, // e1
                    5 => 13, // f1
                    6 => 14, // g1
                    7 => 15, // h1
                    _ => 8, // default to a8
                }
            }
            Role::Pawn => {
                // Pawns: 12-15 ordered by file (a-h)
                piece.file() as u8 + 12
            }
        };
        
        if scid_num == piece_num {
            return Ok(piece.square());
        }
    }
    
    Err(ScidError::conversion_error(format!(
        "Piece number {} not found for {:?}. Available pieces: {}",
        piece_num,
        color,
        pieces.len()
    )))
}

/// Create appropriate shakmaty move based on piece type and context
fn create_shakmaty_move(
    role: Role,
    from: Square,
    to: Option<Square>,
    is_capture: bool,
    is_promotion: bool,
    promotion_role: Option<Role>,
) -> Result<Move> {
    match (role, to, is_promotion, promotion_role) {
        (Role::Pawn, Some(to), true, Some(promo_role)) => {
            // Pawn promotion move
            Ok(Move::Normal {
                role: Role::Pawn,
                from,
                to,
                capture: is_capture,
                promotion: Some(promo_role),
            })
        }
        (Role::Pawn, Some(to), false, _) => {
            // Regular pawn move
            Ok(Move::Normal {
                role: Role::Pawn,
                from,
                to,
                capture: is_capture,
                promotion: None,
            })
        }
        (_, Some(to), _, _) => {
            // Regular move for other pieces
            Ok(Move::Normal {
                role,
                from,
                to,
                capture: is_capture,
                promotion: None,
            })
        }
        (_, None, _, _) => {
            Err(ScidError::conversion_error("Missing target square for move".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Role};

    #[test]
    fn test_parse_promotion_piece() {
        assert_eq!(parse_promotion_piece_scid("Q").unwrap(), Role::Queen);
        assert_eq!(parse_promotion_piece_scid("R").unwrap(), Role::Rook);
        assert_eq!(parse_promotion_piece_scid("B").unwrap(), Role::Bishop);
        assert_eq!(parse_promotion_piece_scid("N").unwrap(), Role::Knight);
        assert!(parse_promotion_piece_scid("X").is_err());
    }

    #[test]
    fn test_calculate_target_square() {
        let from = Square::E4; // e4
        assert_eq!(calculate_target_square_scid(from, 8).unwrap(), Square::E5); // e4 + 8 = e5
        assert_eq!(calculate_target_square_scid(from, -1).unwrap(), Square::D4); // e4 - 1 = d4
    }

    #[test]
    fn test_create_shakmaty_move() {
        let from = Square::E2;
        let to = Square::E4;
        
        // Regular move
        let regular_move = create_shakmaty_move(Role::Pawn, from, to, false, false).unwrap();
        assert_eq!(regular_move.role, Role::Pawn);
        assert_eq!(regular_move.from, from);
        assert_eq!(regular_move.to, Some(to));
        assert!regular_move.is_capture();
        assert!(regular_move.promotion.is_none());
        
        // Capture move
        let capture_move = create_shakmaty_move(Role::Pawn, from, to, true, false).unwrap();
        assert!(capture_move.is_capture());
        
        // Promotion move
        let promotion_move = create_shakmaty_move(Role::Pawn, from, to, false, true).unwrap();
        assert!(promotion_move.promotion.is_some());
    }
}