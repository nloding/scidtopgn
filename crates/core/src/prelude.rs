//! Convenience re-exports for common types
//!
//! The prelude module provides convenient access to the most commonly used types
//! and traits in the library. Import this module to get everything you need:
//!
//! ```
//! use scidtopgn_core::prelude::*;
//! ```
//!
//! # What's Included
//!
//! - Error types: `ScidError`, `Result`
//! - Core types: `GameResult`, `GameDate`
//! - Shakmaty chess types: `Color`, `Square`, `Move`, `Position`, etc.
//! - Main API types (when implemented): `ScidReader`, `PgnOptions`

// Core library types
pub use crate::error::{Result, ScidError};
pub use crate::types::GameDate;
pub use crate::types::GameResult;

// Shakmaty chess types (most commonly used)
pub use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};

// Main API types (will be uncommented in later phases)
// pub use crate::database::reader::ScidReader;
// pub use crate::database::index::GameIndexEntry;
// pub use crate::format::converter::PgnOptions;
// pub use crate::format::pgn::PgnFormatter;
