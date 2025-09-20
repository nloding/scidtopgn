//! Rook move decoder validation tests
//! Tests for Phase 1.3.2 of SCID_TO_PGN_COMPLETION_PLAN.md
//!
//! Validates that our Rook decoder matches SCID behavior exactly

#[cfg(test)]
mod tests {
    use scidtopgn::position::decoder::decode_rook;
    use scidtopgn::position::{PieceType, ScidMove, Square};

    #[test]
    fn test_rook_vertical_moves() {
        // Test vertical moves (value >= 8)
        let mut scid_move = ScidMove {
            from: Square(27), // D4
            to: Square(0),
            moving_piece: PieceType::Rook,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };

        // Test all vertical targets (ranks 0-7)
        for target_rank in 0..8 {
            let move_value = 8 + target_rank;
            let result = decode_rook(move_value, &mut scid_move);
            assert!(
                result.is_ok(),
                "Rook vertical move to rank {} should succeed",
                target_rank
            );

            let expected_square = (target_rank << 3) | 3; // Same file (D), new rank
            assert_eq!(
                scid_move.to.0, expected_square,
                "Rook vertical move to rank {} incorrect: expected {}, got {}",
                target_rank, expected_square, scid_move.to.0
            );
        }
    }

    #[test]
    fn test_rook_horizontal_moves() {
        // Test horizontal moves (value < 8)
        let mut scid_move = ScidMove {
            from: Square(27), // D4
            to: Square(0),
            moving_piece: PieceType::Rook,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };

        // Test all horizontal targets (files 0-7)
        for target_file in 0..8 {
            let move_value = target_file;
            let result = decode_rook(move_value, &mut scid_move);
            assert!(
                result.is_ok(),
                "Rook horizontal move to file {} should succeed",
                target_file
            );

            let expected_square = (3 << 3) | target_file; // Same rank (4), new file
            assert_eq!(
                scid_move.to.0, expected_square,
                "Rook horizontal move to file {} incorrect: expected {}, got {}",
                target_file, expected_square, scid_move.to.0
            );
        }
    }

    #[test]
    fn test_rook_boundary_positions() {
        // Test rook moves from edge squares
        let edge_positions = [
            (0, "A1"),  // Bottom-left corner
            (7, "H1"),  // Bottom-right corner
            (56, "A8"), // Top-left corner
            (63, "H8"), // Top-right corner
            (3, "D1"),  // Bottom edge
            (59, "D8"), // Top edge
            (24, "A4"), // Left edge
            (31, "H4"), // Right edge
        ];

        for (square, name) in edge_positions.iter() {
            let mut scid_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Rook,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };

            // Test all possible rook moves from this position
            for move_value in 0..16 {
                let result = decode_rook(move_value, &mut scid_move);
                // All rook moves should be valid - rook can move to any rank/file
                assert!(
                    result.is_ok(),
                    "Rook move {} from {} should succeed",
                    move_value,
                    name
                );
            }
        }
    }

    #[test]
    fn test_invalid_rook_values() {
        let mut scid_move = ScidMove {
            from: Square(27), // D4
            to: Square(0),
            moving_piece: PieceType::Rook,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };

        // Test invalid move values (16+ should be rejected)
        let invalid_values = [16, 17, 20, 255];
        for move_value in invalid_values.iter() {
            let result = decode_rook(*move_value, &mut scid_move);
            assert!(
                result.is_err(),
                "Invalid rook move value {} should be rejected",
                move_value
            );
        }
    }

    #[test]
    fn test_rook_scid_reference_moves() {
        // Test specific moves that commonly appear in SCID games

        // Rook from A1 (square 0)
        let mut scid_move = ScidMove {
            from: Square(0), // A1
            to: Square(0),
            moving_piece: PieceType::Rook,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 1,
        };

        let test_cases = [
            // Horizontal moves (value < 8)
            (3, (0 << 3) | 3), // A1 to D1 (square 3)
            (7, (0 << 3) | 7), // A1 to H1 (square 7)
            // Vertical moves (value >= 8)
            (8 + 3, (3 << 3) | 0), // A1 to A4 (square 24)
            (8 + 7, (7 << 3) | 0), // A1 to A8 (square 56)
        ];

        for (move_value, expected_target) in test_cases.iter() {
            let result = decode_rook(*move_value, &mut scid_move);
            assert!(
                result.is_ok(),
                "Rook move {} from A1 should succeed",
                move_value
            );
            assert_eq!(
                scid_move.to.0, *expected_target,
                "Rook move {} from A1 target incorrect: expected {}, got {}",
                move_value, *expected_target, scid_move.to.0
            );
        }
    }

    #[test]
    fn test_rook_exact_scid_algorithm() {
        // Verify our algorithm exactly matches SCID's decodeRook
        // Test cases covering the full range of SCID's algorithm

        let test_cases = [
            // From square, move_value, expected target
            (27, 0, (3 << 3) | 0),  // D4 to A4 (horizontal)
            (27, 1, (3 << 3) | 1),  // D4 to B4 (horizontal)
            (27, 7, (3 << 3) | 7),  // D4 to H4 (horizontal)
            (27, 8, (0 << 3) | 3),  // D4 to D1 (vertical)
            (27, 9, (1 << 3) | 3),  // D4 to D2 (vertical)
            (27, 15, (7 << 3) | 3), // D4 to D8 (vertical)
            // Test from corner position
            (0, 0, (0 << 3) | 0),  // A1 to A1 (same square)
            (0, 7, (0 << 3) | 7),  // A1 to H1 (horizontal)
            (0, 15, (7 << 3) | 0), // A1 to A8 (vertical)
            // Test from center position
            (36, 4, (4 << 3) | 4),  // E5 to E5 (same square)
            (36, 2, (4 << 3) | 2),  // E5 to C5 (horizontal)
            (36, 10, (2 << 3) | 4), // E5 to E3 (vertical)
        ];

        for (from_square, move_value, expected_target) in test_cases.iter() {
            let mut scid_move = ScidMove {
                from: Square(*from_square),
                to: Square(0),
                moving_piece: PieceType::Rook,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };

            let result = decode_rook(*move_value, &mut scid_move);
            assert!(
                result.is_ok(),
                "Rook move {} from {} should succeed",
                move_value,
                from_square
            );
            assert_eq!(
                scid_move.to.0, *expected_target,
                "Rook move {} from {} target incorrect: expected {}, got {}",
                move_value, from_square, *expected_target, scid_move.to.0
            );
        }
    }
}
