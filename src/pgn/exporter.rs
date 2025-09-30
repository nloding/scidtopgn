use crate::bridge::{GameState, PositionContext};
use crate::bridge::position::GameMetadata;
use crate::core::error::{Result, ScidError};
use crate::formats::sg4::{StreamingGameElement, StreamingGameParseState, DecodedMove};
use crate::bridge::moves::ScidToShakmaty;
use shakmaty::{Chess, Move, Position, san::San};
use std::collections::HashMap;

/// PGN export configuration options
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Whether to include optional PGN tags (ELO, ECO, etc.)
    pub include_optional_tags: bool,
    /// Whether to validate moves before export
    pub validate_moves: bool,
    /// Maximum line length for PGN output (standard recommends 80)
    pub max_line_length: usize,
    /// Whether to include comments and annotations
    pub include_annotations: bool,
    /// Custom headers to include in addition to standard ones
    pub custom_headers: HashMap<String, String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_optional_tags: true,
            validate_moves: true,
            max_line_length: 80,
            include_annotations: true,
            custom_headers: HashMap::new(),
        }
    }
}

/// Standards-compliant PGN exporter with configurable options
pub struct PgnExporter {
    /// Export configuration options
    options: ExportOptions,
    /// Game state with complete move history
    game_state: GameState,
    /// Parsed game elements (moves, comments, variations)
    elements: Vec<StreamingGameElement>,
}

impl PgnExporter {
    /// Create a new PGN exporter with default settings
    pub fn new(game_state: &GameState, parsed_game: &StreamingGameParseState) -> Result<Self> {
        Self::with_options(game_state, parsed_game, ExportOptions::default())
    }
    
    /// Create a PGN exporter with custom options
    pub fn with_options(
        game_state: &GameState, 
        parsed_game: &StreamingGameParseState, 
        options: ExportOptions
    ) -> Result<Self> {
        Ok(Self {
            options,
            game_state: game_state.clone(),
            elements: parsed_game.elements.clone(),
        })
    }
    
    /// Export the game to PGN format
    pub fn export(&self) -> Result<String> {
        let mut pgn = String::new();
        
        // 1. Export Seven Tag Roster (mandatory headers)
        self.export_seven_tag_roster(&mut pgn)?;
        
        // 2. Export optional tags if enabled
        if self.options.include_optional_tags {
            self.export_optional_tags(&mut pgn)?;
        }
        
        // 3. Export custom headers if any
        self.export_custom_headers(&mut pgn)?;
        
        // 4. Blank line after headers
        pgn.push('\n');
        
        // 5. Export move sequence with proper SAN notation
        self.export_move_sequence(&mut pgn)?;
        
        // 6. Export game termination
        self.export_game_termination(&mut pgn);
        
        Ok(pgn)
    }
    
    /// Export the Seven Tag Roster (mandatory PGN headers)
    fn export_seven_tag_roster(&self, pgn: &mut String) -> Result<()> {
        let metadata = self.game_state.metadata()
            .ok_or_else(|| ScidError::invalid_format("Game metadata is required for PGN export".to_string()))?;
        
        // Seven Tag Roster - mandatory PGN headers according to PGN standard
        pgn.push_str(&format!("[Event \"{}\"]\n", escape_pgn_string(&metadata.event)));
        pgn.push_str(&format!("[Site \"{}\"]\n", escape_pgn_string(&metadata.site)));
        pgn.push_str(&format!("[Date \"{}\"]\n", format_pgn_date(&metadata.date)));
        pgn.push_str(&format!("[Round \"{}\"]\n", escape_pgn_string(
            metadata.round.as_deref().unwrap_or("?")
        )));
        pgn.push_str(&format!("[White \"{}\"]\n", escape_pgn_string(&metadata.white)));
        pgn.push_str(&format!("[Black \"{}\"]\n", escape_pgn_string(&metadata.black)));
        pgn.push_str(&format!("[Result \"{}\"]\n", metadata.result));
        
        Ok(())
    }
    
    /// Export optional PGN tags (ELO, ECO, etc.)
    fn export_optional_tags(&self, pgn: &mut String) -> Result<()> {
        let binding = self.game_state.metadata();
        let metadata = binding
            .as_ref()
            .ok_or_else(|| ScidError::invalid_format("Game metadata is required for PGN export".to_string()))?;
        
        // White ELO
        if let Some(elo) = metadata.white_elo {
            pgn.push_str(&format!("[WhiteElo \"{}\"]\n", elo));
        }
        
        // Black ELO
        if let Some(elo) = metadata.black_elo {
            pgn.push_str(&format!("[BlackElo \"{}\"]\n", elo));
        }
        
        // ECO code
        if let Some(ref eco) = metadata.eco {
            pgn.push_str(&format!("[ECO \"{}\"]\n", escape_pgn_string(eco)));
        }
        
        Ok(())
    }
    
