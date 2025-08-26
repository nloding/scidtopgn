use crate::variation::{VariationTree, Variation, VariationMove};

/// Formats variation trees into proper PGN notation
pub struct VariationFormatter {
    /// Current output buffer
    output: String,
    
    /// Current move number
    current_move_number: usize,
    
    /// Track if we need move numbers
    need_move_number: bool,
}

impl VariationFormatter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            current_move_number: 1,
            need_move_number: true,
        }
    }
    
    /// Format complete variation tree to PGN string
    pub fn format_tree(&mut self, tree: &VariationTree) -> String {
        self.output.clear();
        self.current_move_number = 1;
        self.need_move_number = true;
        
        // Format main line with interspersed variations
        let mut main_move_index = 0;
        
        while main_move_index < tree.main_line.len() {
            let main_move = &tree.main_line[main_move_index];
            
            // Check for variations that start at this position
            let variations_here: Vec<_> = tree.variations.iter()
                .filter(|v| v.start_move_index == main_move_index)
                .collect();
            
            // Format the main move
            self.format_move(main_move);
            
            // Format any variations that start after this move
            for variation in variations_here {
                self.format_variation(variation);
            }
            
            main_move_index += 1;
        }
        
        self.output.clone()
    }
    
    /// Format a single move with proper numbering
    fn format_move(&mut self, mv: &VariationMove) {
        // Add move number if needed
        if self.need_move_number || mv.is_white_move {
            if mv.is_white_move {
                self.output.push_str(&format!("{}.", mv.move_number));
            } else {
                self.output.push_str(&format!("{}...", mv.move_number));
            }
            self.need_move_number = false;
        } else if !mv.is_white_move {
            // For black moves that follow immediately after white moves,
            // we generally don't need the move number unless it's the start
            // of a variation or after comments
        }
        
        // Add the move
        self.output.push_str(&mv.algebraic);
        
        // Add NAG symbols
        for nag in &mv.nags {
            self.output.push_str(&format_nag(*nag));
        }
        
        // Add comments
        for comment in &mv.comments {
            self.output.push_str(&format!(" {{{}}}", comment));
        }
        
        self.output.push(' ');
    }
    
    /// Format a variation with proper parentheses
    fn format_variation(&mut self, variation: &Variation) {
        self.output.push('(');
        
        // Set need_move_number for first move in variation
        self.need_move_number = true;
        
        // Format variation moves
        for (i, mv) in variation.moves.iter().enumerate() {
            self.format_move(mv);
            
            // Check for sub-variations
            let sub_vars_here: Vec<_> = variation.sub_variations.iter()
                .filter(|v| v.start_move_index == i)
                .collect();
            
            for sub_var in sub_vars_here {
                self.format_variation(sub_var);
            }
        }
        
        self.output.push(')');
        self.output.push(' ');
        self.need_move_number = true;
    }
}

