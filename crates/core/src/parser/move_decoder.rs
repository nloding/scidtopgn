//! High-Level SCID Move Decoder
//!
//! This module provides the main move decoder that coordinates:
//! - Position tracking (ScidPosition)
//! - Piece number lookup
//! - Move decoding (piece-specific)
//! - Move validation (shakmaty)
//! - Position updates
//!
//! # Move Byte Structure
//!
//! Each SCID move byte is structured as: `[piece_num:4][move_value:4]`
//! - Upper 4 bits: Piece number (0-15) identifying which piece moves
//! - Lower 4 bits: Move value (meaning depends on piece type)
//!
//! # Decoding Process
//!
//! 1. Extract piece number and move value from byte
//! 2. Look up piece's current square from SCID mapping
//! 3. Determine piece type from shakmaty position
//! 4. Decode move value to target square (piece-specific)
//! 5. Find legal shakmaty Move matching from/to/promotion
//! 6. Apply move and update position

use crate::error::{Result, ScidError};
use crate::parser::byte_stream::ByteStream;
use crate::parser::decoder::*;
use crate::parser::position::ScidPosition;
use shakmaty::{Move, Role};
use std::path::PathBuf;

/// High-level SCID move decoder
///
/// Coordinates position tracking, piece number lookup, move decoding,
/// and move validation via shakmaty.
pub struct ScidMoveDecoder {
    position: ScidPosition,
}

impl ScidMoveDecoder {
    /// Create decoder from standard starting position
    pub fn new() -> Self {
        ScidMoveDecoder {
            position: ScidPosition::new(),
        }
    }

    /// Create decoder from FEN position
    pub fn from_fen(fen: &str) -> Result<Self> {
        Ok(ScidMoveDecoder {
            position: ScidPosition::from_fen(fen)?,
        })
    }

    /// Get current position
    pub fn position(&self) -> &ScidPosition {
        &self.position
    }

    /// Get mutable position (for external manipulation)
    pub fn position_mut(&mut self) -> &mut ScidPosition {
        &mut self.position
    }

    /// Decode single SCID move byte into shakmaty Move
    ///
    /// # Process
    ///
    /// 1. Extract piece number and move value from byte
    /// 2. Look up piece's current square from SCID mapping
    /// 3. Determine piece type from shakmaty position
    /// 4. Decode move value to target square (piece-specific)
    /// 5. Find legal shakmaty Move matching from/to/promotion
    /// 6. Apply move and update position
    ///
    /// # Arguments
    ///
    /// * `move_byte` - The SCID move byte
    /// * `stream` - ByteStream for reading additional bytes (Queen diagonal moves)
    ///
    /// # Returns
    ///
    /// The validated shakmaty Move, or an error if decoding/validation fails
    pub fn decode_move(&mut self, move_byte: u8, stream: &mut ByteStream) -> Result<Move> {
        // Extract piece number and move value
        let piece_num = (move_byte >> 4) & 0x0F;
        let move_value = move_byte & 0x0F;

        // Get piece's current square from SCID mapping
        let from_square =
            self.position
                .get_piece_square(piece_num)
                .ok_or_else(|| ScidError::ParseError {
                    file: PathBuf::from("move_decoder"),
                    offset: 0,
                    message: format!(
                        "Invalid piece number: {} (move_byte: 0x{:02X})",
                        piece_num, move_byte
                    ),
                })?;

        // Get piece at that square from shakmaty
        let piece = self
            .position
            .piece_at(from_square)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!(
                    "No piece at square {:?} for piece_num {} (move_byte: 0x{:02X})",
                    from_square, piece_num, move_byte
                ),
            })?;

        // Decode move based on piece type
        let decoded = match piece.role {
            Role::King => decode_king_move(from_square, move_value, piece.color)?,
            Role::Queen => decode_queen_move(from_square, move_value, stream)?,
            Role::Rook => decode_rook_move(from_square, move_value)?,
            Role::Bishop => decode_bishop_move(from_square, move_value)?,
            Role::Knight => decode_knight_move(from_square, move_value)?,
            Role::Pawn => decode_pawn_move(from_square, move_value, piece.color)?,
        };

        // Find legal shakmaty Move matching our decoded from/to/promotion
        let chess_move = self
            .position
            .find_move(decoded.from, decoded.to, decoded.promotion)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!(
                    "No legal move from {:?} to {:?} (promotion: {:?}), \
                             piece: {:?}, move_byte: 0x{:02X}, piece_num: {}, move_value: {}",
                    decoded.from,
                    decoded.to,
                    decoded.promotion,
                    piece,
                    move_byte,
                    piece_num,
                    move_value
                ),
            })?;

        // Apply move to position (validates and updates state)
        self.position.make_move(&chess_move)?;

        Ok(chess_move)
    }

    /// Decode sequence of SCID move bytes
    ///
    /// Continues decoding until the stream is exhausted or an
    /// end-of-game marker is encountered.
    ///
    /// # Special Bytes
    ///
    /// - 0x00: End of game marker (stops decoding)
    /// - Other special markers are handled by the caller
    pub fn decode_moves(&mut self, move_bytes: &[u8]) -> Result<Vec<Move>> {
        let mut moves = Vec::new();
        let mut stream = ByteStream::new(move_bytes);

        while stream.has_more() {
            let move_byte = stream.get_byte()?;

            // Check for end-of-game marker
            if move_byte == 0x00 {
                break;
            }

            let chess_move = self.decode_move(move_byte, &mut stream)?;
            moves.push(chess_move);
        }

        Ok(moves)
    }

    /// Reset position to standard starting position
    pub fn reset(&mut self) {
        self.position = ScidPosition::new();
    }

    /// Reset position to FEN
    pub fn reset_to_fen(&mut self, fen: &str) -> Result<()> {
        self.position = ScidPosition::from_fen(fen)?;
        Ok(())
    }
}

