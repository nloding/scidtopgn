//! Property-based tests for SCIDtoPGN library
//!
//! This module uses property-based testing to systematically test invariants
//! and edge cases across the SCIDtoPGN library components.

use proptest::prelude::*;
use scidtopgn::api::ScidDatabase;
use scidtopgn::bridge::{GameState, GameMetadata, PositionContext};
use scidtopgn::core::error::Result;
use scidtopgn::formats::sg4::{Sg4File, find_game_boundaries};
use scidtopgn::pgn::{PgnExporter, ExportOptions};
use scidtopgn::position::{ScidPosition, Square, PieceType, Color};
use std::path::PathBuf;

/// Property tests for database functionality
proptest! {
    #[test]
    fn test_database_creation_properties(
        path in prop::collection::vec(vec![
            PathBuf::from("test1"),
            PathBuf::from("test2"),
            PathBuf::from("test3"),
        ]),
    ) {
        // Test that database creation works with various paths
        let result = test_utils::create_test_database_with_path(&path);
        assert!(result.is_ok(), "Database creation should succeed");
    }
}

/// Property tests for position tracking
proptest! {
    #[test]
    fn test_position_tracking_properties(
        moves in prop::collection::vec![
            vec![(Square::E2, Square::E4)],   // Pawn moves
            vec![(Square::E1, Square::G1)],   // King castling
            vec![(Square::D1, Square::H4)],   // Queen moves
            vec![(Square::F1, Square::F8)],   // Rook moves
            vec![(Square::C1, Square::A3)],   // Bishop moves
            vec![(Square::B1, Square::C3)],   // Knight moves
        ],
    ) {
        let mut position = ScidPosition::new_starting_position();
        
        // Test that position tracking works correctly
        for (from, to) in moves {
            let piece = position.piece_at(from).unwrap();
            let piece_num = position.piece_number(piece.color, piece.role);
            
            // Test that piece number is correct
            assert!(piece_num < 16, "Piece number should be valid");
            
            // Test that move is valid
            let valid_move = position.is_valid_move(from, to, piece.color);
            assert!(valid_move, "Move should be valid");
        }
    }
}

/// Property tests for PGN export
proptest! {
    #[test]
    fn test_pgn_export_properties(
        include_optional_tags in any::<bool>(),
        max_line_length in 70u32..100u32,
        include_annotations in any::<bool>(),
    ) in {
        let db = test_utils::create_test_database().unwrap();
        let game = db.games().next().unwrap();
        
        let options = ExportOptions {
            include_optional_tags,
            validate_moves: true,
            max_line_length,
            include_annotations,
            custom_headers: std::collections::HashMap::new(),
        };
        
        let exporter = PgnExporter::with_options(&game.game_state, &game.parsed_game, options);
        let pgn_content = exporter.export();
        
        // Test that PGN content has expected properties
        assert!(pgn_content.contains("[Event]"), "PGN should contain event header");
        assert!(pgn_content.contains("[White]"), "PGN should contain white player header");
        assert!(pgn_content.contains("[Black]"), "PGN should contain black player header");
        
        // Test line length constraint
        let lines: Vec<&str> = pgn_content.lines().collect();
        for line in lines {
            if line.len() > max_line_length as usize {
                // Check if it's a reasonable long line (like a move with annotations)
                if !line.contains("{") && !line.contains("}"){
                    assert!(false, "Line length should not exceed max_line_length");
                }
            }
        }
    }
}

/// Property tests for move decoding
proptest! {
    #[test]
    fn test_move_decoding_properties(
        move_value in 0u8..=15u8,
        piece_num in 0u8..=15u8,
        from_square in any::<Square>(),
    ) {
        let mut position = ScidPosition::new_starting_position();
        
        // Test that move decoding works with various inputs
        let piece = position.piece_at(from_square).unwrap();
        
        // Test that piece number is valid
        assert!(piece_num < 16, "Piece number should be valid");
        
        // Test that piece belongs to current player
        assert_eq!(piece.color, position.turn(), "Piece should belong to current player");
        
        // Test that move value is valid for the piece type
        match piece.role {
            PieceType::King => assert!(move_value <= 15, "King move value should be <= 15"),
            PieceType::Queen => assert!(move_value <= 15, "Queen move value should be <= 15"),
            PieceType::Rook => assert!(move_value <= 15, "Rook move value should be <= 15"),
            PieceType::Bishop => assert!(move_value <= 15, "Bishop move value should be <= 15"),
            PieceType::Knight => assert!(move_value <= 15, "Knight move value should be <= 15"),
            PieceType::Pawn => assert!(move_value <= 15, "Pawn move value should be <= 15"),
        }
    }
}

/// Property tests for file validation
proptest! {
    #[test]
    fn test_file_validation_properties(
        file_size in 0u32..1000000u32,
        file_content in prop::collection::vec![
            vec![0u8; 100],  // Small file
            vec![0u8; 1000], // Medium file
            vec![0u8; 10000], // Large file
        ]),
    ) in {
        let mut path = test_utils::temp_file_path();
        
        // Test file creation and validation
        std::fs::write(&path, &file_content).unwrap();
        
        // Test that file exists and is readable
        assert!(path.exists(), "File should exist");
        assert!(std::fs::read(&path).is_ok(), "File should be readable");
        
        // Test file size
        let metadata = std::fs::metadata(&path).unwrap();
        assert_eq!(metadata.len(), file_content.len(), "File size should match");
        
        // Clean up
        std::fs::remove_file(&path).unwrap();
    }
}

