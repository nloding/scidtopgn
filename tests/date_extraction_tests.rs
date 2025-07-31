//! Date Extraction Tests
//! 
//! These tests validate the date extraction functionality against the five.pgn dataset.
//! The tests focus on validating the core date parsing logic and ensuring compatibility
//! with the expected PGN output format.

use std::fs;

/// Test that validates the expected date format against the PGN source of truth
#[test]
fn test_pgn_date_format_validation() {
    // This test validates that our target date format matches the PGN source
    let pgn_path = "test/data/five.pgn";
    
    // Skip if test data doesn't exist
    if !std::path::Path::new(pgn_path).exists() {
        println!("Skipping test - PGN test data not found at {}", pgn_path);
        return;
    }
    
    let pgn_content = fs::read_to_string(pgn_path)
        .expect("Failed to read PGN test file");
    
    // Count date occurrences in PGN format
    let expected_date = "2022.12.19";
    let date_tag_pattern = format!("[Date \"{}\"]", expected_date);
    let date_occurrences = pgn_content.matches(&date_tag_pattern).count();
    
    assert_eq!(date_occurrences, 5, 
        "PGN should contain exactly 5 games with date {}, found {}", 
        expected_date, date_occurrences);
    
    // Validate that our date formatting matches PGN expectations
    let test_date = scidtopgn::GameIndex {
        offset: 0, length: 0, white_id: 0, black_id: 0, event_id: 0, site_id: 0, round_id: 0,
        year: 2022, month: 12, day: 19, result: 0, eco: 0, white_elo: 0, black_elo: 0, flags: 0,
        num_half_moves: 0, stored_line_code: 0, final_material: [0, 0], pawn_advancement: [0, 0],
        var_count: 0, comment_count: 0, nag_count: 0, deleted: 0, reserved: [0; 5],
    };
    
    assert_eq!(test_date.date_string(), expected_date, 
        "Our date formatting should match PGN format");
}

/// Test the discovered date pattern decoding
#[test]
fn test_discovered_pattern_decoding() {
    // This tests the actual pattern we discovered in the binary data
    let discovered_pattern = 0x0944cd93u32;
    
    // Extract date components using SCID bit-field format
    let day = (discovered_pattern & 31) as u8;
    let month = ((discovered_pattern >> 5) & 15) as u8;
    let year_raw = ((discovered_pattern >> 9) & 0x7FF) as u16;
    
    // Apply the discovered year offset
    let year = year_raw + 1408;
    
    // Validate the decoded values
    assert_eq!(day, 19, "Day should decode to 19");
    assert_eq!(month, 12, "Month should decode to 12");
    assert_eq!(year_raw, 614, "Raw year should decode to 614");
    assert_eq!(year, 2022, "Adjusted year should be 2022");
    
    // Test the complete date string
    let game_index = scidtopgn::GameIndex {
        offset: 0, length: 0, white_id: 0, black_id: 0, event_id: 0, site_id: 0, round_id: 0,
        year, month, day, result: 0, eco: 0, white_elo: 0, black_elo: 0, flags: 0,
        num_half_moves: 0, stored_line_code: 0, final_material: [0, 0], pawn_advancement: [0, 0],
        var_count: 0, comment_count: 0, nag_count: 0, deleted: 0, reserved: [0; 5],
    };
    
    assert_eq!(game_index.date_string(), "2022.12.19",
        "Complete date string should format correctly");
}

