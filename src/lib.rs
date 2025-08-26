// SCID Parser Library
//
// This library provides functionality for parsing SCID chess database files
// and converting them to standard formats like PGN. It includes a bridge layer
// for integrating with the shakmaty chess library.

// Allow dead code for development and debugging utilities
#![allow(dead_code)]

// Core SCID parsing modules
pub mod utils;
pub mod date;
pub mod si4;
pub mod sg4;
pub mod sn4;

// Shakmaty integration modules
pub mod bridge;
pub mod error;

// PGN export module
pub mod pgn;

// Position-aware decoding modules
pub mod position;

// Variation tree building module
pub mod variation_builder;
// Shared variation data structures
pub mod variation;

// Re-export key types for easier access
pub use bridge::{GameState, GameMetadata, PositionContext, ChessNotation, ChessValidation};
pub use error::{ScidError, Result};
pub use pgn::{PgnExporter, ParsedGame};
pub use variation::{VariationTree, Variation, VariationMove, VariationGameElement};