//! SCID Position Wrapper
//!
//! This module provides a wrapper around shakmaty's Chess position that tracks
//! SCID piece numbers alongside the standard chess position.
//!
//! # Piece Number Semantics
//!
//! SCID piece numbers are **side-relative** and **dynamic**:
//! - White to move: White pieces are 0-15
//! - Black to move: Black pieces are 0-15
//! - Captures update the mapping (captured piece number becomes invalid)
//! - Promotions change piece type but keep the number
//!
//! # Standard Initial Numbering
//!
//! ```text
//! White pieces (when White to move):
//!   0 = King (e1)
//!   1 = Queen (d1)
//!   2 = Rook (a1)
//!   3 = Rook (h1)
//!   4 = Bishop (c1)
//!   5 = Bishop (f1)
//!   6 = Knight (b1)
//!   7 = Knight (g1)
//!   8-15 = Pawns (a2-h2)
//!
//! Black pieces (when Black to move):
//!   0 = King (e8)
//!   1 = Queen (d8)
//!   2 = Rook (a8)
//!   3 = Rook (h8)
//!   4 = Bishop (c8)
//!   5 = Bishop (f8)
//!   6 = Knight (b8)
//!   7 = Knight (g8)
//!   8-15 = Pawns (a7-h7)
//! ```

