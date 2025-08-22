use crate::error::{Result, ScidError};
use crate::bridge::{GameState, ChessValidation, ValidationReport, GameTermination, PositionContext};
use crate::bridge::position::GameMetadata;
use shakmaty::Color;

/// Standards-compliant PGN exporter with chess validation
pub struct PgnExporter {
    /// Whether to include optional PGN tags
    include_optional_tags: bool,
    /// Whether to validate moves before export
    validate_moves: bool,
    /// Whether to include comments and variations (future enhancement)
    #[allow(dead_code)]
    include_annotations: bool,
}

/// Represents a parsed game ready for PGN export
pub struct ParsedGame {
    /// Game state with complete move history
    pub game_state: GameState,
    /// Validation report for the game
    pub validation_report: ValidationReport,
    /// Game metadata
    pub metadata: GameMetadata,
}

impl PgnExporter {
    /// Create a new PGN exporter with default settings
    pub fn new() -> Self {
        Self {
            include_optional_tags: true,
            validate_moves: true,
            include_annotations: false,
        }
    }
    
    /// Create a PGN exporter with custom settings
    pub fn with_settings(include_optional_tags: bool, validate_moves: bool) -> Self {
        Self {
            include_optional_tags,
            validate_moves,
            include_annotations: false,
        }
    }
    
    /// Export a game to PGN format with full validation
    /// This is the main method that implements the complete pipeline
    pub fn export_with_validation(&self, parsed_game: &ParsedGame) -> Result<String> {
        // Validate the game if validation is enabled
        if self.validate_moves && !parsed_game.validation_report.is_valid {
            return Err(ScidError::conversion_error(format!(
                "Game contains {} invalid moves and cannot be exported", 
                parsed_game.validation_report.invalid_moves.len()
            )));
        }
        
        let mut pgn = String::new();
        
        // 1. Export validated metadata headers
        self.export_pgn_headers(&mut pgn, &parsed_game.metadata)?;
        
        // 2. Export validated move sequence with proper SAN notation
        self.export_move_sequence(&mut pgn, &parsed_game.game_state)?;
        
        // 3. Add game termination based on validation report
        self.export_game_termination(&mut pgn, &parsed_game.validation_report);
        
        Ok(pgn)
    }
    
    /// Export PGN headers according to PGN standard
    fn export_pgn_headers(&self, pgn: &mut String, metadata: &GameMetadata) -> Result<()> {
        // Seven Tag Roster (STR) - mandatory PGN tags
        pgn.push_str(&format!("[Event \"{}\"]\n", escape_pgn_string(&metadata.event)));
        pgn.push_str(&format!("[Site \"{}\"]\n", escape_pgn_string(&metadata.site)));
        pgn.push_str(&format!("[Date \"{}\"]\n", format_pgn_date(&metadata.date)));
        pgn.push_str(&format!("[Round \"{}\"]\n", escape_pgn_string(&metadata.round.as_deref().unwrap_or("?"))));
        pgn.push_str(&format!("[White \"{}\"]\n", escape_pgn_string(&metadata.white)));
        pgn.push_str(&format!("[Black \"{}\"]\n", escape_pgn_string(&metadata.black)));
        pgn.push_str(&format!("[Result \"{}\"]\n", metadata.result));
        
        // Optional tags (if enabled and available)
        if self.include_optional_tags {
            if let Some(elo) = metadata.white_elo {
                pgn.push_str(&format!("[WhiteElo \"{}\"]\n", elo));
            }
            if let Some(elo) = metadata.black_elo {
                pgn.push_str(&format!("[BlackElo \"{}\"]\n", elo));
            }
            if let Some(ref eco) = metadata.eco {
                pgn.push_str(&format!("[ECO \"{}\"]\n", escape_pgn_string(eco)));
            }
        }
        
        // Blank line after headers
        pgn.push('\n');
        
        Ok(())
    }
    
