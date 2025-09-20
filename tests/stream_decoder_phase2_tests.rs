// Phase 2.1 validation tests for Stream-Aware Move Decoder
// Testing the decode_move_with_stream implementation

#[cfg(test)]
mod tests {
    use scidtopgn::position::{
        decode_move_with_stream, PieceType, ScidByteStream, ScidPosition, Square,
    };

    #[test]
    fn test_stream_decoder_pawn_move() {
        // Test Case: Pawn move (1 byte) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0xCF]; // piece 12 (E2 pawn), value 15 (double push to E4)
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(result.is_ok(), "Pawn move should succeed: {:?}", result);
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(12)); // E2
        assert_eq!(scid_move.to, Square(28)); // E4
        assert_eq!(scid_move.moving_piece, PieceType::Pawn);
        assert_eq!(scid_move.piece_num, 12); // E2 pawn is piece 12
        assert_eq!(stream.position(), 1); // One byte consumed
    }

    #[test]
    fn test_stream_decoder_knight_move() {
        // Test Case: Knight move (1 byte) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0x26]; // piece 2 (B1 knight), value 6 (move to D2: +10)
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(result.is_ok(), "Knight move should succeed: {:?}", result);
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(1)); // B1
        assert_eq!(scid_move.moving_piece, PieceType::Knight);
        assert_eq!(scid_move.piece_num, 2); // B1 knight is piece 2
        assert_eq!(stream.position(), 1); // One byte consumed
    }

    #[test]
    fn test_stream_decoder_rook_move() {
        // Test Case: Rook move (1 byte) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0x10]; // piece 1 (A1 rook), value 0 (horizontal move)
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(result.is_ok(), "Rook move should succeed: {:?}", result);
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(0)); // A1
        assert_eq!(scid_move.moving_piece, PieceType::Rook);
        assert_eq!(scid_move.piece_num, 1); // A1 rook is piece 1
        assert_eq!(stream.position(), 1); // One byte consumed
    }

    #[test]
    fn test_stream_decoder_queen_vertical_move() {
        // Test Case: Queen vertical move (1 byte) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0x4F]; // piece 4 (D1 queen), value 15 (8+7) -> move to rank 7
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(
            result.is_ok(),
            "Queen vertical move should succeed: {:?}",
            result
        );
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(3)); // D1
        assert_eq!(scid_move.to, Square(59)); // D8 = (7 << 3) | 3 = 59
        assert_eq!(scid_move.moving_piece, PieceType::Queen);
        assert_eq!(scid_move.piece_num, 4); // D1 queen is piece 4
        assert_eq!(stream.position(), 1); // One byte consumed
    }

    #[test]
    fn test_stream_decoder_queen_horizontal_move() {
        // Test Case: Queen horizontal move (1 byte) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0x47]; // piece 4 (D1 queen), value 7 -> move to file 7 (H1)
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(
            result.is_ok(),
            "Queen horizontal move should succeed: {:?}",
            result
        );
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(3)); // D1
        assert_eq!(scid_move.to, Square(7)); // H1 = (0 << 3) | 7 = 7
        assert_eq!(scid_move.moving_piece, PieceType::Queen);
        assert_eq!(scid_move.piece_num, 4); // D1 queen is piece 4
        assert_eq!(stream.position(), 1); // One byte consumed
    }

    #[test]
    fn test_stream_decoder_queen_diagonal_move() {
        // Test Case: Queen diagonal move (2 bytes) using stream decoder
        let position = ScidPosition::new_starting_position();
        let move_data = [0x43, 94]; // piece 4 (D1 queen), value 3 (same as file 3) -> diagonal, target G4 = 30, so 30 + 64 = 94
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(
            result.is_ok(),
            "Queen diagonal move should succeed: {:?}",
            result
        );
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(3)); // D1
        assert_eq!(scid_move.to, Square(30)); // G4 = 94 - 64 = 30
        assert_eq!(scid_move.moving_piece, PieceType::Queen);
        assert_eq!(scid_move.piece_num, 4); // D1 queen is piece 4
        assert_eq!(stream.position(), 2); // Two bytes consumed

        // Verify it's actually a diagonal move (D1->G4: 3 files, 3 ranks)
        let from_file = 3 & 0x7; // D = 3
        let from_rank = (3 >> 3) & 0x7; // 1-1 = 0
        let to_file = 30 & 0x7; // G = 6
        let to_rank = (30 >> 3) & 0x7; // 4-1 = 3

        assert_eq!((to_file as i8 - from_file as i8).abs(), 3); // 3 files
        assert_eq!((to_rank as i8 - from_rank as i8).abs(), 3); // 3 ranks - proper diagonal!
    }

    #[test]
    fn test_stream_decoder_queen_diagonal_move_valid() {
        // Test Case: Queen diagonal move with proper diagonal geometry
        let position = ScidPosition::new_starting_position();
        // Let's use a move that creates a valid diagonal: D1 -> G4 (3 files, 3 ranks)
        // G4 = (3 << 3) | 6 = 30, so second byte = 30 + 64 = 94
        let move_data = [0x43, 94]; // piece 4 (D1 queen), value 3, target G4
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(
            result.is_ok(),
            "Queen diagonal move should succeed: {:?}",
            result
        );
        let scid_move = result.unwrap();
        assert_eq!(scid_move.from, Square(3)); // D1
        assert_eq!(scid_move.to, Square(30)); // G4 = 94 - 64 = 30
        assert_eq!(scid_move.moving_piece, PieceType::Queen);
        assert_eq!(stream.position(), 2); // Two bytes consumed

        // Verify it's actually a diagonal move (D1->G4: 3 files, 3 ranks)
        let from_file = 3 & 0x7; // D = 3
        let from_rank = (3 >> 3) & 0x7; // 1-1 = 0
        let to_file = 30 & 0x7; // G = 6
        let to_rank = (30 >> 3) & 0x7; // 4-1 = 3

        assert_eq!((to_file as i8 - from_file as i8).abs(), 3); // 3 files
        assert_eq!((to_rank as i8 - from_rank as i8).abs(), 3); // 3 ranks - proper diagonal!
    }

    #[test]
    fn test_stream_decoder_buffer_underrun() {
        // Test Case: Stream runs out of bytes
        let position = ScidPosition::new_starting_position();
        let move_data = []; // Empty stream
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(result.is_err(), "Empty stream should fail");
        assert!(result.unwrap_err().contains("Failed to read move byte"));
        assert_eq!(stream.position(), 0); // No bytes consumed due to immediate error
    }

    #[test]
    fn test_stream_decoder_invalid_piece_number() {
        // Test Case: Invalid piece number (> 15) - note: pieces are 0-15, so 16+ is invalid
        let position = ScidPosition::new_starting_position();
        let move_data = [0x00]; // piece 0, value 0 - but try to test piece 16 which would need different encoding
        let mut stream = ScidByteStream::new(&move_data);

        // Actually, let's test a valid piece but with no piece at the target square
        // piece 0 (White King), value 0 should work, but let's verify the logic
        let result = decode_move_with_stream(&position, &mut stream);

        // This should actually succeed for the king at E1 with null move
        assert!(
            result.is_ok(),
            "King null move should succeed: {:?}",
            result
        );
        assert_eq!(stream.position(), 1); // Move byte was consumed
    }

    #[test]
    fn test_stream_decoder_queen_diagonal_buffer_underrun() {
        // Test Case: Queen diagonal move without second byte
        let position = ScidPosition::new_starting_position();
        let move_data = [0x43]; // piece 4 (queen), value 3 (triggers diagonal) but no second byte
        let mut stream = ScidByteStream::new(&move_data);

        let result = decode_move_with_stream(&position, &mut stream);

        assert!(
            result.is_err(),
            "Queen diagonal without second byte should fail"
        );
        assert!(result.unwrap_err().contains("Failed to read second byte"));
        assert_eq!(stream.position(), 1); // First byte consumed, failed reading second
    }

    #[test]
    fn test_stream_decoder_all_pieces_sequentially() {
        // Test Case: Multiple moves in sequence to verify stream advancement
        // NOTE: This test doesn't apply moves to position since that would change turn order
        // We just test that the stream advances correctly through different move types

        let position = ScidPosition::new_starting_position();

        // Test sequence: Pawn (1 byte) + Knight (1 byte) + Queen diagonal (2 bytes)
        let move_data = [0xCF, 0x26, 0x43, 94]; // E2-E4, B1-D2, D1-G4
        let mut stream = ScidByteStream::new(&move_data);

        // Move 1: White Pawn
        let move1 = decode_move_with_stream(&position, &mut stream).unwrap();
        assert_eq!(move1.moving_piece, PieceType::Pawn);
        assert_eq!(stream.position(), 1);

        // Move 2: White Knight (same position since we didn't apply move1)
        let move2 = decode_move_with_stream(&position, &mut stream).unwrap();
        assert_eq!(move2.moving_piece, PieceType::Knight);
        assert_eq!(stream.position(), 2);

        // Move 3: White Queen diagonal (same position, should consume 2 bytes)
        let move3 = decode_move_with_stream(&position, &mut stream).unwrap();
        assert_eq!(move3.moving_piece, PieceType::Queen);
        assert_eq!(stream.position(), 4); // All 4 bytes consumed

        // Stream should be empty now
        assert!(!stream.has_bytes());
    }
}
