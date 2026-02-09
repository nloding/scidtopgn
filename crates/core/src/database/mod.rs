//! SCID database file parsing
//!
//! This module handles parsing all three SCID file types:
//! - `.si4` - Index file with game metadata
//! - `.sn4` - Name file with player/event/site/round names
//! - `.sg4` - Game file with chess moves and annotations

// Module declarations
pub mod files;
pub mod games;
pub mod index;
pub mod names;
pub mod reader;
pub mod types;

// Re-exports
pub use index::{
    eco_to_string, game_flags, parse_rating, GameIndexEntry, MaterialSignature, RatingType,
    Si4Header, MATSIG_EMPTY, MATSIG_STANDARD_START,
};

pub use names::{NameDatabase, NameLookupResult, Sn4Header};

pub use games::{decompress_game_data, read_and_decompress_game, read_game_data, GameData};

// Re-export from reader module
pub use reader::{
    ConversionOptions, ConversionStats, ErrorMode, FileAccessMode, GameProcessResult, OpenOptions,
};
