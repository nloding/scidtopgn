// Integration module for position-aware move decoding
// Bridges our SCID-compliant position decoder with existing game parsing

use crate::position::{ScidPosition, ScidMove, decode_move};
use crate::sg4::{DecodedMove, MoveInterpretation};

/// Convert our ScidMove to the existing DecodedMove format
/// This allows seamless integration with existing display code
pub fn scid_move_to_decoded_move(scid_move: &ScidMove, raw_byte: u8) -> DecodedMove {
    // Generate algebraic notation (simplified for now)
    let from_algebraic = scid_move.from.to_algebraic();
    let to_algebraic = scid_move.to.to_algebraic();
    let move_notation = format!("{}-{}", from_algebraic, to_algebraic);
    
    // Convert to the existing MoveInterpretation format based on piece type
    let interpretation = match scid_move.moving_piece {
        crate::position::PieceType::Pawn => {
            let direction = if scid_move.to.0 > scid_move.from.0 { "forward" } else { "capture" };
            let promotion = if scid_move.promote != crate::position::PieceType::Empty {
                Some(format!("{:?}", scid_move.promote))
            } else {
                None
            };
            
            MoveInterpretation::Pawn {
                direction: direction.to_string(),
                promotion,
                description: format!("Pawn {} ({})", move_notation, direction),
                is_en_passant: None, // TODO: Detect en passant
            }
        },
        crate::position::PieceType::King => {
            MoveInterpretation::King {
                direction_code: scid_move.piece_num,
                description: format!("King {} (position-aware)", move_notation),
            }
        },
        crate::position::PieceType::Queen => {
            MoveInterpretation::Queen {
                move_type: "position-aware".to_string(),
                description: format!("Queen {} (position-aware)", move_notation),
            }
        },
        crate::position::PieceType::Rook => {
            MoveInterpretation::Rook {
                target_info: "position-aware".to_string(),
                description: format!("Rook {} (position-aware)", move_notation),
            }
        },
        crate::position::PieceType::Bishop => {
            MoveInterpretation::Bishop {
                direction: "position-aware".to_string(),
                description: format!("Bishop {} (position-aware)", move_notation),
            }
        },
        crate::position::PieceType::Knight => {
            MoveInterpretation::Knight {
                l_shape_code: scid_move.piece_num,
                description: format!("Knight {} (position-aware)", move_notation),
            }
        },
        _ => {
            MoveInterpretation::Unknown {
                reason: format!("Unknown piece type: {:?}", scid_move.moving_piece),
            }
        }
    };
    
    DecodedMove {
        piece_num: scid_move.piece_num,
        move_value: raw_byte & 0x0F, // Extract move value from raw byte
        raw_byte,
        interpretation,
    }
}

/// Position-aware move decoder that integrates with existing game parsing
/// This replaces the static try_decode_move function with position-aware decoding
pub fn decode_move_with_position(
    position: &ScidPosition, 
    raw_byte: u8,
    offset: usize
) -> Result<DecodedMove, String> {
    // Use our position-aware decoder
    match decode_move(position, raw_byte) {
        Ok(scid_move) => {
            // Convert to existing format for compatibility
            Ok(scid_move_to_decoded_move(&scid_move, raw_byte))
        },
        Err(e) => {
            Err(format!("Position-aware decoding failed at offset {}: {}", offset, e))
        }
    }
}

/// Enhanced game element processing with position tracking
/// This maintains a ScidPosition throughout game parsing for accurate move decoding
pub struct PositionTracker {
    position: ScidPosition,
    move_count: u32,
}

impl PositionTracker {
    /// Create new tracker with starting position
    pub fn new() -> Self {
        PositionTracker {
            position: ScidPosition::new_starting_position(),
            move_count: 0,
        }
    }
    
    /// Process a move byte with position-aware decoding
    pub fn process_move(&mut self, raw_byte: u8, offset: usize) -> Result<DecodedMove, String> {
        // Decode the move using current position
        let decoded_move = decode_move_with_position(&self.position, raw_byte, offset)?;
        
        // Convert back to ScidMove to apply to position
        let _piece_num = (raw_byte >> 4) as usize;
        let _move_value = raw_byte & 0x0F;
        
        if let Ok(scid_move) = decode_move(&self.position, raw_byte) {
            // Apply the move to update position
            if let Err(e) = self.position.do_move(&scid_move) {
                return Err(format!("Failed to apply move {}: {}", self.move_count + 1, e));
            }
            self.move_count += 1;
        }
        
        Ok(decoded_move)
    }
    
    /// Get current position (for debugging)
    pub fn current_position(&self) -> &ScidPosition {
        &self.position
    }
    
    /// Get move count
    pub fn move_count(&self) -> u32 {
        self.move_count
    }
}