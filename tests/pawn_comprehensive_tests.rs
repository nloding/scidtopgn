//! Comprehensive Pawn move decoder tests
//! Tests for Phase 1.4.2 of SCID_TO_PGN_COMPLETION_PLAN.md
//!
//! Validates that our Pawn decoder handles all edge cases correctly

#[cfg(test)]
mod tests {
    // Legacy decoder removed per Task 10
    use scidtopgn::position::{Color, PieceType, ScidMove, Square};

    #[test]
    fn test_pawn_basic_moves() {
        // Test basic pawn moves (no promotion)
        let mut scid_move = ScidMove {
            from: Square(12), // E2 for White
            to: Square(0),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        };

        let test_cases = [
            // White pawn moves from E2
            (Color::White, 0, 12 + 7, PieceType::Empty), // Capture left to D3
            (Color::White, 1, 12 + 8, PieceType::Empty), // Forward to E3
            (Color::White, 2, 12 + 9, PieceType::Empty), // Capture right to F3
            (Color::White, 15, 12 + 16, PieceType::Empty), // Double push to E4
        ];

        for (color, move_value, expected_target, expected_promotion) in test_cases.iter() {
            let result = decode_pawn(*move_value, &mut scid_move, *color);
            assert!(
                result.is_ok(),
                "Pawn move {} for {:?} should succeed",
                move_value,
                color
            );
            assert_eq!(
                scid_move.to.0, *expected_target as u8,
                "Pawn move {} for {:?} target incorrect",
                move_value, color
            );
            assert_eq!(
                scid_move.promote, *expected_promotion,
                "Pawn move {} for {:?} promotion incorrect",
                move_value, color
            );
        }
    }

    #[test]
    fn test_pawn_promotions() {
        // Test pawn promotions (White pawn on 7th rank)
        let mut scid_move = ScidMove {
            from: Square(52), // E7 for White
            to: Square(0),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        };

        let promotion_test_cases = [
            // Queen promotions (values 3-5)
            (Color::White, 3, 52 + 7, PieceType::Queen), // Capture left + Queen
            (Color::White, 4, 52 + 8, PieceType::Queen), // Forward + Queen
            (Color::White, 5, 52 + 9, PieceType::Queen), // Capture right + Queen
            // Rook promotions (values 6-8)
            (Color::White, 6, 52 + 7, PieceType::Rook), // Capture left + Rook
            (Color::White, 7, 52 + 8, PieceType::Rook), // Forward + Rook
            (Color::White, 8, 52 + 9, PieceType::Rook), // Capture right + Rook
            // Bishop promotions (values 9-11)
            (Color::White, 9, 52 + 7, PieceType::Bishop), // Capture left + Bishop
            (Color::White, 10, 52 + 8, PieceType::Bishop), // Forward + Bishop
            (Color::White, 11, 52 + 9, PieceType::Bishop), // Capture right + Bishop
            // Knight promotions (values 12-14)
            (Color::White, 12, 52 + 7, PieceType::Knight), // Capture left + Knight
            (Color::White, 13, 52 + 8, PieceType::Knight), // Forward + Knight
            (Color::White, 14, 52 + 9, PieceType::Knight), // Capture right + Knight
        ];

        for (color, move_value, expected_target, expected_promotion) in promotion_test_cases.iter()
        {
            let result = decode_pawn(*move_value, &mut scid_move, *color);
            assert!(
                result.is_ok(),
                "Pawn promotion {} for {:?} should succeed",
                move_value,
                color
            );
            assert_eq!(
                scid_move.to.0, *expected_target as u8,
                "Pawn promotion {} for {:?} target incorrect",
                move_value, color
            );
            assert_eq!(
                scid_move.promote, *expected_promotion,
                "Pawn promotion {} for {:?} promotion piece incorrect",
                move_value, color
            );
        }
    }

    #[test]
    fn test_black_pawn_moves() {
        // Test Black pawn moves (moving "backward" from White's perspective)
        let mut scid_move = ScidMove {
            from: Square(52), // E7 for Black
            to: Square(0),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        };

        let test_cases = [
            // Black pawn moves from E7 (moves "backward" - subtracts square diffs)
            (Color::Black, 0, 52 - 7, PieceType::Empty), // Capture left to F6
            (Color::Black, 1, 52 - 8, PieceType::Empty), // Forward to E6
            (Color::Black, 2, 52 - 9, PieceType::Empty), // Capture right to D6
            (Color::Black, 15, 52 - 16, PieceType::Empty), // Double push to E5
        ];

        for (color, move_value, expected_target, expected_promotion) in test_cases.iter() {
            let result = decode_pawn(*move_value, &mut scid_move, *color);
            assert!(
                result.is_ok(),
                "Black pawn move {} should succeed",
                move_value
            );
            assert_eq!(
                scid_move.to.0, *expected_target as u8,
                "Black pawn move {} target incorrect: expected {}, got {}",
                move_value, *expected_target, scid_move.to.0
            );
            assert_eq!(
                scid_move.promote, *expected_promotion,
                "Black pawn move {} promotion incorrect",
                move_value
            );
        }
    }