    /// Export the move sequence in standard PGN movetext format
    fn export_move_sequence(&self, pgn: &mut String, game_state: &GameState) -> Result<()> {
        let san_moves = game_state.san_moves();
        
        if san_moves.is_empty() {
            // Empty game - just add the result
            return Ok(());
        }
        
        let mut move_text = String::new();
        
        for (i, san_move) in san_moves.iter().enumerate() {
            if i % 2 == 0 {
                // White's move - add move number
                if i > 0 {
                    move_text.push(' ');
                }
                move_text.push_str(&format!("{}. {}", (i / 2) + 1, san_move));
            } else {
                // Black's move
                move_text.push_str(&format!(" {}", san_move));
            }
        }
        
        // Format the movetext with proper line wrapping (80 characters per line)
        let wrapped_text = wrap_movetext(&move_text, 80);
        pgn.push_str(&wrapped_text);
        
        Ok(())
    }
    
    /// Export game termination marker based on validation report
    fn export_game_termination(&self, pgn: &mut String, validation_report: &ValidationReport) {
        // Add a space before the result if there are moves
        if validation_report.total_moves > 0 {
            pgn.push(' ');
        }
        
        // Determine result based on termination type
        let result = match validation_report.termination {
            GameTermination::Checkmate { winner } => {
                match winner {
                    Color::White => "1-0",
                    Color::Black => "0-1",
                }
            }
            GameTermination::Stalemate => "1/2-1/2",
            GameTermination::InsufficientMaterial => "1/2-1/2",
            GameTermination::Repetition => "1/2-1/2",
            GameTermination::FiftyMoveRule => "1/2-1/2",
            GameTermination::InProgress => "*",
            GameTermination::Incomplete => "*",
        };
        
        pgn.push_str(result);
        pgn.push('\n');
    }
    
    /// Export a game state directly (convenience method)
    pub fn export_game_state(&self, game_state: &GameState, metadata: &GameMetadata) -> Result<String> {
        // Create validation report for the game
        let validator = crate::bridge::ChessValidator::new();
        let validation_report = validator.validate_move_sequence(game_state.move_history())?;
        
        let parsed_game = ParsedGame {
            game_state: game_state.clone(), // Note: Consider making this a reference in the future
            validation_report,
            metadata: metadata.clone(),
        };
        
        self.export_with_validation(&parsed_game)
    }
}

impl Default for PgnExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Escape special characters in PGN strings according to PGN specification
fn escape_pgn_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
}

/// Format date according to PGN specification (YYYY.MM.DD)
fn format_pgn_date(date: &str) -> String {
    // If date is already in correct format, return as-is
    if date.matches('.').count() == 2 && date.len() == 10 {
        return date.to_string();
    }
    
    // Handle various date formats and convert to PGN format
    // For now, return the date as-is or use placeholders
    if date.is_empty() || date == "?" {
        "????.??.??".to_string()
    } else {
        date.to_string()
    }
}

