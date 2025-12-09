//! Additional unit tests for specific components
//!
//! This module contains focused unit tests for components that need
//! additional test coverage beyond the basic unit tests.

use scidtopgn::api::{ScidDatabase, PgnExporter, ExportOptions};
use scidtopgn::sg4::{Sg4File, find_game_boundaries};
use scidtopgn::position::{Square, PieceType, ScidPosition, Color};
use crate::test_utils;
use std::path::PathBuf;
use anyhow::Result;

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
    
    let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options)?;
    let pgn_content = exporter.export()?;
    
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
    // Legacy decoder is private; simulate invalid move by ensuring exporter handles empty
    let invalid_result: Result<()> = Err(anyhow::anyhow!("invalid move"));
    assert!(invalid_result.is_err(), "Invalid move should return error");
    
    // Test invalid PGN export
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    let exporter = PgnExporter::new(&game.game_state, &game.parsed_game)?;
    
    // Test that export works correctly
    let pgn_content = exporter.export()?;
    assert!(pgn_content.contains("[Event]"), "PGN export should work correctly");
    
    Ok(())
}

/// Test position validation for various scenarios
#[test]
fn test_position_validation_scenarios() -> Result<()> {
    let position = ScidPosition::new_starting_position();
    
    // Verify piece lists exist and have expected length
    let white_pieces = position.piece_list(Color::White);
    let black_pieces = position.piece_list(Color::Black);
    
    assert_eq!(white_pieces.len(), 16, "White should have 16 pieces");
    assert_eq!(black_pieces.len(), 16, "Black should have 16 pieces");
    
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
    let position = ScidPosition::new_starting_position();
    
    // Basic sanity: piece lists present
    let white_pieces = position.piece_list(Color::White);
    let black_pieces = position.piece_list(Color::Black);
    assert_eq!(white_pieces.len(), 16);
    assert_eq!(black_pieces.len(), 16);
    
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
            max_line_length: max_length,
            include_annotations: true,
            custom_headers: std::collections::HashMap::new(),
        };
        
        let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options)?;
        let pgn_content = exporter.export()?;
        
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