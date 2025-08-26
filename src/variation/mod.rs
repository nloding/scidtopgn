//! Variation data structures used across parsing and PGN formatting.
//!
//! Extracted from sg4.rs to reduce coupling and renamed VariationTreeV2 -> VariationTree.

use crate::position::ScidMove;

/// Complete variation tree structure for PGN export
#[derive(Debug, Clone)]
pub struct VariationTree {
    /// Main line moves (the primary game sequence)
    pub main_line: Vec<VariationMove>,

    /// Variations from the main line
    pub variations: Vec<Variation>,
}

#[derive(Debug, Clone)]
pub struct Variation {
    /// Move number where this variation starts (0-based)
    pub start_move_index: usize,

    /// The variation moves
    pub moves: Vec<VariationMove>,

    /// Nested sub-variations within this variation
    pub sub_variations: Vec<Variation>,

    /// Depth level (0 = main line, 1 = first level variation, etc.)
    pub depth: usize,
}

#[derive(Debug, Clone)]
pub struct VariationMove {
    /// The actual chess move
    pub chess_move: ScidMove,

    /// Move number (1, 2, 3, etc.)
    pub move_number: usize,

    /// Is this a white move (true) or black move (false)
    pub is_white_move: bool,

    /// Comments attached to this move
    pub comments: Vec<String>,

    /// NAG annotations for this move
    pub nags: Vec<u8>,

    /// Move in algebraic notation (e4, Nf3, etc.)
    pub algebraic: String,
}

/// Enhanced game element for variation support
#[derive(Debug, Clone)]
pub enum VariationGameElement {
    Move {
        piece_num: u8,
        move_value: u8,
        raw_bytes: Vec<u8>,
        offset: usize,
        bytes_consumed: usize,
    },
    VariationStart {
        offset: usize,
        depth: usize, // track nesting depth
    },
    VariationEnd {
        offset: usize,
        depth: usize, // track nesting depth
    },
    Comment {
        text: String,
        offset: usize,
    },
    Nag {
        nag_value: u8,
        offset: usize,
    },
    GameEnd {
        offset: usize,
    },
}
