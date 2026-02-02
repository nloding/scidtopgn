use crate::error::{Result, ScidError};
use shakmaty::{Board, CastlingMode, Chess, Color, EnPassantMode, Move, Position, Role, Square};
use std::collections::HashMap;

/// Detect if a FEN string represents a Chess960 position
///
/// Chess960 FENs use file letters for castling rights (e.g., "AHah")
/// when the king or rooks don't start on standard squares.
/// Standard chess uses "KQkq" notation.
///
/// Detection rules:
/// - If castling field contains any lowercase letter a-h: Chess960
/// - If castling field is "-" or contains only K/Q/k/q: Standard
///
/// # Arguments
///
/// * `fen` - Complete FEN string
///
/// # Returns
///
/// `true` if this appears to be a Chess960 position
///
/// # Examples
///
/// ```
/// // Standard chess
/// assert!(!is_chess960_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
///
/// // Chess960 with rook on a-file, king on c-file
/// assert!(is_chess960_fen("rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w ACac - 0 1"));
/// ```
pub fn is_chess960_fen(fen: &str) -> bool {
    let parts: Vec<&str> = fen.split_whitespace().collect();

    if parts.len() < 3 {
        return false;
    }

    let castling_field = parts[2];

    for c in castling_field.chars() {
        match c {
            'K' | 'Q' | 'k' | 'q' | '-' => continue,
            'A'..='H' | 'a'..='h' => return true,
            _ => continue,
        }
    }

    false
}

/// SCID piece number to square mapping
/// Maintains bidirectional mapping between SCID piece numbers and board squares
/// CRITICAL: Each color has its own independent 0-15 piece list
#[derive(Debug, Clone)]
pub struct PieceNumberMapping {
    white_pieces: HashMap<u8, Square>,
    black_pieces: HashMap<u8, Square>,
    white_count: u8,
    black_count: u8,
}

impl PieceNumberMapping {
    pub fn standard_start() -> Self {
        let mut mapping = Self {
            white_pieces: HashMap::with_capacity(16),
            black_pieces: HashMap::with_capacity(16),
            white_count: 16,
            black_count: 16,
        };

        mapping.white_pieces.insert(0, Square::E1);
        mapping.white_pieces.insert(1, Square::A1);
        mapping.white_pieces.insert(2, Square::B1);
        mapping.white_pieces.insert(3, Square::C1);
        mapping.white_pieces.insert(4, Square::D1);
        mapping.white_pieces.insert(5, Square::F1);
        mapping.white_pieces.insert(6, Square::G1);
        mapping.white_pieces.insert(7, Square::H1);

        for file in 0..8 {
            mapping.white_pieces.insert(
                8 + file,
                Square::from_coords(shakmaty::File::new(file as u32), shakmaty::Rank::Second),
            );
        }

        mapping.black_pieces.insert(0, Square::E8);
        mapping.black_pieces.insert(1, Square::A8);
        mapping.black_pieces.insert(2, Square::B8);
        mapping.black_pieces.insert(3, Square::C8);
        mapping.black_pieces.insert(4, Square::D8);
        mapping.black_pieces.insert(5, Square::F8);
        mapping.black_pieces.insert(6, Square::G8);
        mapping.black_pieces.insert(7, Square::H8);

        for file in 0..8 {
            mapping.black_pieces.insert(
                8 + file,
                Square::from_coords(shakmaty::File::new(file as u32), shakmaty::Rank::Seventh),
            );
        }

        mapping
    }

    pub fn from_position(chess: &Chess) -> Result<Self> {
        let mut mapping = Self {
            white_pieces: HashMap::new(),
            black_pieces: HashMap::new(),
            white_count: 0,
            black_count: 0,
        };

        for square in chess.board().by_role(shakmaty::Role::King) {
            if let Some(piece) = chess.board().piece_at(square) {
                if piece.color == Color::White {
                    mapping.white_pieces.insert(0, square);
                } else {
                    mapping.black_pieces.insert(0, square);
                }
            }
        }

        let mut white_num = 1u8;
        let mut black_num = 1u8;

        for rank in (0..8u32).rev() {
            for file in 0..8u32 {
                let square =
                    Square::from_coords(shakmaty::File::new(file), shakmaty::Rank::new(rank));

                if let Some(piece) = chess.board().piece_at(square) {
                    if piece.role == shakmaty::Role::King {
                        continue;
                    }

                    if piece.color == Color::White {
                        mapping.white_pieces.insert(white_num, square);
                        white_num += 1;
                    } else {
                        mapping.black_pieces.insert(black_num, square);
                        black_num += 1;
                    }
                }
            }
        }

        mapping.white_count = white_num;
        mapping.black_count = black_num;

        Ok(mapping)
    }

