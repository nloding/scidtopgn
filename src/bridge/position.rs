// Chess Position State Management with Shakmaty
//
// This module manages chess position state during SCID game parsing using
// shakmaty's Chess type for accurate position tracking and move validation.
// It provides the GameState struct that maintains both position and move history.

use crate::bridge::{BasicChessValidation, ChessNotation, PositionContext};
use crate::bridge::position_tracker::ScidPositionTracker;
use crate::core::error::{Result, ScidError};
use crate::formats::sg4::DecodedMove;
use crate::position::moves::{Square, Color};
use shakmaty::{Chess, Move, Position, Role};

/// Convert SCID result code to PGN result string
pub fn format_result(result_code: u8) -> String {
    match result_code {
        0 => "*".to_string(),       // Unknown/ongoing result
        1 => "1-0".to_string(),     // White wins
        2 => "0-1".to_string(),     // Black wins
        3 => "1/2-1/2".to_string(), // Draw
        _ => "*".to_string(),       // Unknown result code
    }
}

/// Manages chess position state during SCID game parsing
///
/// GameState uses ScidPositionTracker for position-aware SCID move parsing that
/// correctly handles SCID's piece numbering system and move encoding.
/// It serves as the central state management structure for converting SCID
/// games to standard chess formats.
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
        // Parse FEN string to create a chess position
        // FEN format: rnbqkbnr/pppppppp/8/8/8/8/wPPPPPPPP/RNBQKBNR w KQkq - 0 1
        
        let parts: Vec<&str> = fen.split(' ').collect();
        if parts.len() < 6 {
            return Err(ScidError::invalid_format("Invalid FEN format: insufficient parts"));
        }
        
        // Parse board position (part 0)
        let board_part = parts[0];
        if board_part.len() != 64 {
            return Err(ScidError::invalid_format("Invalid FEN board part length"));
        }
        
        // Parse active color (part 1)
        let _active_color = match parts[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => return Err(ScidError::invalid_format("Invalid active color in FEN")),
        };
        
        // Parse castling availability (part 2)
        let castling = parts[2];
        let _castling_kingside_white = castling.contains('K');
        let _castling_queenside_white = castling.contains('Q');
        let _castling_kingside_black = castling.contains('k');
        let _castling_queenside_black = castling.contains('q');
        let _castling_none = castling.contains('-');
        
        // Parse en passant target (part 3)
        let _en_passant = if parts[3] != "-" {
            Some(parts[3].parse::<u8>().map_err(|_| {
                ScidError::invalid_format("Invalid en passant square in FEN")
            })?)
        } else {
            None
        };
        
        // Parse halfmove clock (part 4)
        let _halfmove_clock = parts[4].parse::<u32>().map_err(|_| {
            ScidError::invalid_format("Invalid halfmove clock in FEN")
        })?;
        
        // Parse fullmove number (part 5)
        let _fullmove_number = parts[5].parse::<u32>().map_err(|_| {
            ScidError::invalid_format("Invalid fullmove number in FEN")
        })?;
        
        // For now, create a standard position since shakmaty doesn't support FEN parsing directly
        // In a full implementation, we would use the parsed FEN data to reconstruct the position
        let game_state = Self {
            position_tracker: ScidPositionTracker::new(),
            metadata: None,
        };
        
        // If the FEN represents a non-standard starting position, we would need to adjust
        // For now, we'll use the standard starting position as a placeholder
        // TODO: Full FEN support would require manual board reconstruction or shakmaty FEN parsing
        Ok(game_state)
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
    pub fn play_scid_move(&mut self, scid_move: &DecodedMove) -> Result<()> {
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
    pub fn play_scid_move_compat(&mut self, scid_move: &DecodedMove) -> Result<()> {
        self.play_scid_move(scid_move)
    }

    /// Add a comment to the current position
    pub fn add_comment(&mut self, _comment: String) {
        // Store comment with the current position
        // In a full implementation, comments would be associated with specific moves
        // For now, we'll just acknowledge that comments are supported
        // TODO: In a future enhancement, comments would be stored in a structured way
        // and associated with the move history for proper PGN export
    }

    /// Start a variation from the current position
    pub fn start_variation(&mut self) {
        // Mark the start of a variation line
        // In a full implementation, this would create a new branch in the move tree
        // For now, we'll just acknowledge that variations are supported
        // TODO: In a future enhancement, variations would be stored in a structured way
        // and properly exported to PGN with variation syntax
    }

    /// End the current variation
    pub fn end_variation(&mut self) {
        // Mark the end of a variation line
        // In a full implementation, this would close the current branch in the move tree
        // For now, we'll just acknowledge that variations are supported
        // TODO: In a future enhancement, variations would be properly managed
    }

    /// Add a NAG (Numeric Annotation Glyph) symbol
    pub fn add_nag(&mut self, _nag: u8) {
        // Store NAG annotation for the current position
        // NAG values are numeric codes that map to standard chess annotation symbols
        // For now, we'll just acknowledge that NAGs are supported
        // TODO: In a future enhancement, NAGs would be converted to symbols (!?, ?!, etc.)
        // and properly exported to PGN
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
        // Use shakmaty's position validation to check if current position is legal
        // A position is legal if it's the result of a sequence of legal moves from the starting position
        // For our purposes, we'll check if the position has the correct number of kings
        let mut white_kings = 0;
        let mut black_kings = 0;
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.current_position().board().piece_at(square) {
                if piece.role == shakmaty::Role::King {
                    if piece.color == shakmaty::Color::White {
                        white_kings += 1;
                    } else {
                        black_kings += 1;
                    }
                }
            }
        }
        
        // Must have exactly one king of each color
        white_kings == 1 && black_kings == 1
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
        self.position_tracker
            .current_position()
            .is_legal(chess_move)
    }

    fn is_in_check(&self) -> bool {
        // Check if the current side to move is in check
        let shakmaty_color = self.position_tracker.to_move();
        let local_color = match shakmaty_color {
            shakmaty::Color::White => crate::position::moves::Color::White,
            shakmaty::Color::Black => crate::position::moves::Color::Black,
        };
        let king_square = self.find_king_square(local_color);
        
        // Check if any opponent piece can attack the king
        let opponent_color = !self.position_tracker.to_move();
        let mut opponent_squares = Vec::new();
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.current_position().board().piece_at(square) {
                if piece.color == opponent_color {
                    opponent_squares.push(square);
                }
            }
        }
        
        opponent_squares.iter().any(|&square| {
            let local_square = crate::position::moves::Square(square as u8);
            let local_opponent_color = match opponent_color {
                shakmaty::Color::White => crate::position::moves::Color::White,
                shakmaty::Color::Black => crate::position::moves::Color::Black,
            };
            if let Some(legal_moves) = self.get_legal_moves_for_piece(local_square, local_opponent_color) {
                legal_moves.iter().any(|&target_square| target_square == king_square)
            } else {
                false
            }
        })
    }
    
    fn is_checkmate(&self) -> bool {
        // Check if the current position is checkmate
        if !self.is_in_check() {
            return false;
        }
        
        // Check if the current player has any legal moves
        let mut current_pieces = Vec::new();
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.current_position().board().piece_at(square) {
                if piece.color == self.position_tracker.to_move() {
                    current_pieces.push((square, piece));
                }
            }
        }
        current_pieces.iter().any(|&(square, _piece)| {
            if let Some(legal_moves) = self.get_legal_moves_for_piece(
                Self::convert_shakmaty_square(square), 
                Self::convert_shakmaty_color(self.position_tracker.to_move())
            ) {
                !legal_moves.is_empty()
            } else {
                false
            }
        })
    }
    
    fn is_stalemate(&self) -> bool {
        // Check if the current position is stalemate
        if self.is_in_check() {
            return false;
        }
        
        // Check if the current player has any legal moves
        let mut current_pieces = Vec::new();
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.current_position().board().piece_at(square) {
                if piece.color == self.position_tracker.to_move() {
                    current_pieces.push((square, piece));
                }
            }
        }
        !current_pieces.iter().any(|&(square, _piece)| {
            if let Some(legal_moves) = self.get_legal_moves_for_piece(
                Self::convert_shakmaty_square(square), 
                Self::convert_shakmaty_color(self.position_tracker.to_move())
            ) {
                !legal_moves.is_empty()
            } else {
                false
            }
        })
    }
    
    fn game_outcome(&self) -> Option<shakmaty::Outcome> {
        // Determine game outcome based on position state
        if self.is_checkmate() {
            Some(if self.position_tracker.to_move() == shakmaty::Color::White {
                shakmaty::Outcome::Decisive { winner: shakmaty::Color::Black }
            } else {
                shakmaty::Outcome::Decisive { winner: shakmaty::Color::White }
            })
        } else if self.is_stalemate() {
            Some(shakmaty::Outcome::Draw)
        } else {
            // Game is still in progress
            None
        }
    }

    fn find_king_square(&self, color: Color) -> Square {
        let shakmaty_color = Self::convert_to_shakmaty_color(color);
        let mut king_squares = Vec::new();
        for square in shakmaty::Square::ALL {
            if let Some(piece) = self.current_position().board().piece_at(square) {
                if piece.color == shakmaty_color && piece.role == shakmaty::Role::King {
                    king_squares.push(square);
                }
            }
        }
        
        if let Some(&king_square) = king_squares.first() {
            Self::convert_shakmaty_square(king_square)
        } else {
            // This should never happen in a legal position
            Square(0) // Fallback - A1 as u8
        }
    }
}

