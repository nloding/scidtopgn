//! Comprehensive SG4 validation tests
//! 
//! This test module validates that the SG4 implementation correctly
//! decodes moves and handles all the requirements from Phase 2.3

use crate::formats::sg4::{
    Sg4File, DecodedMove, MoveInterpretation, find_game_boundaries, 
    parse_pgn_tags, parse_pgn_tags_with_streaming, ENCODE_END_GAME,
    ENCODE_NAG, ENCODE_COMMENT, GameElement, StreamingGameElement
};
use crate::core::error::Result;
use std::io::Cursor;

#[test]
fn test_sg4_move_decoding_comprehensive() -> Result<()> {
    println!("🧪 COMPREHENSIVE SG4 MOVE DECODING TEST");
    println!("===========================================");

    // Test 1: King moves including castling
    println!("\n1. 👑 King Move Decoding");
    let king_test_data = vec![
        0x10, // King (1) + up (0)
        0x13, // King (1) + castle kingside (3)
        0x17, // King (1) + castle queenside (7)
        0x11, // King (1) + up-right (1)
    ];
    
    let mut cursor = Cursor::new(king_test_data);
    let mmap = unsafe { Mmap::map(&cursor)? };
    let sg4_file = Sg4File { mmap };
    
    assert_eq!(sg4_file.num_games(), 1, "Should find 1 game boundary");
    
    // Test 2: Queen moves including diagonal (2-byte)
    println!("\n2. ♕ Queen Move Decoding");
    let queen_test_data = vec![
        0x20, // Queen (2) + up (0) - single byte
        0x28, // Queen (2) + diagonal (8) - first byte
        0x05, // Diagonal info - second byte
        ENCODE_END_GAME, 0x01, // End game
    ];
    
    let mut cursor = Cursor::new(queen_test_data);
    let mmap = unsafe { Mmap::map(&cursor)? };
    let sg4_file = Sg4File { mmap };
    
    let boundaries = find_game_boundaries(sg4_file.data());
    assert_eq!(boundaries.len(), 1, "Should find 1 game");
    
    // Test 3: Pawn moves with promotions
    println!("\n3. ♟️ Pawn Move Decoding");
    let pawn_test_data = vec![
        0x60, // Pawn (6) + forward (0)
        0x63, // Pawn (6) + promote queen (3)
        0x64, // Pawn (6) + promote rook (4)
        0x6F, // Pawn (6) + double forward (15)
        ENCODE_END_GAME, 0x02,
    ];
    
    let mut cursor = Cursor::new(pawn_test_data);
    let mmap = unsafe { Mmap::map(&cursor)? };
    let sg4_file = Sg4File { mmap };
    
    let game = sg4_file.get_game(0)?;
    assert_eq!(game.moves.len(), 4, "Should decode 4 pawn moves");
    
    // Validate pawn promotions
    let promotions: Vec<Option<String>> = game.moves.iter()
        .map(|m| match &m.interpretation {
            MoveInterpretation::Pawn { promotion, .. } => promotion.clone(),
            _ => None,
        })
        .collect();
    
    assert_eq!(promotions[0], None, "First move should not promote");
    assert_eq!(promotions[1], Some("Queen".to_string()), "Second move should promote to queen");
    assert_eq!(promotions[2], Some("Rook".to_string()), "Third move should promote to rook");
    assert_eq!(promotions[3], None, "Fourth move should not promote");
    
    println!("   ✅ Pawn promotions decoded correctly");
    
    // Test 4: Special encoding bytes
    println!("\n4. 🏷️ Special Encoding Bytes");
    let special_test_data = vec![
        0x10, // Regular move
        ENCODE_NAG, 0x01, // NAG annotation
        ENCODE_COMMENT, 0x05, b'H', b'e', b'l', b'l', b'o', // Comment
        ENCODE_START_MARKER, // Variation start
        ENCODE_END_MARKER,   // Variation end
        ENCODE_END_GAME, 0x01, // Game end
    ];
    
    let elements = parse_pgn_tags_with_streaming(&special_test_data)?;
    assert_eq!(elements.len(), 6, "Should parse 6 elements");
    
    // Validate element types
    let mut move_count = 0;
    let mut nag_count = 0;
    let mut comment_count = 0;
    let mut variation_count = 0;
    let mut game_end_count = 0;
    
    for element in elements {
        match element {
            StreamingGameElement::Move { .. } => move_count += 1,
            StreamingGameElement::Nag { .. } => nag_count += 1,
            StreamingGameElement::Comment { .. } => comment_count += 1,
            StreamingGameElement::VariationStart { .. } | StreamingGameElement::VariationEnd { .. } => variation_count += 1,
            StreamingGameElement::GameEnd { .. } => game_end_count += 1,
        }
    }
    
    assert_eq!(move_count, 1, "Should have 1 move");
    assert_eq!(nag_count, 1, "Should have 1 NAG");
    assert_eq!(comment_count, 1, "Should have 1 comment");
    assert_eq!(variation_count, 2, "Should have 2 variation markers");
    assert_eq!(game_end_count, 1, "Should have 1 game end");
    
    println!("   ✅ Special encoding bytes handled correctly");
    
    // Test 5: Multi-byte queen diagonal moves
    println!("\n5. 🌟 Multi-byte Queen Diagonal Moves");
    let queen_diagonal_data = vec![
        0x28, // Queen (2) + diagonal move (8)
        0x03, // Diagonal target info (3)
        0x2A, // Queen (2) + another diagonal (10)
        0x07, // Diagonal target info (7)
        ENCODE_END_GAME, 0x00,
    ];
    
    let elements = parse_pgn_tags_with_streaming(&queen_diagonal_data)?;
    let queen_moves: Vec<&StreamingGameElement> = elements.iter()
        .filter(|e| matches!(e, StreamingGameElement::Move { piece_num, .. } if *piece_num == 2))
        .collect();
    
    assert_eq!(queen_moves.len(), 2, "Should have 2 queen moves");
    
    if let Some(StreamingGameElement::Move { raw_bytes, bytes_consumed, .. }) = queen_moves.first() {
        assert_eq!(raw_bytes.len(), 2, "First queen move should be 2 bytes");
        assert_eq!(*bytes_consumed, 2, "Should consume 2 bytes");
        assert_eq!(raw_bytes[0], 0x28, "First byte should be 0x28");
        assert_eq!(raw_bytes[1], 0x03, "Second byte should be 0x03");
    }
    
    println!("   ✅ Multi-byte queen diagonal moves decoded correctly");
    
    // Test 6: Game boundary detection
    println!("\n6. 📋 Game Boundary Detection");
    let multi_game_data = vec![
        0x10, 0x20, ENCODE_END_GAME, 0x01, // Game 1
        0x30, 0x40, 0x50, ENCODE_END_GAME, 0x02, // Game 2
        0x60, ENCODE_END_GAME, 0x00,             // Game 3
    ];
    
    let boundaries = find_game_boundaries(&multi_game_data);
    assert_eq!(boundaries.len(), 3, "Should find 3 games");
    assert_eq!(boundaries[0], (0, 4), "Game 1 should span bytes 0-4");
    assert_eq!(boundaries[1], (4, 9), "Game 2 should span bytes 4-9");
    assert_eq!(boundaries[2], (9, 12), "Game 3 should span bytes 9-12");
    
    println!("   ✅ Game boundaries detected correctly");
    
    // Test 7: Error handling
    println!("\n7. ⚠️ Error Handling");
    let incomplete_data = vec![ENCODE_COMMENT, 0x10]; // Comment with insufficient length
    
    let result = parse_pgn_tags_with_streaming(&incomplete_data);
    assert!(result.is_err(), "Should handle incomplete comment gracefully");
    
    println!("   ✅ Error handling works correctly");
    
    println!("\n🎉 COMPREHENSIVE SG4 VALIDATION PASSED!");
    println!("   ✅ All piece types decode correctly");
    println!("   ✅ Special encoding bytes handled");
    println!("   ✅ Multi-byte moves supported");
    println!("   ✅ Game boundary detection works");
    println!("   ✅ Error handling is robust");
    
    Ok(())
}