    pub fn get_square(&self, piece_num: u8, color: Color) -> Option<Square> {
        match color {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    pub fn update_after_move(&mut self, chess_move: &Move, color: Color) {
        if chess_move.is_capture() {
            let capture_square = if chess_move.is_en_passant() {
                let target = chess_move.to();
                let ep_rank = match color {
                    Color::White => shakmaty::Rank::Fifth,
                    Color::Black => shakmaty::Rank::Fourth,
                };
                Square::from_coords(target.file(), ep_rank)
            } else {
                chess_move.to()
            };

            let (enemy_pieces, enemy_count) = match color {
                Color::White => (&mut self.black_pieces, &mut self.black_count),
                Color::Black => (&mut self.white_pieces, &mut self.white_count),
            };

            let captured_slot = enemy_pieces
                .iter()
                .find(|(_, &sq)| sq == capture_square)
                .map(|(&num, _)| num);

            if let Some(captured_num) = captured_slot {
                *enemy_count -= 1;
                let last_slot = *enemy_count;

                if captured_num != last_slot {
                    if let Some(&last_square) = enemy_pieces.get(&last_slot) {
                        enemy_pieces.insert(captured_num, last_square);
                    }
                }

                enemy_pieces.remove(&last_slot);
            }
        }

        let own_pieces = match color {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces,
        };

        match chess_move {
            Move::Normal { from, to, .. } => {
                let piece_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
            }
            Move::Castle { king, rook } => {
                let king_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::G, king.rank())
                } else {
                    Square::from_coords(shakmaty::File::C, king.rank())
                };

                let rook_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::F, king.rank())
                } else {
                    Square::from_coords(shakmaty::File::D, king.rank())
                };

                own_pieces.insert(0, king_to);

                let rook_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *rook)
                    .map(|(&num, _)| num);

                if let Some(num) = rook_num {
                    own_pieces.insert(num, rook_to);
                }
            }
            Move::EnPassant { from, to } => {
                let piece_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
            }
            Move::Put { .. } => {}
        }
    }
}

#[test]
fn test_standard_piece_numbering() {
    let mapping = PieceNumberMapping::standard_start();

    assert_eq!(mapping.get_square(0, Color::White), Some(Square::E1));
    assert_eq!(mapping.get_square(1, Color::White), Some(Square::A1));
    assert_eq!(mapping.get_square(2, Color::White), Some(Square::B1));
    assert_eq!(mapping.get_square(3, Color::White), Some(Square::C1));
    assert_eq!(mapping.get_square(4, Color::White), Some(Square::D1));
    assert_eq!(mapping.get_square(5, Color::White), Some(Square::F1));
    assert_eq!(mapping.get_square(6, Color::White), Some(Square::G1));
    assert_eq!(mapping.get_square(7, Color::White), Some(Square::H1));
    assert_eq!(mapping.get_square(8, Color::White), Some(Square::A2));
    assert_eq!(mapping.get_square(15, Color::White), Some(Square::H2));

    assert_eq!(mapping.get_square(0, Color::Black), Some(Square::E8));
    assert_eq!(mapping.get_square(1, Color::Black), Some(Square::A8));
    assert_eq!(mapping.get_square(4, Color::Black), Some(Square::D8));
}

#[test]
fn test_piece_mapping_after_move() {
    let mut mapping = PieceNumberMapping::standard_start();

    let chess_move = Move::Normal {
        role: shakmaty::Role::Pawn,
        from: Square::E2,
        capture: None,
        to: Square::E4,
        promotion: None,
    };

    mapping.update_after_move(&chess_move, Color::White);

    assert_eq!(mapping.get_square(12, Color::White), Some(Square::E4));
}

/// Wrapper around shakmaty::Chess that tracks SCID piece numbers
///
/// This type combines shakmaty's complete chess engine with SCID's
/// piece numbering system. Shakmaty handles all chess rules while
/// we maintain the SCID piece number to square mapping.
#[derive(Debug, Clone)]
pub struct ScidPosition {
    /// Shakmaty chess position (handles ALL chess logic)
    chess: Chess,

