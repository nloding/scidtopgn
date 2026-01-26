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
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let reader = ScidReader::open("database.si4")?;
//! let pgn = reader.to_pgn(PgnOptions::default())?;
//! println!("{}", pgn);
//! # Ok(())
//! # }
//! ```

// Public modules
pub mod database;
pub mod format;
pub mod parser;

// Core types and errors
pub mod error;
pub mod prelude;
mod types;

// Re-exports for convenience
pub use error::{Result, ScidError};
pub use types::{GameDate, GameResult};

// Main API (will be implemented in later phases)
// pub use database::reader::ScidReader;
// pub use format::converter::PgnOptions;
