use crate::sg4::{VariationMove, nag_processor::NagProcessor};

/// Formats annotations for PGN output
pub struct AnnotationFormatter;

impl AnnotationFormatter {
    /// Format a move with all its annotations for PGN
    pub fn format_move_with_annotations(mv: &VariationMove) -> String {
        let mut result = String::new();
        
        // Add the basic move
        result.push_str(&mv.algebraic);
        
        // Add NAG symbols immediately after the move
        for nag in &mv.nags {
            if NagProcessor::is_symbol_nag(*nag) {
                let symbol = NagProcessor::nag_to_symbol(*nag);
                if !symbol.is_empty() {
                    result.push_str(symbol);
                }
            }
        }
        
        // Add comments
        for comment in &mv.comments {
            if !comment.trim().is_empty() {
                result.push(' ');
                result.push_str(comment);
            }
        }
        
        // Add descriptive NAGs as comments
        for nag in &mv.nags {
            if !NagProcessor::is_symbol_nag(*nag) {
                if let Some(description) = NagProcessor::nag_to_description(*nag) {
                    result.push_str(&format!(" {{{}}}", description));
                }
            }
        }
        
        result
    }
    
    /// Format move number with proper spacing
    pub fn format_move_number(move_number: usize, is_white_move: bool, needs_number: bool) -> String {
        if needs_number {
            if is_white_move {
                format!("{}.", move_number)
            } else {
                format!("{}...", move_number)
            }
        } else if is_white_move {
            format!("{}.", move_number)
        } else {
            String::new()
        }
    }
    
    /// Format a complete move line with move number and annotations
    pub fn format_complete_move(mv: &VariationMove, needs_move_number: bool) -> String {
        let mut result = String::new();
        
        // Add move number if needed
        if needs_move_number || mv.is_white_move {
            let move_num_str = Self::format_move_number(mv.move_number, mv.is_white_move, needs_move_number);
            if !move_num_str.is_empty() {
                result.push_str(&move_num_str);
            }
        }
        
        // Add the annotated move
        result.push_str(&Self::format_move_with_annotations(mv));
        
        result
    }
    
    /// Format a sequence of moves with proper spacing
    pub fn format_move_sequence(moves: &[VariationMove]) -> String {
        let mut result = String::new();
        let mut needs_move_number = true;
        
        for (i, mv) in moves.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            
            let formatted = Self::format_complete_move(mv, needs_move_number);
            result.push_str(&formatted);
            
            // Only the first move or white moves need move numbers in a sequence
            needs_move_number = false;
        }
        