    /// Maps SCID piece numbers to current squares
    piece_mapping: PieceNumberMapping,
}

impl ScidPosition {
    /// Create position from standard starting position
    pub fn new() -> Self {
        ScidPosition {
            chess: Chess::default(),
            piece_mapping: PieceNumberMapping::standard_start(),
        }
    }

    /// Create position from FEN string
    pub fn from_fen(fen: &str) -> Result<Self> {
        Self::from_fen_auto(fen)
    }

    /// Create position from FEN with automatic Chess960 detection
    ///
    /// Examines the FEN's castling rights field to determine whether
    /// to use standard or Chess960 mode. This ensures correct castling
    /// validation for both game types.
    ///
    /// # Arguments
    ///
    /// * `fen` - FEN string (standard or Chess960)
    ///
    /// # Returns
    ///
    /// ScidPosition configured for the correct game variant
    ///
    /// # Example
    ///
    /// ```
    /// // Standard position - uses CastlingMode::Standard
    /// let pos = ScidPosition::from_fen_auto(
    ///     "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
    /// )?;
    ///
    /// // Chess960 position - uses CastlingMode::Chess960
    /// let pos = ScidPosition::from_fen_auto(
    ///     "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w ACac - 0 1"
    /// )?;
    /// ```
    pub fn from_fen_auto(fen: &str) -> Result<Self> {
        let is_960 = is_chess960_fen(fen);

        let castling_mode = if is_960 {
            CastlingMode::Chess960
        } else {
            CastlingMode::Standard
        };

        let parsed_fen: shakmaty::fen::Fen = fen.parse().map_err(|e| ScidError::ParseError {
            file: "position".into(),
            offset: 0,
            message: format!("Invalid FEN: {:?}", e),
        })?;

        let chess = parsed_fen
            .into_position(castling_mode)
            .map_err(|e| ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Invalid position: {:?}", e),
            })?;

        let piece_mapping = PieceNumberMapping::from_position(&chess)?;

        Ok(ScidPosition {
            chess,
            piece_mapping,
        })
    }

    /// Get square where SCID piece number is currently located
    pub fn get_piece_square(&self, piece_num: u8) -> Option<Square> {
        self.piece_mapping.get_square(piece_num, self.chess.turn())
    }

    /// Get piece at a specific square (delegates to shakmaty)
    pub fn piece_at(&self, square: Square) -> Option<shakmaty::Piece> {
        self.chess.board().piece_at(square)
    }

    /// Get whose turn it is (delegates to shakmaty)
    pub fn turn(&self) -> Color {
        self.chess.turn()
    }

    /// Get the board state (delegates to shakmaty)
    pub fn board(&self) -> &Board {
        self.chess.board()
    }

    /// Get all legal moves (delegates to shakmaty)
    pub fn legal_moves(&self) -> Vec<Move> {
        self.chess.legal_moves().to_vec()
    }

    /// Check if a move is legal (delegates to shakmaty)
    pub fn is_legal(&self, chess_move: &Move) -> bool {
        self.chess.is_legal(chess_move)
    }

    /// Apply a move and return new position
    ///
    /// This is the critical method that:
    /// 1. Validates move legality via shakmaty
    /// 2. Applies move to chess position
    /// 3. Updates SCID piece number mapping
    pub fn make_move(&mut self, chess_move: &Move) -> Result<()> {
        if !self.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: "position".into(),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        let new_position =
            self.chess
                .clone()
                .play(chess_move)
                .map_err(|e| ScidError::ParseError {
                    file: "position".into(),
                    offset: 0,
                    message: format!("Failed to apply move: {:?}", e),
                })?;

        self.piece_mapping
            .update_after_move(chess_move, self.chess.turn());

        self.chess = new_position;

        Ok(())
    }

    /// Find legal move matching from/to squares and optional promotion
    ///
    /// Used to convert SCID decoded moves (from/to squares) into
    /// shakmaty Move objects
    pub fn find_move(&self, from: Square, to: Square, promotion: Option<Role>) -> Option<Move> {
        let legals = self.legal_moves();

        legals
            .into_iter()
            .find(|m| m.from() == Some(from) && m.to() == to && m.promotion() == promotion)
    }

    /// Get castling rights (delegates to shakmaty)
    pub fn castling_rights(&self) -> shakmaty::Bitboard {
        self.chess.castles().castling_rights()
    }

    /// Get en passant target square if available (delegates to shakmaty)
    pub fn ep_square(&self) -> Option<Square> {
        self.chess.ep_square(EnPassantMode::Legal)
    }

    /// Check if position is checkmate (delegates to shakmaty)
    pub fn is_checkmate(&self) -> bool {
        self.chess.is_checkmate()
    }

    /// Check if position is stalemate (delegates to shakmaty)
    pub fn is_stalemate(&self) -> bool {
        self.chess.is_stalemate()
    }

    /// Check if position is in check (delegates to shakmaty)
    pub fn is_check(&self) -> bool {
        self.chess.is_check()
    }
}

