use crate::bridge::GameState;

impl GameState {
	/// Generate PGN output for the game
	pub fn to_pgn(&self) -> String {
		let mut pgn = String::new();
		// Add headers
		if let Some(ref metadata) = self.metadata() {
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
		for (i, san_move) in self.san_moves().iter().enumerate() {
			if i % 2 == 0 {
				pgn.push_str(&format!("{}. ", (i / 2) + 1));
			}
			pgn.push_str(san_move);
			pgn.push(' ');
		}
		// Add result at end
		if let Some(ref metadata) = self.metadata() {
			pgn.push_str(&metadata.result);
		} else {
			pgn.push_str("*"); // Unknown result for empty game
		}
		pgn
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

// Placeholder for PGN generation functionality
// Implementation will be added in later steps