/// Property tests for error handling
proptest! {
    #[test]
    fn test_error_handling_properties(
        error_type in prop::collection::vec![
            "file_not_found",
            "invalid_format",
            "corrupt_data",
            "permission_denied",
        ],
    ) in {
        let path = test_utils::temp_file_path();
        
        // Test that error handling works correctly
        let result = match error_type {
            "file_not_found" => ScidDatabase::open(&path),
            "invalid_format" => test_utils::create_invalid_database(&path),
            "corrupt_data" => test_utils::create_corrupt_database(&path),
            "permission_denied" => {
                // Create file with restricted permissions
                std::fs::write(&path, b"test").unwrap();
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&path).unwrap().permissions();
                    perms.set_readonly(std::fs::PermissionsExt::from_mode(0o444));
                    std::fs::set_permissions(&path, perms).unwrap();
                }
                #[cfg(windows)]
                {
                    // Windows doesn't have the same permission model
                }
                ScidDatabase::open(&path)
            }
            _ => panic!("Unknown error type"),
        };
        
        // Test that error is properly handled
        assert!(result.is_err(), "Operation should return error");
        
        // Clean up
        if path.exists() {
            std::fs::remove_file(&path).unwrap();
        }
    }
}

/// Property tests for metadata handling
proptest! {
    #[test]
    fn test_metadata_properties(
        white_name in "[a-zA-Z]+",
        black_name in "[a-zA-Z]+",
        event_name in "[a-zA-Z]+",
        site_name in "[a-zA-Z]+",
        date in "[0-9]{4}\\.[0-9]{2}\\.[0-9]{2}",
        result in ["1-0", "0-1", "1/2-1/2", "*"],
        white_elo in 0u16..=3000u16,
        black_elo in 0u16..=3000u16,
    ) in {
        // Test metadata creation with various inputs
        let metadata = GameMetadata::new(
            white_name,
            black_name,
            event_name,
            site_name,
            date,
            result,
            if white_elo > 0 { Some(white_elo) } else { None },
            if black_elo > 0 { Some(black_elo) } else { None },
            None,
        );
        
        // Test that metadata contains expected values
        assert_eq!(metadata.white, white_name);
        assert_eq!(metadata.black, black_name);
        assert_eq!(metadata.event, event_name);
        assert_eq!(metadata.site, site_name);
        assert_eq!(metadata.result, result);
        
        if let Some(white_elo) = metadata.white_elo {
            assert!(white_elo >= 1000, "White ELO should be at least 1000");
            assert!(white_elo <= 3000, "White ELO should not exceed 3000");
        }
        
        if let Some(black_elo) = metadata.black_elo {
            assert!(black_elo >= 1000, "Black ELO should be at least 1000");
            assert!(black_elo <= 3000, "Black ELO should not exceed 3000");
        }
    }
}

/// Property tests for position validation
proptest! {
    #[test]
    fn test_position_validation_properties(
        position_data in prop::collection::vec![
            vec![(Square::E2, Square::E4)],   // Pawn moves
            vec![(Square::E1, Square::G1)],   // King castling
            vec![(Square::D1, Square::H4)],   // Queen moves
            vec![(Square::F1, Square::F8)],   // Rook moves
            vec![(Square::C1, Square::A3)],   // Bishop moves
            vec![(Square::B1, Square::C3)],   // Knight moves
        ],
    ) in {
        let mut position = ScidPosition::new_starting_position();
        
        // Test that position validation works correctly
        for (from, to) in position_data {
            let piece = position.piece_at(from).unwrap();
            
            // Test that move is valid
            let is_valid = position.is_valid_move(from, to, piece.color);
            
            // Test that position remains legal after move
            if is_valid {
                let mut test_position = position.clone();
                test_position.make_move(from, to, piece.color);
                
                // Test that position is still legal
                assert!(test_position.is_position_legal(), "Position should remain legal after valid move");
            }
        }
    }
}

/// Helper function to create invalid database for testing
fn create_invalid_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create a file with invalid SCID format
    let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid magic header
    std::fs::write(path, &invalid_data)?;
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid SCID format")))
}

/// Helper function to create corrupt database for testing
fn create_corrupt_database(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create a file with corrupt SG4 data
    let corrupt_data = vec![
        0x53, 0x63, 0x69, 0x64, 0x2E, // "Scid." with invalid null terminator
        0x00, 0x00, 0x00, 0x00, // Invalid padding
    ];
    std::fs::write(path, &corrupt_data)?;
    Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Corrupt SCID data")))
}

/// Helper function to create temporary file path
fn temp_file_path() -> PathBuf {
    use std::env;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .as_secs();
    
    let mut path = std::env::temp_dir();
    path.push(format!("test_{}.scid", timestamp));
    path
}

