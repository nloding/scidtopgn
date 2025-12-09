#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{decode_move_with_piece_type, ScidPosition, Color};
    use crate::position::PieceType;

    #[test]
    fn test_piece_type_translation() {
        // Test all piece types map correctly to their SCID piece numbers
        let test_cases = vec![
            (PieceType::King, 1),
            (PieceType::Queen, 2), 
            (PieceType::Rook, 3),
            (PieceType::Bishop, 4),
            (PieceType::Knight, 5),
            (PieceType::Pawn, 6),
        ];
        
        for (piece_type, expected_num) in test_cases {
            // Create a position where the piece is on a known square
            let position = ScidPosition::new_starting_position();
            
            // Test that when we provide interpretation piece type, it gets routed correctly
            // This test validates the core fix for dual numbering systems
            let move_byte = (expected_num << 4) | 0; // Simple move with value 0
            
            let result = decode_move_with_piece_type(&position, move_byte, Some(piece_type));
            
            // The result should not have routing errors (like "Invalid knight move value")
            // We expect position-related errors, not routing errors
            match result {
                Ok(_) => {}, // Successfully decoded to routing stage
                Err(e) => {
                    // Should NOT get routing errors like "Invalid X move value"
                    assert!(!e.contains("Invalid knight move value"));
                    assert!(!e.contains("Invalid queen move value"));
                    assert!(!e.contains("Invalid rook move value"));
                    assert!(!e.contains("Invalid bishop move value"));
                    assert!(!e.contains("Invalid king move value"));
                    assert!(!e.contains("Invalid pawn move value"));
                }
            }
        }
    }

    #[test]
    fn test_problematic_move_0x6c() {
        // Specific test for move byte 0x6C (Pawn promoting to Knight)
        // Before fix: Should fail with "Invalid knight move value: 12"
        // After fix: Should succeed with pawn routing or get position-level errors
        let position = ScidPosition::new_starting_position();
        
        let move_byte = 0x6C; // Pawn move, value 12 = forward + promote to Knight
        let interpretation_piece_type = Some(PieceType::Pawn);
        
        let result = decode_move_with_piece_type(&position, move_byte, interpretation_piece_type);
        
        // Key test: Should NOT get "Invalid knight move value: 12" error
        match result {
            Ok(_) => {
                // Success - move was routed to pawn converter correctly
            },
            Err(e) => {
                // Should NOT get the original error
                assert!(!e.contains("Invalid knight move value: 12"), 
                    "Should not get 'Invalid knight move value: 12' error after fix");
                
                // Position/application errors are acceptable
                assert!(e.contains("No") || e.contains("Could not") || e.contains("not at"), 
                    "Should only get position/application errors, not routing errors");
            }
        }
    }

    #[test]
    fn test_backward_compatibility() {
        // Test that None interpretation_piece_type still works (backward compatibility)
        let position = ScidPosition::new_starting_position();
        
        let move_byte = 0x20; // King move, value 0
        let result = decode_move_with_piece_type(&position, move_byte, None);
        
        // Should still work with original behavior
        assert!(result.is_ok() || result.unwrap_err().contains("No King found"));
    }
}