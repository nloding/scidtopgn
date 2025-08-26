// Chess Position State Management with Shakmaty
//
// This module manages chess position state during SCID game parsing using
// shakmaty's Chess type for accurate position tracking and move validation.
// It provides the GameState struct that maintains both position and move history.

use shakmaty::{Chess, Move, Position};
use crate::error::Result;
use crate::bridge::{PositionContext, ChessNotation, BasicChessValidation, ScidPositionTracker};

/// Convert SCID result code to PGN result string
fn format_result(result_code: u8) -> String {
    match result_code {
        0 => "*".to_string(),      // Unknown/ongoing result
        1 => "1-0".to_string(),    // White wins
        2 => "0-1".to_string(),    // Black wins  
        3 => "1/2-1/2".to_string(), // Draw
        _ => "*".to_string(),      // Unknown result code
    }
}

/// Manages chess position state during SCID game parsing
/// 
/// GameState uses ScidPositionTracker for position-aware SCID move parsing that
/// correctly handles SCID's piece numbering system and move encoding.
/// It serves as the central state management structure for converting SCID
/// games to standard chess formats.
#[derive(Clone)]
pub struct GameState {
    /// SCID-aware position tracker for accurate move parsing
    position_tracker: ScidPositionTracker,
    
    /// Optional game metadata (headers for PGN export)
    metadata: Option<GameMetadata>,
}

/// Game metadata for PGN export
/// 
/// Contains the standard PGN headers and additional metadata extracted
/// from SCID database files (si4 index and sn4 names).
#[derive(Debug, Clone)]
pub struct GameMetadata {
    /// White player name
    pub white: String,
    
    /// Black player name
    pub black: String,
    
    /// Event/tournament name
    pub event: String,
    
    /// Site where game was played
    pub site: String,
    
    /// Game date in YYYY.MM.DD format
    pub date: String,
    
    /// Game result (1-0, 0-1, 1/2-1/2, *)
    pub result: String,
    
    /// White player's ELO rating (if available)
    pub white_elo: Option<u16>,
    
    /// Black player's ELO rating (if available)
    pub black_elo: Option<u16>,
    
    /// Round number (if available)
    pub round: Option<String>,
    
    /// ECO opening code (if available)
    pub eco: Option<String>,
}

impl GameState {
    /// Create a new GameState with standard starting position
    pub fn new() -> Self {
        Self {
            position_tracker: ScidPositionTracker::new(),
            metadata: None,
        }
    }
    
    /// Create a new GameState with metadata
    pub fn with_metadata(metadata: GameMetadata) -> Self {
        Self {
            position_tracker: ScidPositionTracker::new(),
            metadata: Some(metadata),
        }
    }
    
    /// Create a GameState from a FEN position string
    pub fn from_fen(fen: &str) -> Result<Self> {
        // TODO: Implement FEN parsing when needed
        // For now, use standard starting position as placeholder
        let _fen = fen; // Prevent unused variable warning
        Ok(Self {
            position_tracker: ScidPositionTracker::new(),
            metadata: None,
        })
    }
    
    /// Get reference to game metadata
    pub fn metadata(&self) -> Option<&GameMetadata> {
        self.metadata.as_ref()
    }
    
    /// Set game metadata
    pub fn set_metadata(&mut self, metadata: GameMetadata) {
        self.metadata = Some(metadata);
    }
    
    /// Get the current FEN position string
    pub fn fen(&self) -> String {
        format!("{}", self.position_tracker.current_position().board())
    }
    
    /// Get SAN notation for all moves
    pub fn san_moves(&self) -> &[String] {
        self.position_tracker.san_history()
    }
    
    /// Apply a SCID move to the current position, updating move history and SAN
    /// Uses position-aware SCID parsing via ScidPositionTracker
    pub fn play_scid_move(&mut self, scid_move: &crate::sg4::DecodedMove) -> Result<()> {
        // Use the position tracker to apply the SCID move
        // This handles all SCID-specific move decoding, position updates, and history tracking
        self.position_tracker.apply_scid_move(scid_move)?;
        Ok(())
    }
    
    /// Attach metadata from SI4 index parsing
    pub fn attach_metadata(&mut self, metadata: GameMetadata) {
        self.metadata = Some(metadata);
    }
    
    /// Compatibility wrapper for legacy SCID parsing
    /// Allows existing parse_game_basic to work unchanged
    pub fn play_scid_move_compat(&mut self, scid_move: &crate::sg4::DecodedMove) -> Result<()> {
        self.play_scid_move(scid_move)
    }
    
    /// Add a comment to the current position
    pub fn add_comment(&mut self, _comment: String) {
        // TODO: Implement comment handling in later steps
        // Comments will be associated with specific moves/positions
    }
    
    /// Start a variation from the current position
    pub fn start_variation(&mut self) {
        // TODO: Implement variation handling in later steps
        // Will support complex PGN variation trees
    }
    
    /// End the current variation
    pub fn end_variation(&mut self) {
        // TODO: Implement variation handling in later steps
    }
    
