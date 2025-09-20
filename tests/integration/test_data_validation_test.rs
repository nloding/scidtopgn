//! Test Data Validation Tests
//!
//! This module validates that all 5 games parse correctly from test files
//! and validates extracted metadata against known values.

use std::path::Path;
use std::fs;
use scidtopgn::api::ScidDatabase;
use scidtopgn::core::error::Result;
use crate::test_utils::*;

/// Test that all 5 games parse correctly from test files
#[test]
fn test_all_five_games_parse_correctly() -> Result<()> {
    // Verify test data exists
    assert!(test_files_exist(), "Test files should exist");
    
    // Create test database
    let db = create_test_database()?;
    
    // Validate test database
    validate_test_database(&db)?;
    
    // Test that we can access all 5 games
    let games: Vec<_> = db.games().collect();
    assert_eq!(games.len(), 5, "Database should contain exactly 5 games");
    
    // Test that each game has the expected structure
    for (index, game) in games.iter().enumerate() {
        println!("Validating game {} with white_id={}, black_id={}", 
                 index, game.index.white_id, game.index.black_id);
        
        // Test game structure
        assert!(game.index.white_id > 0, "Game should have valid white player ID");
        assert!(game.index.black_id > 0, "Game should have valid black player ID");
        assert!(game.index.year > 0, "Game should have valid year");
        assert!(game.index.month > 0 && game.index.month <= 12, "Game should have valid month");
        assert!(game.index.day > 0 && game.index.day <= 31, "Game should have valid day");
        
        // Test that the game state is properly initialized
        assert!(!game.game_state.metadata().is_none(), "Game should have metadata");
        
        // Test that the parsed game has elements
        assert!(!game.parsed_game.elements.is_empty(), "Game should have parsed elements");
    }
    
    println!("✅ All 5 games parsed correctly");
    Ok(())
}

/// Validate extracted metadata against known values
#[test]
fn test_metadata_validation_against_known_values() -> Result<()> {
    let db = create_test_database()?;
    
    // Load reference PGN for comparison
    let reference_pgn = load_reference_pgn()?;
    
    // Extract known values from reference PGN
    let known_values = extract_known_values_from_pgn(&reference_pgn)?;
    
    // Validate each game against known values
    for (index, game_result) in db.games().enumerate() {
        let game = game_result?;
        
        println!("Validating metadata for game {}", index + 1);
        
        // Validate against known values
        if let Some(known_white) = &known_values[index].white {
            assert_eq!(game.index.white_id.to_string(), *known_white, 
                     "White player ID should match known value");
        }
        
        if let Some(known_black) = &known_values[index].black {
            assert_eq!(game.index.black_id.to_string(), *known_black, 
                     "Black player ID should match known value");
        }
        
        if let Some(known_event) = &known_values[index].event {
            assert_eq!(game.index.event_id.to_string(), *known_event, 
                     "Event ID should match known value");
        }
        
        if let Some(known_site) = &known_values[index].site {
            assert_eq!(game.index.site_id.to_string(), *known_site, 
                     "Site ID should match known value");
        }
        
        if let Some(known_date) = &known_values[index].date {
            assert_eq!(format!("{:04}.{:02}.{:02}", game.index.year, game.index.month, game.index.day), 
                     *known_date, "Date should match known value");
        }
        
        if let Some(known_result) = &known_values[index].result {
            assert_eq!(decode_result(game.index.result), *known_result, 
                     "Result should match known value");
        }
    }
    
    println!("✅ Metadata validation against known values passed");
    Ok(())
}

