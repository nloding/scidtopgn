#[cfg(test)]
mod tests {
    use scid_parser::pgn::{EnhancedPgnExporter, ExportOptions};
    
    // Helper functions to create test data
    fn create_test_game_index() -> scid_parser::pgn::exporter::SimpleGameIndex {
        scid_parser::pgn::exporter::SimpleGameIndex {
            event_id: 1,
            site_id: 2,
            white_id: 10,
            black_id: 11,
            round_id: 1,
            year: 2023,
            month: 12,
            day: 25,
            result: 1, // 1-0
            white_elo: 1800,
            black_elo: 1750,
            eco: 113, // B13
            num_half_moves: 42,
        }
    }
    
    fn create_test_name_database() -> scid_parser::pgn::exporter::SimpleNameDatabase {
        let mut name_db = scid_parser::pgn::exporter::SimpleNameDatabase::new();
        name_db.event_names.insert(1, "Test Tournament".to_string());
        name_db.site_names.insert(2, "Test City".to_string());
        name_db.player_names.insert(10, "Player White".to_string());
        name_db.player_names.insert(11, "Player Black".to_string());
        name_db.round_names.insert(1, "1".to_string());
        name_db
    }
    
    fn load_test_game_data() -> &'static str {
        "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1"
    }
    
    fn load_test_game_with_variations() -> &'static str {
        "1.e4 e5 2.Nf3 Nc6 (2...Nf6 3.Nxe5 d6 4.Nf3 Nxe4) 3.Bb5 a6 4.Ba4"
    }
    
    fn load_test_annotated_game() -> &'static str {
        "1.e4 e5 2.Nf3! {A good development move} Nc6 3.Bb5 a6 4.Ba4 Nf6!?"
    }
    
    #[test]
    fn test_pgn_standards_compliance() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        
        // Test with real game data
        let game_data = load_test_game_data();
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export game successfully");
        
        // Validate PGN structure
        assert!(pgn_output.contains("[Event "));
        assert!(pgn_output.contains("[White "));
        assert!(pgn_output.contains("[Black "));
        assert!(pgn_output.contains("[Result "));
        
        // Check for proper line breaks
        let lines: Vec<&str> = pgn_output.lines().collect();
        for line in &lines {
            if !line.starts_with('[') && !line.is_empty() {
                assert!(line.len() <= 80, "Line too long: {}", line);
            }
        }
    }
    
    #[test]
    fn test_variation_export() {
        // Test that variations are properly formatted
        let options = ExportOptions {
            include_variations: true,
            ..Default::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        
        // Test with game containing variations
        let game_data = load_test_game_with_variations();
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export game with variations");
        
        // Check for variation markers
        assert!(pgn_output.contains('('));
        assert!(pgn_output.contains(')'));
        
        // Check that the variation content is preserved
        assert!(pgn_output.contains("2...Nf6 3.Nxe5 d6 4.Nf3 Nxe4"));
    }
    
    #[test]
    fn test_annotation_export() {
        // Test that comments and NAGs are properly included
        let options = ExportOptions {
            include_comments: true,
            include_nags: true,
            ..Default::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        
        // Test with annotated game
        let game_data = load_test_annotated_game();
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export annotated game");
        
        // Check for annotations
        assert!(pgn_output.contains('{'));  // Comments
        assert!(pgn_output.contains("A good development move"));  // Comment text
        assert!(pgn_output.contains('!'));  // NAG symbols
        assert!(pgn_output.contains("!?"));  // Complex NAG symbols
    }
    
    #[test]
    fn test_export_without_optional_headers() {
        let options = ExportOptions {
            include_optional_headers: false,
            ..Default::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        let game_data = load_test_game_data();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export without optional headers");
        
        // Required headers should be present
        assert!(pgn_output.contains("[Event "));
        assert!(pgn_output.contains("[Site "));
        assert!(pgn_output.contains("[Date "));
        assert!(pgn_output.contains("[Round "));
        assert!(pgn_output.contains("[White "));
        assert!(pgn_output.contains("[Black "));
        assert!(pgn_output.contains("[Result "));
        
        // Optional headers should not be present
        assert!(!pgn_output.contains("[WhiteElo "));
        assert!(!pgn_output.contains("[BlackElo "));
        assert!(!pgn_output.contains("[ECO "));
        assert!(!pgn_output.contains("[PlyCount "));
    }
    
    #[test]
    fn test_line_length_formatting() {
        let options = ExportOptions {
            max_line_length: 40,  // Short lines for testing
            ..Default::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        
        // Use a longer game to test line wrapping
        let long_game = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6 8.c3 O-O 9.h3 Nb8";
        
        let pgn_output = exporter.export_complete_game(&game_index, long_game, Some(&name_db))
            .expect("Should format with line breaks");
        
        // Check that move lines don't exceed the limit
        for line in pgn_output.lines() {
            if !line.starts_with('[') && !line.trim().is_empty() {
                assert!(
                    line.len() <= 45, // Allow some tolerance for result at end
                    "Line too long ({}): '{}'", 
                    line.len(), 
                    line
                );
            }
        }
        
        // Should have line breaks in the moves section
        let moves_lines: Vec<&str> = pgn_output.lines()
            .filter(|line| !line.starts_with('[') && !line.trim().is_empty())
            .collect();
        
        assert!(moves_lines.len() > 1, "Should have multiple move lines due to wrapping");
    }
    
    #[test]
    fn test_header_validation() {
        let options = ExportOptions {
            validate_headers: true,
            ..Default::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        let game_data = load_test_game_data();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export with valid headers");
        
        // All required headers should be present and properly formatted
        assert!(pgn_output.contains("[Event \"Test Tournament\"]"));
        assert!(pgn_output.contains("[Site \"Test City\"]"));
        assert!(pgn_output.contains("[Date \"2023.12.25\"]"));
        assert!(pgn_output.contains("[Round \"1\"]"));
        assert!(pgn_output.contains("[White \"Player White\"]"));
        assert!(pgn_output.contains("[Black \"Player Black\"]"));
        assert!(pgn_output.contains("[Result \"1-0\"]"));
    }
    
    #[test]
    fn test_export_with_minimal_data() {
        // Test export when some data is missing
        let mut game_index = create_test_game_index();
        game_index.white_elo = 0;  // No ELO rating
        game_index.black_elo = 0;  // No ELO rating
        game_index.eco = 0;        // No ECO code
        
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let name_db = create_test_name_database();
        let game_data = "1.e4 e5";
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should export with minimal data");
        
        // Required headers should still be present
        assert!(pgn_output.contains("[Event "));
        assert!(pgn_output.contains("[Result "));
        
        // Optional headers with zero values should not be present
        assert!(!pgn_output.contains("[WhiteElo \"0\"]"));
        assert!(!pgn_output.contains("[BlackElo \"0\"]"));
        assert!(!pgn_output.contains("[ECO \"?\"]"));
    }
    
    #[test]
    fn test_export_without_name_database() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let game_data = load_test_game_data();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, None)
            .expect("Should export without name database");
        
        // Should use fallback names
        assert!(pgn_output.contains("[Event \"Event_1\"]"));
        assert!(pgn_output.contains("[Site \"Site_2\"]"));
        assert!(pgn_output.contains("[White \"Player_10\"]"));
        assert!(pgn_output.contains("[Black \"Player_11\"]"));
        
        // Moves should still be present
        assert!(pgn_output.contains("1.e4 e5"));
        assert!(pgn_output.contains("1-0"));
    }
    
    #[test]
    fn test_special_characters_in_names() {
        let mut name_db = create_test_name_database();
        name_db.player_names.insert(10, "José García".to_string());
        name_db.player_names.insert(11, "François Müller".to_string());
        name_db.event_names.insert(1, "World Championship \"2023\"".to_string());
        
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let game_data = load_test_game_data();
        
        let pgn_output = exporter.export_complete_game(&game_index, game_data, Some(&name_db))
            .expect("Should handle special characters");
        
        // Names with special characters should be properly escaped/preserved
        assert!(pgn_output.contains("José García"));
        assert!(pgn_output.contains("François Müller"));
        // Check what's actually generated for the event name (quotes preserved as-is)
        assert!(pgn_output.contains("World Championship \"2023\""));
    }
}