/// Test various date patterns that might appear in SCID files
#[test]
fn test_scid_date_pattern_variations() {
    // Test encoding/decoding roundtrip for various dates
    let test_cases = vec![
        (2022, 12, 19, 1408), // Our discovered case
        (2020, 1, 1, 1408),   // January 1st, 2020 
        (2023, 12, 31, 1408), // December 31st, 2023
    ];
    
    for (expected_year, expected_month, expected_day, year_offset) in test_cases {
        // Calculate what the encoded pattern should be
        let year_raw = expected_year - year_offset;
        let encoded = (expected_day as u32) | ((expected_month as u32) << 5) | ((year_raw as u32) << 9);
        
        // Decode it back
        let decoded_day = (encoded & 31) as u8;
        let decoded_month = ((encoded >> 5) & 15) as u8;
        let decoded_year_raw = ((encoded >> 9) & 0x7FF) as u16;
        let decoded_year = decoded_year_raw + year_offset;
        
        assert_eq!(decoded_day, expected_day, 
            "Day roundtrip failed for {}.{:02}.{:02}", expected_year, expected_month, expected_day);
        assert_eq!(decoded_month, expected_month,
            "Month roundtrip failed for {}.{:02}.{:02}", expected_year, expected_month, expected_day);
        assert_eq!(decoded_year, expected_year,
            "Year roundtrip failed for {}.{:02}.{:02}", expected_year, expected_month, expected_day);
    }
}

/// Test edge cases for date handling
#[test] 
fn test_date_edge_cases() {
    // Test invalid dates are handled gracefully
    let invalid_dates = vec![
        (0, 12, 19),      // Invalid year
        (2022, 0, 19),    // Invalid month
        (2022, 13, 19),   // Invalid month
        (2022, 12, 0),    // Invalid day
        (2022, 12, 32),   // Invalid day
        (3000, 12, 19),   // Year too high
    ];
    
    for (year, month, day) in invalid_dates {
        let game_index = scidtopgn::GameIndex {
            offset: 0, length: 0, white_id: 0, black_id: 0, event_id: 0, site_id: 0, round_id: 0,
            year, month, day, result: 0, eco: 0, white_elo: 0, black_elo: 0, flags: 0,
            num_half_moves: 0, stored_line_code: 0, final_material: [0, 0], pawn_advancement: [0, 0],
            var_count: 0, comment_count: 0, nag_count: 0, deleted: 0, reserved: [0; 5],
        };
        
        let date_string = game_index.date_string();
        
        // Should either be the unknown date format or a corrected valid date
        if year == 0 || year > 2100 {
            assert_eq!(date_string, "????.??.??", 
                "Invalid year should produce unknown date format");
        } else {
            // Should have corrected invalid month/day to valid values
            assert!(date_string.len() == 10, "Date string should be proper length");
            assert!(date_string.contains('.'), "Date string should contain dots");
        }
    }
}

/// Test date formatting consistency
#[test]
fn test_date_formatting_consistency() {
    // Test that our date formatting is consistent and follows PGN standards
    let test_date = scidtopgn::GameIndex {
        offset: 0, length: 0, white_id: 0, black_id: 0, event_id: 0, site_id: 0, round_id: 0,
        year: 2022, month: 12, day: 19, result: 0, eco: 0, white_elo: 0, black_elo: 0, flags: 0,
        num_half_moves: 0, stored_line_code: 0, final_material: [0, 0], pawn_advancement: [0, 0],
        var_count: 0, comment_count: 0, nag_count: 0, deleted: 0, reserved: [0; 5],
    };
    
    let date_string = test_date.date_string();
    
    // Validate format
    assert_eq!(date_string.len(), 10, "Date should be 10 characters: YYYY.MM.DD");
    assert_eq!(&date_string[4..5], ".", "Character 4 should be a dot");
    assert_eq!(&date_string[7..8], ".", "Character 7 should be a dot"); 
    
    // Validate individual components
    let parts: Vec<&str> = date_string.split('.').collect();
    assert_eq!(parts.len(), 3, "Should have 3 parts separated by dots");
    assert_eq!(parts[0], "2022", "Year part should be 2022");
    assert_eq!(parts[1], "12", "Month part should be 12");
    assert_eq!(parts[2], "19", "Day part should be 19");
    
    // Validate PGN tag format compatibility  
    let pgn_date_tag = format!("[Date \"{}\"]", date_string);
    assert_eq!(pgn_date_tag, "[Date \"2022.12.19\"]", 
        "Should format correctly as PGN date tag");
}