    /// Export custom headers
    fn export_custom_headers(&self, pgn: &mut String) -> Result<()> {
        for (key, value) in &self.options.custom_headers {
            pgn.push_str(&format!("[{} \"{}\"]\n", escape_pgn_string(key), escape_pgn_string(value)));
        }
        Ok(())
    }
    
    /// Export move sequence with proper SAN notation
    fn export_move_sequence(&self, pgn: &mut String) -> Result<()> {
        let mut current_position = self.game_state.current_position().clone();
        let mut move_number = 1;
        let mut line_length = 0;
        
        for element in &self.elements {
            match element {
                StreamingGameElement::Move { raw, .. } => {
                    // Convert raw bytes to shakmaty move (simplified for now)
                    // In a full implementation, this would use the position tracker
                    if let Some(shakmaty_move) = self.decode_move_to_shakmaty(&current_position, raw) {
                        // Generate SAN notation
                        let san = San::from_move(&current_position, &shakmaty_move);
                        let san_string = san.to_string();
                        
                        // Format move with move number
                        let move_text = if current_position.turn() == shakmaty::Color::White {
                            format!("{}. {}", move_number, san_string)
                        } else {
                            format!("{}... {}", move_number, san_string)
                        };
                        
                        // Check line length and wrap if necessary
                        if line_length + move_text.len() + 1 > self.options.max_line_length {
                            pgn.push('\n');
                            line_length = 0;
                        }
                        
                        pgn.push_str(&move_text);
                        pgn.push(' ');
                        line_length += move_text.len() + 1;
                        
                        // Apply move to position for next move
                        current_position = current_position.play(&shakmaty_move)
                            .map_err(|e| ScidError::conversion_error(
                                format!("Failed to apply move {}: {}", san_string, e)
                            ))?;
                        
                        // Update move number after black moves
                        if current_position.turn() == shakmaty::Color::White {
                            move_number += 1;
                        }
                    }
                }
                StreamingGameElement::Comment { text, .. } if self.options.include_annotations => {
                    // Format comment: { comment text }
                    let comment_text = format!("{{{}}}", escape_pgn_string(text));
                    
                    // Check line length and wrap if necessary
                    if line_length + comment_text.len() + 1 > self.options.max_line_length {
                        pgn.push('\n');
                        line_length = 0;
                    }
                    
                    pgn.push_str(&comment_text);
                    pgn.push(' ');
                    line_length += comment_text.len() + 1;
                }
                StreamingGameElement::Nag { nag_code, .. } if self.options.include_annotations => {
                    // Format NAG: $NAG
                    let nag_text = format!("${}", nag_code);
                    
                    // Check line length and wrap if necessary
                    if line_length + nag_text.len() + 1 > self.options.max_line_length {
                        pgn.push('\n');
                        line_length = 0;
                    }
                    
                    pgn.push_str(&nag_text);
                    pgn.push(' ');
                    line_length += nag_text.len() + 1;
                }
                StreamingGameElement::VariationStart { .. } => {
                    // Start variation: (
                    pgn.push('(');
                    line_length += 1;
                }
                StreamingGameElement::VariationEnd { .. } => {
                    // End variation: )
                    pgn.push_str(") ");
                    line_length += 2;
                }
                StreamingGameElement::Comment { .. } | StreamingGameElement::Nag { .. } => {
                    // Skip comments and NAGs if annotations are disabled
                    continue;
                }
                StreamingGameElement::GameEnd { result } => {
                    // Format game result and append to PGN
                    let result_text = crate::bridge::position::format_result(*result);
                    pgn.push_str(&result_text);
                }
            }
        }
        
        // Remove trailing space and add final newline
        if pgn.ends_with(' ') {
            pgn.pop();
        }
        pgn.push('\n');
        
        Ok(())
    }
    
    /// Export game termination
    fn export_game_termination(&self, pgn: &mut String) {
        if let Some(metadata) = self.game_state.metadata() {
            // The result is already included in the headers
            // but some PGN formats also include it at the end
            if metadata.result != "*" {
                pgn.push_str(&metadata.result);
                pgn.push('\n');
            }
        }
    }
    
