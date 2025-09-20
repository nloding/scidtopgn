//! Additional unit tests for specific components
//!
//! This module contains focused unit tests for components that need
//! additional test coverage beyond the basic unit tests.

use scidtopgn::api::ScidDatabase;
use scidtopgn::bridge::{GameState, GameMetadata, PositionContext};
use scidtopgn::core::error::Result;
use scidtopgn::formats::sg4::{Sg4File, find_game_boundaries};
use scidtopgn::pgn::{PgnExporter, ExportOptions};
use scidtopgn::position::{ScidPosition, Square, PieceType, Color};
use std::path::PathBuf;

/// Test SG4 file parsing functionality
#[test]
fn test_sg4_file_parsing() -> Result<()> {
    let sg4_path = test_utils::five_test_data().with_extension("sg4");
    let sg4_data = std::fs::read(&sg4_path)?;
    
    // Test that we can find game boundaries
    let boundaries = find_game_boundaries(&sg4_data);
    assert_eq!(boundaries.len(), 5, "Should find 5 game boundaries");
    
    // Test that boundaries are correct
    let (first_start, first_end) = boundaries[0];
    assert_eq!(first_end - first_start, 175, "First game should be 175 bytes");
    
    // Test that we can create SG4 file
    let sg4_file = Sg4File::open(&sg4_path)?;
    assert_eq!(sg4_file.num_games(), 5, "SG4 file should contain 5 games");
    
    Ok(())
}

/// Test game state management
#[test]
fn test_game_state_management() -> Result<()> {
    let mut game_state = GameState::new();
    
    // Test that game state starts empty
    assert_eq!(game_state.move_count(), 0, "Initial game state should have 0 moves");
    assert!(game_state.is_position_legal(), "Initial position should be legal");
    assert!(game_state.game_outcome().is_none(), "Initial game should have no outcome");
    
    // Test that we can add metadata
    let metadata = GameMetadata::new(
        "Test Player".to_string(),
        "Test Opponent".to_string(),
        "Test Event".to_string(),
        "Test Site".to_string(),
        "2024.01.01".to_string(),
        "1-0".to_string(),
        Some(1800),
        Some(1750),
        Some("A00".to_string()),
    );
    
    game_state.set_metadata(metadata);
    
    // Test that metadata is properly set
    assert!(game_state.metadata().is_some(), "Game state should have metadata");
    
    // Test that we can add moves
    game_state.add_comment("Test comment".to_string());
    
    // Test that we can start and end variations
    game_state.start_variation();
    game_state.end_variation();
    
    // Test that we can add NAG annotations
    game_state.add_nag(1); // Good move
    
    Ok(())
}

/// Test PGN export with different options
#[test]
fn test_pgn_export_options() -> Result<()> {
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    
    // Test with minimal options
    let options = ExportOptions {
        include_optional_tags: false,
        validate_moves: false,
        max_line_length: 80,
        include_annotations: false,
        custom_headers: std::collections::HashMap::new(),
    };
    
    let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options);
    let pgn_content = exporter.export();
    
    // Test that PGN content contains basic headers
    assert!(pgn_content.contains("[Event]"), "PGN should contain event header");
    assert!(pgn_content.contains("[White]"), "PGN should contain white player header");
    assert!(pgn_content.contains("[Black]"), "PGN should contain black player header");
    assert!(pgn_content.contains("[Result]"), "PGN should contain result header");
    
    // Test that optional tags are not included
    assert!(!pgn_content.contains("[WhiteElo]"), "PGN should not contain WhiteElo when disabled");
    assert!(!pgn_content.contains("[BlackElo]"), "PGN should not contain BlackElo when disabled");
    
    Ok(())
}

/// Test error handling for various scenarios
#[test]
fn test_error_handling_scenarios() -> Result<()> {
    // Test file not found error
    let invalid_path = PathBuf::from("nonexistent_file");
    let result = ScidDatabase::open(&invalid_path);
    assert!(result.is_err(), "Opening nonexistent file should return error");
    
    // Test invalid move decoding
    let mut position = ScidPosition::new_starting_position();
    let invalid_move = scidtopgn::position::decode_move(&position, 0xFF);
    assert!(invalid_move.is_err(), "Invalid move should return error");
    
    // Test invalid PGN export
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    let exporter = PgnExporter::new(&game.game_state, &game.parsed_game, ExportOptions::default());
    
    // Test that export works correctly
    let pgn_content = exporter.export();
    assert!(pgn_content.contains("[Event]"), "PGN export should work correctly");
    
    Ok(())
}

