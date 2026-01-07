//! SCID Database Parser and PGN Converter
//!
//! This library provides functionality to parse SCID (Shane's Chess Information Database)
//! files and convert them to standard PGN (Portable Game Notation) format.
//!
//! # Quick Start
//!
//! ```ignore
//! use scidtopgn_core::prelude::*;
//!
//! let reader = ScidReader::open("database.si4")?;
//! let pgn = reader.to_pgn(PgnOptions::default())?;
//! println!("{}", pgn);
//! ```

// Public modules
pub mod database;
pub mod format;
pub mod parser;

// Core types and errors
mod error;
pub mod prelude;
mod types;

// Re-exports for convenience
pub use error::{Result, ScidError};
pub use types::{GameDate, GameResult};
