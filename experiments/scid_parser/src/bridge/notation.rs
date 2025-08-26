use crate::bridge::{GameState, ChessNotation};

impl GameState {
	/// Generate PGN output for the game
	pub fn to_pgn(&self) -> String {
		// Delegate to the canonical ChessNotation implementation to avoid duplication
		<GameState as ChessNotation>::to_pgn(self)
	}
}
// PGN Notation Generation using Shakmaty
//
// This module handles generating standard chess notation (PGN format) from
// game state using shakmaty's notation capabilities. It converts game history
// and metadata into proper PGN format suitable for export.

// use shakmaty::{Chess, Move, Position};  // Commented out - unused imports
// Note: GameState import will be added when implementing PGN generation
// use crate::bridge::GameState;

// Placeholder comments remain for context; functionality now delegated to ChessNotation impl