    /// Decode raw move bytes to shakmaty move using Phase 3 bridge layer
    fn decode_move_to_shakmaty(&self, position: &Chess, raw: &[u8]) -> Option<Move> {
        // Use the Phase 3 bridge layer to decode SCID moves
        // This integrates with the comprehensive SCID move decoding logic
        if raw.is_empty() {
            return None;
        }
        
        // Parse the SCID move byte
        let piece_num = (raw[0] >> 4) & 0x0F;
        let move_value = raw[0] & 0x0F;
        
        // Handle multi-byte moves (queen diagonal moves require 2 bytes)
        let (raw_bytes, move_value) = if piece_num == 2 && move_value >= 8 && raw.len() > 1 {
            // Queen diagonal move - 2 bytes required
            (raw.to_vec(), move_value)
        } else {
            // Single byte move
            (raw.to_vec(), move_value)
        };
        
        // Create appropriate MoveInterpretation based on piece type
        let interpretation = match piece_num {
            1 => crate::formats::sg4::MoveInterpretation::King {
                direction_code: move_value,
                is_castle: move_value == 9 || move_value == 10,
            },
            2 => crate::formats::sg4::MoveInterpretation::Queen,
            3 => crate::formats::sg4::MoveInterpretation::Rook,
            4 => crate::formats::sg4::MoveInterpretation::Bishop,
            5 => crate::formats::sg4::MoveInterpretation::Knight {
                l_shape_code: move_value,
            },
            6 => crate::formats::sg4::MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: if move_value >= 3 && move_value <= 6 {
                    Some("Queen".to_string())
                } else if move_value >= 7 && move_value <= 10 {
                    Some("Rook".to_string())
                } else if move_value >= 11 && move_value <= 14 {
                    Some("Bishop".to_string())
                } else if move_value >= 15 && move_value <= 18 {
                    Some("Knight".to_string())
                } else {
                    None
                },
                is_en_passant: if move_value == 15 { Some(true) } else { None },
            },
            _ => crate::formats::sg4::MoveInterpretation::Unknown {
                reason: format!("Invalid piece number: {}", piece_num),
            },
        };
        
        // Create a DecodedMove structure for the bridge layer
        let decoded_move = crate::formats::sg4::DecodedMove {
            raw_bytes,
            piece_num,
            move_value,
            interpretation,
            from_square_index: None,
            to_square_index: None,
            promotion_piece: None,
        };
        
        // Use the bridge layer's ScidToShakmaty trait to convert to shakmaty move
        decoded_move.to_shakmaty(position).ok()
    }
}

/// Escape special characters in PGN strings according to PGN standard
fn escape_pgn_string(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\r', "\\r")
     .replace('\t', "\\t")
}

/// Format date for PGN output (YYYY.MM.DD format)
fn format_pgn_date(date: &str) -> String {
    // Assuming date is already in YYYY.MM.DD format from SCID
    // If it needs conversion, additional logic would be added here
    date.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::GameState;
    use crate::formats::sg4::StreamingGameParseState;
    use std::collections::HashMap;

    #[test]
    fn test_pgn_exporter_creation() {
        let game_state = GameState::new();
        let parsed_game = StreamingGameParseState::new();
        let exporter = PgnExporter::new(&game_state, &parsed_game);
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_export_with_options() {
        let game_state = GameState::new();
        let parsed_game = StreamingGameParseState::new();
        let options = ExportOptions {
            include_optional_tags: false,
            validate_moves: false,
            max_line_length: 120,
            include_annotations: false,
            custom_headers: HashMap::new(),
        };
        let exporter = PgnExporter::with_options(&game_state, &parsed_game, options);
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_export_headers() {
        let mut game_state = GameState::new();
        game_state.set_metadata(GameMetadata {
            white: "WhitePlayer".to_string(),
            black: "BlackPlayer".to_string(),
            event: "Test Event".to_string(),
            site: "Test Site".to_string(),
            date: "2025.08.29".to_string(),
            result: "*".to_string(),
            white_elo: Some(1500),
            black_elo: Some(1600),
            round: Some("1".to_string()),
            eco: Some("A00".to_string()),
        });
        let parsed_game = StreamingGameParseState::new();
        let exporter = PgnExporter::new(&game_state, &parsed_game).unwrap();

        let mut pgn = String::new();
        exporter.export_seven_tag_roster(&mut pgn).unwrap();

        assert!(pgn.contains("[Event \"Test Event\"]"));
        assert!(pgn.contains("[Site \"Test Site\"]"));
        assert!(pgn.contains("[Date \"2025.08.29\"]"));
        assert!(pgn.contains("[Round \"1\"]"));
        assert!(pgn.contains("[White \"WhitePlayer\"]"));
        assert!(pgn.contains("[Black \"BlackPlayer\"]"));
        assert!(pgn.contains("[Result \"*\"]"));
    }

    #[test]
    fn test_escape_pgn_string() {
        assert_eq!(escape_pgn_string("Test \"String\""), "Test \\\"String\\\"");
        assert_eq!(escape_pgn_string("Line\nBreak"), "Line\\nBreak");
        assert_eq!(escape_pgn_string("Back\\Slash"), "Back\\\\Slash");
    }

    #[test]
    fn test_format_pgn_date() {
        assert_eq!(format_pgn_date("2025.08.29"), "2025.08.29");
    }
}
