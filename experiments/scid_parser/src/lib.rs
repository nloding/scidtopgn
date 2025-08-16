// SCID Parser Library
//
// This library provides functionality for parsing SCID chess database files
// and converting them to standard formats like PGN. It includes a bridge layer
// for integrating with the shakmaty chess library.

// Core SCID parsing modules
pub mod utils;
pub mod date;
pub mod si4;
pub mod sg4;
pub mod sn4;
pub mod position;

// Shakmaty integration modules
pub mod bridge;
pub mod error;

// PGN export module
pub mod pgn;

// Re-export key types for easier access
pub use bridge::{GameState, GameMetadata, PositionContext, ChessNotation, ChessValidation};
pub use error::{ScidError, Result};
pub use pgn::{PgnExporter, ParsedGame};