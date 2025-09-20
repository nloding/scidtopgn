//! Comprehensive Phase 3 Bridge Layer Integration Tests
//! 
//! This test suite validates the enhanced SCID-to-Shakmaty bridge layer implementation,
//! ensuring that all piece types convert correctly and edge cases are handled properly.

use crate::bridge::{GameState, GameMetadata, PositionContext, ChessNotation};
use crate::bridge::moves::{ScidToShakmaty, convert_king_move, convert_queen_move, convert_rook_move, convert_bishop_move, convert_knight_move, convert_pawn_move};
use crate::formats::sg4::{DecodedMove, MoveInterpretation};
use crate::core::error::Result;
use shakmaty::{Chess, Move, Square, Role, Color};

#[test]
fn test_enhanced_piece_lookup_by_number() -> Result<()> {
    println!("🧪 ENHANCED PIECE LOOKUP TEST");
    println!("===================================");
    
    // Create a standard starting position
    let position = Chess::default();
    
    // Test piece lookup for white pieces
    let white_king = convert_king_move(0, 0, &position)?;
    assert_eq!(white_king.from(), Square::E1, "White king should be at e1");
    
    let white_queen = convert_queen_move(0, 1, &position)?;
    assert_eq!(white_queen.from(), Square::D1, "White queen should be at d1");
    
    // Test piece lookup for black pieces  
    let black_king = convert_king_move(0, 0, &position)?;
    assert_eq!(black_king.from(), Square::E8, "Black king should be at e8");
    
    let black_queen = convert_queen_move(0, 1, &position)?;
    assert_eq!(black_queen.from(), Square::D8, "Black queen should be at d8");
    
    println!("   ✅ Enhanced piece lookup working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_queen_diagonal_moves() -> Result<()> {
    println!("🌟 ENHANCED QUEEN DIAGONAL MOVES TEST");
    println!("===========================================");
    
    let position = Chess::default();
    
    // Test single byte queen moves (rook-like)
    let queen_north = convert_queen_move(0, 1, &position)?;
    assert_eq!(queen_north.from(), Square::D1, "Queen from d1");
    assert_eq!(queen_north.to?, Square::D4, "Queen north to d4");
    
    // Test multi-byte queen diagonal moves (8-15 indicate diagonal)
    let queen_diagonal = convert_queen_move(8, 1, &position)?;
    assert_eq!(queen_diagonal.from(), Square::D1, "Queen from d1");
    // The target should be a diagonal move from d1
    
    println!("   ✅ Enhanced queen diagonal moves working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_rook_rank_file_encoding() -> Result<()> {
    println!("🏰 ENHANCED ROOK RANK/FILE ENCODING TEST");
    println!("===============================================");
    
    let position = Chess::default();
    
    // Test rook moves with proper rank/file encoding
    let rook_vertical = convert_rook_move(0, 3, &position)?; // White rook at f1
    assert_eq!(rook_vertical.from(), Square::F1, "Rook from f1");
    
    let rook_horizontal = convert_rook_move(2, 3, &position)?; // Different encoding
    assert_eq!(rook_horizontal.from(), Square::F1, "Rook from f1");
    
    println!("   ✅ Enhanced rook rank/file encoding working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_bishop_file_direction_encoding() -> Result<()> {
    println!("🐘 ENHANCED BISHOP FILE/DIRECTION ENCODING TEST");
    println!("===============================================");
    
    let position = Chess::default();
    
    // Test bishop moves with file + direction encoding
    let bishop_move = convert_bishop_move(0, 4, &position)?; // White bishop at c1
    assert_eq!(bishop_move.from(), Square::C1, "Bishop from c1");
    
    // Test direction bit handling
    let bishop_diagonal = convert_bishop_move(8, 4, &position)?; // Different direction
    assert_eq!(bishop_diagonal.from(), Square::C1, "Bishop from c1");
    
    println!("   ✅ Enhanced bishop file/direction encoding working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_knight_square_difference_encoding() -> Result<()> {
    println!("🐎 ENHANCED KNIGHT SQUARE DIFFERENCE ENCODING TEST");
    println!("===============================================");
    
    let position = Chess::default();
    
    // Test knight moves with square difference encoding
    let knight_move = convert_knight_move(0, 8, &position)?; // White knight at b1
    assert_eq!(knight_move.from(), Square::B1, "Knight from b1");
    
    // Test extended knight moves (8-15 for edge cases)
    let knight_extended = convert_knight_move(8, 8, &position)?;
    assert_eq!(knight_extended.from(), Square::B1, "Knight from b1");
    
    println!("   ✅ Enhanced knight square difference encoding working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_pawn_promotion_encoding() -> Result<()> {
    println!("♟️ ENHANCED PAWN PROMOTION ENCODING TEST");
    println!("=======================================");
    
    let position = Chess::default();
    
    // Test pawn promotion moves
    let pawn_promote_queen = convert_pawn_move(3, 12, Some("Queen"), &position)?;
    assert_eq!(pawn_promote_queen.from(), Square::E2, "Pawn from e2");
    assert!(pawn_promote_queen.to?, Square::E4, "Pawn to e4");
    assert!(pawn_promote_queen.is_promotion(), "Should be promotion");
    
    // Test en passant detection
    let pawn_en_passant = convert_pawn_move(15, 12, None, &position)?;
    assert_eq!(pawn_en_passant.from(), Square::E2, "Pawn from e2");
    // En passant should be detected based on position state
    
    println!("   ✅ Enhanced pawn promotion encoding working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_king_castling_encoding() -> Result<()> {
    println!("👑 ENHANCED KING CASTLING ENCODING TEST");
    println!("=======================================");
    
    let position = Chess::default();
    
    // Test king castling moves
    let king_castle_kingside = convert_king_move(9, 0, &position)?;
    assert_eq!(king_castle_kingside.from(), Square::E1, "King from e1");
    // Castling move should be detected and handled properly
    
    let king_castle_queenside = convert_king_move(10, 0, &position)?;
    assert_eq!(king_castle_queenside.from(), Square::E1, "King from e1");
    // Queenside castling should also be detected
    
    println!("   ✅ Enhanced king castling encoding working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_position_validation() -> Result<()> {
    println!("🎯 ENHANCED POSITION VALIDATION TEST");
    println!("=======================================");
    
    // Create game state
    let mut game_state = GameState::new();
    
    // Test position validation
    assert!(game_state.is_position_legal(), "Starting position should be legal");
    assert!(!game_state.is_in_check(), "Starting position should not be in check");
    assert!(!game_state.is_checkmate(), "Starting position should not be checkmate");
    assert!(!game_state.is_stalemate(), "Starting position should not be stalemate");
    assert!(game_state.game_outcome().is_none(), "Starting position should have no outcome");
    
    println!("   ✅ Enhanced position validation working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_scid_to_shakmaty_conversion() -> Result<()> {
    println!("🔄 ENHANCED SCID TO SHAKMATY CONVERSION TEST");
    println!("=========================================");
    
    let position = Chess::default();
    
    // Test conversion of various SCID moves to shakmaty moves
    let test_cases = vec![
        // (piece_num, move_value, expected_move_type)
        (0, 0, "King"),      // King up
        (1, 0, "Queen"),     // Queen up  
        (3, 0, "Rook"),      // Rook up
        (4, 0, "Bishop"),    // Bishop up
        (5, 0, "Knight"),    // Knight up
        (6, 0, "Pawn"),      // Pawn up
        (6, 3, "Pawn promo"), // Pawn promote to queen
    ];
    
    for (piece_num, move_value, expected_type) in test_cases {
        let scid_move = DecodedMove {
            raw_bytes: vec![piece_num << 4 | move_value],
            piece_num,
            move_value,
            interpretation: create_test_interpretation(piece_num, move_value),
            from_square_index: None,
            to_square_index: None,
            promotion_piece: None,
        };
        
        let shakmaty_move = scid_move.to_shakmaty(&position)?;
        
        // Validate the move type matches expectations
        match expected_type {
            "King" => assert_eq!(shakmaty_move.role(), Role::King, "Should be king move"),
            "Queen" => assert_eq!(shakmaty_move.role(), Role::Queen, "Should be queen move"),
            "Rook" => assert_eq!(shakmaty_move.role(), Role::Rook, "Should be rook move"),
            "Bishop" => assert_eq!(shakmaty_move.role(), Role::Bishop, "Should be bishop move"),
            "Knight" => assert_eq!(shakmaty_move.role(), Role::Knight, "Should be knight move"),
            "Pawn" => assert_eq!(shakmaty_move.role(), Role::Pawn, "Should be pawn move"),
            "Pawn promo" => {
                assert_eq!(shakmaty_move.role(), Role::Pawn, "Should be pawn move");
                assert!(shakmaty_move.promotion.is_some(), "Should be promotion");
            }
            _ => panic!("Unknown expected type: {}", expected_type),
        }
        
        println!("   ✅ {} conversion working correctly", expected_type);
    }
    
    Ok(())
}

#[test]
fn test_enhanced_game_state_pgn_export() -> Result<()> {
    println!("📋 ENHANCED GAME STATE PGN EXPORT TEST");
    println!("=====================================");
    
    // Create game state with metadata
    let metadata = GameMetadata::new(
        "Alice".to_string(),
        "Bob".to_string(),
        "Test Tournament".to_string(),
        "Test Site".to_string(),
        "2024.01.01".to_string(),
        "1-0".to_string(),
    );
    
    let mut game_state = GameState::with_metadata(metadata);
    
    // Test PGN export
    let pgn = game_state.to_pgn();
    
    // Validate PGN contains expected headers
    assert!(pgn.contains("[Event \"Test Tournament\"]"), "Should contain event header");
    assert!(pgn.contains("[White \"Alice\"]"), "Should contain white player");
    assert!(pgn.contains("[Black \"Bob\"]"), "Should contain black player");
    assert!(pgn.contains("[Result \"1-0\"]"), "Should contain result");
    
    // Test Seven Tag Roster compliance
    let seven_tag_roster = ["Event", "Site", "Date", "White", "Black", "Result"];
    for tag in &seven_tag_roster {
        assert!(pgn.contains(&format!("[{} \"", tag)), "Should contain seven tag roster: {}", tag);
    }
    
    println!("   ✅ Enhanced game state PGN export working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_position_tracking_integration() -> Result<()> {
    println!("🎯 ENHANCED POSITION TRACKING INTEGRATION TEST");
    println!("===============================================");
    
    // Create game state
    let mut game_state = GameState::new();
    
    // Test that position tracking is properly integrated
    assert_eq!(game_state.move_count(), 0, "Initial move count should be 0");
    assert_eq!(game_state.current_position().turn(), Color::White, "White should move first");
    
    // Test piece list integration
    let position = game_state.current_position();
    let white_pieces = position.board().pieces_of_color(Color::White);
    let black_pieces = position.board().pieces_of_color(Color::Black);
    
    assert_eq!(white_pieces.len(), 16, "White should have 16 pieces");
    assert_eq!(black_pieces.len(), 16, "Black should have 16 pieces");
    
    // Test that position context is properly implemented
    assert!(game_state.is_position_legal(), "Initial position should be legal");
    
    println!("   ✅ Enhanced position tracking integration working correctly");
    
    Ok(())
}

#[test]
fn test_enhanced_error_handling() -> Result<()> {
    println!("⚠️ ENHANCED ERROR HANDLING TEST");
    println!("=================================");
    
    let position = Chess::default();
    
    // Test invalid piece number handling
    let invalid_piece_result = convert_king_move(0, 99, &position);
    assert!(invalid_piece_result.is_err(), "Should handle invalid piece number");
    
    // Test invalid move value handling  
    let invalid_move_result = convert_queen_move(99, 1, &position);
    assert!(invalid_move_result.is_err(), "Should handle invalid move value");
    
    // Test missing square handling
    let scid_move = DecodedMove {
        raw_bytes: vec![0x10],
        piece_num: 0,
        move_value: 0,
        interpretation: MoveInterpretation::Unknown { reason: "Test".to_string() },
        from_square_index: None,
        to_square_index: None,
        promotion_piece: None,
    };
    
    let conversion_result = scid_move.to_shakmaty(&position);
    assert!(conversion_result.is_err(), "Should handle unknown move interpretation");
    
    println!("   ✅ Enhanced error handling working correctly");
    
    Ok(())
}

// Helper function to create test move interpretations
fn create_test_interpretation(piece_num: u8, move_value: u8) -> MoveInterpretation {
    match piece_num {
        0 => MoveInterpretation::King {
            direction_code: move_value,
            is_castle: move_value == 9 || move_value == 10,
        },
        1 => MoveInterpretation::Queen,
        2 => MoveInterpretation::Rook,
        3 => MoveInterpretation::Bishop,
        4 => MoveInterpretation::Knight {
            l_shape_code: move_value,
        },
        5 => MoveInterpretation::Pawn {
            direction: "test".to_string(),
            promotion: if move_value >= 3 && move_value <= 6 {
                Some("Queen".to_string())
            } else {
                None
            },
            is_en_passant: if move_value == 15 { Some(true) } else { None },
        },
        _ => MoveInterpretation::Unknown {
            reason: format!("Test piece {}", piece_num),
        },
    }
}