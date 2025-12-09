// Integration module for position-aware move decoding
// Bridges our SCID-compliant position decoder with existing game parsing

use super::moves::ScidMove;
use super::{ScidByteStream, ScidPosition};
use crate::formats::sg4::{DecodedMove, MoveInterpretation};

/// Convert our ScidMove to the existing DecodedMove format
/// This allows seamless integration with existing display code
#[allow(dead_code)]
pub fn scid_move_to_decoded_move(scid_move: &ScidMove, raw_bytes: &[u8]) -> DecodedMove {
    // Generate algebraic notation (simplified for now)
    let _from_algebraic = scid_move.from.to_algebraic();
    let _to_algebraic = scid_move.to.to_algebraic();

    // Convert to the existing MoveInterpretation format based on piece type
    let interpretation = match scid_move.moving_piece {
        crate::position::PieceType::Pawn => {
            let promotion = if scid_move.promote != crate::position::PieceType::Empty {
                Some(format!("{:?}", scid_move.promote))
            } else {
                None
            };

            MoveInterpretation::Pawn { 
                direction: "unknown".to_string(), 
                promotion,
                is_en_passant: None,
            }
        }
        crate::position::PieceType::King => {
            MoveInterpretation::King {
                direction_code: scid_move.piece_num,
                is_castle: false, // TODO: Detect castling
            }
        }
        crate::position::PieceType::Queen => MoveInterpretation::Queen,
        crate::position::PieceType::Rook => MoveInterpretation::Rook,
        crate::position::PieceType::Bishop => MoveInterpretation::Bishop,
        crate::position::PieceType::Knight => MoveInterpretation::Knight {
            l_shape_code: scid_move.piece_num,
        },
        _ => MoveInterpretation::Unknown {
            reason: format!("Unknown piece type: {:?}", scid_move.moving_piece),
        },
    };

    DecodedMove {
        piece_num: scid_move.piece_num,
        move_value: raw_bytes.first().map_or(0, |b| b & 0x0F), // Extract move value from raw byte
        raw_bytes: raw_bytes.to_vec(),
        interpretation,
        from_square_index: Some(scid_move.from.0),
        to_square_index: Some(scid_move.to.0),
        promotion_piece: None, // TODO
        piece_type: None, // Not available in this context
    }
}

/// Position-aware move decoder that integrates with existing game parsing
/// This replaces the static try_decode_move function with position-aware decoding
#[allow(dead_code)]
pub fn decode_move_with_position(
    position: &ScidPosition,
    raw_byte: u8,
    _offset: usize,
) -> Result<DecodedMove, String> {
    eprintln!("DEBUG: decode_move_with_position called with raw_byte: 0x{:02X}", raw_byte);
    // Use our position-aware decoder
    // Legacy decode_move removed; this will be refactored to SG4Parser.
    match Err::<ScidMove, String>("legacy decoder removed".to_string()) {
        Ok(scid_move) => {
            eprintln!("DEBUG: decode_move returned ScidMove with moving_piece: {:?}", scid_move.moving_piece);
            // Convert to existing format for compatibility
            Ok(scid_move_to_decoded_move(&scid_move, &[raw_byte]))
        }
        Err(e) => Err(format!("Position-aware decoding failed: {}", e)),
    }
}

/// Back-compat shim used by tests: tracks a position and processes moves from bytes/streams
#[allow(dead_code)]
pub struct PositionTracker {
    position: ScidPosition,
    move_count: usize,
}

#[allow(dead_code)]
impl PositionTracker {
    pub fn new() -> Self {
        Self {
            position: ScidPosition::new_starting_position(),
            move_count: 0,
        }
    }

    pub fn current_position(&self) -> &ScidPosition {
        &self.position
    }

    pub fn move_count(&self) -> usize {
        self.move_count
    }

    /// Process a move from a byte stream starting at current position
    pub fn process_move_from_stream(
        &mut self,
        stream: &mut ScidByteStream,
        _offset: usize,
    ) -> Result<DecodedMove, String> {
        let start = stream.position();
        let raw_byte = match stream.get_byte() {
            Ok(b) => b,
            Err(_) => return Err("No byte available in stream".to_string()),
        };
        let result: Result<DecodedMove, String> = Err::<DecodedMove, String>("legacy decoder removed".to_string());
        let bytes_consumed = stream.position().saturating_sub(start);
        match result {
            Ok(decoded) => {
                eprintln!("DEBUG: decoded DecodedMove: moving_piece {:?}", decoded.piece_type);
                // apply to internal position; ignore apply errors for now
                let _ = self
                    .position
                    .do_move(&self.last_scid_move_cache(decoded.clone()));
                self.move_count += 1;
                Ok(decoded)
            }
            Err(e) => Err(e),
        }
    }

    /// Process a single raw byte move (legacy API)
    pub fn process_move(&mut self, raw_byte: u8, _offset: usize) -> Result<DecodedMove, String> {
        let bytes = [raw_byte];
        let mut tmp = ScidByteStream::new(&bytes);
        self.process_move_from_stream(&mut tmp, 0)
    }

    fn scid_move_to_decoded(
        &self,
        scid_move: ScidMove,
        stream: &ScidByteStream,
        start: usize,
        _bytes_consumed: usize,
    ) -> DecodedMove {
        // Build a Decoded interpretation variant with useful info
        let from_sq = scid_move.from.to_algebraic();
        let to_sq = scid_move.to.to_algebraic();
        let piece_type = format!("{:?}", scid_move.moving_piece);
        let raw = stream.get_consumed_bytes(start);
        DecodedMove {
            piece_num: scid_move.piece_num,
            move_value: 0,
            raw_bytes: raw.to_vec(),
            interpretation: MoveInterpretation::Decoded {
                from_square: Some(from_sq),
                to_square: Some(to_sq),
                piece_type: Some(piece_type),
                is_capture: false,
                is_promotion: scid_move.promote != crate::position::PieceType::Empty,
            },
            from_square_index: Some(scid_move.from.0),
            to_square_index: Some(scid_move.to.0),
            promotion_piece: None,
            piece_type: None, // Not available in this context
        }
    }

    // Helper to rebuild a ScidMove from a DecodedMove-like info for applying to position
    fn last_scid_move_cache(&self, decoded: DecodedMove) -> ScidMove {
        // This is a best-effort reconstruction; for tests we only need move count to advance
        // Use decode_move on raw_byte as fallback
        let raw = decoded.raw_bytes.first().copied().unwrap_or(0);
        match Err::<ScidMove, String>("legacy decoder removed".to_string()) {
            Ok(m) => m,
            Err(_) => ScidMove {
                piece_num: decoded.piece_num,
                moving_piece: crate::position::PieceType::Pawn,
                from: crate::position::Square(0),
                to: crate::position::Square(0),
                promote: crate::position::PieceType::Empty,
                captured_piece: crate::position::PieceType::Empty,
            },
        }
    }
}

#[cfg(test)]
mod tests {}