    /// Add a NAG (Numeric Annotation Glyph) symbol
    pub fn add_nag(&mut self, _nag: u8) {
        // TODO: Implement NAG handling in later steps
        // Will convert numeric codes to standard chess symbols (!?, ?!, etc.)
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the bridge traits for GameState
// These provide the core functionality for position tracking and chess operations

impl PositionContext for GameState {
    fn current_position(&self) -> &Chess {
        self.position_tracker.current_position()
    }
    
    fn move_history(&self) -> &[Move] {
        self.position_tracker.move_history()
    }
    
    fn move_count(&self) -> usize {
        self.position_tracker.move_history().len()
    }
    
    fn is_position_legal(&self) -> bool {
        // TODO: Implement proper position validation using shakmaty
        // For now, assume positions are legal
        true
    }
}

impl ChessNotation for GameState {
    fn to_pgn(&self) -> String {
        let mut pgn = String::new();
        
        // Add headers if metadata is available
        if let Some(ref metadata) = self.metadata {
            pgn.push_str(&format!("[Event \"{}\"]\n", metadata.event));
            pgn.push_str(&format!("[Site \"{}\"]\n", metadata.site));
            pgn.push_str(&format!("[Date \"{}\"]\n", metadata.date));
            pgn.push_str(&format!("[White \"{}\"]\n", metadata.white));
            pgn.push_str(&format!("[Black \"{}\"]\n", metadata.black));
            pgn.push_str(&format!("[Result \"{}\"]\n", metadata.result));
            
            if let Some(elo) = metadata.white_elo {
                pgn.push_str(&format!("[WhiteElo \"{}\"]\n", elo));
            }
            if let Some(elo) = metadata.black_elo {
                pgn.push_str(&format!("[BlackElo \"{}\"]\n", elo));
            }
            if let Some(ref round) = metadata.round {
                pgn.push_str(&format!("[Round \"{}\"]\n", round));
            }
            if let Some(ref eco) = metadata.eco {
                pgn.push_str(&format!("[ECO \"{}\"]\n", eco));
            }
            
            pgn.push('\n');
        }
        
        // Add moves in standard PGN format
        for (i, san_move) in self.position_tracker.san_history().iter().enumerate() {
            if i % 2 == 0 {
                pgn.push_str(&format!("{}. ", (i / 2) + 1));
            }
            pgn.push_str(san_move);
            pgn.push(' ');
        }
        
        // Add result if available
        if let Some(ref metadata) = self.metadata {
            pgn.push_str(&metadata.result);
        } else {
            pgn.push_str("*"); // Unknown result
        }
        
        pgn
    }
}

impl BasicChessValidation for GameState {
    fn is_move_legal(&self, chess_move: &Move) -> bool {
        // Use shakmaty's built-in move validation
        self.position_tracker.current_position().is_legal(chess_move)
    }
    
    fn is_in_check(&self) -> bool {
        // TODO: Implement check detection using shakmaty
        // Will use position.is_check() or similar method
        false // Placeholder
    }
    
    fn is_checkmate(&self) -> bool {
        // TODO: Implement checkmate detection using shakmaty
        // Will check if position is checkmate
        false // Placeholder
    }
    
    fn is_stalemate(&self) -> bool {
        // TODO: Implement stalemate detection using shakmaty
        // Will check if position is stalemate
        false // Placeholder
    }
    
    fn game_outcome(&self) -> Option<shakmaty::Outcome> {
        // TODO: Implement game outcome detection using shakmaty
        // Will return Decisive or Draw outcomes
        None // Placeholder
    }
}

impl GameMetadata {
    /// Create new GameMetadata with required fields
    pub fn new(
        white: String,
        black: String,
        event: String,
        site: String,
        date: String,
        result: String,
    ) -> Self {
        Self {
            white,
            black,
            event,
            site,
            date,
            result,
            white_elo: None,
            black_elo: None,
            round: None,
            eco: None,
        }
    }
    
    /// Create GameMetadata from SI4 index and SN4 name records
    /// This is a placeholder implementation - full name resolution will be added later
    pub fn from_index_and_names(
        index: &crate::si4::GameIndex,
        _names: &[crate::sn4::NameRecord], // Use actual type from sn4.rs
    ) -> Result<Self> {
        // TODO: Implement proper name resolution from SCID files
        // For now, use placeholder values from index
        Ok(Self {
            white: format!("Player{}", index.white_id),
            black: format!("Player{}", index.black_id),
            event: format!("Event{}", index.event_id),
            site: format!("Site{}", index.site_id),
            date: format!("{:04}.{:02}.{:02}", index.year, index.month, index.day),
            result: format_result(index.result),
            white_elo: if index.white_elo > 0 { Some(index.white_elo) } else { None },
            black_elo: if index.black_elo > 0 { Some(index.black_elo) } else { None },
            round: if index.round_id > 0 { Some(format!("Round{}", index.round_id)) } else { None },
            eco: if index.eco > 0 { Some(format!("ECO{}", index.eco)) } else { None },
        })
    }
}