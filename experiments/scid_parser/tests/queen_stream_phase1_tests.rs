// Phase 1 validation tests for Queen stream decoder
// Testing the ByteStream and decode_queen_with_stream implementation

#[cfg(test)]
mod tests {
    use scid_parser::position::{ScidPosition, ScidMove, PieceType, Square, Color, ScidByteStream, decode_queen_with_stream};
    
    /// Helper function to create a test Queen move structure
    fn create_test_queen_move(from_square: u8) -> ScidMove {
        ScidMove {
            from: Square(from_square),
            to: Square(from_square), // Will be updated by decoder
            moving_piece: PieceType::Queen,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 4, // Assume Queen is piece 4 in standard position
        }
    }
    
    #[test]
    fn test_queen_vertical_move_with_stream() {
        // Test Case 1: Queen vertical move (1 byte) - should work same as before
        let move_data = []; // No additional bytes needed for vertical moves
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        // move_value = 15 (8 + 7) -> move to rank 7 (D8)
        let result = decode_queen_with_stream(15, &mut queen_move, &mut stream);
        
        assert!(result.is_ok(), "Vertical queen move should succeed");
        assert_eq!(queen_move.to, Square(59)); // D8 = (7 << 3) | 3 = 59
        assert_eq!(stream.position(), 0); // No bytes consumed from stream
    }
    
    #[test]
    fn test_queen_horizontal_move_with_stream() {
        // Test Case 2: Queen horizontal move (1 byte) - should work same as before
        let move_data = []; // No additional bytes needed for horizontal moves
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        // move_value = 7 -> move to file 7 (H4)
        let result = decode_queen_with_stream(7, &mut queen_move, &mut stream);
        
        assert!(result.is_ok(), "Horizontal queen move should succeed");
        assert_eq!(queen_move.to, Square(31)); // H4 = (3 << 3) | 7 = 31
        assert_eq!(stream.position(), 0); // No bytes consumed from stream
    }
    
    #[test]
    fn test_queen_diagonal_move_with_stream() {
        // Test Case 3: Queen diagonal move (2 bytes) - THE NEW FUNCTIONALITY
        // Queen at D4 (square 27, file 3), move_value = 3 (same as file) triggers diagonal
        let move_data = [118]; // Second byte: G7 = 54, so 54 + 64 = 118
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4 (file 3)
        
        // move_value = 3 (same as file) -> triggers diagonal mode
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        assert!(result.is_ok(), "Diagonal queen move should succeed: {:?}", result);
        assert_eq!(queen_move.to, Square(54)); // 118 - 64 = 54 (G7), valid diagonal D4->G7
        assert_eq!(stream.position(), 1); // One byte consumed from stream
    }
    
    #[test]
    fn test_queen_diagonal_move_valid_target() {
        // Test with a valid diagonal move: D4 -> G7
        let move_data = [118]; // Second byte: G7 = 54, so 54 + 64 = 118
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4 (square 27, file 3)
        
        // move_value = 3 triggers diagonal mode
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        assert!(result.is_ok(), "Valid diagonal queen move should succeed: {:?}", result);
        assert_eq!(queen_move.to, Square(54)); // 118 - 64 = 54 (G7)
        assert_eq!(stream.position(), 1); // One byte consumed
        
        // Verify it's actually a diagonal move (D4->G7: 3 files, 3 ranks)
        let from_file = 27 & 0x7; // D = 3
        let from_rank = (27 >> 3) & 0x7; // 4-1 = 3
        let to_file = 54 & 0x7; // G = 6  
        let to_rank = (54 >> 3) & 0x7; // 7-1 = 6
        
        assert_eq!((to_file as i8 - from_file as i8).abs(), 3); // 3 files
        assert_eq!((to_rank as i8 - from_rank as i8).abs(), 3); // 3 ranks
    }
    
    #[test]
    fn test_queen_diagonal_invalid_target_byte() {
        // Test with invalid second byte (outside range 64-127)
        let move_data = [63]; // Invalid: below 64
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        assert!(result.is_err(), "Invalid target byte should fail");
        assert!(result.unwrap_err().contains("Invalid Queen diagonal target byte"));
        assert_eq!(stream.position(), 1); // Byte was consumed before validation failed
    }
    
    #[test]
    fn test_queen_diagonal_invalid_target_byte_high() {
        // Test with invalid second byte (above 127)
        let move_data = [128]; // Invalid: above 127
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        assert!(result.is_err(), "Invalid high target byte should fail");
        assert!(result.unwrap_err().contains("Invalid Queen diagonal target byte"));
        assert_eq!(stream.position(), 1); // Byte was consumed before validation
    }
    
    #[test]
    fn test_queen_diagonal_buffer_underrun() {
        // Test with empty stream when diagonal move expected
        let move_data = []; // No bytes available
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        assert!(result.is_err(), "Buffer underrun should fail");
        assert!(result.unwrap_err().contains("Failed to read second byte"));
        assert_eq!(stream.position(), 0); // No bytes consumed due to immediate error
    }
    
    #[test]
    fn test_queen_diagonal_geometry_validation() {
        // Test move that has valid byte range but invalid diagonal geometry
        let move_data = [65]; // Valid range, target = square 1 (B1)
        let mut stream = ScidByteStream::new(&move_data);
        let mut queen_move = create_test_queen_move(27); // D4
        
        let result = decode_queen_with_stream(3, &mut queen_move, &mut stream);
        
        // This should fail because D4->B1 is not a valid diagonal
        // D4 = file 3, rank 3; B1 = file 1, rank 0
        // File diff = 2, rank diff = 3 -> not diagonal
        assert!(result.is_err(), "Invalid diagonal geometry should fail");
        assert!(result.unwrap_err().contains("Invalid Queen diagonal move geometry"));
    }
    
    #[test] 
    fn test_all_three_queen_cases_sequentially() {
        // Test that all three cases work correctly in sequence
        
        // Case 1: Vertical move
        let mut queen_move1 = create_test_queen_move(27); // D4
        let mut stream1 = ScidByteStream::new(&[]);
        assert!(decode_queen_with_stream(10, &mut queen_move1, &mut stream1).is_ok()); // move_value = 10 (8+2) -> D3
        
        // Case 2: Horizontal move  
        let mut queen_move2 = create_test_queen_move(27); // D4
        let mut stream2 = ScidByteStream::new(&[]);
        assert!(decode_queen_with_stream(0, &mut queen_move2, &mut stream2).is_ok()); // move_value = 0 -> A4
        
        // Case 3: Diagonal move
        let mut queen_move3 = create_test_queen_move(27); // D4
        let mut stream3 = ScidByteStream::new(&[118]); // G7
        assert!(decode_queen_with_stream(3, &mut queen_move3, &mut stream3).is_ok()); // move_value = 3 (same as file) -> diagonal
        
        // Verify stream consumption
        assert_eq!(stream1.position(), 0); // Vertical: no stream bytes
        assert_eq!(stream2.position(), 0); // Horizontal: no stream bytes  
        assert_eq!(stream3.position(), 1); // Diagonal: one stream byte consumed
    }
}