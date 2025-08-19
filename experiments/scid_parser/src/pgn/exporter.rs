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