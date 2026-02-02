use crate::error::{Result, ScidError};
use crate::parser::byte_stream::ByteStream;
use crate::parser::decoder::*;
use crate::parser::position::ScidPosition;
use shakmaty::{Move, Role};
use std::path::PathBuf;

/// High-level SCID move decoder
///
/// Coordinates:
/// - Position tracking (ScidPosition)
/// - Piece number lookup
/// - Move decoding (piece-specific)
/// - Move validation (shakmaty)
/// - Position updates
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

    /// Decode single SCID move byte into shakmaty Move
    ///
    /// Process:
    /// 1. Extract piece number and move value from byte
    /// 2. Look up piece's current square
    /// 3. Determine piece type from position
    /// 4. Decode move value to target square (piece-specific)
    /// 5. Find legal shakmaty Move matching from/to/promotion
    /// 6. Apply move and update position
    pub fn decode_move(&mut self, move_byte: u8, stream: &mut ByteStream) -> Result<Move> {
        let piece_num = (move_byte >> 4) & 0x0F;
        let move_value = move_byte & 0x0F;

        let from_square =
            self.position
                .get_piece_square(piece_num)
                .ok_or_else(|| ScidError::ParseError {
                    file: PathBuf::from("move_decoder"),
                    offset: 0,
                    message: format!("Invalid piece number: {}", piece_num),
                })?;

        let piece = self
            .position
            .piece_at(from_square)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!("No piece at square {:?}", from_square),
            })?;

        let decoded = match piece.role {
            Role::King => decode_king_move(from_square, move_value, piece.color)?,
            Role::Queen => decode_queen_move(from_square, move_value, stream)?,
            Role::Rook => decode_rook_move(from_square, move_value)?,
            Role::Bishop => decode_bishop_move(from_square, move_value)?,
            Role::Knight => decode_knight_move(from_square, move_value)?,
            Role::Pawn => decode_pawn_move(from_square, move_value, piece.color)?,
        };

        let chess_move = self
            .position
            .find_move(decoded.from, decoded.to, decoded.promotion)
            .ok_or_else(|| ScidError::ParseError {
                file: PathBuf::from("move_decoder"),
                offset: 0,
                message: format!(
                    "No legal move from {:?} to {:?} (promotion: {:?})",
                    decoded.from, decoded.to, decoded.promotion
                ),
            })?;

        self.position.make_move(&chess_move)?;

        Ok(chess_move)
    }

    /// Decode sequence of SCID move bytes
    pub fn decode_moves(&mut self, move_bytes: &[u8]) -> Result<Vec<Move>> {
        let mut moves = Vec::new();
        let mut stream = ByteStream::new(move_bytes);

        while stream.has_more() {
            let move_byte = stream.get_byte()?;
            let chess_move = self.decode_move(move_byte, &mut stream)?;
            moves.push(chess_move);
        }

        Ok(moves)
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
    use shakmaty::{Color, Square};

    // ===== KING MOVE TESTS =====

    #[test]
    fn test_king_move_one_square() {
        let from = Square::E1;
        let move_value = 0x01;
        let decoded =
            decode_king_move(from, move_value, Color::White).expect("Should decode king move");

        assert_eq!(decoded.from, from);
        assert_eq!(decoded.to, Square::E2);
        assert_eq!(decoded.piece_type, PieceType::King);
    }

    #[test]
    fn test_king_castling_kingside_white() {
        let from = Square::E1;
        let move_value = 0x0E;

        let decoded = decode_king_move(from, move_value, Color::White)
            .expect("Should decode kingside castle");

        assert_eq!(decoded.from, Square::E1);
        assert_eq!(decoded.to, Square::G1);
        assert!(decoded.is_castling);
    }

    #[test]
    fn test_king_castling_queenside_white() {
        let from = Square::E1;
        let move_value = 0x0F;

        let decoded = decode_king_move(from, move_value, Color::White)
            .expect("Should decode queenside castle");

        assert_eq!(decoded.from, Square::E1);
        assert_eq!(decoded.to, Square::C1);
        assert!(decoded.is_castling);
    }

    #[test]
    fn test_king_castling_black() {
        let from = Square::E8;
        let move_value = 0x0E;

        let decoded =
            decode_king_move(from, move_value, Color::Black).expect("Should decode black castle");

        assert_eq!(decoded.from, Square::E8);
        assert_eq!(decoded.to, Square::G8);
    }

    // ===== QUEEN MOVE TESTS =====

    #[test]
    fn test_queen_move_horizontal() {
        let from = Square::D1;
        let move_value = 0x04;
        let mut stream = ByteStream::new(&[]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode horizontal queen move");

        assert_eq!(decoded.from, Square::D1);
        assert_eq!(decoded.to, Square::H1);
    }

    #[test]
    fn test_queen_move_vertical() {
        let from = Square::D1;
        let move_value = 0x17;
        let mut stream = ByteStream::new(&[]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode vertical queen move");

        assert_eq!(decoded.from, Square::D1);
        assert_eq!(decoded.to, Square::D8);
    }

    #[test]
    fn test_queen_move_diagonal_short() {
        let from = Square::D4;
        let move_value = 0x20;
        let mut stream = ByteStream::new(&[]);

        let decoded =
            decode_queen_move(from, move_value, &mut stream).expect("Should decode short diagonal");

        assert_eq!(decoded.from, Square::D4);
    }

    #[test]
    fn test_queen_move_diagonal_long() {
        let from = Square::A1;
        let move_value = 0xFF;

        let second_byte = 0x07;
        let mut stream = ByteStream::new(&[second_byte]);

        let decoded = decode_queen_move(from, move_value, &mut stream)
            .expect("Should decode long diagonal from stream");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H8);
    }

    // ===== ROOK MOVE TESTS =====

    #[test]
    fn test_rook_move_horizontal() {
        let from = Square::A1;
        let move_value = 0x07;

        let decoded = decode_rook_move(from, move_value).expect("Should decode rook move");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H1);
    }

    #[test]
    fn test_rook_move_vertical() {
        let from = Square::A1;
        let move_value = 0x17;

        let decoded = decode_rook_move(from, move_value).expect("Should decode vertical rook move");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::A8);
    }

    // ===== BISHOP MOVE TESTS =====

    #[test]
    fn test_bishop_move_diagonal() {
        let from = Square::C1;
        let move_value = 0x03;

        let decoded = decode_bishop_move(from, move_value).expect("Should decode bishop move");

        assert_eq!(decoded.from, Square::C1);
        assert_eq!(decoded.to, Square::F4);
    }

    #[test]
    fn test_bishop_move_long_diagonal() {
        let from = Square::A1;
        let move_value = 0x07;

        let decoded = decode_bishop_move(from, move_value).expect("Should decode long diagonal");

        assert_eq!(decoded.from, Square::A1);
        assert_eq!(decoded.to, Square::H8);
    }

    // ===== KNIGHT MOVE TESTS =====

    #[test]
    fn test_knight_all_possible_moves() {
        let from = Square::D4;

        let expected_targets = vec![
            (0x00, Square::E6),
            (0x01, Square::F5),
            (0x02, Square::F3),
            (0x03, Square::E2),
            (0x04, Square::C2),
            (0x05, Square::B3),
            (0x06, Square::B5),
            (0x07, Square::C6),
        ];

        for (move_value, expected_to) in expected_targets {
            let decoded = decode_knight_move(from, move_value)
                .expect(&format!("Should decode knight move {}", move_value));

            assert_eq!(decoded.from, from);
            assert_eq!(
                decoded.to, expected_to,
                "Knight move {} should go to {}",
                move_value, expected_to
            );
        }
    }

    #[test]
    fn test_knight_edge_squares() {
        let from = Square::A1;

        let decoded = decode_knight_move(from, 0x00).expect("Should decode from corner");

        assert_eq!(decoded.from, Square::A1);
        assert!(
            decoded.to == Square::B3 || decoded.to == Square::C2,
            "Should be legal knight move from corner"
        );
    }

    // ===== PAWN MOVE TESTS =====

    #[test]
    fn test_pawn_move_one_square() {
        let from = Square::E2;
        let move_value = 0x01;

        let decoded =
            decode_pawn_move(from, move_value, Color::White).expect("Should decode pawn move");

        assert_eq!(decoded.from, Square::E2);
        assert_eq!(decoded.to, Square::E3);
        assert!(!decoded.is_capture);
        assert_eq!(decoded.promotion, None);
    }

    #[test]
    fn test_pawn_move_two_squares() {
        let from = Square::E2;
        let move_value = 0x02;

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode two-square pawn move");

        assert_eq!(decoded.from, Square::E2);
        assert_eq!(decoded.to, Square::E4);
    }

    #[test]
    fn test_pawn_capture_left() {
        let from = Square::E4;
        let move_value = 0x11;

        let decoded =
            decode_pawn_move(from, move_value, Color::White).expect("Should decode pawn capture");

        assert_eq!(decoded.from, Square::E4);
        assert_eq!(decoded.to, Square::D5);
        assert!(decoded.is_capture);
    }

    #[test]
    fn test_pawn_capture_right() {
        let from = Square::E4;
        let move_value = 0x12;

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode pawn capture right");

        assert_eq!(decoded.from, Square::E4);
        assert_eq!(decoded.to, Square::F5);
        assert!(decoded.is_capture);
    }

    #[test]
    fn test_pawn_promotion_queen() {
        let from = Square::E7;
        let move_value = 0x81;

        let decoded =
            decode_pawn_move(from, move_value, Color::White).expect("Should decode promotion");

        assert_eq!(decoded.from, Square::E7);
        assert_eq!(decoded.to, Square::E8);
        assert_eq!(decoded.promotion, Some(PieceType::Queen));
    }

    #[test]
    fn test_pawn_promotion_knight() {
        let from = Square::E7;
        let move_value = 0x84;

        let decoded = decode_pawn_move(from, move_value, Color::White)
            .expect("Should decode knight promotion");

        assert_eq!(decoded.promotion, Some(PieceType::Knight));
    }

    #[test]
    fn test_pawn_en_passant() {
        let from = Square::E5;
        let move_value = 0xE1;

        let decoded =
            decode_pawn_move(from, move_value, Color::White).expect("Should decode en passant");

        assert_eq!(decoded.from, Square::E5);
        assert_eq!(decoded.to, Square::D6);
        assert!(decoded.is_capture);
        assert!(decoded.is_en_passant);
    }

    #[test]
    fn test_pawn_black_moves() {
        let from = Square::E7;
        let move_value = 0x01;

        let decoded = decode_pawn_move(from, move_value, Color::Black)
            .expect("Should decode black pawn move");

        assert_eq!(decoded.from, Square::E7);
        assert_eq!(decoded.to, Square::E6);
    }

    // ===== SCID POSITION INTEGRATION TESTS =====

    #[test]
    fn test_scid_position_initial() {
        let pos = ScidPosition::new();

        assert_eq!(pos.side_to_move(), Color::White);
        assert!(pos.can_castle_kingside(Color::White));
        assert!(pos.can_castle_queenside(Color::White));

        assert_eq!(pos.piece_count(), 32);
    }

    #[test]
    fn test_scid_position_piece_numbers() {
        let pos = ScidPosition::new();

        assert_eq!(pos.piece_square(0), Some(Square::E1));
        assert_eq!(pos.piece_square(1), Some(Square::D1));
        assert_eq!(pos.piece_square(12), Some(Square::E2));
        assert_eq!(pos.piece_square(16), Some(Square::E8));
    }

    #[test]
    fn test_scid_position_make_move() {
        let mut pos = ScidPosition::new();

        pos.make_move(12, 0x02).expect("Should make legal move");

        assert_eq!(pos.piece_square(12), Some(Square::E4));
        assert_eq!(pos.side_to_move(), Color::Black);
    }

    #[test]
    fn test_scid_position_capture_updates_pieces() {
        let mut pos = ScidPosition::new();

        pos.make_move(13, 0x01).unwrap();
        pos.make_move(28, 0x02).unwrap();
        pos.make_move(14, 0x02).unwrap();

        pos.make_move(17, 0x40).unwrap();

        assert_eq!(pos.piece_square(14), None);
    }

    #[test]
    fn test_scid_position_castling() {
        let mut pos = ScidPosition::new();

        pos.make_move(12, 0x02).unwrap();
        pos.make_move(28, 0x02).unwrap();
        pos.make_move(6, 0x01).unwrap();
        pos.make_move(22, 0x01).unwrap();
        pos.make_move(5, 0x02).unwrap();
        pos.make_move(21, 0x02).unwrap();

        pos.make_move(0, 0x0E).unwrap();

        assert_eq!(pos.piece_square(0), Some(Square::G1));
        assert_eq!(pos.piece_square(7), Some(Square::F1));
    }

    #[test]
    fn test_scid_position_promotion() {
        let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
        let mut pos = ScidPosition::from_fen(fen).expect("Should parse FEN");

        pos.make_move(12, 0x81).unwrap();

        assert_eq!(pos.piece_square(12), Some(Square::E8));
        assert_eq!(pos.piece_type(12), Some(PieceType::Queen));
    }

    #[test]
    fn test_scid_position_checkmate_detection() {
        let mut pos = ScidPosition::new();

        pos.make_move(13, 0x01).unwrap();
        pos.make_move(28, 0x02).unwrap();
        pos.make_move(14, 0x02).unwrap();
        pos.make_move(17, 0x40).unwrap();

        assert!(pos.is_checkmate());
        assert!(!pos.is_stalemate());
    }

    #[test]
    fn test_scid_position_stalemate_detection() {
        let fen = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1";
        let pos = ScidPosition::from_fen(fen).expect("Should parse FEN");

        assert!(pos.is_stalemate());
        assert!(!pos.is_checkmate());
    }

    // ===== DECODER HIGH-LEVEL TESTS =====

    #[test]
    fn test_decode_e4() {
        let mut decoder = ScidMoveDecoder::new();

        let move_byte = (12 << 4) | 15;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(move_byte, &mut stream).unwrap();

        assert_eq!(chess_move.from(), Some(Square::E2));
        assert_eq!(chess_move.to(), Square::E4);

        assert_eq!(decoder.position().turn(), Color::Black);
    }

    #[test]
    fn test_decode_knight_f3() {
        let mut decoder = ScidMoveDecoder::new();

        let e4_byte = (12 << 4) | 15;
        let mut stream = ByteStream::new(&[]);
        decoder.decode_move(e4_byte, &mut stream).unwrap();

        let e5_byte = (12 << 4) | 15;
        decoder.decode_move(e5_byte, &mut stream).unwrap();

        let nf3_byte = (7 << 4) | 6;

        let chess_move = decoder.decode_move(nf3_byte, &mut stream).unwrap();
        assert_eq!(chess_move.to(), Square::F3);
    }

    #[test]
    fn test_decode_queen_diagonal() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        let first_byte = (1 << 4) | 3;

        let data = vec![first_byte, 103];
        let mut stream = ByteStream::new(&data);

        let chess_move = decoder.decode_move(first_byte, &mut stream).unwrap();
        assert_eq!(chess_move.to(), Square::H5);
    }

    #[test]
    fn test_decode_castling() {
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        let castle_byte = (0 << 4) | 10;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        assert_eq!(chess_move.from(), Some(Square::E1));
        assert_eq!(chess_move.to(), Square::G1);
    }

    #[test]
    fn test_decode_queenside_castling() {
        let fen = "r3kbnr/pppqpppp/2n5/3p1b2/3P1B2/2N5/PPPQPPPP/R3KBNR w KQkq - 6 5";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        let castle_byte = (0 << 4) | 9;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(castle_byte, &mut stream).unwrap();

        assert_eq!(chess_move.from(), Some(Square::E1));
        assert_eq!(chess_move.to(), Square::C1);
    }

    #[test]
    fn test_decode_pawn_promotion() {
        let fen = "8/4P3/8/8/8/8/8/4K2k w - - 0 1";
        let mut decoder = ScidMoveDecoder::from_fen(fen).unwrap();

        let promo_byte = (8 << 4) | 4;

        let mut stream = ByteStream::new(&[]);
        let chess_move = decoder.decode_move(promo_byte, &mut stream).unwrap();

        assert_eq!(chess_move.to(), Square::E8);
    }

    #[test]
    fn test_decode_moves_sequence() {
        let mut decoder = ScidMoveDecoder::new();

        let moves_bytes = vec![(12 << 4) | 15, (12 << 4) | 15, (7 << 4) | 6, (7 << 4) | 7];

        let moves = decoder.decode_moves(&moves_bytes).unwrap();

        assert_eq!(moves.len(), 4);
        assert_eq!(moves[0].from(), Some(Square::E2));
        assert_eq!(moves[0].to(), Square::E4);
    }

    #[test]
    fn test_default_decoder() {
        let decoder = ScidMoveDecoder::default();
        assert_eq!(decoder.position().turn(), Color::White);
    }
}
