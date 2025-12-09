// Phase 2 verification tests - piece-specific decoders

#[cfg(test)]
mod tests {
    // Legacy decoder removed per Task 10
    use scidtopgn::position::ScidMove;
    use scidtopgn::position::{PieceType, Square};

    /// Helper function to create a test move
    fn create_test_move(from_square: u8, piece_type: PieceType, piece_num: u8) -> ScidMove {
        ScidMove {
            from: Square(from_square),
            to: Square(from_square), // Will be updated by decoder
            moving_piece: piece_type,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num,
        }
    }

    #[test]
    fn test_king_moves() {
        // Test King move decoder
        // King at E1 (square 4), test various moves
        let mut move_data = create_test_move(4, PieceType::King, 0); // E1

        // Test null move (value 0)
        let _ = &mut move_data;
        assert_eq!(move_data.to, Square(4)); // Stays at E1

        // Test basic king moves using SCID's square difference array
        // Value 1: -9 (up-left): E1 -> D2 (4 + (-9) = -5, invalid but testing decoder logic)
        // Let's test from E4 instead (square 28)
        move_data.from = Square(28); // E4

        // Value 2: -8 (up): E4 -> E5 (28 + (-8) = 20, which is E3, not E5...)
        // Actually, SCID uses different coordinate system. Let me check...
        // From SCID: up is -8, so E4 (28) + (-8) = 20, but that's wrong
        // Let me test with known good values first

        // Value 6: +7 (down-right diagonal): should work
        // legacy decoder removed
        let expected_square = 28 + 7; // Should be valid
        assert_eq!(move_data.to.0, expected_square as u8);

        // Value 7: +8 (down): E4 -> E3
        move_data.from = Square(28);
        // legacy decoder removed
        assert_eq!(move_data.to.0, 36); // 28 + 8 = 36
    }

    #[test]
    fn test_knight_moves() {
        // Test Knight move decoder
        // Knight at E4 (square 28)
        let mut move_data = create_test_move(28, PieceType::Knight, 2); // E4

        // Value 1: -17 (knight jump)
        // legacy decoder removed
        assert_eq!(move_data.to.0, 11); // 28 + (-17) = 11

        // Value 8: +17 (knight jump)
        move_data.from = Square(28);
        // legacy decoder removed
        assert_eq!(move_data.to.0, 45); // 28 + 17 = 45

        // Test invalid values
        move_data.from = Square(28);
        // legacy decoder removed
    }

    #[test]
    fn test_rook_moves() {
        // Test Rook move decoder
        // Rook at E4 (square 28)
        let mut move_data = create_test_move(28, PieceType::Rook, 1); // E4

        // Test horizontal move: value 0 = move to A4 (file 0)
        // legacy decoder removed
        // E4 = file 4, rank 3. Move to file 0, rank 3 = A4 = (3 << 3) | 0 = 24
        assert_eq!(move_data.to.0, 24);

        // Test vertical move: value 8 = move to E1 (rank 0)
        move_data.from = Square(28); // Reset
        // legacy decoder removed
        // E4 = file 4, rank 3. Move to file 4, rank 0 = E1 = (0 << 3) | 4 = 4
        assert_eq!(move_data.to.0, 4);

        // Test vertical move: value 15 = move to E8 (rank 7)
        move_data.from = Square(28);
        // legacy decoder removed
        // Move to file 4, rank 7 = E8 = (7 << 3) | 4 = 60
        assert_eq!(move_data.to.0, 60);
    }

    #[test]
    fn test_bishop_moves() {
        // Test Bishop move decoder
        // Bishop at E4 (square 28)
        let mut move_data = create_test_move(28, PieceType::Bishop, 3); // E4

        // Test diagonal move: value 0 (target file 0)
        // From E4 (file 4) to A-file (file 0): file_diff = 0 - 4 = -4
        // Since value < 8: target = from + 9 * file_diff = 28 + 9 * (-4) = 28 - 36 = -8
        // This should be invalid (negative), so let's test a valid move

        // Test value 7 (target file 7 = H-file)
        // file_diff = 7 - 4 = 3
        // target = 28 + 9 * 3 = 28 + 27 = 55 (valid)
        // legacy decoder removed
        assert_eq!(move_data.to.0, 55);

        // Test value 13 (target file 5 = F-file, direction >= 8)
        move_data.from = Square(28); // E4
                                     // file_diff = 5 - 4 = 1, target = 28 - 7 * 1 = 21 (D3)
        // legacy decoder removed
        assert_eq!(move_data.to.0, 21);
    }

    #[test]
    fn test_queen_rook_moves() {
        // Test Queen move decoder (rook-like moves only)
        // Queen at E4 (square 28)
        let mut move_data = create_test_move(28, PieceType::Queen, 4); // E4

        // Test horizontal move: value 0 = move to A4
        // legacy decoder removed
        assert_eq!(move_data.to.0, 24); // Same as rook test

        // Test vertical move: value 8 = move to E1
        move_data.from = Square(28);
        // legacy decoder removed
        assert_eq!(move_data.to.0, 4); // Same as rook test

        // Test diagonal move case (should fail - not implemented)
        move_data.from = Square(28);
        // When value == from_file, it's a diagonal move requiring 2 bytes
        // legacy decoder removed
    }

    #[test]
    fn test_scid_coordinate_system() {
        // Verify our understanding of SCID's coordinate system
        // square = (rank << 3) | file, where a1 = 0, h8 = 63

        // A1 = (0 << 3) | 0 = 0
        assert_eq!(0 & 0x7, 0); // file A
        assert_eq!((0 >> 3) & 0x7, 0); // rank 1

        // E1 = (0 << 3) | 4 = 4
        assert_eq!(4 & 0x7, 4); // file E
        assert_eq!((4 >> 3) & 0x7, 0); // rank 1

        // E4 = (3 << 3) | 4 = 28
        assert_eq!(28 & 0x7, 4); // file E
        assert_eq!((28 >> 3) & 0x7, 3); // rank 4

        // H8 = (7 << 3) | 7 = 63
        assert_eq!(63 & 0x7, 7); // file H
        assert_eq!((63 >> 3) & 0x7, 7); // rank 8
    }
}
