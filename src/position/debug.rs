// Position debugging and validation utilities
// Phase 1 Step 1.3 from POSITION_TRACKING_IMPLEMENTATION_PLAN.md

use crate::position::{Color, PieceType, ScidPosition};

impl ScidPosition {
    /// Generate human-readable position description for debugging
    pub fn debug_position_state(&self) -> String {
        let mut result = String::new();

        result.push_str("=== POSITION STATE DEBUG ===\n");
        result.push_str(&format!("Turn: {:?}\n", self.to_move));
        result.push_str(&format!("Move: {}\n", self.fullmove_number));
        result.push_str(&format!("Hash: {:016x}\n", self.position_hash));
        result.push_str(&format!("En passant: {:?}\n", self.en_passant_target));

        // Board visualization
        result.push_str("\nBOARD:\n");
        for rank in (0..8).rev() {
            result.push_str(&format!("{} ", rank + 1));
            for file in 0..8 {
                let square = (rank << 3) | file;
                let piece_char = match self.board[square as usize] {
                    PieceType::Empty => '.',
                    PieceType::King => 'K',
                    PieceType::Queen => 'Q',
                    PieceType::Rook => 'R',
                    PieceType::Bishop => 'B',
                    PieceType::Knight => 'N',
                    PieceType::Pawn => 'P',
                };
                result.push_str(&format!("{} ", piece_char));
            }
            result.push('\n');
        }
        result.push_str("  a b c d e f g h\n");

        // Piece lists
        result.push_str("\nPIECE LISTS:\n");
        for color in [Color::White, Color::Black] {
            let color_idx = color as usize;
            result.push_str(&format!(
                "{} ({} pieces):\n",
                color.to_string(),
                self.piece_counts[color_idx]
            ));
            for i in 0..self.piece_counts[color_idx] {
                if i >= 16 {
                    break;
                }
                let square = self.piece_lists[color_idx][i];
                let piece = self.board[square.0 as usize];
                result.push_str(&format!(
                    "  {}: {} at {}\n",
                    i,
                    piece.to_string(),
                    square.to_algebraic()
                ));
            }
        }

        // Move history
        if !self.move_history.is_empty() {
            result.push_str(&format!(
                "\nMOVE HISTORY ({} moves):\n",
                self.move_history.len()
            ));
            for (i, mv) in self.move_history.iter().enumerate() {
                result.push_str(&format!(
                    "  {}: {} -> {} (piece {})\n",
                    i + 1,
                    mv.from.to_algebraic(),
                    mv.to.to_algebraic(),
                    mv.piece_num
                ));
            }
        }

        result.push_str("===========================\n");
        result
    }

    /// Validate piece list consistency with board
    pub fn validate_piece_lists(&self) -> Result<(), String> {
        for color in [Color::White, Color::Black] {
            let color_idx = color as usize;

            // Check each piece in the list
            for i in 0..self.piece_counts[color_idx] {
                if i >= 16 {
                    return Err(format!(
                        "Too many pieces for {}: {}",
                        color.to_string(),
                        self.piece_counts[color_idx]
                    ));
                }

                let square = self.piece_lists[color_idx][i];
                if square.0 >= 64 {
                    return Err(format!(
                        "Invalid square {} for {} piece {}",
                        square.0,
                        color.to_string(),
                        i
                    ));
                }

                let piece_on_board = self.board[square.0 as usize];
                if piece_on_board == PieceType::Empty {
                    return Err(format!(
                        "Piece list shows {} piece {} at {} but board is empty",
                        color.to_string(),
                        i,
                        square.to_algebraic()
                    ));
                }

                // Check reverse lookup
                if self.list_pos[square.0 as usize] != i as u8 {
                    return Err(format!(
                        "Reverse lookup mismatch at {}: expected {}, got {}",
                        square.to_algebraic(),
                        i,
                        self.list_pos[square.0 as usize]
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if position matches SCID starting position
    pub fn is_starting_position(&self) -> bool {
        // Quick checks
        if self.to_move != Color::White
            || self.fullmove_number != 1
            || self.halfmove_clock != 0
            || self.piece_counts[0] != 16
            || self.piece_counts[1] != 16
        {
            return false;
        }

        // Check piece locations match initial setup
        for (i, (expected_piece, expected_square)) in
            super::SCID_WHITE_PIECE_INIT.iter().enumerate()
        {
            if self.piece_lists[0][i] != *expected_square
                || self.board[expected_square.0 as usize] != *expected_piece
            {
                return false;
            }
        }

        for (i, (expected_piece, expected_square)) in
            super::SCID_BLACK_PIECE_INIT.iter().enumerate()
        {
            if self.piece_lists[1][i] != *expected_square
                || self.board[expected_square.0 as usize] != *expected_piece
            {
                return false;
            }
        }

        true
    }

    /// Generate comparison between two positions for debugging move application
    pub fn compare_positions(&self, other: &ScidPosition, move_description: &str) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "=== POSITION COMPARISON: {} ===\n",
            move_description
        ));

        // Compare basic state
        result.push_str(&format!(
            "Turn: {:?} -> {:?}\n",
            self.to_move, other.to_move
        ));
        result.push_str(&format!(
            "Move: {} -> {}\n",
            self.fullmove_number, other.fullmove_number
        ));
        result.push_str(&format!(
            "Halfmove: {} -> {}\n",
            self.halfmove_clock, other.halfmove_clock
        ));

        // Compare piece counts
        for color in [Color::White, Color::Black] {
            let color_idx = color as usize;
            let before = self.piece_counts[color_idx];
            let after = other.piece_counts[color_idx];
            if before != after {
                result.push_str(&format!(
                    "{} pieces: {} -> {} ({})\n",
                    color.to_string(),
                    before,
                    after,
                    if after > before { "gained" } else { "lost" }
                ));
            }
        }

        // Find moved pieces
        for color in [Color::White, Color::Black] {
            let color_idx = color as usize;
            for i in 0..std::cmp::min(self.piece_counts[color_idx], other.piece_counts[color_idx]) {
                if i >= 16 {
                    break;
                }
                let before_square = self.piece_lists[color_idx][i];
                let after_square = other.piece_lists[color_idx][i];
                if before_square != after_square {
                    result.push_str(&format!(
                        "{} piece {}: {} -> {}\n",
                        color.to_string(),
                        i,
                        before_square.to_algebraic(),
                        after_square.to_algebraic()
                    ));
                }
            }
        }

        result.push_str("=================================\n");
        result
    }
}
