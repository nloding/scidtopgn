//! Unit tests for individual components
//!
//! This module contains focused unit tests for individual functions and components
//! across the SCIDtoPGN library, ensuring each piece works correctly in isolation.

use scidtopgn::api::{ScidDatabase, PgnExporter, ExportOptions};
use scidtopgn::bridge::{GameState, GameMetadata, PositionContext};
use scidtopgn::core::error::Result;
use scidtopgn::sg4::{Sg4File, find_game_boundaries};
use scidtopgn::position::{ScidPosition, Square, PieceType};
use std::path::PathBuf;

mod test_utils;

/// Test database opening functionality
#[test]
fn test_database_opening() -> Result<()> {
    mod test_utils; // ensure visibility
    let test_db_path = test_utils::five_test_data();
    
    // Test that database opens successfully
    let db = ScidDatabase::open(&test_db_path)?;
    
    // Test that we can access basic information
    assert_eq!(db.num_games(), 5, "Database should contain 5 games");
    assert!(db.is_validated(), "Database should be validated");
    
    Ok(())
}

/// Test game access functionality
#[test]
fn test_game_access() -> Result<()> {
    let db = test_utils::create_test_database()?;
    
    // Test that we can access individual games
    let games: Vec<_> = db.games().take(3).collect();
    assert_eq!(games.len(), 3, "Should access 3 games");
    
    // Test that each game has the expected structure
    for (index, game) in games.iter().enumerate() {
        assert!(game.index.white_id > 0, "Game should have valid white player ID");
        assert!(game.index.black_id > 0, "Game should have valid black player ID");
        assert!(game.index.year > 0, "Game should have valid year");
        assert!(game.index.month > 0 && game.index.month <= 12, "Game should have valid month");
        assert!(game.index.day > 0 && game.index.day <= 31, "Game should have valid day");
    }
    
    Ok(())
}

/// Test PGN export functionality
#[test]
fn test_pgn_export() -> Result<()> {
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    
    // Test PGN export with default options
    let exporter = PgnExporter::new(&game.game_state, &game.parsed_game)?;
    let pgn_content = exporter.export()?;
    
    // Test that PGN content contains expected elements
    assert!(pgn_content.contains("[Event"), "PGN should contain event header");
    assert!(pgn_content.contains("[White"), "PGN should contain white player header");
    assert!(pgn_content.contains("[Black"), "PGN should contain black player header");
    assert!(pgn_content.contains("[Result"), "PGN should contain result header");
    
    Ok(())
}

/// Test position tracking functionality
#[test]
fn test_position_tracking() -> Result<()> {
    let mut position = ScidPosition::new_starting_position();
    
    // Test that position tracking works correctly
    let white_pieces = position.piece_list(scidtopgn::position::Color::White);
    let black_pieces = position.piece_list(scidtopgn::position::Color::Black);
    
    // Test that piece lists have correct length
    assert_eq!(white_pieces.len(), 16, "White should have 16 pieces");
    assert_eq!(black_pieces.len(), 16, "Black should have 16 pieces");
    
    // Test that key pieces are in correct positions
    let e1: Square = "e1".parse().unwrap();
    let e2: Square = "e2".parse().unwrap();
    let e7: Square = "e7".parse().unwrap();
    let e8: Square = "e8".parse().unwrap();
    assert_eq!(white_pieces[0], e1, "White king should be at E1");
    assert_eq!(white_pieces[12], e2, "White E2 pawn should be at E2");
    assert_eq!(black_pieces[0], e8, "Black king should be at E8");
    assert_eq!(black_pieces[12], e7, "Black E7 pawn should be at E7");
    
    Ok(())
}

/// Test metadata functionality
#[test]
fn test_metadata_handling() -> Result<()> {
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
    
    // Test that metadata contains expected values
    assert_eq!(metadata.white, "Test Player");
    assert_eq!(metadata.black, "Test Opponent");
    assert_eq!(metadata.event, "Test Event");
    assert_eq!(metadata.site, "Test Site");
    assert_eq!(metadata.result, "1-0");
    assert_eq!(metadata.white_elo, Some(1800));
    assert_eq!(metadata.black_elo, Some(1750));
    assert_eq!(metadata.eco, Some("A00".to_string()));
    
    Ok(())
}

/// Test error handling for invalid files
#[test]
fn test_invalid_file_handling() -> Result<()> {
    let invalid_path = PathBuf::from("nonexistent_file");
    
    // Test that opening invalid file returns appropriate error
    let result = ScidDatabase::open(&invalid_path);
    assert!(result.is_err(), "Opening invalid file should return error");
    
    Ok(())
}

