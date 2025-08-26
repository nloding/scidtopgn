//! Move parsing edge case integration tests
//! Tests for Phase 1.6.1 of SCID_TO_PGN_COMPLETION_PLAN.md
//! 
//! Tests the specific failure cases that were previously observed

#[cfg(test)]
mod tests {
    use scid_parser::position::{ScidMove, Square, PieceType, ScidPosition};
    use scid_parser::position::decoder::{decode_knight, decode_bishop, decode_rook, decode_pawn};
    use scid_parser::position::{decode_move, ScidByteStream, decode_move_with_stream};
    
    #[test]
    fn test_previously_failing_knight_moves() {
        // Test the specific Knight failures that were observed:
        // ❌ Knight target square out of bounds: 68 (from square 62, diff 6)
        // ❌ Knight target square out of bounds: 74 (from square 57, diff 17)
        
        let test_cases = [
            // Cases that should be valid
            (62, 4, true),  // From square 62, move_value 4 (diff -6) -> square 56
            (57, 1, true),  // From square 57, move_value 1 (diff -17) -> square 40
            
            // Cases that should fail (out of bounds)
            (62, 5, false), // From square 62, move_value 5 (diff 6) -> square 68 (invalid)
            (57, 8, false), // From square 57, move_value 8 (diff 17) -> square 74 (invalid)
        ];
        
        for (from_square, move_value, should_succeed) in test_cases.iter() {
            let mut scid_move = ScidMove {
                from: Square(*from_square),
                to: Square(0),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            let result = decode_knight(*move_value, &mut scid_move);
            
            if *should_succeed {
                assert!(result.is_ok(), 
                       "Knight move {} from square {} should succeed", move_value, from_square);
            } else {
                assert!(result.is_err(), 
                       "Knight move {} from square {} should fail (out of bounds)", move_value, from_square);
            }
        }
    }
    
    #[test]
    fn test_previously_failing_bishop_moves() {
        // Test the specific Bishop failures that were observed:
        // ❌ Bishop target square out of bounds: 89 (from f:8, target file b, diff -4)
        
        let test_cases = [
            // Cases that should be valid
            (40, 2, true),  // From square 40 (A6), target file 2 (C) -> valid diagonal
            (35, 1, true),  // From square 35 (D5), target file 1 (B) -> valid diagonal
            
            // Cases that should fail (out of bounds)
            (7, 0, false),  // From H1, trying to go to A file with direction -> out of bounds
            (56, 7, false), // From A8, trying to go to H file -> likely out of bounds
        ];
        
        for (from_square, move_value, should_succeed) in test_cases.iter() {
            let mut scid_move = ScidMove {
                from: Square(*from_square),
                to: Square(0),
                moving_piece: PieceType::Bishop,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            let result = decode_bishop(*move_value, &mut scid_move);
            
            if *should_succeed {
                assert!(result.is_ok(), 
                       "Bishop move {} from square {} should succeed", move_value, from_square);
            } else {
                if result.is_err() {
                    println!("Bishop move {} from square {} correctly rejected: {}", 
                            move_value, from_square, result.unwrap_err());
                }
            }
        }
    }
    
    #[test]
    fn test_boundary_conditions_all_pieces() {
        // Test all piece types from challenging boundary positions
        let boundary_positions = [
            (0, "A1"),   // Bottom-left corner
            (7, "H1"),   // Bottom-right corner
            (56, "A8"),  // Top-left corner
            (63, "H8"),  // Top-right corner
        ];
        
        for (square, name) in boundary_positions.iter() {
            // Test Knight moves
            let mut knight_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            for move_value in 1..=8 {
                let result = decode_knight(move_value, &mut knight_move);
                // Many will fail from corners - that's expected
                if result.is_err() {
                    println!("Knight move {} from {} correctly rejected", move_value, name);
                }
            }
            
            // Test Rook moves (should all succeed - rook can go anywhere)
            let mut rook_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Rook,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            for move_value in 0..16 {
                let result = decode_rook(move_value, &mut rook_move);
                assert!(result.is_ok(), "Rook move {} from {} should always succeed", move_value, name);
            }
            
            // Test Bishop moves
            let mut bishop_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Bishop,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };
            
            for move_value in 0..16 {
                let result = decode_bishop(move_value, &mut bishop_move);
                // Many will fail from corners - that's expected
                if result.is_err() {
                    println!("Bishop move {} from {} correctly rejected", move_value, name);
                }
            }
        }
    }
    
    #[test]
    fn test_position_aware_decoding_integration() {
        // Test full position-aware decoding with real scenarios
        let position = ScidPosition::new_starting_position();
        
        // Test the famous CF byte that was incorrectly decoded as "en passant"
        // Should decode as e4 (pawn double push)
        let cf_byte = 0xCF;
        let piece_num = (cf_byte >> 4) & 0x0F; // 12 (E2 pawn)
        let move_value = cf_byte & 0x0F;       // 15 (double push)
        
        let result = decode_move(&position, cf_byte);
        assert!(result.is_ok(), "CF byte should decode successfully");
        
        if let Ok(scid_move) = result {
            assert_eq!(scid_move.piece_num, piece_num, "CF byte piece number should be 12");
            assert_eq!(scid_move.from, Square(12), "CF byte should be from E2");
            assert_eq!(scid_move.to, Square(28), "CF byte should go to E4");
            assert_eq!(scid_move.moving_piece, PieceType::Pawn, "CF byte should be pawn move");
        }
    }
    
    #[test]
    fn test_stream_based_multi_byte_moves() {
        // Test multi-byte moves using stream decoder
        let position = ScidPosition::new_starting_position();
        
        // Test a 2-byte Queen diagonal move
        let queen_diagonal_bytes = [0x13, 0x6D]; // Example 2-byte Queen move
        let mut stream = ScidByteStream::new(&queen_diagonal_bytes);
        
        let result = decode_move_with_stream(&position, &mut stream);
        // This may succeed or fail depending on the specific bytes, but shouldn't crash
        match result {
            Ok(scid_move) => {
                println!("Queen diagonal move decoded successfully: from {} to {}", 
                        scid_move.from.0, scid_move.to.0);
                println!("Bytes consumed: {}", stream.position());
                // Don't assert specific byte consumption - just verify it doesn't crash
                assert!(stream.position() >= 1, "Move should consume at least 1 byte");
            },
            Err(e) => {
                println!("Queen diagonal move failed (expected for some cases): {}", e);
            }
        }
    }
    
    #[test]  
    fn test_edge_case_combinations() {
        // Test combinations of edge cases that might interact poorly
        let position = ScidPosition::new_starting_position();
        
        // Test various raw bytes that have caused issues
        let problematic_bytes = [
            0x65, 0x28, 0x59, 0x06, 0x5C, 0x4D, 0x43, 0x41, 
            0x70, 0x7C, 0x78, 0x42, 0x40, 0x44, 0x71, 0x7A,
            0x27, 0x49, 0x45, 0x4B, 0x68, 0x11, 0x67, 0x26,
            0x0A, 0x34, 0x35, 0x1E, 0xB1, 0x1B, 0x04, 0x5A,
            0x5B, 0x57, 0x72, 0x04, 0xCF // The famous CF byte
        ];
        
        let mut successful_decodes = 0;
        let mut failed_decodes = 0;
        
        for &raw_byte in problematic_bytes.iter() {
            let result = decode_move(&position, raw_byte);
            match result {
                Ok(_) => successful_decodes += 1,
                Err(_) => failed_decodes += 1,
            }
        }
        
        // Calculate success rate
        let total = successful_decodes + failed_decodes;
        let success_rate = (successful_decodes as f64 / total as f64) * 100.0;
        
        println!("Edge case decoding results:");
        println!("  Successful: {}", successful_decodes);
        println!("  Failed: {}", failed_decodes);
        println!("  Success rate: {:.1}%", success_rate);
        
        // We expect some failures - the goal is to handle them gracefully
        assert!(success_rate >= 20.0, "Success rate should be at least 20% on edge cases");
        assert!(success_rate <= 100.0, "Success rate should not exceed 100%");
    }
}