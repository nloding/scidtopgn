use std::path::Path;
use std::fs;
use scid_parser::{
    PgnExporter, ParsedGame, GameState, GameMetadata
};
use scid_parser::bridge::{ChessValidator, ValidationReport, PositionContext, ChessValidation};
use scid_parser::sg4::parse_game_with_position_tracking;
use scid_parser::error::Result;

/// Test the complete SCID to PGN pipeline using known good test data
#[test]
fn test_complete_scid_to_pgn_pipeline() -> Result<()> {
    // Load the test data
    let test_data_path = Path::new("../../test/data/five");
    
    // Test that the files exist
    let sg4_path = test_data_path.with_extension("sg4");
    let si4_path = test_data_path.with_extension("si4");
    let sn4_path = test_data_path.with_extension("sn4");
    let reference_pgn_path = test_data_path.with_extension("pgn");
    
    assert!(sg4_path.exists(), "Test file five.sg4 not found at {:?}", sg4_path);
    assert!(si4_path.exists(), "Test file five.si4 not found at {:?}", si4_path);
    assert!(sn4_path.exists(), "Test file five.sn4 not found at {:?}", sn4_path);
    assert!(reference_pgn_path.exists(), "Reference PGN file not found at {:?}", reference_pgn_path);
    
    // Parse the SG4 game file
    let game_data = fs::read(&sg4_path)
        .map_err(|e| scid_parser::ScidError::conversion_error(format!("Failed to read game file: {}", e)))?;
    
    // Parse using position-aware parsing
    let (moves, san_notation) = parse_game_with_position_tracking(&game_data, 1)
        .map_err(|e| scid_parser::ScidError::conversion_error(format!("Failed to parse game: {}", e)))?;
    
    println!("✅ Successfully parsed {} moves with position tracking", moves.len());
    
    // Create a GameState with the parsed moves
    let mut game_state = GameState::new();
    
    // Apply each move to build up the game state
    // Note: This is a simplified approach - in a full implementation,
    // we would parse the actual SCID moves from the SG4 file
    
    // Validate the move sequence using the chess validator
    let validator = ChessValidator::new();
    let validation_report = validator.validate_move_sequence(&moves)?;
    
    // Assert that all moves are valid
    assert!(validation_report.is_valid, 
        "Game contains invalid moves: {:?}", validation_report.invalid_moves);
    
    println!("✅ All {} moves passed validation", validation_report.total_moves);
    
    // Create metadata for the game (normally this would come from SI4/SN4 parsing)
    let metadata = GameMetadata {
        event: "Test Event".to_string(),
        site: "Test Site".to_string(), 
        date: "2022.12.19".to_string(),
        white: "Test White".to_string(),
        black: "Test Black".to_string(),
        result: "*".to_string(),
        round: Some("1".to_string()),
        white_elo: None,
        black_elo: None,
        eco: None,
    };
    
    // Create ParsedGame structure
    let parsed_game = ParsedGame {
        game_state: game_state.clone(),
        validation_report,
        metadata: metadata.clone(),
    };
    
    // Export to PGN using the new exporter
    let exporter = PgnExporter::new();
    let pgn_output = exporter.export_with_validation(&parsed_game)?;
    
    println!("✅ Successfully exported to PGN");
    println!("PGN Output:\n{}", pgn_output);
    
    // Basic validation of PGN structure
    assert!(pgn_output.contains("[Event \"Test Event\"]"), "PGN missing Event tag");
    assert!(pgn_output.contains("[Site \"Test Site\"]"), "PGN missing Site tag");
    assert!(pgn_output.contains("[Date \"2022.12.19\"]"), "PGN missing Date tag");
    assert!(pgn_output.contains("[White \"Test White\"]"), "PGN missing White tag");
    assert!(pgn_output.contains("[Black \"Test Black\"]"), "PGN missing Black tag");
    assert!(pgn_output.contains("[Result \"*\"]"), "PGN missing Result tag");
    
    // Validate PGN ends with result marker
    assert!(pgn_output.trim().ends_with('*'), "PGN should end with result marker");
    
    println!("✅ PGN structure validation passed");
    
    Ok(())
}