/// Test move decoding functionality
#[test]
fn test_move_decoding() -> Result<()> {
    use scidtopgn::position::decode_move;
    
    // Test knight move decoding
    let mut position = ScidPosition::new_starting_position();
    let knight_move = decode_move(&position, 0x50).unwrap();
    
    assert_eq!(knight_move.piece_num, 5, "Knight should be piece 5");
    assert_eq!(knight_move.from, Square::B1, "Knight should start from B1");
    assert_eq!(knight_move.to, Square::A3, "Knight should move to A3");
    assert_eq!(knight_move.moving_piece, PieceType::Knight, "Moving piece should be knight");
    
    // Test rook move decoding
    let rook_move = decode_move(&position, 0x30).unwrap();
    assert_eq!(rook_move.piece_num, 3, "Rook should be piece 3");
    assert_eq!(rook_move.from, Square::F1, "Rook should start from F1");
    assert_eq!(rook_move.moving_piece, PieceType::Rook, "Moving piece should be rook");
    
    Ok(())
}

/// Test SG4 file parsing
#[test]
fn test_sg4_parsing() -> Result<()> {
    let sg4_path = test_utils::five_test_data().with_extension("sg4");
    let sg4_data = std::fs::read(&sg4_path)?;
    
    // Test that we can find game boundaries
    let boundaries = find_game_boundaries(&sg4_data);
    assert_eq!(boundaries.len(), 5, "Should find 5 game boundaries");
    
    // Test that boundaries are correct
    let (first_start, first_end) = boundaries[0];
    assert_eq!(first_end - first_start, 175, "First game should be 175 bytes");
    
    Ok(())
}

/// Test file validation
#[test]
fn test_file_validation() -> Result<()> {
    let (si4_path, sn4_path, sg4_path, pgn_path) = test_utils::five_test_files();
    
    // Test that all required files exist
    assert!(si4_path.exists(), "SI4 file should exist");
    assert!(sn4_path.exists(), "SN4 file should exist");
    assert!(sg4_path.exists(), "SG4 file should exist");
    assert!(pgn_path.exists(), "PGN file should exist");
    
    // Test that files are readable
    assert!(std::fs::read(&si4_path).is_ok(), "SI4 file should be readable");
    assert!(std::fs::read(&sn4_path).is_ok(), "SN4 file should be readable");
    assert!(std::fs::read(&sg4_path).is_ok(), "SG4 file should be readable");
    assert!(std::fs::read(&pgn_path).is_ok(), "PGN file should be readable");
    
    Ok(())
}

/// Test export options
#[test]
fn test_export_options() -> Result<()> {
    let db = test_utils::create_test_database()?;
    let game = db.games().next().unwrap();
    
    // Test export with different options
    let options = ExportOptions {
        include_optional_tags: true,
        validate_moves: true,
        max_line_length: 80,
        include_annotations: true,
        custom_headers: std::collections::HashMap::new(),
    };
    
    let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options)?;
    let pgn_content = exporter.export()?;
    
    // Test that PGN content contains optional tags
    assert!(pgn_content.contains("[WhiteElo"), "PGN should contain White ELO");
    assert!(pgn_content.contains("[BlackElo"), "PGN should contain Black ELO");
    assert!(pgn_content.contains("[ECO"), "PGN should contain ECO");
    
    Ok(())
}

/// Test move decoding for different piece types
#[test]
fn test_move_validation_by_piece_type() -> Result<()> {
    let mut position = ScidPosition::new_starting_position();
    
    // Test pawn moves
    let pawn_moves = vec![
        ("e2".parse().unwrap(), "e4".parse().unwrap()),   // Forward move
        ("e2".parse().unwrap(), "e3".parse().unwrap()),   // Single square move
        ("e4".parse().unwrap(), "e5".parse().unwrap()),   // Double pawn move
    ];
    
    for (from, to) in pawn_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Pawn, "Should be pawn");
        assert!(position.is_valid_move(from, to, Color::White), "Pawn move should be valid");
    }
    
    // Test knight moves
    let knight_moves = vec![
        ("b1".parse().unwrap(), "c3".parse().unwrap()),   // Forward-right
        ("b1".parse().unwrap(), "a3".parse().unwrap()),   // Forward-left
        ("g1".parse().unwrap(), "f3".parse().unwrap()),   // Forward-right
        ("g1".parse().unwrap(), "h3".parse().unwrap()),   // Forward-left
    ];
    
    for (from, to) in knight_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Knight, "Should be knight");
        assert!(position.is_valid_move(from, to, Color::White), "Knight move should be valid");
    }
    
    // Test rook moves
    let rook_moves = vec![
        ("a1".parse().unwrap(), "a8".parse().unwrap()),   // Vertical move
        ("a1".parse().unwrap(), "h1".parse().unwrap()),   // Horizontal move
    ];
    
    for (from, to) in rook_moves {
        let piece = position.piece_at(from).unwrap();
        assert_eq!(piece.role, PieceType::Rook, "Should be rook");
        assert!(position.is_valid_move(from, to, Color::White), "Rook move should be valid");
    }
    
    Ok(())
}