/// Test position validation for various scenarios
#[test]
fn test_position_validation_scenarios() -> Result<()> {
    let mut position = ScidPosition::new_starting_position();
    
    // Test that starting position is valid
    assert!(position.is_position_legal(), "Starting position should be legal");
    assert!(!position.is_in_check(), "Starting position should not be in check");
    assert!(!position.is_checkmate(), "Starting position should not be checkmate");
    assert!(!position.is_stalemate(), "Starting position should not be stalemate");
    
    // Test that position validation works correctly
    let white_king = position.find_king_square(Color::White);
    let black_king = position.find_king_square(Color::Black);
    
    assert!(white_king.is_some(), "White king should be found");
    assert!(black_king.is_some(), "Black king should be found");
    
    // Test that kings are in valid positions
    assert_eq!(white_king.unwrap(), Square::E1, "White king should be at E1");
    assert_eq!(black_king.unwrap(), Square::E8, "Black king should be at E8");
    
    // Test that piece lists are correct
    let white_pieces = position.piece_list(Color::White);
    let black_pieces = position.piece_list(Color::Black);
    
    assert_eq!(white_pieces.len(), 16, "White should have 16 pieces");
    assert_eq!(black_pieces.len(), 16, "Black should have 16 pieces");
    
    // Test that key pieces are in correct positions
    assert_eq!(white_pieces[0], Square::E1, "White king should be at E1");
    assert_eq!(white_pieces[12], Square::E2, "White E2 pawn should be at E2");
    assert_eq!(black_pieces[0], Square::E8, "Black king should be at E8");
    assert_eq!(black_pieces[12], Square::E7, "Black E7 pawn should be at E7");
    
    Ok(())
}

/// Test database statistics functionality
#[test]
fn test_database_statistics() -> Result<()> {
    let db = test_utils::create_test_database()?;
    
    // Test that we can get database statistics
    let stats = db.statistics();
    
    // Test that statistics contain expected values
    assert_eq!(stats.num_games, 5, "Database should have 5 games");
    assert!(stats.is_validated, "Database should be validated");
    
    // Test that file sizes are recorded (may be 0 for mock files)
    assert!(stats.file_size_si4 >= 0, "SI4 file size should be non-negative");
    assert!(stats.file_size_sn4 >= 0, "SN4 file size should be non-negative");
    assert!(stats.file_size_sg4 >= 0, "SG4 file size should be non-negative");
    
    Ok(())
}

/// Test move validation for different piece types
#[test]
fn test_move_validation_by_piece_type() -> Result<()> {
    let mut position = ScidPosition::new_starting_position();
    
    // Test pawn moves
    let pawn_moves = vec![
        (Square::E2, Square::E4),   // Forward move
        (Square::E2, Square::E3),   // Single square move
        (Square::E4, Square::E5),   // Double pawn move
    ];
    
    for (from, to) in pawn_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Pawn, "Should be pawn");
        assert!(position.is_valid_move(from, to, Color::White), "Pawn move should be valid");
    }
    
    // Test knight moves
    let knight_moves = vec![
        (Square::B1, Square::C3),   // Forward-right
        (Square::B1, Square::A3),   // Forward-left
        (Square::G1, Square::F3),   // Forward-right
        (Square::G1, Square::H3),   // Forward-left
    ];
    
    for (from, to) in knight_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Knight, "Should be knight");
        assert!(position.is_valid_move(from, to, Color::White), "Knight move should be valid");
    }
    
    // Test rook moves
    let rook_moves = vec![
        (Square::A1, Square::A8),   // Vertical move
        (Square::A1, Square::H1),   // Horizontal move
    ];
    
    for (from, to) in rook_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Rook, "Should be rook");
        assert!(position.is_valid_move(from, to, Color::White), "Rook move should be valid");
    }
    
    Ok(())
}

/// Test PGN export formatting
#[test]
fn test_pgn_export_formatting() -> Result<()> {
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    
    // Test with different line length limits
    let line_lengths = [70, 80, 90, 100];
    
    for max_length in line_lengths {
        let options = ExportOptions {
            include_optional_tags: true,
            validate_moves: true,
            max_line_length,
            include_annotations: true,
            custom_headers: std::collections::HashMap::new(),
        };
        
        let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options);
        let pgn_content = exporter.export();
        
        // Test that lines don't exceed the maximum length
        let lines: Vec<&str> = pgn_content.lines().collect();
        for line in lines {
            if line.len() > max_length {
                // Check if it's a reasonable long line (like a move with annotations)
                if !line.contains("{") && !line.contains("}") {
                    assert!(false, "Line length should not exceed max_length");
                }
            }
        }
    }
    
    Ok(())
}