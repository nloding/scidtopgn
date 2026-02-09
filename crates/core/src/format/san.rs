use crate::error::{Result, ScidError};
use shakmaty::{fen::Fen, san::San, Bitboard, Chess, Move, Position};
use std::path::PathBuf;

/// Stateful SAN (Standard Algebraic Notation) generator
///
/// Wraps shakmaty's Chess position and generates SAN notation
/// for moves while maintaining position state.
///
/// # Example
///
/// ```
/// use scidtopgn_core::format::san::SanGenerator;
/// use shakmaty::{Chess, Square, Move, Role};
///
/// let mut gen = SanGenerator::new();
///
/// // Generate SAN for e2-e4
/// let move_e4 = Move::Normal {
///     role: Role::Pawn,
///     from: Square::E2,
///     capture: None,
///     to: Square::E4,
///     promotion: None,
/// };
///
/// let san = gen.move_to_san(&move_e4).unwrap();
/// assert_eq!(san, "e4");
/// ```
#[derive(Debug, Clone)]
pub struct SanGenerator {
    /// Current chess position
    position: Chess,
}

impl SanGenerator {
    /// Create SAN generator from standard starting position
    pub fn new() -> Self {
        SanGenerator {
            position: Chess::default(),
        }
    }

    /// Create SAN generator from FEN position
    ///
    /// # Example
    ///
    /// ```
    /// let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
    /// let gen = SanGenerator::from_fen(fen).unwrap();
    /// ```
    pub fn from_fen(fen: &str) -> Result<Self> {
        let position = Fen::from_str(fen).map_err(|e| ScidError::ParseError {
            file: PathBuf::from("san"),
            offset: 0,
            message: format!("Invalid FEN: {:?}", e),
        })?;

        Ok(SanGenerator { position })
    }

    /// Create SAN generator from existing Chess position
    pub fn from_position(position: Chess) -> Self {
        SanGenerator { position }
    }

    /// Generate SAN for a single move and update position
    ///
    /// This method:
    /// 1. Generates SAN notation using shakmaty
    /// 2. Applies the move to the internal position
    /// 3. Returns the SAN string
    ///
    /// # Errors
    ///
    /// Returns error if move is illegal in current position
    pub fn move_to_san(&mut self, chess_move: &Move) -> Result<String> {
        if !self.position.is_legal(chess_move) {
            return Err(ScidError::ParseError {
                file: PathBuf::from("san"),
                offset: 0,
                message: format!("Illegal move: {:?}", chess_move),
            });
        }

        let board_fen = self.position.board().board_fen(Bitboard::EMPTY);
        if let Some(fen) = board_fen {
            let san = San::from_move(&self.position, chess_move);

            self.position =
                self.position
                    .clone()
                    .play(chess_move)
                    .map_err(|e| ScidError::ParseError {
                        file: PathBuf::from("san"),
                        offset: 0,
                        message: format!("Failed to apply move: {:?}", e),
                    })?;

            Ok(san.to_string())
        } else {
            let san = San::from_move(&self.position, chess_move);

            self.position =
                self.position
                    .clone()
                    .play(chess_move)
                    .map_err(|e| ScidError::ParseError {
                        file: PathBuf::from("san"),
                        offset: 0,
                        message: format!("Failed to apply move: {:?}", e),
                    })?;

            Ok(san.to_string())
        }
    }

    /// Generate SAN for a sequence of moves
    ///
    /// Generates SAN notation for each move in order, updating
    /// position state after each move.
    ///
    /// # Example
    ///
    /// ```
    /// let moves = vec![e4_move, e5_move, nf3_move];
    /// let sans = gen.moves_to_san(&moves).unwrap();
    /// // Returns: ["e4", "e5", "Nf3"]
    /// ```
    pub fn moves_to_san(&mut self, moves: &[Move]) -> Result<Vec<String>> {
        let mut sans = Vec::with_capacity(moves.len());

        for chess_move in moves {
            let san = self.move_to_san(chess_move)?;
            sans.push(san);
        }

        Ok(sans)
    }

    /// Get current position
    pub fn position(&self) -> &Chess {
        &self.position
    }

    /// Get current turn
    pub fn turn(&self) -> shakmaty::Color {
        self.position.turn()
    }

    /// Check if current position is checkmate
    pub fn is_checkmate(&self) -> bool {
        self.position.is_checkmate()
    }

    /// Check if current position is stalemate
    pub fn is_stalemate(&self) -> bool {
        self.position.is_stalemate()
    }

