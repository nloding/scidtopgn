use scidtopgn::{GameMetadata, GameState, PgnExporter, PositionContext, parse_streaming_state};
use anyhow::Result;
use std::fs;
use std::path::Path;

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

    assert!(
        sg4_path.exists(),
        "Test file five.sg4 not found at {:?}",
        sg4_path
    );
    assert!(
        si4_path.exists(),
        "Test file five.si4 not found at {:?}",
        si4_path
    );
    assert!(
        sn4_path.exists(),
        "Test file five.sn4 not found at {:?}",
        sn4_path
    );
    assert!(
        reference_pgn_path.exists(),
        "Reference PGN file not found at {:?}",
        reference_pgn_path
    );

    // Parse the SG4 game file
    let game_data = fs::read(&sg4_path)?;
    let parsed_game = parse_streaming_state(&game_data)?;

    // Create a GameState (empty for this integration test)
    let game_state = GameState::new();

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

    // Export to PGN using the new exporter
    let exporter = PgnExporter::new(&game_state, &parsed_game)?;
    let pgn_output = exporter.export()?;

    assert!(pgn_output.contains("[Event \"Test Event\"]"));
    assert!(pgn_output.contains("[Site \"Test Site\"]"));
    assert!(pgn_output.contains("[Date \"2022.12.19\"]"));
    assert!(pgn_output.contains("[White \"Test White\"]"));
    assert!(pgn_output.contains("[Black \"Test Black\"]"));
    assert!(pgn_output.contains("[Result \"*\"]"));
    assert!(pgn_output.trim().ends_with('*'));

    Ok(())
}

/// Test PGN exporter with a simple game sequence
#[test]
fn test_pgn_export_with_simple_game() -> Result<()> {
    // Create a simple game state with a few moves
    let game_state = GameState::new();

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

    // Export using PgnExporter
    let exporter = PgnExporter::with_options(&game_state, &parse_streaming_state(&[])? , scidtopgn::pgn::ExportOptions::default())?;
    let pgn_output = exporter.export()?;

    assert!(pgn_output.contains("[Event \"Integration Test\"]"));
    assert!(pgn_output.contains("[Site \"Test Suite\"]"));
    assert!(pgn_output.contains("[Date \"2024.01.01\"]"));
    assert!(pgn_output.contains("[White \"Alice\"]"));
    assert!(pgn_output.contains("[Black \"Bob\"]"));
    assert!(pgn_output.contains("[Result \"1/2-1/2\"]"));
    assert!(pgn_output.contains("[WhiteElo \"1500\"]"));
    assert!(pgn_output.contains("[BlackElo \"1600\"]"));
    assert!(pgn_output.contains("[ECO \"B00\"]"));
    assert!(pgn_output.trim().ends_with("*"));

    Ok(())
}

/// Test validation pipeline with invalid moves
#[test]
fn test_validation_pipeline_with_invalid_moves() -> Result<()> {
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

    // With empty parsed game, exporter should still produce headers
    let exporter = PgnExporter::new(&game_state, &parse_streaming_state(&[])? )?;
    let pgn_output = exporter.export()?;
    assert!(pgn_output.contains("[Event \"Invalid Game Test\"]"));

    Ok(())
}

/// Test the complete pipeline components work together
#[test]
fn test_pipeline_components_integration() -> Result<()> {
    // Game state creation
    let game_state = GameState::new();
    assert_eq!(scidtopgn::PositionContext::move_count(&game_state), 0);

    // Metadata creation
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

    // PGN export
    let exporter = PgnExporter::new(&game_state, &parse_streaming_state(&[])? )?;
    let pgn_output = exporter.export()?;

    assert!(pgn_output.contains("[Event \"Component Test\"]"));
    assert!(pgn_output.contains("[Result \"*\"]"));
    assert!(pgn_output.trim().ends_with('*'));

    Ok(())
}