impl Default for ScidPosition {
    fn default() -> Self {
        Self::new()
    }
}

#[test]
fn test_scid_position_creation() {
    let pos = ScidPosition::new();

    assert_eq!(pos.turn(), Color::White);
    assert!(!pos.is_check());
    assert!(!pos.is_checkmate());

    let piece = pos.piece_at(Square::E2).unwrap();
    assert_eq!(piece.role, Role::Pawn);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_piece_number_lookup() {
    let pos = ScidPosition::new();

    assert_eq!(pos.get_piece_square(0), Some(Square::E1));
    assert_eq!(pos.get_piece_square(1), Some(Square::A1));
    assert_eq!(pos.get_piece_square(4), Some(Square::D1));
    assert_eq!(pos.get_piece_square(12), Some(Square::E2));
}

#[test]
fn test_make_move() {
    let mut pos = ScidPosition::new();

    let e4_move = pos
        .find_move(Square::E2, Square::E4, None)
        .expect("e2-e4 should be legal");

    pos.make_move(&e4_move).unwrap();

    assert_eq!(pos.turn(), Color::Black);
    assert!(pos.piece_at(Square::E2).is_none());
    assert!(pos.piece_at(Square::E4).is_some());

    assert_eq!(pos.get_piece_square(12), Some(Square::E4));
}

#[test]
fn test_from_fen() {
    let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    let pos = ScidPosition::from_fen(fen).unwrap();

    assert_eq!(pos.turn(), Color::Black);
    assert_eq!(pos.ep_square(), Some(Square::E3));

    let piece = pos.piece_at(Square::E4).unwrap();
    assert_eq!(piece.role, Role::Pawn);
    assert_eq!(piece.color, Color::White);
}

#[test]
fn test_illegal_move_rejected() {
    let mut pos = ScidPosition::new();

    let illegal_move = Move::Normal {
        role: Role::Pawn,
        from: Square::E2,
        capture: None,
        to: Square::E5,
        promotion: None,
    };

    assert!(!pos.is_legal(&illegal_move));
}

#[cfg(test)]
mod chess960_tests {
    use super::*;

    #[test]
    fn test_detect_standard_fen() {
        assert!(!is_chess960_fen(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ));

        assert!(!is_chess960_fen(
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 4 4"
        ));

        assert!(!is_chess960_fen("8/8/8/8/8/8/8/4K2k w - - 0 1"));
    }

    #[test]
    fn test_detect_chess960_fen() {
        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w AHah - 0 1"
        ));

        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w Hh - 0 1"
        ));

        assert!(is_chess960_fen(
            "rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w KQah - 0 1"
        ));
    }

    #[test]
    fn test_from_fen_auto_standard() {
        let pos =
            ScidPosition::from_fen_auto("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
                .unwrap();

        assert_eq!(pos.turn(), Color::White);
        assert_eq!(pos.get_piece_square(0), Some(Square::E1));
    }

    #[test]
    fn test_from_fen_auto_chess960() {
        let pos =
            ScidPosition::from_fen_auto("rnbkqbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKQBNR w HAha - 0 1");

        assert!(pos.is_ok());
    }

    #[test]
    fn test_chess960_castling_decoding() {
        let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w AHah - 0 1";
        let pos = ScidPosition::from_fen_auto(fen).unwrap();

        assert!(
            pos.castling_rights().contains(Square::A1)
                || pos.castling_rights().contains(Square::H1)
        );
    }
}