impl Default for VariationFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert NAG number to symbol
fn format_nag(nag: u8) -> &'static str {
    match nag {
        1 => "!",    // Good move
        2 => "?",    // Poor move  
        3 => "!!",   // Excellent move
        4 => "??",   // Blunder
        5 => "!?",   // Interesting move
        6 => "?!",   // Dubious move
        7 => "□",    // Forced move
        8 => "□",    // Only move
        9 => "⩲",    // Worst move
        10 => "=",   // Equal position
        11 => "=",   // Equal position
        12 => "=",   // Equal position
        13 => "∞",   // Unclear position
        14 => "⩲",   // White has slight advantage
        15 => "⩱",   // Black has slight advantage
        16 => "±",   // White has moderate advantage
        17 => "∓",   // Black has moderate advantage
        18 => "+-",  // White has decisive advantage
        19 => "-+",  // Black has decisive advantage
        _ => "",     // Unknown NAG
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{ScidMove, Square, PieceType};

    #[test]
    fn test_simple_main_line_formatting() {
        let mut formatter = VariationFormatter::new();
        
        // Create a simple main line: 1.e4 e5
        let move1 = VariationMove {
            chess_move: ScidMove {
                from: Square(12),
                to: Square(28),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: true,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "e4".to_string(),
        };
        
        let move2 = VariationMove {
            chess_move: ScidMove {
                from: Square(52),
                to: Square(36),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: false,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "e5".to_string(),
        };
        
    let tree = VariationTree {
            main_line: vec![move1, move2],
            variations: Vec::new(),
        };
        
        let pgn = formatter.format_tree(&tree);
        assert_eq!(pgn.trim(), "1.e4 e5");
    }
    
    #[test]
    fn test_variation_formatting() {
        let mut formatter = VariationFormatter::new();
        
        // Create main line: 1.e4 with variation (1.d4)
        let main_move = VariationMove {
            chess_move: ScidMove {
                from: Square(12),
                to: Square(28),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: true,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "e4".to_string(),
        };
        
        let variation_move = VariationMove {
            chess_move: ScidMove {
                from: Square(11),
                to: Square(27),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 11,
            },
            move_number: 1,
            is_white_move: true,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "d4".to_string(),
        };
        
        let variation = Variation {
            start_move_index: 0,
            moves: vec![variation_move],
            sub_variations: Vec::new(),
            depth: 1,
        };
        
    let tree = VariationTree {
            main_line: vec![main_move],
            variations: vec![variation],
        };
        
        let pgn = formatter.format_tree(&tree);
        assert!(pgn.contains("1.e4"));
        assert!(pgn.contains("(1.d4"));
    }
    
    #[test]
    fn test_nested_variation_formatting() {
        let mut formatter = VariationFormatter::new();
        
        // Create main line: 1.e4 with variation (1.d4 d5 (1...Nf6)) 
        let main_move = VariationMove {
            chess_move: ScidMove {
                from: Square(12),
                to: Square(28),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: true,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "e4".to_string(),
        };
        
        // Variation move 1: d4
        let var_move1 = VariationMove {
            chess_move: ScidMove {
                from: Square(11),
                to: Square(27),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 11,
            },
            move_number: 1,
            is_white_move: true,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "d4".to_string(),
        };
        
        // Variation move 2: d5
        let var_move2 = VariationMove {
            chess_move: ScidMove {
                from: Square(51),
                to: Square(35),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 11,
            },
            move_number: 1,
            is_white_move: false,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "d5".to_string(),
        };
        
        // Sub-variation move: Nf6
        let sub_var_move = VariationMove {
            chess_move: ScidMove {
                from: Square(62),
                to: Square(45),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 6,
            },
            move_number: 1,
            is_white_move: false,
            comments: Vec::new(),
            nags: Vec::new(),
            algebraic: "Nf6".to_string(),
        };
        
        // Create sub-variation (1...Nf6)
        let sub_variation = Variation {
            start_move_index: 1, // starts after d5
            moves: vec![sub_var_move],
            sub_variations: Vec::new(),
            depth: 2,
        };
        
        // Create main variation (1.d4 d5)
        let main_variation = Variation {
            start_move_index: 0,
            moves: vec![var_move1, var_move2],
            sub_variations: vec![sub_variation],
            depth: 1,
        };
        
    let tree = VariationTree {
            main_line: vec![main_move],
            variations: vec![main_variation],
        };
        
        let pgn = formatter.format_tree(&tree);
        
        // Should contain main line
        assert!(pgn.contains("1.e4"));
        // Should contain main variation
        assert!(pgn.contains("(1.d4"));
        assert!(pgn.contains("d5"));
        // Should contain nested variation
        assert!(pgn.contains("(1...Nf6"));
    }
    
    #[test]
    fn test_nag_formatting() {
        assert_eq!(format_nag(1), "!");
        assert_eq!(format_nag(2), "?");
        assert_eq!(format_nag(3), "!!");
        assert_eq!(format_nag(4), "??");
        assert_eq!(format_nag(5), "!?");
        assert_eq!(format_nag(6), "?!");
        assert_eq!(format_nag(255), ""); // Unknown NAG
    }
}