        result
    }
    
    /// Separate NAGs into symbol and descriptive categories
    pub fn categorize_nags(nags: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let mut symbol_nags = Vec::new();
        let mut descriptive_nags = Vec::new();
        
        for &nag in nags {
            if NagProcessor::is_symbol_nag(nag) {
                symbol_nags.push(nag);
            } else {
                descriptive_nags.push(nag);
            }
        }
        
        (symbol_nags, descriptive_nags)
    }
    
    /// Check if a move has any annotations
    pub fn has_annotations(mv: &VariationMove) -> bool {
        !mv.comments.is_empty() || !mv.nags.is_empty()
    }
    
    /// Get annotation count for a move
    pub fn annotation_count(mv: &VariationMove) -> usize {
        mv.comments.len() + mv.nags.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{ScidMove, Square, PieceType};

    fn create_test_move() -> VariationMove {
        VariationMove {
            chess_move: ScidMove {
                from: Square(6),
                to: Square(21),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 6,
            },
            move_number: 1,
            is_white_move: true,
            comments: vec!["{A good developing move}".to_string()],
            nags: vec![1], // Good move
            algebraic: "Nf3".to_string(),
        }
    }
    
    #[test]
    fn test_move_with_annotations_formatting() {
        let mv = create_test_move();
        let formatted = AnnotationFormatter::format_move_with_annotations(&mv);
        
        // Should include the move, NAG symbol, and comment
        assert!(formatted.contains("Nf3"));
        assert!(formatted.contains("!"));
        assert!(formatted.contains("{A good developing move}"));
        
        // Should be in correct order: move, NAG, comment
        let nf3_pos = formatted.find("Nf3").unwrap();
        let exclamation_pos = formatted.find("!").unwrap();
        let comment_pos = formatted.find("{A good developing move}").unwrap();
        
        assert!(nf3_pos < exclamation_pos);
        assert!(exclamation_pos < comment_pos);
    }
    
    #[test]
    fn test_move_number_formatting() {
        // White move with number needed
        let result = AnnotationFormatter::format_move_number(1, true, true);
        assert_eq!(result, "1.");
        
        // Black move with number needed
        let result = AnnotationFormatter::format_move_number(1, false, true);
        assert_eq!(result, "1...");
        
        // White move without number needed but still white
        let result = AnnotationFormatter::format_move_number(2, true, false);
        assert_eq!(result, "2.");
        
        // Black move without number needed
        let result = AnnotationFormatter::format_move_number(2, false, false);
        assert_eq!(result, "");
    }
    
    #[test]
    fn test_complete_move_formatting() {
        let mv = create_test_move();
        let formatted = AnnotationFormatter::format_complete_move(&mv, true);
        
        // Should include move number, move, NAG, and comment
        assert!(formatted.contains("1."));
        assert!(formatted.contains("Nf3"));
        assert!(formatted.contains("!"));
        assert!(formatted.contains("{A good developing move}"));
    }
    
    #[test]
    fn test_move_sequence_formatting() {
        let mv1 = VariationMove {
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
        
        let mv2 = VariationMove {
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
        
        let sequence = vec![mv1, mv2];
        let formatted = AnnotationFormatter::format_move_sequence(&sequence);
        
        // Should format as "1.e4 e5"
        assert!(formatted.contains("1.e4"));
        assert!(formatted.contains("e5"));
        assert!(!formatted.contains("1...e5")); // Black move shouldn't have number
    }
    
    #[test]
    fn test_nag_categorization() {
        let nags = vec![1, 2, 14, 30, 44]; // Mix of symbol and descriptive NAGs
        let (symbol_nags, descriptive_nags) = AnnotationFormatter::categorize_nags(&nags);
        
        // Symbol NAGs: 1, 2, 14
        assert_eq!(symbol_nags.len(), 3);
        assert!(symbol_nags.contains(&1));
        assert!(symbol_nags.contains(&2));
        assert!(symbol_nags.contains(&14));
        
        // Descriptive NAGs: 30, 44
        assert_eq!(descriptive_nags.len(), 2);
        assert!(descriptive_nags.contains(&30));
        assert!(descriptive_nags.contains(&44));
    }
    
    #[test]
    fn test_annotation_utilities() {
        let mv_with_annotations = create_test_move();
        assert!(AnnotationFormatter::has_annotations(&mv_with_annotations));
        assert!(AnnotationFormatter::annotation_count(&mv_with_annotations) > 0);
        
        let mv_without_annotations = VariationMove {
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
        
        assert!(!AnnotationFormatter::has_annotations(&mv_without_annotations));
        assert_eq!(AnnotationFormatter::annotation_count(&mv_without_annotations), 0);
    }
    
    #[test]
    fn test_multiple_nag_types() {
        let mut mv = create_test_move();
        mv.nags = vec![1, 14, 30]; // Good move, White better, Initiative
        
        let formatted = AnnotationFormatter::format_move_with_annotations(&mv);
        
        // Should have symbol NAGs (1, 14) inline
        assert!(formatted.contains("!"));
        assert!(formatted.contains("⩲"));
        
        // Should have descriptive NAG (30) as comment
        assert!(formatted.contains("{Initiative}"));
    }
    
    #[test]
    fn test_multiple_comments() {
        let mut mv = create_test_move();
        mv.comments = vec![
            "{First comment}".to_string(),
            "{Second comment}".to_string(),
        ];
        
        let formatted = AnnotationFormatter::format_move_with_annotations(&mv);
        
        assert!(formatted.contains("{First comment}"));
        assert!(formatted.contains("{Second comment}"));
    }
}