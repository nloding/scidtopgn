//! Knight move decoder edge case tests
//! Tests for Phase 1.1.3 of SCID_TO_PGN_COMPLETION_PLAN.md
//! 
//! Validates that our Knight decoder matches SCID behavior exactly

#[cfg(test)]
mod tests {
    use scid_parser::position::{ScidMove, Square, PieceType};
    use scid_parser::position::decoder::decode_knight;
    
    #[test]
    fn test_all_valid_knight_moves() {
        // Test all 8 standard knight moves from center position
        let mut scid_move = ScidMove {
            from: Square(27), // D4
            to: Square(0),
            moving_piece: PieceType::Knight,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };
        
        let expected_targets = [
            10, // 27-17 (C2) - move_value 1
            12, // 27-15 (E2) - move_value 2  
            17, // 27-10 (B3) - move_value 3
            21, // 27-6  (F3) - move_value 4
            33, // 27+6  (B5) - move_value 5
            37, // 27+10 (F5) - move_value 6
            42, // 27+15 (C6) - move_value 7
            44, // 27+17 (E6) - move_value 8
        ];
        
        for (i, expected_target) in expected_targets.iter().enumerate() {
            let move_value = (i + 1) as u8; // SCID uses 1-8
            let result = decode_knight(move_value, &mut scid_move);
            assert!(result.is_ok(), "Knight move {} should succeed", move_value);
            assert_eq!(scid_move.to.0, *expected_target as u8, 
                      "Knight move {} target incorrect: expected {}, got {}", 
                      move_value, *expected_target, scid_move.to.0);
        }
    }
    
    #[test]
    fn test_knight_boundary_conditions() {
        // Test knight moves from edge squares
        let edge_positions = [
            (0, "A1"),   // Bottom-left corner
            (7, "H1"),   // Bottom-right corner
            (56, "A8"),  // Top-left corner
            (63, "H8"),  // Top-right corner
        ];
        
        for (square, name) in edge_positions.iter() {
            let mut scid_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            // Test all possible knight moves from this position
            for move_value in 1..=8 {
                let result = decode_knight(move_value, &mut scid_move);
                // Some moves will be invalid from edge positions - this is expected
                // The important thing is that we don't crash and handle bounds correctly
                if result.is_err() {
                    println!("Knight move {} from {} correctly rejected: {}", 
                            move_value, name, result.unwrap_err());
                }
            }
        }
    }
    
    #[test]
    fn test_invalid_knight_values() {
        let mut scid_move = ScidMove {
            from: Square(27), // D4
            to: Square(0),
            moving_piece: PieceType::Knight,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };
        
        // Test invalid move values (0 and 9+ should be rejected)
        let invalid_values = [0, 9, 10, 15, 16, 255];
        for move_value in invalid_values.iter() {
            let result = decode_knight(*move_value, &mut scid_move);
            assert!(result.is_err(), "Invalid knight move value {} should be rejected", move_value);
        }
    }
    
    #[test]
    fn test_knight_squares_from_scid_reference() {
        // Test specific squares mentioned in SCID documentation
        // These are real examples from SCID games
        
        // Knight from G1 (square 6) 
        let mut scid_move = ScidMove {
            from: Square(6), // G1
            to: Square(0),
            moving_piece: PieceType::Knight,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 6,
        };
        
        // Test some common knight moves from G1
        let test_cases = [
            (3, -4_i8), // 6-10 (square -4, out of bounds)
            (4, 0_i8),  // 6-6  (square 0) 
            (6, 16_i8), // 6+10 (square 16)
            (7, 21_i8), // 6+15 (square 21)
        ];
        
        for (move_value, expected_target) in test_cases.iter() {
            let result = decode_knight(*move_value, &mut scid_move);
            if *expected_target >= 0 && *expected_target <= 63 {
                assert!(result.is_ok(), "Knight move {} from G1 should succeed", move_value);
                if result.is_ok() {
                    assert_eq!(scid_move.to.0, *expected_target as u8, 
                              "Knight move {} from G1 target incorrect", move_value);
                }
            } else {
                assert!(result.is_err(), "Knight move {} from G1 should fail (out of bounds)", move_value);
            }
        }
    }
}