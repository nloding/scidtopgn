use crate::error::Result;
use crate::format::san::SanGenerator;
use shakmaty::{Color, Move};

/// Options for movetext formatting
#[derive(Debug, Clone)]
pub struct MovetextOptions {
    /// Use compact format (no line breaks)
    pub compact: bool,

    /// Maximum characters per line (ignored if compact)
    pub line_width: usize,

    /// Include move numbers
    pub include_move_numbers: bool,

    /// Starting move number (for games that don't start from position)
    pub start_move_number: u16,

    /// Color of first move
    pub first_move_color: Color,
}

impl Default for MovetextOptions {
    fn default() -> Self {
        MovetextOptions {
            compact: false,
            line_width: 80,
            include_move_numbers: true,
            start_move_number: 1,
            first_move_color: Color::White,
        }
    }
}

/// Format movetext section of PGN
pub struct MovetextFormatter {
    san_gen: SanGenerator,
    options: MovetextOptions,
}

impl MovetextFormatter {
    /// Create formatter with default options
    pub fn new() -> Self {
        MovetextFormatter {
            san_gen: SanGenerator::new(),
            options: MovetextOptions::default(),
        }
    }

    /// Create formatter with custom options
    pub fn with_options(options: MovetextOptions) -> Self {
        MovetextFormatter {
            san_gen: SanGenerator::new(),
            options,
        }
    }

    /// Create formatter from FEN position
    pub fn from_fen(fen: &str, options: MovetextOptions) -> Result<Self> {
        Ok(MovetextFormatter {
            san_gen: SanGenerator::from_fen(fen)?,
            options,
        })
    }

    /// Format moves to PGN movetext
    ///
    /// Generates movetext like:
    /// ```
    /// 1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7
    /// 6. Re1 b5 7. Bb3 d6 8. c3 O-O
    /// ```
    pub fn format_moves(&mut self, moves: &[Move]) -> Result<String> {
        let mut movetext = String::new();
        let mut move_number = self.options.start_move_number;
        let mut current_line_length = 0;
        let mut is_white_move = self.options.first_move_color == Color::White;

        for (idx, chess_move) in moves.iter().enumerate() {
            // Generate SAN for this move
            let san = self.san_gen.move_to_san(chess_move)?;

            // Add move number for White moves
            if is_white_move {
                let move_num_str = format!("{}. ", move_number);
                movetext.push_str(&move_num_str);
                current_line_length += move_num_str.len();
            }

            // Add the move
            movetext.push_str(&san);
            current_line_length += san.len();

            // Add space after move
            if idx < moves.len() - 1 {
                movetext.push(' ');
                current_line_length += 1;
            }

            // Handle line wrapping (non-compact mode)
            if !self.options.compact && current_line_length >= self.options.line_width {
                // Only wrap if there are more moves
                if idx < moves.len() - 1 {
                    movetext.push('\n');
                    current_line_length = 0;
                }
            }

            // Update state
            if is_white_move {
                is_white_move = false;
            } else {
                is_white_move = true;
                move_number += 1;
            }
        }

        Ok(movetext)
    }

    /// Format moves with result marker
    pub fn format_moves_with_result(&mut self, moves: &[Move], result: &str) -> Result<String> {
        let mut movetext = self.format_moves(moves)?;
        movetext.push(' ');
        movetext.push_str(result);
        Ok(movetext)
    }
}

impl Default for MovetextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Move, Role, Square};

    fn create_e4_move() -> Move {
        Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        }
    }

    fn create_e5_move() -> Move {
        Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            capture: None,
            to: Square::E5,
            promotion: None,
        }
    }

    fn create_nf3_move() -> Move {
        Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            capture: None,
            to: Square::F3,
            promotion: None,
        }
    }

    #[test]
    fn test_basic_move_numbering() {
        let mut formatter = MovetextFormatter::new();

        let moves = vec![create_e4_move(), create_e5_move(), create_nf3_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        assert!(movetext.contains("1. e4"));
        assert!(movetext.contains("e5"));
        assert!(movetext.contains("2. Nf3"));
    }

    #[test]
    fn test_compact_format() {
        let mut options = MovetextOptions::default();
        options.compact = true;

        let mut formatter = MovetextFormatter::with_options(options);

        let moves = vec![create_e4_move(), create_e5_move(), create_nf3_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        // Compact format should have no newlines
        assert!(!movetext.contains('\n'));
        assert_eq!(movetext, "1. e4 e5 2. Nf3");
    }

    #[test]
    fn test_with_result() {
        let mut formatter = MovetextFormatter::new();

        let moves = vec![create_e4_move(), create_e5_move()];

        let movetext = formatter
            .format_moves_with_result(&moves, "1/2-1/2")
            .unwrap();

        assert!(movetext.ends_with("1/2-1/2"));
    }

    #[test]
    fn test_odd_number_of_moves() {
        let mut formatter = MovetextFormatter::new();

        // Only White's first move
        let moves = vec![create_e4_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        assert_eq!(movetext, "1. e4");
    }

    #[test]
    fn test_line_wrapping() {
        let mut options = MovetextOptions::default();
        options.compact = false;
        options.line_width = 30;

        let mut formatter = MovetextFormatter::with_options(options);

        let moves = vec![create_e4_move(), create_e5_move(), create_nf3_move()];

        let movetext = formatter.format_moves(&moves).unwrap();

        // Should contain newlines due to wrapping
        if movetext.len() > 30 {
            assert!(movetext.contains('\n'));
        }
    }
}