/// Wrap movetext to specified line width
fn wrap_movetext(movetext: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_line = String::new();
    
    for word in movetext.split(' ') {
        if current_line.len() + word.len() + 1 > width && !current_line.is_empty() {
            result.push_str(&current_line);
            result.push('\n');
            current_line.clear();
        }
        
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    
    if !current_line.is_empty() {
        result.push_str(&current_line);
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::{ChessValidator, GameState};
    use shakmaty::{Square, Role};
    
    #[test]
    fn test_pgn_exporter_creation() {
        let exporter = PgnExporter::new();
        assert!(exporter.include_optional_tags);
        assert!(exporter.validate_moves);
        
        let custom_exporter = PgnExporter::with_settings(false, false);
        assert!(!custom_exporter.include_optional_tags);
        assert!(!custom_exporter.validate_moves);
    }
    
    #[test]
    fn test_escape_pgn_string() {
        assert_eq!(escape_pgn_string("Normal text"), "Normal text");
        assert_eq!(escape_pgn_string("Text with \"quotes\""), "Text with \\\"quotes\\\"");
        assert_eq!(escape_pgn_string("Text with \\ backslash"), "Text with \\\\ backslash");
        assert_eq!(escape_pgn_string("Text with\nnewline"), "Text with\\nnewline");
    }
    
    #[test]
    fn test_format_pgn_date() {
        assert_eq!(format_pgn_date("2022.12.19"), "2022.12.19");
        assert_eq!(format_pgn_date(""), "????.??.??");
        assert_eq!(format_pgn_date("?"), "????.??.??");
        assert_eq!(format_pgn_date("invalid"), "invalid");
    }
    
    #[test]
    fn test_wrap_movetext() {
        let text = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 4. Ba4 Nf6 5. O-O Be7";
        let wrapped = wrap_movetext(text, 20);
        let lines: Vec<&str> = wrapped.split('\n').collect();
        
        // Each line should be under or at the limit
        for line in &lines {
            assert!(line.len() <= 20);
        }
        
        // Should contain the original content when joined
        let rejoined = lines.join(" ");
        assert!(rejoined.contains("1. e4 e5"));
    }
    
    #[test]
    fn test_empty_game_export() {
        let exporter = PgnExporter::new();
        let game_state = GameState::new();
        let metadata = GameMetadata {
            event: "Test Event".to_string(),
            site: "Test Site".to_string(),
            date: "2022.12.19".to_string(),
            white: "White Player".to_string(),
            black: "Black Player".to_string(),
            result: "*".to_string(),
            round: Some("1".to_string()),
            white_elo: Some(1500),
            black_elo: Some(1600),
            eco: Some("B00".to_string()),
        };
        
        let pgn = exporter.export_game_state(&game_state, &metadata).unwrap();
        
        // Should contain all required headers
        assert!(pgn.contains("[Event \"Test Event\"]"));
        assert!(pgn.contains("[White \"White Player\"]"));
        assert!(pgn.contains("[Black \"Black Player\"]"));
        assert!(pgn.contains("[Result \"*\"]"));
        assert!(pgn.contains("[WhiteElo \"1500\"]"));
        assert!(pgn.contains("[BlackElo \"1600\"]"));
        
        // Should end with result
        assert!(pgn.trim().ends_with('*'));
    }
}

// ==========================================
// PHASE 4: ENHANCED PGN EXPORTER
// Complete PGN export with standards compliance
// From SCID_TO_PGN_COMPLETION_PLAN.md Phase 4.2
// ==========================================

use crate::pgn::{VariationFormatter, AnnotationFormatter, PgnStandardsChecker};

/// Enhanced PGN exporter with complete feature support
pub struct EnhancedPgnExporter {
    /// Standards compliance checker
    standards_checker: PgnStandardsChecker,
    
    /// Export options
    options: ExportOptions,
}

#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include variations in output
    pub include_variations: bool,
    
    /// Include comments in output
    pub include_comments: bool,
    
    /// Include NAG annotations
    pub include_nags: bool,
    
    /// Maximum line length for formatting
    pub max_line_length: usize,
    
    /// Include optional headers (ECO, WhiteElo, etc.)
    pub include_optional_headers: bool,
    
    /// Validate headers according to PGN standards
    pub validate_headers: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_variations: true,
            include_comments: true,
            include_nags: true,
            max_line_length: 80,
            include_optional_headers: true,
            validate_headers: true,
        }
    }
}

/// Simple game index structure for enhanced exporter
#[derive(Debug, Clone)]
pub struct SimpleGameIndex {
    pub event_id: u32,
    pub site_id: u32,
    pub white_id: u32,
    pub black_id: u32,
    pub round_id: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub result: u8,  // 0=*, 1=1-0, 2=0-1, 3=1/2-1/2
    pub white_elo: u16,
    pub black_elo: u16,
    pub eco: u16,
    pub num_half_moves: u16,
}

