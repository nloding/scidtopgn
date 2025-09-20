#[cfg(test)]
mod tests {
    use crate::position::decoder::decode_move;
    use crate::position::{Color, PieceType, ScidPosition, Square};

    #[test]
    fn test_scid_piece_numbering() {
        let pos = ScidPosition::new_starting_position();

        // Verify SCID piece numbering - VERIFICATION CRITERIA from plan
        assert_eq!(pos.piece_list(Color::White)[0], Square(4)); // King at E1
        assert_eq!(pos.piece_list(Color::White)[12], Square(12)); // E2 pawn - KEY TEST

        // Verify piece types
        assert_eq!(pos.piece_at(Square(4)), Some(PieceType::King)); // E1 king
        assert_eq!(pos.piece_at(Square(12)), Some(PieceType::Pawn)); // E2 pawn
    }

    #[test]
    fn test_square_conversions() {
        // Test square number conversions - VERIFICATION CRITERIA from plan
        assert_eq!(Square::from_algebraic("e4").unwrap(), Square(28));
        assert_eq!(Square(12).to_algebraic(), "e2");
        assert_eq!(Square(28).to_algebraic(), "e4");
    }

    #[test]
    fn test_cf_byte_decodes_to_e4() {
        let pos = ScidPosition::new_starting_position();

        // This is THE CRITICAL TEST - CF should decode to e4, not "en passant"
        let scid_move = decode_move(&pos, 0xCF).unwrap();

        // Verify the move components
        assert_eq!(scid_move.from, Square(12)); // E2
        assert_eq!(scid_move.to, Square(28)); // E4
        assert_eq!(scid_move.moving_piece, PieceType::Pawn);
        assert_eq!(scid_move.piece_num, 12);

        // Verify algebraic notation
        assert_eq!(scid_move.to_algebraic(&pos), "e4");
    }

    #[test]
    fn test_move_byte_extraction() {
        // Test extraction from CF byte - VERIFICATION CRITERIA from plan
        let move_byte = 0xCF;
        let piece_num = (move_byte >> 4) as usize; // Should be 12
        let move_value = move_byte & 0x0F; // Should be 15

        assert_eq!(piece_num, 12);
        assert_eq!(move_value, 15);
    }

    #[test]
    fn test_pawn_double_push_calculation() {
        // Verify the mathematical calculation: E2 + 16 = E4
        let e2_square = 12; // E2 square number
        let target_square = e2_square + 16; // Add 16 for double push
        let e4_square = 28; // E4 square number

        assert_eq!(target_square, e4_square);

        // Verify via Square conversion
        assert_eq!(Square(12).to_algebraic(), "e2");
        assert_eq!(Square(28).to_algebraic(), "e4");
    }
}