impl Default for ScidMoveDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Color, Role, Square};

    #[test]
    fn test_decode_e4() {
        let mut decoder = ScidMoveDecoder::new();

        // SCID move byte for e2-e4 (pawn 12, double push value 15)
        // Piece 12 = e-pawn, move value 15 = double push
        let move_byte = (12 << 4) | 15; // 0xCF

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(move_byte, &mut stream).unwrap();

        // Verify move
        assert_eq!(chess_move.from(), Some(Square::E2));
        assert_eq!(chess_move.to(), Square::E4);

        // Verify position updated
        assert_eq!(decoder.position().turn(), Color::Black);
    }

    #[test]
    fn test_decode_nf3() {
        let mut decoder = ScidMoveDecoder::new();

        // First play e4
        let e4_byte = (12 << 4) | 15;
        let mut stream = ByteStream::new(&[]);
        decoder.decode_move(e4_byte, &mut stream).unwrap();

        // Black plays e5
        let e5_byte = (12 << 4) | 15; // Black's e-pawn double push
        decoder.decode_move(e5_byte, &mut stream).unwrap();

        // White plays Nf3 (Knight from g1 to f3)
        // Knight at g1 is piece number 7
        // Knight move: need to find correct move_value for g1-f3
        // From g1: file 6, rank 0
        // To f3: file 5, rank 2
        // Delta: (-1, +2) = move value 0 (2 up, 1 left)
        let nf3_byte = (7 << 4) | 0;

        let chess_move = decoder.decode_move(nf3_byte, &mut stream).unwrap();
        assert_eq!(chess_move.to(), Square::F3);
    }

    #[test]
    fn test_decode_castling() {
        // Set up position where castling is legal
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Kingside castle (king piece 0, move value 10)
        let castle_byte = (0 << 4) | 10;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        // Verify castling move
        match &chess_move {
            Move::Castle { king, rook } => {
                assert_eq!(*king, Square::E1);
                assert_eq!(*rook, Square::H1);
            }
            _ => panic!("Expected castling move, got {:?}", chess_move),
        }
    }

    #[test]
    fn test_decode_queenside_castle() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Queenside castle (king piece 0, move value 11)
        let castle_byte = (0 << 4) | 11;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        match &chess_move {
            Move::Castle { king, rook } => {
                assert_eq!(*king, Square::E1);
                assert_eq!(*rook, Square::A1);
            }
            _ => panic!("Expected castling move"),
        }
    }

    #[test]
    fn test_decode_pawn_promotion() {
        // Set up position with pawn ready to promote
        let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // The e7 pawn should be piece 0 (first pawn found after king)
        // In from_position, pieces are assigned by Role order
        // King first (piece 0), then Pawn (piece 1)
        // So e7 pawn is piece 1

        // Queen promotion forward = move value 4
        let promo_byte = (1 << 4) | 4;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(promo_byte, &mut stream).unwrap();

        assert_eq!(chess_move.to(), Square::E8);
        assert_eq!(chess_move.promotion(), Some(Role::Queen));
    }

    #[test]
    fn test_decode_moves_sequence() {
        let mut decoder = ScidMoveDecoder::new();

        // Encode: e4 e5 Nf3
        let moves = vec![
            (12 << 4) | 15, // e4: pawn 12, double push (15)
            (12 << 4) | 15, // e5: pawn 12, double push (15)
            (7 << 4) | 0,   // Nf3: knight 7, move value 0
        ];

        let decoded = decoder.decode_moves(&moves).unwrap();

        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[0].to(), Square::E4);
        assert_eq!(decoded[1].to(), Square::E5);
        assert_eq!(decoded[2].to(), Square::F3);
    }

    #[test]
    fn test_decode_with_end_marker() {
        let mut decoder = ScidMoveDecoder::new();

        // e4 followed by end marker
        let moves = vec![
            (12 << 4) | 15, // e4
            0x00,           // end marker
            (12 << 4) | 15, // this shouldn't be decoded
        ];

        let decoded = decoder.decode_moves(&moves).unwrap();

        // Should only have 1 move (stopped at end marker)
        assert_eq!(decoded.len(), 1);
    }

    #[test]
    fn test_invalid_piece_number() {
        // After a capture, piece number might become invalid
        // But at start, all 16 piece numbers are valid
        // Let's test with a position that has fewer pieces

        let fen = "4k3/8/8/8/8/8/8/4K3 w - - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        // Try to move piece 5 (doesn't exist - only king exists)
        let bad_byte = (5 << 4) | 1;

        let mut stream = ByteStream::new(&[]);
        let result = decoder.decode_move(bad_byte, &mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_reset() {
        let mut decoder = ScidMoveDecoder::new();

        // Play e4
        let e4_byte = (12 << 4) | 15;
        let mut stream = ByteStream::new(&[]);
        decoder.decode_move(e4_byte, &mut stream).unwrap();

        // Verify it's Black's turn
        assert_eq!(decoder.position().turn(), Color::Black);

        // Reset
        decoder.reset();

        // Verify it's White's turn again
        assert_eq!(decoder.position().turn(), Color::White);
    }
}