/// Simple name database for lookups
#[derive(Debug, Clone)]
pub struct SimpleNameDatabase {
    pub event_names: std::collections::HashMap<u32, String>,
    pub site_names: std::collections::HashMap<u32, String>,
    pub player_names: std::collections::HashMap<u32, String>,
    pub round_names: std::collections::HashMap<u32, String>,
}

impl SimpleNameDatabase {
    pub fn new() -> Self {
        Self {
            event_names: std::collections::HashMap::new(),
            site_names: std::collections::HashMap::new(),
            player_names: std::collections::HashMap::new(),
            round_names: std::collections::HashMap::new(),
        }
    }
    
    pub fn get_event_name(&self, id: u32) -> Option<String> {
        self.event_names.get(&id).cloned()
    }
    
    pub fn get_site_name(&self, id: u32) -> Option<String> {
        self.site_names.get(&id).cloned()
    }
    
    pub fn get_player_name(&self, id: u32) -> Option<String> {
        self.player_names.get(&id).cloned()
    }
    
    pub fn get_round_name(&self, id: u32) -> Option<String> {
        self.round_names.get(&id).cloned()
    }
}

impl EnhancedPgnExporter {
    pub fn new(options: ExportOptions) -> Self {
        let mut standards_checker = PgnStandardsChecker::new();
        if options.max_line_length != 80 {
            standards_checker = PgnStandardsChecker::with_max_line_length(options.max_line_length);
        }
        
        Self {
            standards_checker,
            options,
        }
    }
    
