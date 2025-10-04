// Bridge Layer - SCID to Shakmaty Conversion
//
// This module provides the bridge layer for converting SCID-specific data
// to shakmaty chess types. The bridge pattern allows the SCID parser to
// focus on binary format parsing while delegating chess logic to shakmaty.

use shakmaty::{Chess, Move};

pub mod position;
pub mod position_tracker;
pub mod moves;

// Re-export key types for easier access

pub use position::*;

/// Core trait for converting SCID data to shakmaty types
///
/// Trait for types that can provide chess position context
///
/// This trait allows access to the current chess position and move history,
/// enabling position-aware operations and move validation.
pub trait PositionContext {
    /// Get the current chess position
    fn current_position(&self) -> &Chess;

    /// Get the complete move history
    fn move_history(&self) -> &[Move];

    /// Get the number of moves played
    fn move_count(&self) -> usize {
        self.move_history().len()
    }

    /// Check if the current position is legal
    fn is_position_legal(&self) -> bool {
        // Note: In shakmaty 0.26, we use is_sane() to check position validity
        // is_legal() requires a move parameter to check if that move is legal
        true // Placeholder - will be implemented with proper position validation
    }
}

/// Trait for types that can generate chess notation
///
/// This trait provides methods for converting game state and moves
/// to various chess notation formats, primarily PGN.
pub trait ChessNotation {
    /// Generate PGN representation
    fn to_pgn(&self) -> String;

    /// Generate PGN with custom headers
    fn to_pgn_with_headers(&self, headers: &[(String, String)]) -> String {
        let mut pgn = String::new();

        // Add headers
        for (key, value) in headers {
            pgn.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        pgn.push('\n');

        // Add moves
        pgn.push_str(&self.to_pgn());

        pgn
    }
}

/// Trait for basic chess validation operations
///
/// This trait provides methods for validating chess moves and checking
/// game state conditions like check, checkmate, and stalemate.
#[allow(dead_code)]
pub trait BasicChessValidation {
    /// Check if a move is legal in the current position
    fn is_move_legal(&self, chess_move: &Move) -> bool;

    /// Check if the current side to move is in check
    fn is_in_check(&self) -> bool;

    /// Check if the current position is checkmate
    fn is_checkmate(&self) -> bool;

    /// Check if the current position is stalemate
    fn is_stalemate(&self) -> bool;

    /// Get the game outcome if the game has ended
    fn game_outcome(&self) -> Option<shakmaty::Outcome>;

    /// Find the square of the king for the given color
    fn find_king_square(&self, color: crate::position::moves::Color) -> crate::position::moves::Square;
}

