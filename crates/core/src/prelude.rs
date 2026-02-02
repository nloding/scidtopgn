//! Convenience re-exports for common types and functions.
//!
//! The prelude makes it easy to get started with the library by
//! importing the most commonly used types in one line:
//!
//! ```
//! use scidtopgn_core::prelude::*;
//!
//! let reader = ScidReader::open("database")?;
//! let options = PgnOptions::default();
//! let pgn = reader.to_pgn(&options)?;
//! # Ok::<(), ScidError>(())
//! ```
//!
//! # What's in the Prelude?
//!
//! ## Core Types
//!
//! - **[`ScidReader`]**: Main entry point for reading SCID databases
//! - **[`Game`]**: Single game with metadata and moves
//! - **[`ScidError`]**: Error type for all operations
//! - **[`Result<T>`]**: Convenient Result type alias
//!
//! ## Formatting
//!
//! - **[`PgnOptions`]**: PGN output formatting options
//!
//! ## Chess Types
//!
//! - **[`ChessMove`]**: Chess move from shakmaty
//! - **[`Color`]**: Chess color
//! - **[`Role`]**: Chess piece role
//! - **[`Square`]**: Chess square
//!
//! ## Database Types
//!
//! - **[`GameDate`]**: Game date with partial support
//! - **[`GameResult`]**: Game outcome
//! - **[`MaterialSignature`]**: Position material encoding
//! - **[`RatingType`]**: Rating system (Elo, USCF, etc.)
//! - **[`MATSIG_EMPTY`]**: Empty material signature constant
//! - **[`MATSIG_STANDARD_START`]**: Standard opening material signature
//!
//! ## Rating Parsing
//!
//! - **[`parse_rating()`]**: Parse rating from database strings
//!
//! ## Error Recovery
//!
//! - **[`ErrorMode`]**: How to handle errors during conversion
//! - **[`GameProcessResult`]**: Result of processing a single game
//! - **[`ConversionOptions`]**: Conversion configuration
//! - **[`ConversionStats`]**: Statistics from conversion
//!
//! ## File Access
//!
//! - **[`FileAccessMode`]**: File access strategy
//! - **[`OpenOptions`]**: File opening options
//!
//! # Common Import Patterns
//!
//! ## Basic usage
//!
//! ```rust
//! use scidtopgn_core::prelude::*;
//! # fn main() -> Result<(), ScidError> {
//! let reader = ScidReader::open("database.si4")?;
//! let game = reader.game(0)?;
//! println!("{} vs {}", game.white(), game.black());
//! # Ok(())
//! # }
//! ```
//!
//! ## Converting to PGN
//!
//! ```rust
//! use scidtopgn_core::prelude::*;
//! # fn main() -> Result<(), ScidError> {
//! let reader = ScidReader::open("database.si4")?;
//! let output = reader.to_pgn(&PgnOptions::default())?;
//! println!("{}", output);
//! # Ok(())
//! # }
//! ```
//!
//! ## Filtering by player
//!
//! ```rust
//! use scidtopgn_core::prelude::*;
//! # fn main() -> Result<(), ScidError> {
//! let reader = ScidReader::open("database.si4")?;
//!
//! for game in reader.games() {
//!     let game = game?;
//!     if game.white().contains("Carlsen") || game.black().contains("Carlsen") {
//!         let pgn = game.to_pgn()?;
//!         println!("{}", pgn);
//!     }
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Why Use the Prelude?
//!
//! ## Simplicity
//!
//! Instead of:
//!
//! ```rust
//! use scidtopgn_core::{ScidReader, Game, ScidError, PgnOptions, ChessMove};
//! ```
//!
//! You can use:
//!
//! ```rust
//! use scidtopgn_core::prelude::*;
//! ```
//!
//! ## Consistency
//!
//! The prelude provides a consistent set of types across the library,
//! making it easier to remember what's available without constantly
//! checking the documentation.
//!
//! ## Discoverability
//!
//! By importing from the prelude, you'll be exposed to common types
//! that may be useful in your code, encouraging better patterns.
//!
//! # Performance Impact
//!
//! The prelude has zero runtime cost. It only re-exports types at
//! compile-time. There's no performance difference between using
//! the prelude or importing types individually.

// Re-export main types
pub use crate::Game;
pub use crate::Result;
pub use crate::ScidError;
pub use crate::ScidReader;

// Re-export format types
pub use crate::format::pgn::PgnOptions;

// Re-export common data types
pub use crate::types::{GameDate, GameResult};

// Re-export rating and material types
pub use crate::database::{
    parse_rating, MaterialSignature, RatingType, MATSIG_EMPTY, MATSIG_STANDARD_START,
};

// Re-export error recovery types (Gap 13)
pub use crate::reader::{ConversionOptions, ConversionStats, ErrorMode, GameProcessResult};

// Re-export file access types (Gap 14)
pub use crate::reader::{FileAccessMode, OpenOptions};

// Re-export commonly used shakmaty types for convenience
pub use shakmaty::{Color, Move as ChessMove, Role, Square};