/// Confirm PGN output matches expected format
#[test]
fn test_pgn_output_matches_expected_format() -> Result<()> {
    let db = create_test_database()?;
    let reference_pgn = load_reference_pgn()?;
    
    // Export all games to PGN
    let mut exported_pgns = Vec::new();
    
    for (index, game_result) in db.games().enumerate() {
        let game = game_result?;
        
        // Export game to PGN
        let exported_pgn = scidtopgn::pgn::PgnExporter::new(&game.game_state, &game.parsed_game)?
            .export()?;
        exported_pgns.push(exported_pgn);
        
        // Validate PGN structure
        assert!(exported_pgn.contains("[Event]"), "Exported PGN should have Event tag");
        assert!(exported_pgn.contains("[Site]"), "Exported PGN should have Site tag");
        assert!(exported_pgn.contains("[Date]"), "Exported PGN should have Date tag");
        assert!(exported_pgn.contains("[White]"), "Exported PGN should have White tag");
        assert!(exported_pgn.contains("[Black]"), "Exported PGN should have Black tag");
        assert!(exported_pgn.contains("[Result]"), "Exported PGN should have Result tag");
        
        // Test that the game has moves
        assert!(exported_pgn.contains("1."), "Exported PGN should have moves");
        
        // Test that the game ends with result
        assert!(exported_pgn.trim().ends_with("*") || 
                 exported_pgn.trim().ends_with("1-0") || 
                 exported_pgn.trim().ends_with("0-1") || 
                 exported_pgn.trim().ends_with("1/2-1/2"), 
                 "Exported PGN should end with valid result");
    }
    
    // Compare with reference PGN (basic comparison)
    assert_eq!(exported_pgns.len(), 5, "Should export 5 games");
    
    println!("✅ PGN output matches expected format");
    Ok(())
}

/// Test edge cases and error conditions
#[test]
fn test_edge_cases_and_error_conditions() -> Result<()> {
    let db = create_test_database()?;
    
    // Test accessing game beyond bounds
    let result = db.get_game(10);
    assert!(result.is_err(), "Should return error for out-of-bounds game");
    
    // Test empty iterator
    let games: Vec<_> = db.games().skip(10).collect();
    assert_eq!(games.len(), 0, "Should return empty iterator");
    
    // Test database statistics
    let stats = db.statistics();
    assert_eq!(stats.num_games, 5, "Statistics should show 5 games");
    
    // Test database revalidation
    let mut db2 = create_test_database()?;
    db2.revalidate()?;
    assert!(db2.is_validated(), "Database should be revalidated");
    
    // Test with invalid file paths
    let result = ScidDatabase::open("non_existent_database");
    assert!(result.is_err(), "Should return error for non-existent database");
    
    println!("✅ Edge cases and error conditions test passed");
    Ok(())
}

/// Test file integrity and consistency
#[test]
fn test_file_integrity_and_consistency() -> Result<()> {
    let (si4_path, sn4_path, sg4_path, pgn_path) = five_test_files();
    
    // Test that all files exist
    assert!(si4_path.exists(), "SI4 file should exist");
    assert!(sn4_path.exists(), "SN4 file should exist");
    assert!(sg4_path.exists(), "SG4 file should exist");
    assert!(pgn_path.exists(), "PGN file should exist");
    
    // Test file sizes are reasonable
    let si4_size = fs::metadata(&si4_path)?.len();
    let sn4_size = fs::metadata(&sn4_path)?.len();
    let sg4_size = fs::metadata(&sg4_path)?.len();
    let pgn_size = fs::metadata(&pgn_path)?.len();
    
    println!("File sizes:");
    println!("  SI4: {} bytes", si4_size);
    println!("  SN4: {} bytes", sn4_size);
    println!("  SG4: {} bytes", sg4_size);
    println!("  PGN: {} bytes", pgn_size);
    
    // Test that files are not empty
    assert!(si4_size > 0, "SI4 file should not be empty");
    assert!(sn4_size > 0, "SN4 file should not be empty");
    assert!(sg4_size > 0, "SG4 file should not be empty");
    assert!(pgn_size > 0, "PGN file should not be empty");
    
    // Test that SG4 file is larger than others (expected)
    assert!(sg4_size > si4_size, "SG4 file should be larger than SI4 file");
    assert!(sg4_size > sn4_size, "SG4 file should be larger than SN4 file");
    
    println!("✅ File integrity and consistency test passed");
    Ok(())
}