    /// Export complete game to PGN
    pub fn export_complete_game(
        &self,
        game_index: &SimpleGameIndex,
        game_moves: &str, // Pre-formatted moves string
        name_database: Option<&SimpleNameDatabase>,
    ) -> Result<String> {
        let mut pgn = String::new();
        
        // Generate headers
        let headers = self.generate_headers(game_index, name_database)?;
        
        // Validate headers if requested
        if self.options.validate_headers {
            let header_refs: Vec<(&str, &str)> = headers.iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            self.standards_checker.validate_headers(&header_refs)
                .map_err(|e| ScidError::invalid_format(format!("Header validation failed: {}", e)))?;
        }
        
        // Write headers
        for (key, value) in &headers {
            pgn.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        
        pgn.push('\n');
        
        // Format moves according to PGN standards
        let formatted_moves = if game_moves.trim().is_empty() {
            String::new()
        } else {
            self.standards_checker.format_pgn(game_moves)
        };
        
        pgn.push_str(&formatted_moves);
        
        // Add result if not already present
        if !formatted_moves.trim().is_empty() && !formatted_moves.contains(&self.format_result(game_index.result)) {
            pgn.push(' ');
            pgn.push_str(&self.format_result(game_index.result));
        }
        
        pgn.push('\n');
        
        Ok(pgn)
    }
    
    /// Generate complete PGN headers
    fn generate_headers(&self, game_index: &SimpleGameIndex, name_database: Option<&SimpleNameDatabase>) -> Result<Vec<(String, String)>> {
        let mut headers = Vec::new();
        
        // Required headers
        headers.push(("Event".to_string(), self.get_event_name(game_index.event_id, name_database)));
        headers.push(("Site".to_string(), self.get_site_name(game_index.site_id, name_database)));
        headers.push(("Date".to_string(), self.format_date(game_index.year, game_index.month, game_index.day)));
        headers.push(("Round".to_string(), self.get_round_name(game_index.round_id, name_database)));
        headers.push(("White".to_string(), self.get_player_name(game_index.white_id, name_database)));
        headers.push(("Black".to_string(), self.get_player_name(game_index.black_id, name_database)));
        headers.push(("Result".to_string(), self.format_result(game_index.result)));
        
        // Optional headers
        if self.options.include_optional_headers {
            if game_index.white_elo > 0 {
                headers.push(("WhiteElo".to_string(), game_index.white_elo.to_string()));
            }
            if game_index.black_elo > 0 {
                headers.push(("BlackElo".to_string(), game_index.black_elo.to_string()));
            }
            if game_index.eco > 0 {
                headers.push(("ECO".to_string(), self.format_eco(game_index.eco)));
            }
            if game_index.num_half_moves > 0 {
                headers.push(("PlyCount".to_string(), game_index.num_half_moves.to_string()));
            }
        }
        
        Ok(headers)
    }
    
    // Helper methods for name lookups
    fn get_event_name(&self, event_id: u32, name_db: Option<&SimpleNameDatabase>) -> String {
        if let Some(name_db) = name_db {
            name_db.get_event_name(event_id).unwrap_or_else(|| format!("Event_{}", event_id))
        } else {
            format!("Event_{}", event_id)
        }
    }
    
    fn get_site_name(&self, site_id: u32, name_db: Option<&SimpleNameDatabase>) -> String {
        if let Some(name_db) = name_db {
            name_db.get_site_name(site_id).unwrap_or_else(|| format!("Site_{}", site_id))
        } else {
            format!("Site_{}", site_id)
        }
    }
    
    fn get_player_name(&self, player_id: u32, name_db: Option<&SimpleNameDatabase>) -> String {
        if let Some(name_db) = name_db {
            name_db.get_player_name(player_id).unwrap_or_else(|| format!("Player_{}", player_id))
        } else {
            format!("Player_{}", player_id)
        }
    }
    
    fn get_round_name(&self, round_id: u32, name_db: Option<&SimpleNameDatabase>) -> String {
        if let Some(name_db) = name_db {
            name_db.get_round_name(round_id).unwrap_or_else(|| {
                if round_id == 0 {
                    "?".to_string()
                } else {
                    round_id.to_string()
                }
            })
        } else if round_id == 0 {
            "?".to_string()
        } else {
            round_id.to_string()
        }
    }
    
    fn format_date(&self, year: u16, month: u8, day: u8) -> String {
        let year_str = if year == 0 { "????".to_string() } else { year.to_string() };
        let month_str = if month == 0 { "??".to_string() } else { format!("{:02}", month) };
        let day_str = if day == 0 { "??".to_string() } else { format!("{:02}", day) };
        
        format!("{}.{}.{}", year_str, month_str, day_str)
    }
    
    fn format_result(&self, result: u8) -> String {
        match result {
            1 => "1-0".to_string(),
            2 => "0-1".to_string(),
            3 => "1/2-1/2".to_string(),
            _ => "*".to_string(),
        }
    }
    
    fn format_eco(&self, eco: u16) -> String {
        // ECO codes are A00-E99
        if eco == 0 {
            return "?".to_string();
        }
        
        // Convert to ECO format using standard encoding
        let section = match eco / 100 {
            0 => 'A',
            1 => 'B', 
            2 => 'C',
            3 => 'D',
            4 => 'E',
            _ => '?',
        };
        
        let number = eco % 100;
        format!("{}{:02}", section, number)
    }
}

#[cfg(test)]
mod enhanced_exporter_tests {
    use super::*;

