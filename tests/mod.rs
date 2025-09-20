//! Test module for SCIDtoPGN library
//! 
//! This module contains comprehensive tests for the SCIDtoPGN library,
//! including integration tests, unit tests, and property-based tests.

// Import test modules
mod integration;
mod unit;
mod property;

// Re-export common test utilities
pub use integration::*;
pub use test_utils::*;

/// Common test utilities and fixtures
pub mod test_utils {
    use std::path::PathBuf;
    use std::fs;
    use scidtopgn::api::ScidDatabase;
    use scidtopgn::core::error::Result;
    
    /// Get the path to test data directory
    pub fn test_data_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("Failed to find manifest directory")
            .join("tests")
            .join("data")
    }
    
    /// Get the path to the five test database files
    pub fn five_test_data() -> PathBuf {
        test_data_dir().join("five")
    }
    
    /// Get the path to the five test database files
    pub fn five_test_files() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        let base = five_test_data();
        (
            base.with_extension("si4"),
            base.with_extension("sn4"),
            base.with_extension("sg4"),
            base.with_extension("pgn"),
        )
    }
    
    /// Check if test data files exist
    pub fn test_files_exist() -> bool {
        let (si4_path, sn4_path, sg4_path, pgn_path) = five_test_files();
        si4_path.exists() && sn4_path.exists() && sg4_path.exists() && pgn_path.exists()
    }
    
    /// Create a test database from the five test data
    pub fn create_test_database() -> Result<ScidDatabase> {
        let base = five_test_data();
        ScidDatabase::open(&base)
    }
    
    /// Load reference PGN content
    pub fn load_reference_pgn() -> Result<String> {
        let (_, _, _, pgn_path) = five_test_files();
        fs::read_to_string(&pgn_path)
            .map_err(|e| scidtopgn::core::error::ScidError::file_error(format!("Failed to read reference PGN: {}", e)))
    }
    
    /// Test database creation and validation
    pub fn validate_test_database(db: &ScidDatabase) -> Result<()> {
        // Check that the database has the expected number of games
        assert_eq!(db.num_games(), 5, "Test database should have exactly 5 games");
        
        // Check that the database is validated
        assert!(db.is_validated(), "Test database should be validated");
        
        // Test that we can access all games
        let games: Vec<_> = db.games().collect();
        assert_eq!(games.len(), 5, "Should be able to access all 5 games");
        
        Ok(())
    }
    
    /// Test game export functionality
    pub fn test_game_export(db: &ScidDatabase) -> Result<()> {
        let games: Vec<_> = db.games().collect();
        assert_eq!(games.len(), 5, "Should have 5 games to test");
        
        for (index, game) in games.iter().enumerate() {
            // Test that each game has the expected structure
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
        
        Ok(())
    }
    
    /// Create a temporary file path for testing
    pub fn temp_file_path() -> PathBuf {
        use std::env;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .as_secs();
        
        let mut path = std::env::temp_dir();
        path.push(format!("test_{}.scid", timestamp));
        path
    }
    
    /// Create a test database with custom path
    pub fn create_test_database_with_path(path: &PathBuf) -> Result<ScidDatabase> {
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
            si4_file: create_mock_si4_file(&game_index),
            sn4_file: create_mock_sn4_file(),
            sg4_file: create_mock_sg4_file(),
            game_index,
            game_state,
            parsed_game,
        })
    }
    
    /// Create a mock SI4 file for testing
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
    
    /// Create a mock SN4 file for testing
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
    
    /// Create a mock SG4 file for testing
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
    
    /// Test game export functionality
    pub fn test_game_export(db: &ScidDatabase) -> Result<()> {
        for (index, game_result) in db.games().enumerate() {
            let game = game_result?;
            
            // Test that each game has the expected structure
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
        
        Ok(())
    }
    
    /// Test PGN export accuracy
    pub fn test_pgn_export_accuracy(db: &ScidDatabase) -> Result<()> {
        let reference_pgn = load_reference_pgn()?;
        
        for (index, game_result) in db.games().enumerate() {
            let game = game_result?;
            
            // Export game to PGN
            let exported_pgn = scidtopgn::pgn::PgnExporter::new(&game.game_state, &game.parsed_game)?
                .export()?;
            
            // Basic validation of exported PGN
            assert!(exported_pgn.contains("[Event"), "Exported PGN should have Event tag");
            assert!(exported_pgn.contains("[Site]"), "Exported PGN should have Site tag");
            assert!(exported_pgn.contains("[Date]"), "Exported PGN should have Date tag");
            assert!(exported_pgn.contains("[White]"), "Exported PGN should have White tag");
            assert!(exported_pgn.contains("[Black]"), "Exported PGN should have Black tag");
            assert!(exported_pgn.contains("[Result]"), "Exported PGN should have Result tag");
            
            // Test that the game has moves
            assert!(exported_pgn.contains("1."), "Exported PGN should have moves");
        }
        
        Ok(())
    }
    
    /// Test error handling for corrupt files
    pub fn test_error_handling() -> Result<()> {
        // Test with non-existent database
        let result = ScidDatabase::open("non_existent_database");
        assert!(result.is_err(), "Should return error for non-existent database");
        
        // Test with invalid file paths
        let mut invalid_path = five_test_data();
        invalid_path.set_extension("invalid");
        
        let result = ScidDatabase::open(&invalid_path);
        assert!(result.is_err(), "Should return error for invalid database files");
        
        Ok(())
    }
    
    /// Test performance with large databases
    pub fn test_performance_with_large_database() -> Result<()> {
        let db = create_test_database()?;
        
        // Test game iteration performance
        let start_time = std::time::Instant::now();
        let game_count: usize = db.games().count();
        let iteration_time = start_time.elapsed();
        
        // Performance assertions (adjust thresholds as needed)
        assert!(iteration_time.as_millis() < 1000, "Game iteration should be fast");
        assert!(game_count == 5, "Should count all 5 games");
        
        // Test PGN export performance
        let export_start = std::time::Instant::now();
        let mut successful_exports = 0;
        
        for game_result in db.games() {
            if let Ok(game) = game_result {
                if scidtopgn::pgn::PgnExporter::new(&game.game_state, &game.parsed_game).is_ok() {
                    successful_exports += 1;
                }
            }
        }
        let export_time = export_start.elapsed();
        
        // Performance assertions
        assert!(export_time.as_millis() < 1000, "PGN export should be fast");
        assert_eq!(successful_exports, 5, "Should export all games successfully");
        
        Ok(())
    }
}