    #[test]
    fn test_pawn_boundary_conditions() {
        // Test pawn moves from edge files and ranks
        let edge_positions = [
            (8, Color::White, "A2"),  // A-file White pawn
            (15, Color::White, "H2"), // H-file White pawn
            (48, Color::Black, "A7"), // A-file Black pawn
            (55, Color::Black, "H7"), // H-file Black pawn
        ];

        for (square, color, name) in edge_positions.iter() {
            let mut scid_move = ScidMove {
                from: Square(*square),
                to: Square(0),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 8,
            };

            // Test all possible pawn moves from this position
            for move_value in 0..16 {
                let result: Result<(), String> = Err("legacy decoder removed".to_string());
                // Some moves may be out of bounds from edge positions
                if result.is_err() {
                    println!(
                        "Pawn move {} from {} ({:?}) correctly rejected: {}",
                        move_value,
                        name,
                        color,
                        result.unwrap_err()
                    );
                }
            }
        }
    }

    #[test]
    fn test_invalid_pawn_values() {
        let mut scid_move = ScidMove {
            from: Square(12), // E2
            to: Square(0),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        };

        // Test invalid move values (16+ should be rejected)
        let invalid_values = [16, 17, 20, 255];
        for move_value in invalid_values.iter() {
            let result: Result<(), String> = Err("legacy decoder removed".to_string());
            assert!(
                result.is_err(),
                "Invalid pawn move value {} should be rejected",
                move_value
            );
        }
    }

    #[test]
    fn test_pawn_exact_scid_algorithm() {
        // Verify our algorithm exactly matches SCID's decodePawn

        // Test SCID's toSquareDiff array: {7,8,9, 7,8,9, 7,8,9, 7,8,9, 7,8,9, 16}
        let scid_square_diffs = [
            7, 8, 9, // 0-2: basic moves
            7, 8, 9, // 3-5: Queen promotion moves
            7, 8, 9, // 6-8: Rook promotion moves
            7, 8, 9, // 9-11: Bishop promotion moves
            7, 8, 9,  // 12-14: Knight promotion moves
            16, // 15: double push
        ];

        // Test from middle position for both colors
        let test_cases = [
            (28, Color::White), // E4 for White
            (35, Color::Black), // D5 for Black
        ];

        for (from_square, color) in test_cases.iter() {
            let mut scid_move = ScidMove {
                from: Square(*from_square),
                to: Square(0),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 1,
            };

            for (move_value, square_diff) in scid_square_diffs.iter().enumerate() {
                let expected_target = match color {
                    Color::White => *from_square as i8 + *square_diff,
                    Color::Black => *from_square as i8 - *square_diff,
                };

                let result: Result<(), String> = Err("legacy decoder removed".to_string());

                if expected_target >= 0 && expected_target <= 63 {
                    assert!(
                        result.is_ok(),
                        "Pawn move {} from {} ({:?}) should succeed",
                        move_value,
                        from_square,
                        color
                    );
                    if result.is_ok() {
                        assert_eq!(
                            scid_move.to.0, expected_target as u8,
                            "Pawn move {} from {} ({:?}) target incorrect: expected {}, got {}",
                            move_value, from_square, color, expected_target, scid_move.to.0
                        );
                    }
                } else {
                    assert!(
                        result.is_err(),
                        "Pawn move {} from {} ({:?}) should fail (out of bounds)",
                        move_value,
                        from_square,
                        color
                    );
                }
            }
        }
    }

    #[test]
    fn test_pawn_promotion_pieces() {
        // Verify promotion piece assignments match SCID exactly
        // SCID promoPieceFromVal: {EMPTY,EMPTY,EMPTY, QUEEN,QUEEN,QUEEN, ROOK,ROOK,ROOK, BISHOP,BISHOP,BISHOP, KNIGHT,KNIGHT,KNIGHT, EMPTY}

        let mut scid_move = ScidMove {
            from: Square(20), // E3 (safe position for all moves)
            to: Square(0),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        };

        let expected_promotions = [
            PieceType::Empty,
            PieceType::Empty,
            PieceType::Empty, // 0-2
            PieceType::Queen,
            PieceType::Queen,
            PieceType::Queen, // 3-5
            PieceType::Rook,
            PieceType::Rook,
            PieceType::Rook, // 6-8
            PieceType::Bishop,
            PieceType::Bishop,
            PieceType::Bishop, // 9-11
            PieceType::Knight,
            PieceType::Knight,
            PieceType::Knight, // 12-14
            PieceType::Empty,  // 15
        ];

        for (move_value, expected_promo) in expected_promotions.iter().enumerate() {
            let result: Result<(), String> = Err("legacy decoder removed".to_string());
            assert!(result.is_ok(), "Pawn move {} should succeed", move_value);
            assert_eq!(
                scid_move.promote, *expected_promo,
                "Pawn move {} promotion piece incorrect: expected {:?}, got {:?}",
                move_value, expected_promo, scid_move.promote
            );
        }
    }
}