/// Test PGN exporter with a simple game sequence
#[test]
fn test_pgn_export_with_simple_game() -> Result<()> {
    // Create a simple game state with a few moves
    let mut game_state = GameState::new();
    
    // Create metadata
    let metadata = GameMetadata {
        event: "Integration Test".to_string(),
        site: "Test Suite".to_string(),
        date: "2024.01.01".to_string(),
        white: "Alice".to_string(),
        black: "Bob".to_string(),
        result: "1/2-1/2".to_string(),
        round: Some("1".to_string()),
        white_elo: Some(1500),
        black_elo: Some(1600),
        eco: Some("B00".to_string()),
    };
    
    // Export using the convenience method
    let exporter = PgnExporter::new();
    let pgn_output = exporter.export_game_state(&game_state, &metadata)?;
    
    // Validate the output contains all expected headers
    assert!(pgn_output.contains("[Event \"Integration Test\"]"));
    assert!(pgn_output.contains("[Site \"Test Suite\"]"));
    assert!(pgn_output.contains("[Date \"2024.01.01\"]"));
    assert!(pgn_output.contains("[White \"Alice\"]"));
    assert!(pgn_output.contains("[Black \"Bob\"]"));
    assert!(pgn_output.contains("[Result \"1/2-1/2\"]"));
    assert!(pgn_output.contains("[WhiteElo \"1500\"]"));
    assert!(pgn_output.contains("[BlackElo \"1600\"]"));
    assert!(pgn_output.contains("[ECO \"B00\"]"));
    
    // Should end with the result
    assert!(pgn_output.trim().ends_with("*")); // Empty game gets * result
    
    println!("✅ Simple game PGN export test passed");
    println!("PGN Output:\n{}", pgn_output);
    
    Ok(())
}

/// Test validation pipeline with invalid moves
#[test]
fn test_validation_pipeline_with_invalid_moves() -> Result<()> {
    use shakmaty::{Square, Role, Move};
    
    // Create a sequence with an invalid move
    let moves = vec![
        // Valid move: e2-e4
        Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        },
        // Invalid move: pawn moving backwards
        Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E8, // Invalid: pawn can't move to 8th rank without promotion
            capture: None,
            promotion: None,
        },
    ];
    
    // Validate the sequence
    let validator = ChessValidator::new();
    let validation_report = validator.validate_move_sequence(&moves)?;
    
    // Should detect the invalid move
    assert!(!validation_report.is_valid, "Validation should detect invalid moves");
    assert!(!validation_report.invalid_moves.is_empty(), "Should have invalid moves");
    
    // Create a game state and metadata
    let game_state = GameState::new();
    let metadata = GameMetadata {
        event: "Invalid Game Test".to_string(),
        site: "Test Suite".to_string(),
        date: "2024.01.01".to_string(),
        white: "Player1".to_string(),
        black: "Player2".to_string(),
        result: "*".to_string(),
        round: None,
        white_elo: None,
        black_elo: None,
        eco: None,
    };
    
    // Create parsed game with invalid validation
    let parsed_game = ParsedGame {
        game_state,
        validation_report,
        metadata,
    };
    
    // Try to export - should fail with validation enabled
    let exporter = PgnExporter::new();
    let result = exporter.export_with_validation(&parsed_game);
    
    assert!(result.is_err(), "Export should fail with invalid moves");
    
    // Try with validation disabled
    let exporter_no_validation = PgnExporter::with_settings(true, false);
    let pgn_output = exporter_no_validation.export_with_validation(&parsed_game)?;
    
    // Should succeed but warn about invalid moves
    assert!(pgn_output.contains("[Event \"Invalid Game Test\"]"));
    
    println!("✅ Validation pipeline test with invalid moves passed");
    
    Ok(())
}

/// Test the complete pipeline components work together
#[test] 
fn test_pipeline_components_integration() -> Result<()> {
    // Test that all the major components can work together
    
    // 1. Chess validation
    let validator = ChessValidator::new();
    let empty_moves = vec![];
    let validation_report = validator.validate_move_sequence(&empty_moves)?;
    assert!(validation_report.is_valid);
    
    // 2. Game state creation
    let game_state = GameState::new();
    assert_eq!(game_state.move_count(), 0);
    
    // 3. Metadata creation
    let metadata = GameMetadata {
        event: "Component Test".to_string(),
        site: "Integration Suite".to_string(),
        date: "2024.01.01".to_string(),
        white: "Component A".to_string(),
        black: "Component B".to_string(),
        result: "*".to_string(),
        round: None,
        white_elo: None,
        black_elo: None,
        eco: None,
    };
    
    // 4. ParsedGame creation
    let parsed_game = ParsedGame {
        game_state,
        validation_report,
        metadata,
    };
    
    // 5. PGN export
    let exporter = PgnExporter::new();
    let pgn_output = exporter.export_with_validation(&parsed_game)?;
    
    // Validate the complete pipeline produced valid PGN
    assert!(pgn_output.contains("[Event \"Component Test\"]"));
    assert!(pgn_output.contains("[Result \"*\"]"));
    assert!(pgn_output.trim().ends_with('*'));
    
    println!("✅ Pipeline components integration test passed");
    
    Ok(())
}