    fn create_test_game_index() -> SimpleGameIndex {
        SimpleGameIndex {
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
    
    fn create_test_name_database() -> SimpleNameDatabase {
        let mut name_db = SimpleNameDatabase::new();
        name_db.event_names.insert(1, "Test Tournament".to_string());
        name_db.site_names.insert(2, "Test City".to_string());
        name_db.player_names.insert(10, "Player White".to_string());
        name_db.player_names.insert(11, "Player Black".to_string());
        name_db.round_names.insert(1, "1".to_string());
        name_db
    }

    #[test]
    fn test_enhanced_exporter_basic_export() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        let moves = "1.e4 e5 2.Nf3 Nc6 3.Bb5";
        
        let pgn = exporter.export_complete_game(&game_index, moves, Some(&name_db))
            .expect("Should export game successfully");
        
        // Check required headers
        assert!(pgn.contains("[Event \"Test Tournament\"]"));
        assert!(pgn.contains("[Site \"Test City\"]"));
        assert!(pgn.contains("[Date \"2023.12.25\"]"));
        assert!(pgn.contains("[Round \"1\"]"));
        assert!(pgn.contains("[White \"Player White\"]"));
        assert!(pgn.contains("[Black \"Player Black\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));
        
        // Check optional headers
        assert!(pgn.contains("[WhiteElo \"1800\"]"));
        assert!(pgn.contains("[BlackElo \"1750\"]"));
        assert!(pgn.contains("[ECO \"B13\"]"));
        assert!(pgn.contains("[PlyCount \"42\"]"));
        
        // Check moves
        assert!(pgn.contains("1.e4 e5 2.Nf3 Nc6 3.Bb5"));
        assert!(pgn.contains("1-0"));
    }
    
    #[test]
    fn test_enhanced_exporter_without_name_database() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let moves = "1.d4 d5";
        
        let pgn = exporter.export_complete_game(&game_index, moves, None)
            .expect("Should export game without name database");
        
        // Should use fallback names
        assert!(pgn.contains("[Event \"Event_1\"]"));
        assert!(pgn.contains("[Site \"Site_2\"]"));
        assert!(pgn.contains("[White \"Player_10\"]"));
        assert!(pgn.contains("[Black \"Player_11\"]"));
    }
    
    #[test]
    fn test_enhanced_exporter_minimal_options() {
        let options = ExportOptions {
            include_variations: false,
            include_comments: false,
            include_nags: false,
            include_optional_headers: false,
            validate_headers: true,
            ..ExportOptions::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        let game_index = create_test_game_index();
        let moves = "1.e4 e5";
        
        let pgn = exporter.export_complete_game(&game_index, moves, None)
            .expect("Should export with minimal options");
        
        // Should have required headers only
        assert!(pgn.contains("[Event "));
        assert!(pgn.contains("[Result "));
        
        // Should not have optional headers
        assert!(!pgn.contains("[WhiteElo"));
        assert!(!pgn.contains("[ECO"));
    }
    
    #[test]
    fn test_date_formatting() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        
        // Full date
        assert_eq!(exporter.format_date(2023, 12, 25), "2023.12.25");
        
        // Partial dates
        assert_eq!(exporter.format_date(2023, 12, 0), "2023.12.??");
        assert_eq!(exporter.format_date(2023, 0, 0), "2023.??.??");
        assert_eq!(exporter.format_date(0, 0, 0), "????.??.??");
    }
    
    #[test]
    fn test_result_formatting() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        
        assert_eq!(exporter.format_result(1), "1-0");
        assert_eq!(exporter.format_result(2), "0-1");
        assert_eq!(exporter.format_result(3), "1/2-1/2");
        assert_eq!(exporter.format_result(0), "*");
        assert_eq!(exporter.format_result(99), "*");
    }
    
    #[test]
    fn test_eco_formatting() {
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        
        assert_eq!(exporter.format_eco(0), "?");
        assert_eq!(exporter.format_eco(13), "A13");
        assert_eq!(exporter.format_eco(150), "B50");
        assert_eq!(exporter.format_eco(299), "C99");
        assert_eq!(exporter.format_eco(300), "D00");
        assert_eq!(exporter.format_eco(499), "E99");
    }
    
    #[test]
    fn test_line_length_formatting() {
        let options = ExportOptions {
            max_line_length: 20,
            ..ExportOptions::default()
        };
        
        let exporter = EnhancedPgnExporter::new(options);
        let game_index = create_test_game_index();
        let long_moves = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6";
        
        let pgn = exporter.export_complete_game(&game_index, long_moves, None)
            .expect("Should format with line breaks");
        
        // Check that move lines don't exceed the limit
        for line in pgn.lines() {
            if !line.starts_with('[') && !line.trim().is_empty() {
                assert!(
                    line.len() <= 25, // Allow some tolerance for result at end
                    "Line too long ({}): '{}'", 
                    line.len(), 
                    line
                );
            }
        }
    }
}