use crate::error::{Result, ScidError};
use shakmaty::fen::Fen;
use shakmaty::{
    Board, Castles, CastlingMode, Chess, Color, EnPassantMode, Move, Piece, Position, Role, Square,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// SCID piece number to square mapping
///
/// Maintains bidirectional mapping between SCID piece numbers and board squares.
#[derive(Debug, Clone)]
pub struct PieceNumberMapping {
    /// Maps SCID piece number to current square (for White)
    white_pieces: HashMap<u8, Square>,
    /// Maps SCID piece number to current square (for Black)
    black_pieces: HashMap<u8, Square>,
}

impl PieceNumberMapping {
    /// Initialize from standard starting position
    pub fn standard_start() -> Self {
        let mut mapping = Self {
            white_pieces: HashMap::with_capacity(16),
            black_pieces: HashMap::with_capacity(16),
        };

        // White pieces
        mapping.white_pieces.insert(0, Square::E1); // King
        mapping.white_pieces.insert(1, Square::D1); // Queen
        mapping.white_pieces.insert(2, Square::A1); // Rook a1
        mapping.white_pieces.insert(3, Square::H1); // Rook h1
        mapping.white_pieces.insert(4, Square::C1); // Bishop c1
        mapping.white_pieces.insert(5, Square::F1); // Bishop f1
        mapping.white_pieces.insert(6, Square::B1); // Knight b1
        mapping.white_pieces.insert(7, Square::G1); // Knight g1

        // White pawns (a2-h2 = piece numbers 8-15)
        mapping.white_pieces.insert(8, Square::A2);
        mapping.white_pieces.insert(9, Square::B2);
        mapping.white_pieces.insert(10, Square::C2);
        mapping.white_pieces.insert(11, Square::D2);
        mapping.white_pieces.insert(12, Square::E2);
        mapping.white_pieces.insert(13, Square::F2);
        mapping.white_pieces.insert(14, Square::G2);
        mapping.white_pieces.insert(15, Square::H2);

        // Black pieces
        mapping.black_pieces.insert(0, Square::E8); // King
        mapping.black_pieces.insert(1, Square::D8); // Queen
        mapping.black_pieces.insert(2, Square::A8); // Rook a8
        mapping.black_pieces.insert(3, Square::H8); // Rook h8
        mapping.black_pieces.insert(4, Square::C8); // Bishop c8
        mapping.black_pieces.insert(5, Square::F8); // Bishop f8
        mapping.black_pieces.insert(6, Square::B8); // Knight b8
        mapping.black_pieces.insert(7, Square::G8); // Knight g8

        // Black pawns (a7-h7 = piece numbers 8-15)
        mapping.black_pieces.insert(8, Square::A7);
        mapping.black_pieces.insert(9, Square::B7);
        mapping.black_pieces.insert(10, Square::C7);
        mapping.black_pieces.insert(11, Square::D7);
        mapping.black_pieces.insert(12, Square::E7);
        mapping.black_pieces.insert(13, Square::F7);
        mapping.black_pieces.insert(14, Square::G7);
        mapping.black_pieces.insert(15, Square::H7);

        mapping
    }

    /// Initialize from FEN position
    ///
    /// Assigns piece numbers based on piece type and square.
    /// Order: King, Queen, Rooks (a-file first), Bishops, Knights, Pawns (a-file first)
    pub fn from_position(chess: &Chess) -> Result<Self> {
        let mut mapping = Self {
            white_pieces: HashMap::new(),
            black_pieces: HashMap::new(),
        };

        // Process each color separately
        for color in [Color::White, Color::Black] {
            let pieces = match color {
                Color::White => &mut mapping.white_pieces,
                Color::Black => &mut mapping.black_pieces,
            };

            let mut piece_num = 0u8;

            // Assign piece numbers in priority order
            for role in [
                Role::King,
                Role::Queen,
                Role::Rook,
                Role::Bishop,
                Role::Knight,
                Role::Pawn,
            ] {
                // Get squares with this piece type and color, sorted by index
                let mut squares: Vec<Square> = chess
                    .board()
                    .by_piece(Piece { color, role })
                    .into_iter()
                    .collect();
                squares.sort_by_key(|s| u32::from(*s));

                for square in squares {
                    if piece_num < 16 {
                        pieces.insert(piece_num, square);
                        piece_num += 1;
                    }
                }
            }
        }

        Ok(mapping)
    }

    /// Get square for piece number (for current side to move)
    pub fn get_square(&self, piece_num: u8, color: Color) -> Option<Square> {
        match color {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    /// Update mapping after a move
    pub fn update_after_move(&mut self, chess_move: &Move, color: Color) {
        // Get the piece mapping for the moving side
        let (moving_pieces, opponent_pieces) = match color {
            Color::White => (&mut self.white_pieces, &mut self.black_pieces),
            Color::Black => (&mut self.black_pieces, &mut self.white_pieces),
        };

        match chess_move {
            Move::Normal { from, to, .. } | Move::EnPassant { from, to } => {
                // Find piece number that moved from this square
                let piece_num = moving_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    // Update to new square
                    moving_pieces.insert(num, *to);
                }

                // Handle captures (remove captured piece from opponent mapping)
                if chess_move.is_capture() {
                    // For en passant, the captured pawn is not on 'to' square
                    let capture_square = if let Move::EnPassant { to, .. } = chess_move {
                        // En passant: captured pawn is on same file but previous rank
                        match color {
                            Color::White => Square::from_coords(to.file(), shakmaty::Rank::Fifth),
                            Color::Black => Square::from_coords(to.file(), shakmaty::Rank::Fourth),
                        }
                    } else {
                        *to
                    };
                    opponent_pieces.retain(|_, &mut sq| sq != capture_square);
                }
            }
            Move::Castle { king, rook } => {
                // Find king piece number
                let king_num = moving_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *king)
                    .map(|(&num, _)| num);

                // Find rook piece number
                let rook_num = moving_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *rook)
                    .map(|(&num, _)| num);

                // Determine castling type and new positions
                let (king_to, rook_to) = if rook.file() > king.file() {
                    // Kingside castling
                    (
                        Square::from_coords(shakmaty::File::G, king.rank()),
                        Square::from_coords(shakmaty::File::F, king.rank()),
                    )
                } else {
                    // Queenside castling
                    (
                        Square::from_coords(shakmaty::File::C, king.rank()),
                        Square::from_coords(shakmaty::File::D, king.rank()),
                    )
                };

                if let Some(num) = king_num {
                    moving_pieces.insert(num, king_to);
                }
                if let Some(num) = rook_num {
                    moving_pieces.insert(num, rook_to);
                }
            }
            Move::Put { .. } => {
                // Not used in standard chess
            }
        }
    }
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
        let parsed_fen: Fen = fen.parse().map_err(|e| ScidError::ParseError {
            file: PathBuf::from("position"),
            offset: 0,
            message: format!("Invalid FEN: {:?}", e),
        })?;

        let chess: Chess = parsed_fen
            .into_position(CastlingMode::Standard)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("position"),
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
    pub fn piece_at(&self, square: Square) -> Option<Piece> {
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
        self.chess.legal_moves().into_iter().collect()
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
        // Validate move is legal using shakmaty
        if !self.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("position"),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        // Get the current turn before applying the move
        let color = self.chess.turn();

        // Update piece number mapping
        self.piece_mapping.update_after_move(chess_move, color);

        // Apply move using shakmaty (this handles ALL chess rules)
        self.chess = self
            .chess
            .clone()
            .play(chess_move)
            .map_err(|e| ScidError::ParseError {
                file: PathBuf::from("position"),
                offset: 0,
                message: format!("Failed to apply move: {:?}", e),
            })?;

        Ok(())
    }

    /// Find legal move matching from/to squares and optional promotion
    ///
    /// Used to convert SCID decoded moves (from/to squares) into
    /// shakmaty Move objects.
    ///
    /// Note: For castling moves, 'to' is the king's destination square
    /// (g1/g8 for kingside, c1/c8 for queenside), not the rook square.
    pub fn find_move(&self, from: Square, to: Square, promotion: Option<Role>) -> Option<Move> {
        let legals = self.legal_moves();

        legals.into_iter().find(|m| {
            match m {
                Move::Normal {
                    from: f,
                    to: t,
                    promotion: p,
                    ..
                } => *f == from && *t == to && *p == promotion,
                Move::EnPassant { from: f, to: t } => *f == from && *t == to && promotion.is_none(),
                Move::Castle { king, rook } => {
                    // Castling: 'from' is king's start, 'to' is king's destination
                    if *king != from {
                        return false;
                    }
                    // Determine expected king destination based on rook position
                    let king_dest = if rook.file() > king.file() {
                        // Kingside: king goes to g-file
                        Square::from_coords(shakmaty::File::G, king.rank())
                    } else {
                        // Queenside: king goes to c-file
                        Square::from_coords(shakmaty::File::C, king.rank())
                    };
                    to == king_dest && promotion.is_none()
                }
                Move::Put { .. } => false,
            }
        })
    }

    /// Get castling rights (delegates to shakmaty)
    pub fn castles(&self) -> &Castles {
        self.chess.castles()
    }

    /// Get en passant target square if available (delegates to shakmaty)
    pub fn ep_square(&self, mode: EnPassantMode) -> Option<Square> {
        self.chess.ep_square(mode)
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

    /// Get the underlying shakmaty Chess position
    pub fn chess(&self) -> &Chess {
        &self.chess
    }
}

impl Default for ScidPosition {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_piece_numbering() {
        let mapping = PieceNumberMapping::standard_start();

        // Verify White pieces
        assert_eq!(mapping.get_square(0, Color::White), Some(Square::E1)); // King
        assert_eq!(mapping.get_square(1, Color::White), Some(Square::D1)); // Queen
        assert_eq!(mapping.get_square(2, Color::White), Some(Square::A1)); // Rook a1
        assert_eq!(mapping.get_square(3, Color::White), Some(Square::H1)); // Rook h1
        assert_eq!(mapping.get_square(8, Color::White), Some(Square::A2)); // a-pawn
        assert_eq!(mapping.get_square(12, Color::White), Some(Square::E2)); // e-pawn
        assert_eq!(mapping.get_square(15, Color::White), Some(Square::H2)); // h-pawn

        // Verify Black pieces
        assert_eq!(mapping.get_square(0, Color::Black), Some(Square::E8)); // King
        assert_eq!(mapping.get_square(1, Color::Black), Some(Square::D8)); // Queen
        assert_eq!(mapping.get_square(8, Color::Black), Some(Square::A7)); // a-pawn
    }

    #[test]
    fn test_piece_mapping_after_move() {
        let mut mapping = PieceNumberMapping::standard_start();

        // Move e2 pawn to e4 (piece number 12)
        let chess_move = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        };

        mapping.update_after_move(&chess_move, Color::White);

        // Verify pawn moved
        assert_eq!(mapping.get_square(12, Color::White), Some(Square::E4));
    }

    #[test]
    fn test_scid_position_creation() {
        let pos = ScidPosition::new();

        // Verify starting position
        assert_eq!(pos.turn(), Color::White);
        assert!(!pos.is_check());
        assert!(!pos.is_checkmate());

        // Verify piece at starting square
        let piece = pos.piece_at(Square::E2).unwrap();
        assert_eq!(piece.role, Role::Pawn);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_piece_number_lookup() {
        let pos = ScidPosition::new();

        // White king should be at e1 (piece number 0)
        assert_eq!(pos.get_piece_square(0), Some(Square::E1));

        // White queen should be at d1 (piece number 1)
        assert_eq!(pos.get_piece_square(1), Some(Square::D1));

        // White e-pawn should be at e2 (piece number 12)
        assert_eq!(pos.get_piece_square(12), Some(Square::E2));
    }

    #[test]
    fn test_make_move() {
        let mut pos = ScidPosition::new();

        // Find legal e4 move
        let e4_move = pos
            .find_move(Square::E2, Square::E4, None)
            .expect("e2-e4 should be legal");

        // Apply move
        pos.make_move(&e4_move).unwrap();

        // Verify position updated
        assert_eq!(pos.turn(), Color::Black);
        assert!(pos.piece_at(Square::E2).is_none());
        assert!(pos.piece_at(Square::E4).is_some());

        // After move, we're Black to move, so get_piece_square returns Black pieces
        // The white pawn is still at E4, but we need to check White's mapping
        // Since get_piece_square uses current turn, we need different approach
    }

    #[test]
    fn test_from_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let pos = ScidPosition::from_fen(fen).unwrap();

        // Verify position matches FEN
        assert_eq!(pos.turn(), Color::Black);
        // Use Always mode since Legal mode only returns ep square if capture is actually possible
        assert_eq!(pos.ep_square(EnPassantMode::Always), Some(Square::E3));

        // Verify piece at e4
        let piece = pos.piece_at(Square::E4).unwrap();
        assert_eq!(piece.role, Role::Pawn);
        assert_eq!(piece.color, Color::White);
    }

    #[test]
    fn test_illegal_move_rejected() {
        let mut pos = ScidPosition::new();

        // Try to make illegal move (pawn to e5 from starting position)
        let illegal_move = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E5,
            capture: None,
            promotion: None,
        };

        let result = pos.make_move(&illegal_move);
        assert!(result.is_err());
    }

    #[test]
    fn test_capture_removes_from_mapping() {
        let mut mapping = PieceNumberMapping::standard_start();

        // Simulate a capture: White pawn on e4 captures Black pawn on d5
        // First, move white e-pawn to e4
        mapping.update_after_move(
            &Move::Normal {
                role: Role::Pawn,
                from: Square::E2,
                to: Square::E4,
                capture: None,
                promotion: None,
            },
            Color::White,
        );

        // Then black d-pawn to d5
        mapping.update_after_move(
            &Move::Normal {
                role: Role::Pawn,
                from: Square::D7,
                to: Square::D5,
                capture: None,
                promotion: None,
            },
            Color::Black,
        );

        // Now white captures on d5
        mapping.update_after_move(
            &Move::Normal {
                role: Role::Pawn,
                from: Square::E4,
                to: Square::D5,
                capture: Some(Role::Pawn),
                promotion: None,
            },
            Color::White,
        );

        // Black's d-pawn (piece 11) should be removed from mapping
        assert_eq!(mapping.get_square(11, Color::Black), None);

        // White's e-pawn (piece 12) should now be on d5
        assert_eq!(mapping.get_square(12, Color::White), Some(Square::D5));
    }
}
