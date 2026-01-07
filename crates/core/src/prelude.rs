//! Convenience re-exports for common types
//!
//! Import this module to get commonly used types:
//!
//! ```
//! use scidtopgn_core::prelude::*;
//! ```

// Core library types
pub use crate::error::{Result, ScidError};
pub use crate::types::{GameDate, GameResult};

// Shakmaty chess types
pub use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};