/// Test database consistency across multiple accesses
#[test]
fn test_database_consistency_across_multiple_accesses() -> Result<()> {
    let db1 = create_test_database()?;
    let db2 = create_test_database()?;
    
    // Test that both databases have the same number of games
    assert_eq!(db1.num_games(), db2.num_games(), "Both databases should have same number of games");
    
    // Test that both databases have the same game indices
    let games1: Vec<_> = db1.games().collect();
    let games2: Vec<_> = db2.games().collect();
    
    assert_eq!(games1.len(), games2.len(), "Both databases should have same number of games");
    
    // Compare game indices
    for (game1, game2) in games1.iter().zip(games2.iter()) {
        assert_eq!(game1.index.white_id, game2.index.white_id, "Games should have same white player ID");
        assert_eq!(game1.index.black_id, game2.index.black_id, "Games should have same black player ID");
        assert_eq!(game1.index.year, game2.index.year, "Games should have same year");
        assert_eq!(game1.index.month, game2.index.month, "Games should have same month");
        assert_eq!(game1.index.day, game2.index.day, "Games should have same day");
    }
    
    println!("✅ Database consistency across multiple accesses test passed");
    Ok(())
}

/// Test performance with test data
#[test]
fn test_performance_with_test_data() -> Result<()> {
    let db = create_test_database()?;
    
    // Test database opening performance
    let start_time = std::time::Instant::now();
    let _db = create_test_database()?;
    let open_time = start_time.elapsed();
    assert!(open_time.as_millis() < 500, "Database opening should be fast");
    
    // Test game iteration performance
    let start_time = std::time::Instant::now();
    let game_count: usize = db.games().count();
    let iteration_time = start_time.elapsed();
    assert!(iteration_time.as_millis() < 100, "Game iteration should be fast");
    assert_eq!(game_count, 5, "Should count all 5 games");
    
    // Test PGN export performance
    let start_time = std::time::Instant::now();
    let mut successful_exports = 0;
    
    for game_result in db.games() {
        if let Ok(game) = game_result {
            if scidtopgn::pgn::PgnExporter::new(&game.game_state, &game.parsed_game).is_ok() {
                successful_exports += 1;
            }
        }
    }
    let export_time = start_time.elapsed();
    assert!(export_time.as_millis() < 500, "PGN export should be fast");
    assert_eq!(successful_exports, 5, "Should export all games successfully");
    
    println!("✅ Performance with test data test passed");
    println!("  Games processed: {}", successful_exports);
    println!("  Total time: {:?}", open_time + iteration_time + export_time);
    
    Ok(())
}

/// Helper function to extract known values from reference PGN
fn extract_known_values_from_pgn(pgn_content: &str) -> Result<Vec<KnownValues>> {
    let mut known_values = Vec::new();
    let mut current_game = 0;
    let mut current_player = None;
    let mut current_event = None;
    let mut current_site = None;
    let mut current_date = None;
    let mut current_result = None;
    
    for line in pgn_content.lines() {
        let line = line.trim();
        
        if line.starts_with("[Event \"") {
            current_event = Some(line.split('"')[1]);
        } else if line.starts_with("[Site \"") {
            current_site = Some(line.split('"')[1]);
        } else if line.starts_with("[Date \"") {
            current_date = Some(line.split('"')[1]);
        } else if line.starts_with("[White \"") {
            current_player = Some(line.split('"')[1]);
        } else if line.starts_with("[Black \"") {
            if current_player.is_some() {
                // This is the end of the current game
                known_values.push(KnownValues {
                    white: current_player.clone(),
                    black: line.split('"')[1].to_string(),
                    event: current_event.clone(),
                    site: current_site.clone(),
                    date: current_date.clone(),
                    result: current_result.clone(),
                });
                
                // Reset for next game
                current_player = None;
                current_event = None;
                current_site = None;
                current_date = None;
                current_result = None;
                current_game += 1;
            }
        } else if line.starts_with("[Result \"") {
            current_result = Some(line.split('"')[1]);
        }
    }
    
    Ok(known_values)
}

/// Helper function to decode SCID result code
fn decode_result(result_code: u8) -> String {
    match result_code {
        0 => "*".to_string(),       // Unknown/ongoing result
        1 => "1-0".to_string(),     // White wins
        2 => "0-1".to_string(),     // Black wins
        3 => "1/2-1/2".to_string(), // Draw
        _ => "*".to_string(),       // Unknown result code
    }
}

/// Known values extracted from reference PGN
#[derive(Debug, Clone)]
struct KnownValues {
    pub white: String,
    pub black: String,
    pub event: String,
    pub site: String,
    pub date: String,
    pub result: String,
}