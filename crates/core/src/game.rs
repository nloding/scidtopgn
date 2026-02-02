//! Game struct for holding parsed game data
//!
//! This module provides the Game struct that wraps parsed SCID game data
//! and provides convenient methods for accessing game information.

use crate::database::GameIndexEntry;
use crate::error::Result;
use crate::format::movetext::MovetextFormatter;
use crate::format::pgn::PgnOptions;
use crate::format::tags::{SevenTagRoster, SupplementalTags};
use std::collections::HashMap;

/// Represents a single parsed chess game
///
/// This struct contains all the metadata and move data for a single game
/// extracted from a SCID database file.
#[derive(Debug, Clone)]
pub struct Game {
    /// Game metadata from the index file
    pub index_entry: GameIndexEntry,

    /// Name database containing player, event, site, and round names
    pub names: crate::database::NameDatabase,

    /// Game data including tags and moves
    pub game_data: crate::database::GameData,

    /// Formatting options for PGN output
    pub options: PgnOptions,
}

impl Game {
    /// Create a new Game from raw data
    pub fn new(
        index_entry: GameIndexEntry,
        names: crate::database::NameDatabase,
        game_data: crate::database::GameData,
        options: PgnOptions,
    ) -> Self {
        Self {
            index_entry,
            names,
            game_data,
            options,
        }
    }

    /// Format this game as complete PGN
    ///
    /// # Returns
    ///
    /// A string containing the complete PGN representation of this game
    pub fn to_pgn(&self) -> Result<String> {
        crate::format::pgn::PgnFormatter::format_game(
            &self.index_entry,
            &self.names,
            &self.game_data,
            &self.options,
        )
    }

    /// Format this game as PGN and write to a writer
    ///
    /// # Arguments
    ///
    /// * `writer` - A mutable reference to a writer implementing std::io::Write
    pub fn write_pgn<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
        crate::format::pgn::PgnFormatter::write_game(
            writer,
            &self.index_entry,
            &self.names,
            &self.game_data,
            &self.options,
        )
    }
}