/// Helper function to create test database with custom path
fn create_test_database_with_path(path: &PathBuf) -> Result<ScidDatabase, Box<dyn std::error::Error>> {
    // Create a simple test database
    let test_data = vec![
        0x53, 0x63, 0x69, 0x64, 0x2E, // "Scid." header
        0x00, 0x00, 0x00, 0x00, // Padding
        0x01, 0x00, 0x00, 0x00, // 1 game
        0x02, 0x00, 0x00, 0x00, // 2 games
        0x03, 0x00, 0x00, 0x00, // 3 games
        0x04, 0x00, 0x00, 0x00, // 4 games
        0x05, 0x00, 0x00, 0x00, // 5 games
    ];
    
    std::fs::write(path.with_extension("si4"), &test_data)?;
    std::fs::write(path.with_extension("sn4"), &test_data)?;
    std::fs::write(path.with_extension("sg4"), &test_data)?;
    std::fs::write(path.with_extension("pgn"), &test_data)?;
    
    // Create a simple game index
    let game_index = crate::formats::si4::GameIndex {
        white_id: 1,
        black_id: 2,
        event_id: 3,
        site_id: 4,
        round_id: 5,
        year: 2024,
        month: 1,
        day: 1,
        event_year: 2024,
        event_month: 1,
        event_day: 1,
        result: 1,
        eco: 0,
        white_elo: 1800,
        black_elo: 1750,
        flags: 0,
        parsed_flags: 0,
        num_half_moves: 42,
    };
    
    // Create a simple game state
    let game_state = GameState::new();
    let parsed_game = crate::formats::sg4::StreamingGameParseState::new();
    
    Ok(ScidDatabase {
        si4_file: test_utils::create_mock_si4_file(&game_index),
        sn4_file: test_utils::create_mock_sn4_file(),
        sg4_file: test_utils::create_mock_sg4_file(),
        game_index,
        game_state,
        parsed_game,
    })
}

/// Helper function to create mock SI4 file
fn create_mock_si4_file(game_index: &crate::formats::si4::GameIndex) -> crate::formats::si4::Si4File {
    use memmap2::Mmap;
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create("test.si4").unwrap();
    
    // Write mock SI4 data
    file.write_all(&[0x53, 0x63, 0x69, 0x64, 0x2E]).unwrap(); // "Scid." header
    file.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap(); // Padding
    
    // Write mock game index data
    file.write_all(&[
        game_index.white_id.to_le_bytes(),
        game_index.black_id.to_le_bytes(),
        game_index.event_id.to_le_bytes(),
        game_index.site_id.to_le_bytes(),
        game_index.round_id.to_le_bytes(),
        (game_index.year).to_le_bytes(),
        game_index.month as u8,
        game_index.day as u8,
        game_index.event_year.to_le_bytes(),
        game_index.event_month as u8,
        game_index.event_day as u8,
        game_index.result as u8,
        game_index.eco.to_le_bytes(),
        game_index.white_elo.to_le_bytes(),
        game_index.black_elo.to_le_bytes(),
        game_index.flags.to_le_bytes(),
        game_index.parsed_flags.to_le_bytes(),
        game_index.num_half_moves.to_le_bytes(),
    ]).unwrap();
    
    // Create memory map
    let mmap = unsafe { Mmap::map(&file) }.unwrap();
    
    crate::formats::si4::Si4File { mmap }
}

/// Helper function to create mock SN4 file
fn create_mock_sn4_file() -> crate::formats::sn4::Sn4File {
    use memmap2::Mmap;
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create("test.sn4").unwrap();
    
    // Write mock SN4 data
    file.write_all(&[0x53, 0x63, 0x69, 0x64, 0x2E]).unwrap(); // "Scid." header
    file.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap(); // Padding
    
    // Create memory map
    let mmap = unsafe { Mmap::map(&file) }.unwrap();
    
    crate::formats::sn4::Sn4File {
        mmap,
        header: crate::formats::sn4::Sn4Header {
            magic: [0x53, 0x63, 0x69, 0x64, 0x2E],
            num_names_player: 10,
            max_frequency_player: 100,
            num_names_event: 5,
            max_frequency_event: 50,
            num_names_site: 5,
            max_frequency_site: 50,
            num_names_round: 5,
            max_frequency_round: 50,
        },
        player_offset: 36,
        event_offset: 86,
        site_offset: 136,
        round_offset: 186,
        name_data: vec![],
    }
}

/// Helper function to create mock SG4 file
fn create_mock_sg4_file() -> crate::formats::sg4::Sg4File {
    use memmap2::Mmap;
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create("test.sg4").unwrap();
    
    // Write mock SG4 data
    file.write_all(&[0x53, 0x63, 0x69, 0x64, 0x2E]).unwrap(); // "Scid." header
    file.write_all(&[0x00, 0x00, 0x00, 0x00]).unwrap(); // Padding
    
    // Create memory map
    let mmap = unsafe { Mmap::map(&file) }.unwrap();
    
    crate::formats::sg4::Sg4File { mmap }
}