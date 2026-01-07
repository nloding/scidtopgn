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
