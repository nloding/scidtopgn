use std::path::Path;

// Integration tests for the SCID database parsing
// This tests the complete workflow from reading SCID files to validating against PGN output

/// Test data validation - ensures our test dataset matches expected values
#[test]
fn test_date_extraction_with_five_dataset() {
    // Load the SCID database from test data
    let test_data_path = Path::new("test/data/five");
    
    // Verify all required test files exist
    assert!(test_data_path.with_extension("si4").exists(), "five.si4 test file is missing");
    assert!(test_data_path.with_extension("sg4").exists(), "five.sg4 test file is missing");
    assert!(test_data_path.with_extension("sn4").exists(), "five.sn4 test file is missing");
    assert!(test_data_path.with_extension("pgn").exists(), "five.pgn test file is missing");
    
    // Load the SCID database
    let database = scidtopgn::scid::database::ScidDatabase::load(test_data_path)
        .expect("Failed to load SCID database from test data");
    
    // Verify we have exactly 5 games as expected
    assert_eq!(database.num_games(), 5, "Expected exactly 5 games in test dataset");
    
    // Test date extraction for all games
    let expected_date = "2022.12.19";
    
    for game_id in 0..database.num_games() {
        let game_index = database.game_index(game_id)
            .expect(&format!("Failed to get game index for game {}", game_id));
        
        // Test the date extraction
        let actual_date = game_index.date_string();
        assert_eq!(actual_date, expected_date, 
            "Game {} date mismatch: expected '{}', got '{}'", game_id, expected_date, actual_date);
        
        // Validate individual date components
        assert_eq!(game_index.year, 2022, "Game {} year should be 2022", game_id);
        assert_eq!(game_index.month, 12, "Game {} month should be 12", game_id);
        assert_eq!(game_index.day, 19, "Game {} day should be 19", game_id);
    }
}

/// Test that validates extracted dates against the PGN source of truth
#[test]
fn test_date_validation_against_pgn_source() {
    use std::fs;
    
    // Read the PGN source of truth
    let pgn_content = fs::read_to_string("test/data/five.pgn")
        .expect("Failed to read five.pgn source of truth file");
    
    // Count occurrences of the expected date in PGN
    let expected_date = "2022.12.19";
    let date_occurrences = pgn_content.matches(&format!("[Date \"{}\"]", expected_date)).count();
    
    // Should have exactly 5 games with this date
    assert_eq!(date_occurrences, 5, 
        "PGN source should contain exactly 5 games with date {}, found {}", 
        expected_date, date_occurrences);
    
    // Load and test SCID database
    let database = scidtopgn::scid::database::ScidDatabase::load("test/data/five")
        .expect("Failed to load SCID database");
    
    // Verify our extraction matches PGN
    assert_eq!(database.num_games(), 5, "Database should have 5 games");
    
    for game_id in 0..database.num_games() {
        let game_index = database.game_index(game_id).unwrap();
        let extracted_date = game_index.date_string();
        
        assert_eq!(extracted_date, expected_date,
            "Extracted date for game {} should match PGN source", game_id);
    }
}

/// Test the raw binary date pattern extraction
#[test]
fn test_raw_date_pattern_extraction() {
    use scidtopgn::scid::index::IndexFile;
    
    // Load just the index file
    let index_file = IndexFile::load("test/data/five.si4")
        .expect("Failed to load index file");
    
    // Test that all games have consistent date extraction
    let game_indices = index_file.game_indices();
    assert_eq!(game_indices.len(), 5, "Should have 5 game indices");
    
    // All games should have the same date since they're from the same event
    for (i, game_index) in game_indices.iter().enumerate() {
        assert_eq!(game_index.year, 2022, "Game {} year extraction failed", i);
        assert_eq!(game_index.month, 12, "Game {} month extraction failed", i);
        assert_eq!(game_index.day, 19, "Game {} day extraction failed", i);
        
        // Test the date string formatting
        let date_str = game_index.date_string();
        assert_eq!(date_str, "2022.12.19", "Game {} date string formatting failed", i);
    }
}

/// Test edge cases and error handling
#[test]
fn test_date_extraction_error_handling() {
    use scidtopgn::scid::index::IndexFile;
    
    // Test with non-existent file
    let result = IndexFile::load("test/data/nonexistent.si4");
    assert!(result.is_err(), "Should fail when loading non-existent file");
    
    // Load valid file for boundary testing
    let index_file = IndexFile::load("test/data/five.si4").unwrap();
    
    // Test accessing game indices
    let game_indices = index_file.game_indices();
    
    // Test boundary access
    assert!(index_file.game_index(0).is_some(), "Should have game 0");
    assert!(index_file.game_index(4).is_some(), "Should have game 4");
    assert!(index_file.game_index(5).is_none(), "Should not have game 5");
    assert!(index_file.game_index(100).is_none(), "Should not have game 100");
}

/// Performance test for date extraction
#[test]
fn test_date_extraction_performance() {
    use std::time::Instant;
    use scidtopgn::scid::index::IndexFile;
    
    let start = Instant::now();
    
    // Load and parse the index file
    let index_file = IndexFile::load("test/data/five.si4")
        .expect("Failed to load index file for performance test");
    
    // Extract dates from all games
    let game_indices = index_file.game_indices();
    let mut date_count = 0;
    
    for game_index in game_indices {
        let _date_str = game_index.date_string();
        date_count += 1;
    }
    
    let duration = start.elapsed();
    
    assert_eq!(date_count, 5, "Should have processed 5 games");
    
    // Performance assertion - should complete well under 1 second for 5 games
    assert!(duration.as_millis() < 1000, 
        "Date extraction took too long: {:?}", duration);
    
    println!("Date extraction performance: {:?} for {} games", duration, date_count);
}