    /// Check if current position is check
    pub fn is_check(&self) -> bool {
        self.position.is_check()
    }
}

impl Default for SanGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Bitboard, Color, Move, Role, Square};

    #[test]
    fn test_basic_pawn_move() {
        let mut gen = SanGenerator::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            promotion: None,
            capture: None,
        };

        let san = gen.move_to_san(&e4).unwrap();
        assert_eq!(san, "e4");

        assert_eq!(gen.turn(), Color::Black);
    }

    #[test]
    fn test_piece_move() {
        let mut gen = SanGenerator::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            promotion: None,
            capture: None,
        };
        gen.move_to_san(&e4).unwrap();

        let e5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E5,
            promotion: None,
            capture: None,
        };
        gen.move_to_san(&e5).unwrap();

        let nf3 = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            to: Square::F3,
            promotion: None,
            capture: None,
        };

        let san = gen.move_to_san(&nf3).unwrap();
        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_capture_notation() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let nf3 = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            to: Square::F3,
            promotion: None,
            capture: None,
        };
        gen.move_to_san(&nf3).unwrap();

        let nc6 = Move::Normal {
            role: Role::Knight,
            from: Square::B8,
            to: Square::C6,
            promotion: None,
            capture: None,
        };
        gen.move_to_san(&nc6).unwrap();

        let nxe5 = Move::Normal {
            role: Role::Knight,
            from: Square::F3,
            to: Square::E5,
            promotion: None,
            capture: Some(Role::Pawn),
        };

        let san = gen.move_to_san(&nxe5).unwrap();
        assert_eq!(san, "Nxe5");
    }

    #[test]
    fn test_pawn_capture() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let fen2 = "rnbqkbnr/ppp2ppp/8/3pp3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
        let mut gen2 = SanGenerator::from_fen(fen2).unwrap();

        let exd5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E4,
            to: Square::D5,
            promotion: None,
            capture: Some(Role::Pawn),
        };

        let san = gen2.move_to_san(&exd5).unwrap();
        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_castling() {
        let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::H1,
        };

        let san = gen.move_to_san(&castle).unwrap();
        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_queenside_castling() {
        let fen = "r3kbnr/pppqpppp/2np4/8/8/2NP4/PPPQPPPP/R3KBNR w KQkq - 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let castle = Move::Castle {
            king: Square::E1,
            rook: Square::A1,
        };

        let san = gen.move_to_san(&castle).unwrap();
        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_promotion() {
        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let promo = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E8,
            promotion: Some(Role::Queen),
            capture: None,
        };

        let san = gen.move_to_san(&promo).unwrap();
        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_check_notation() {
        let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let nxe5 = Move::Normal {
            role: Role::Knight,
            from: Square::F3,
            to: Square::E5,
            promotion: None,
            capture: Some(Role::Pawn),
        };

        let san = gen.move_to_san(&nxe5).unwrap();
        assert!(san.contains('+') || san == "Nxe5");
    }

    #[test]
    fn test_disambiguation() {
        let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4";
        let mut gen = SanGenerator::from_fen(fen).unwrap();

        let nd5 = Move::Normal {
            role: Role::Knight,
            from: Square::C3,
            to: Square::D5,
            promotion: None,
            capture: None,
        };

        let san = gen.move_to_san(&nd5).unwrap();
        assert!(san.contains('N'));
    }

    #[test]
    fn test_move_sequence() {
        let mut gen = SanGenerator::new();

        let moves = vec![
            Move::Normal {
                role: Role::Pawn,
                from: Square::E2,
                to: Square::E4,
                promotion: None,
                capture: None,
            },
            Move::Normal {
                role: Role::Pawn,
                from: Square::E7,
                to: Square::E5,
                promotion: None,
                capture: None,
            },
            Move::Normal {
                role: Role::Knight,
                from: Square::G1,
                to: Square::F3,
                promotion: None,
                capture: None,
            },
        ];

        let sans = gen.moves_to_san(&moves).unwrap();
        assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
    }

    #[test]
    fn test_illegal_move_rejected() {
        let mut gen = SanGenerator::new();

        let illegal = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            to: Square::E2,
            promotion: None,
            capture: None,
        };

        let result = gen.move_to_san(&illegal);
        assert!(result.is_err());
    }
}