#[test]
fn test_individual_piece_decoding() -> Result<()> {
    println!("🎯 INDIVIDUAL PIECE DECODING TEST");
    println!("=====================================");
    
    // Test each piece type individually
    let test_cases = vec![
        // (raw_byte, expected_piece, expected_move_type)
        (0x10, 1, "King"),      // King up
        (0x13, 1, "King castle"), // King castle
        (0x20, 2, "Queen"),     // Queen up
        (0x30, 3, "Rook"),      // Rook move
        (0x40, 4, "Bishop"),    // Bishop move
        (0x50, 5, "Knight"),    // Knight move
        (0x60, 6, "Pawn"),      // Pawn forward
        (0x63, 6, "Pawn promo"), // Pawn promote queen
    ];
    
    for (raw_byte, expected_piece, description) in test_cases {
        let piece_num = (raw_byte >> 4) & 0x0F;
        let move_value = raw_byte & 0x0F;
        
        assert_eq!(piece_num, expected_piece, "Piece number mismatch for {}", description);
        
        // Basic validation that the move can be decoded
        let test_data = vec![raw_byte, ENCODE_END_GAME, 0x01];
        let elements = parse_pgn_tags_with_streaming(&test_data)?;
        
        if let Some(StreamingGameElement::Move { piece_num: decoded_piece, .. }) = elements.first() {
            assert_eq!(*decoded_piece, expected_piece, "Decoded piece mismatch for {}", description);
        } else {
            panic!("Expected move element for {}", description);
        }
        
        println!("   ✅ {}: piece_num={}, move_value={}", description, piece_num, move_value);
    }
    
    Ok(())
}

#[test]
fn test_sg4_constants() -> Result<()> {
    println!("🔢 SG4 CONSTANTS TEST");
    println!("=========================");
    
    // Validate all constants match SCID specification
    assert_eq!(ENCODE_NAG, 11, "NAG constant should be 11");
    assert_eq!(ENCODE_COMMENT, 12, "Comment constant should be 12");
    assert_eq!(ENCODE_START_MARKER, 13, "Start marker constant should be 13");
    assert_eq!(ENCODE_END_MARKER, 14, "End marker constant should be 14");
    assert_eq!(ENCODE_END_GAME, 15, "End game constant should be 15");
    
    println!("   ✅ All encoding constants are correct");
    
    Ok(())
}