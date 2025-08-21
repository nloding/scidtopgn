// Integration module for position-aware move decoding
// Bridges our SCID-compliant position decoder with existing game parsing

use crate::position::{ScidPosition, ScidMove, decode_move, decode_move_with_stream, ScidByteStream};
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
    
    /// Process move from byte stream (supports multi-byte moves)
    pub fn process_move_from_stream(&mut self, stream: &mut ScidByteStream, _offset: usize) -> Result<DecodedMove, String> {
        // Record stream position before decoding (for debugging)
        let initial_position = stream.position();
        
        // Decode move using stream-aware decoder
        let scid_move = decode_move_with_stream(&self.position, stream)?;
        
        // Calculate bytes consumed (for multi-byte moves)
        let bytes_consumed = stream.position() - initial_position;
        
        // Apply the move to update position
        if let Err(e) = self.position.do_move(&scid_move) {
            return Err(format!("Failed to apply move {}: {}", self.move_count + 1, e));
        }
        self.move_count += 1;
        
        // Create DecodedMove for compatibility with existing code
        let decoded_move = DecodedMove {
            piece_num: scid_move.piece_num,
            move_value: 0, // Not meaningful for multi-byte moves
            raw_byte: 0,   // Not meaningful for multi-byte moves  
            interpretation: MoveInterpretation::Decoded {
                description: scid_move.to_algebraic(&self.position),
                from_square: Some(scid_move.from.to_algebraic()),
                to_square: Some(scid_move.to.to_algebraic()),
                piece_type: Some(format!("{:?}", scid_move.moving_piece)),
                is_capture: scid_move.captured_piece != crate::position::PieceType::Empty,
                is_promotion: scid_move.promote != crate::position::PieceType::Empty,
                bytes_consumed, // 🔥 NEW: Track how many bytes this move consumed
            },
        };
        
        Ok(decoded_move)
    }
    
    /// Process a move byte with position-aware decoding (backward compatibility)
    pub fn process_move(&mut self, raw_byte: u8, offset: usize) -> Result<DecodedMove, String> {
        // Create temporary stream with single byte
        let byte_array = [raw_byte];
        let mut stream = ScidByteStream::new(&byte_array);
        self.process_move_from_stream(&mut stream, offset)
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