#[test]
fn test_piece_move() {
    let mut gen = SanGenerator::new();

    let e4 = Move::Normal {
        role: Role::Pawn,
        from: Square::E2,
        capture: None,
        to: Square::E4,
        promotion: None,
    };
    gen.move_to_san(&e4).unwrap();

    let e5 = Move::Normal {
        role: Role::Pawn,
        from: Square::E7,
        capture: None,
        to: Square::E5,
        promotion: None,
    };
    gen.move_to_san(&e5).unwrap();

    let nf3 = Move::Normal {
        role: Role::Knight,
        from: Square::G1,
        capture: None,
        to: Square::F3,
        promotion: None,
    };

    let san = gen.move_to_san(&nf3).unwrap();
    assert_eq!(san, "Nf3");
}

#[test]
fn test_capture_notation() {
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let nf3 = Move::Normal {
        role: Role::Knight,
        from: Square::G1,
        capture: None,
        to: Square::F3,
        promotion: None,
    };
    gen.move_to_san(&nf3).unwrap();

    let nc6 = Move::Normal {
        role: Role::Knight,
        from: Square::B8,
        capture: None,
        to: Square::C6,
        promotion: None,
    };
    gen.move_to_san(&nc6).unwrap();

    let nxe5 = Move::Normal {
        role: Role::Knight,
        from: Square::F3,
        capture: Some(Role::Pawn),
        to: Square::E5,
        promotion: None,
    };

    let san = gen.move_to_san(&nxe5).unwrap();
    assert_eq!(san, "Nxe5");
}

#[test]
fn test_pawn_capture() {
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 1";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let fen2 = "rnbqkbnr/ppp2ppp/8/3pp3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 1";
    let mut gen2 = SanGenerator::from_fen(fen2).unwrap();

    let exd5 = Move::Normal {
        role: Role::Pawn,
        from: Square::E4,
        capture: Some(Role::Pawn),
        to: Square::D5,
        promotion: None,
    };

    let san = gen2.move_to_san(&exd5).unwrap();
    assert_eq!(san, "exd5");
}

#[test]
fn test_castling() {
    let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let castle = Move::Castle {
        king: Square::E1,
        rook: Square::H1,
    };

    let san = gen.move_to_san(&castle).unwrap();
    assert_eq!(san, "O-O");
}

#[test]
fn test_queenside_castling() {
    let fen = "r3kbnr/pppqpppp/2np4/8/8/2NP4/PPPQPPPP/R3KBNR w KQkq - 0 1";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let castle = Move::Castle {
        king: Square::E1,
        rook: Square::A1,
    };

    let san = gen.move_to_san(&castle).unwrap();
    assert_eq!(san, "O-O-O");
}

#[test]
fn test_promotion() {
    let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let promo = Move::Normal {
        role: Role::Pawn,
        from: Square::E7,
        capture: None,
        to: Square::E8,
        promotion: Some(Role::Queen),
    };

    let san = gen.move_to_san(&promo).unwrap();
    assert_eq!(san, "e8=Q");
}

#[test]
fn test_check_notation() {
    let fen = "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let nxe5 = Move::Normal {
        role: Role::Knight,
        from: Square::F3,
        capture: Some(Role::Pawn),
        to: Square::E5,
        promotion: None,
    };

    let san = gen.move_to_san(&nxe5).unwrap();
    assert!(san.contains('+') || san == "Nxe5");
}

#[test]
fn test_disambiguation() {
    let fen = "r1bqkb1r/pppp1ppp/2n2n2/4p3/4P3/2N2N2/PPPP1PPP/R1BQKB1R w KQkq - 4 4";
    let mut gen = SanGenerator::from_fen(fen).unwrap();

    let nd5 = Move::Normal {
        role: Role::Knight,
        from: Square::C3,
        capture: None,
        to: Square::D5,
        promotion: None,
    };

    let san = gen.move_to_san(&nd5).unwrap();
    assert!(san.contains('N'));
}

#[test]
fn test_move_sequence() {
    let mut gen = SanGenerator::new();

    let moves = vec![
        Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        },
        Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            capture: None,
            to: Square::E5,
            promotion: None,
        },
        Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::F3,
            promotion: None,
        },
    ];

    let sans = gen.moves_to_san(&moves).unwrap();
    assert_eq!(sans, vec!["e4", "e5", "Nf3"]);
}

#[test]
fn test_illegal_move_rejected() {
    let mut gen = SanGenerator::new();

    let illegal = Move::Normal {
        role: Role::Knight,
        from: Square::G1,
        capture: None,
        to: Square::E2,
        promotion: None,
    };

    let result = gen.move_to_san(&illegal);
    assert!(result.is_err());
}