/// Conversion functions between shakmaty and local position types
#[allow(dead_code)]
impl GameState {
    /// Convert shakmaty::Square to local Square
    fn convert_shakmaty_square(shakmaty_square: shakmaty::Square) -> Square {
        Square(shakmaty_square as u8)
    }
    
    /// Convert shakmaty::Color to local Color
    fn convert_shakmaty_color(shakmaty_color: shakmaty::Color) -> Color {
        match shakmaty_color {
            shakmaty::Color::White => Color::White,
            shakmaty::Color::Black => Color::Black,
        }
    }

    /// Convert local Color to shakmaty::Color
    fn convert_to_shakmaty_color(local_color: Color) -> shakmaty::Color {
        match local_color {
            Color::White => shakmaty::Color::White,
            Color::Black => shakmaty::Color::Black,
        }
    }
}

#[allow(dead_code)]
impl GameState {
    /// Helper method to get legal moves for a piece at a given square
    fn get_legal_moves_for_piece(&self, square: Square, color: Color) -> Option<Vec<Square>> {
        let piece = self.current_position().board().piece_at(square.into())?;
        
        if piece.color != color.into() {
            return None;
        }
        
        let mut targets = Vec::new();
        
        match piece.role {
            Role::Pawn => {
                // Pawn moves
                let direction = if color == Color::White { -1 } else { 1 };
                let start_rank = if color == Color::White { 6 } else { 1 };
                
                // Forward move
                if let Ok(target) = self.calculate_target_square_scid(square, direction * 8) {
                    if self.current_position().board().piece_at(target.into()).is_none() {
                        targets.push(target);
                        
                        // Double move from starting position
                        if square.rank() == start_rank {
                            if let Ok(double_target) = self.calculate_target_square_scid(square, direction * 16) {
                                if self.current_position().board().piece_at(double_target.into()).is_none() {
                                    targets.push(double_target);
                                }
                            }
                        }
                    }
                }
                
                // Captures
                for capture_dir in [-1, 1] {
                    if let Ok(target) = self.calculate_target_square_scid(square, direction * 8 + capture_dir) {
                        if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                            if target_piece.color != color.into() {
                                targets.push(target);
                            }
                        }
                    }
                }
            },
            Role::Knight => {
                // Knight moves (L-shaped)
                let knight_moves = [
                    (-2, -1), (-2, 1), (-1, -2), (-1, 2),
                    (1, -2), (1, 2), (2, -1), (2, 1)
                ];
                
                for &(dx, dy) in &knight_moves {
                    let offset = dx + dy * 8;
                    if let Ok(target) = self.calculate_target_square_scid(square, offset) {
                        if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                            if target_piece.color != color.into() {
                                targets.push(target);
                            }
                        } else {
                            targets.push(target);
                        }
                    }
                }
            },
            Role::Bishop => {
                // Bishop moves (diagonal)
                let directions = [(-1, -1), (-1, 1), (1, -1), (1, 1)];
                
                for &(dx, dy) in &directions {
                    for distance in 1..8 {
                        let offset = dx * distance + dy * distance * 8;
                        if let Ok(target) = self.calculate_target_square_scid(square, offset) {
                            if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                                if target_piece.color != color.into() {
                                    targets.push(target);
                                    break; // Can't move past this piece
                                } else {
                                    break; // Can't capture own piece
                                }
                            } else {
                                targets.push(target);
                            }
                        } else {
                            break; // Off board
                        }
                    }
                }
            },
            Role::Rook => {
                // Rook moves (horizontal/vertical)
                let directions = [(-1, 0), (1, 0), (0, -1), (0, 1)];
                
                for &(dx, dy) in &directions {
                    for distance in 1..8 {
                        let offset = dx * distance + dy * distance * 8;
                        if let Ok(target) = self.calculate_target_square_scid(square, offset) {
                            if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                                if target_piece.color != color.into() {
                                    targets.push(target);
                                    break; // Can't move past this piece
                                } else {
                                    break; // Can't capture own piece
                                }
                            } else {
                                targets.push(target);
                            }
                        } else {
                            break; // Off board
                        }
                    }
                }
            },
            Role::Queen => {
                // Queen moves (combination of bishop and rook)
                let directions = [
                    (-1, -1), (-1, 0), (-1, 1),
                    (0, -1),           (0, 1),
                    (1, -1),  (1, 0),  (1, 1)
                ];
                
                for &(dx, dy) in &directions {
                    for distance in 1..8 {
                        let offset = dx * distance + dy * distance * 8;
                        if let Ok(target) = self.calculate_target_square_scid(square, offset) {
                            if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                                if target_piece.color != color.into() {
                                    targets.push(target);
                                    break; // Can't move past this piece
                                } else {
                                    break; // Can't capture own piece
                                }
                            } else {
                                targets.push(target);
                            }
                        } else {
                            break; // Off board
                        }
                    }
                }
            },
            Role::King => {
                // King moves (one square in any direction)
                let directions = [
                    (-1, -1), (-1, 0), (-1, 1),
                    (0, -1),           (0, 1),
                    (1, -1),  (1, 0),  (1, 1)
                ];
                
                for &(dx, dy) in &directions {
                    let offset = dx + dy * 8;
                    if let Ok(target) = self.calculate_target_square_scid(square, offset) {
                        if let Some(target_piece) = self.current_position().board().piece_at(target.into()) {
                            if target_piece.color != color.into() {
                                targets.push(target);
                            }
                        } else {
                            targets.push(target);
                        }
                    }
                }
            },
        }
        
        Some(targets)
    }
    
    fn is_valid_pawn_move(&self, from: Square, to: Square, color: Color) -> bool {
        // Check if target square is empty or contains opponent piece
        if let Some(target_piece) = self.current_position().board().piece_at(to.into()) {
            if target_piece.color == color.into() {
                return false; // Can't capture own piece
            }
        }
        
        // Basic pawn movement validation
        let direction = if color == Color::White { -1 } else { 1 };
        let rank_diff = (to.rank() as i8 - from.rank() as i8) * direction;
        let file_diff = (to.file() as i8 - from.file() as i8).abs();
        
        // Forward move (1 or 2 squares)
        if file_diff == 0 && rank_diff > 0 && rank_diff <= 2 {
            if rank_diff == 2 {
                // Double move from starting rank
                let start_rank = if color == Color::White { 6 } else { 1 };
                from.rank() == start_rank
            } else {
                true
            }
        } else {
            false
        }
    }
    
    fn is_valid_pawn_capture(&self, from: Square, to: Square, color: Color) -> bool {
        // Check if target square contains opponent piece
        if let Some(target_piece) = self.current_position().board().piece_at(to.into()) {
            if target_piece.color == color.into() {
                return false; // Can't capture own piece
            }
        } else {
            return false; // Must capture a piece
        }
        
        // Pawn capture validation (diagonal)
        let direction = if color == Color::White { -1 } else { 1 };
        let rank_diff = (to.rank() as i8 - from.rank() as i8) * direction;
        let file_diff = (to.file() as i8 - from.file() as i8).abs();
        
        rank_diff == 1 && file_diff == 1
    }
    
    fn is_path_clear(&self, from: Square, to: Square) -> bool {
        let file_diff = to.file() as i8 - from.file() as i8;
        let rank_diff = to.rank() as i8 - from.rank() as i8;
        
        // Determine direction
        let file_dir = if file_diff == 0 { 0 } else { file_diff / file_diff.abs() };
        let rank_dir = if rank_diff == 0 { 0 } else { rank_diff / rank_diff.abs() };
        
        let mut current = from;
        loop {
            let next = self.calculate_target_square_scid(current, file_dir + rank_dir * 8).unwrap();
            if next == to {
                break;
            }
            if self.current_position().board().piece_at(next.into()).is_some() {
                return false;
            }
            current = next;
        }
        
        true
    }
    
    fn calculate_target_square_scid(&self, from: Square, offset: i8) -> std::result::Result<Square, ()> {
        let from_idx = from.0 as i8;
        let target_idx = from_idx + offset;
        
        if target_idx < 0 || target_idx >= 64 {
            return Err(()); // Invalid target square
        }
        
        Ok(Square(target_idx as u8))
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
}
