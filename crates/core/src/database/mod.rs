//! SCID database file parsing
//!
//! This module handles parsing all three SCID file types:
//! - `.si4` - Index file with game metadata
//! - `.sn4` - Name file with player/event/site/round names
//! - `.sg4` - Game file with chess moves and annotations

pub mod files;
pub mod games;
pub mod index;
pub mod names;
pub mod reader;
pub mod types;

// Re-exports for convenience
pub use games::{flags, parse_game_structure, parse_game_tags, read_game_data, GameData};
pub use index::{
    parse_game_index_entry, parse_si4_file, parse_si4_header, GameIndexEntry, IndexEntryIter,
    Si4Header, GAME_ENTRY_SIZE, SCID_VERSION, SI4_HEADER_SIZE, SI4_MAGIC,
};
pub use names::{parse_name_database, NameDatabase, Sn4Header, SN4_HEADER_SIZE, SN4_MAGIC};
