use shakmaty::{fen::Fen, Move, Position};

use crate::error::{Result, ScidError};

use super::position::PieceNumberMapping;

/// Complete game representation with variations
#[derive(Debug, Clone)]
pub struct GameTree {
    /// Starting position (None for standard start, Some(fen) for custom)
    pub start_fen: Option<String>,

    /// Root of the move tree (main line + variations)
    pub root: MoveNode,
}

/// Single node in move tree
#[derive(Debug, Clone)]
pub struct MoveNode {
    /// The chess move (None for root node before first move)
    pub chess_move: Option<Move>,

    /// Comment attached to this move (from ENCODE_COMMENT markers)
    pub comment: Option<String>,

    /// NAG annotations (from ENCODE_NAG markers)
    pub nags: Vec<u8>,

    /// Continuation (next move in this line)
    pub continuation: Option<Box<MoveNode>>,

    /// Alternative variations starting from this position
    /// Created when ENCODE_START_MARKER (0x0D) is encountered
    pub variations: Vec<MoveNode>,
}

impl MoveNode {
    /// Create root node (before first move)
    pub fn root() -> Self {
        MoveNode {
            chess_move: None,
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }
    }

    /// Create node with a move
    pub fn with_move(chess_move: Move) -> Self {
        MoveNode {
            chess_move: Some(chess_move),
            comment: None,
            nags: Vec::new(),
            continuation: None,
            variations: Vec::new(),
        }
    }

    /// Add a move as continuation and return mutable reference to it
    pub fn add_continuation(&mut self, chess_move: Move) -> &mut MoveNode {
        self.continuation = Some(Box::new(MoveNode::with_move(chess_move)));
        self.continuation.as_mut().unwrap()
    }

    /// Add a variation and return mutable reference to first move
    pub fn add_variation(&mut self, first_move: Move) -> &mut MoveNode {
        self.variations.push(MoveNode::with_move(first_move));
        self.variations.last_mut().unwrap()
    }

    /// Iterate over main line moves
    pub fn main_line(&self) -> MainLineIter {
        MainLineIter {
            current: Some(self),
        }
    }
}

/// Iterator over main line moves
pub struct MainLineIter<'a> {
    current: Option<&'a MoveNode>,
}

impl<'a> Iterator for MainLineIter<'a> {
    type Item = &'a MoveNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.current?;
        self.current = node.continuation.as_deref();
        Some(node)
    }
}

impl GameTree {
    /// Create new game tree with standard starting position
    pub fn new() -> Self {
        GameTree {
            start_fen: None,
            root: MoveNode::root(),
        }
    }

    /// Create game tree with custom starting position
    pub fn with_fen(fen: String) -> Self {
        GameTree {
            start_fen: Some(fen),
            root: MoveNode::root(),
        }
    }

    /// Get main line moves as vector
    pub fn main_line_moves(&self) -> Vec<&Move> {
        self.root
            .main_line()
            .filter_map(|node| node.chess_move.as_ref())
            .collect()
    }

    /// Count total moves including variations (DFS)
    pub fn total_move_count(&self) -> usize {
        fn count_node(node: &MoveNode) -> usize {
            let mut count = if node.chess_move.is_some() { 1 } else { 0 };
            if let Some(ref cont) = node.continuation {
                count += count_node(cont);
            }
            for var in &node.variations {
                count += count_node(var);
            }
            count
        }
        count_node(&self.root)
    }
}

/// Variation parsing state machine with position tracking (Gap 1)
///
/// # CRITICAL: Position State Restoration
///
/// When entering a variation (START_MARKER 0x0D), we must save the COMPLETE
/// chess position state. When exiting (END_MARKER 0x0E), we restore it.
///
/// This is essential because variations are alternative continuations from
/// a specific position. Without restoring, the position would be corrupted
/// by the variation's moves.
///
/// ## Why Position State Matters
///
/// ```text
/// Main line: 1.e4 e5 2.Nf3
///                      ↓
///              ┌───────┴────────┐
///              │                │
///           2...Nc6          [VAR: 2...d6 3.d4]
///              ↓                  ↓
///           3.Bb5             END_MARKER (0x0E)
///                                 ↓
///                             ← MUST restore position after 2.Nf3!
/// ```
///
/// ## Position State Stack
///
/// Each stack entry includes the complete position state:
/// - Board configuration (piece placement)
/// - Side to move
/// - Castling rights
/// - En passant square
/// - Halfmove clock
/// - Fullmove number
/// - SCID piece number mapping (for continued decoding)
#[derive(Debug)]
pub struct VariationParseState {
    /// Stack of saved positions for variation restore
    position_stack: Vec<VariationSnapshot>,

    /// Current chess position
    current_position: shakmaty::Chess,

    /// Current SCID piece mapping (for move decoding)
    current_piece_mapping: PieceNumberMapping,

    /// Current node in tree being built
    current_node: *mut MoveNode,

    /// Variation depth (for debugging)
    depth: usize,
}

/// Snapshot of state to restore after variation ends
#[derive(Debug, Clone)]
pub struct VariationSnapshot {
    /// Chess position at variation start
    pub position: shakmaty::Chess,

    /// SCID piece mapping at variation start
    pub piece_mapping: PieceNumberMapping,

    /// Node to return to (parent of variation)
    pub return_node: *mut MoveNode,
}

impl VariationParseState {
    /// Create new parse state with starting position
    pub fn new(start_position: shakmaty::Chess) -> Self {
        let piece_mapping = PieceNumberMapping::from_position(&start_position).unwrap();
        Self {
            position_stack: Vec::new(),
            current_position: start_position,
            current_piece_mapping: piece_mapping,
            current_node: std::ptr::null_mut(),
            depth: 0,
        }
    }

    /// Create from FEN string
    pub fn from_fen(fen: &str) -> Result<Self> {
        let parsed: Fen = fen
            .parse()
            .map_err(|e| ScidError::InvalidFormat(format!("FEN parse error: {:?}", e)))?;
        let position: shakmaty::Chess = parsed
            .into_position(shakmaty::CastlingMode::Standard)
            .map_err(|e| ScidError::InvalidFormat(format!("Invalid position: {:?}", e)))?;
        Ok(Self::new(position))
    }

    /// Handle START_MARKER (0x0D) - entering a variation
    ///
    /// Saves complete position state before processing variation moves.
    pub fn start_variation(&mut self) {
        let snapshot = VariationSnapshot {
            position: self.current_position.clone(),
            piece_mapping: self.current_piece_mapping.clone(),
            return_node: self.current_node,
        };
        self.position_stack.push(snapshot);

        self.depth += 1;
    }

    /// Handle END_MARKER (0x0E) - exiting a variation
    ///
    /// Restores complete position state to continue main line.
    pub fn end_variation(&mut self) -> bool {
        if let Some(snapshot) = self.position_stack.pop() {
            self.current_position = snapshot.position;
            self.current_piece_mapping = snapshot.piece_mapping;
            self.current_node = snapshot.return_node;
            self.depth = self.depth.saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// Apply a move to current position
    pub fn apply_move(&mut self, chess_move: &shakmaty::Move) -> Result<(), ScidError> {
        self.current_piece_mapping
            .update_after_move(chess_move, self.current_position.turn());

        self.current_position = self
            .current_position
            .clone()
            .play(chess_move)
            .map_err(|e| ScidError::InvalidFormat(format!("Illegal move: {:?}", e)))?;

        Ok(())
    }

    /// Get current position for move validation
    pub fn position(&self) -> &shakmaty::Chess {
        &self.current_position
    }

    /// Get current piece mapping for move decoding
    pub fn piece_mapping(&self) -> &PieceNumberMapping {
        &self.current_piece_mapping
    }

    /// Current variation depth
    pub fn variation_depth(&self) -> usize {
        self.depth
    }
}

/// Common NAG (Numeric Annotation Glyph) values
pub mod nag {
    pub const GOOD_MOVE: u8 = 1;
    pub const MISTAKE: u8 = 2;
    pub const BRILLIANT_MOVE: u8 = 3;
    pub const BLUNDER: u8 = 4;
    pub const INTERESTING_MOVE: u8 = 5;
    pub const DUBIOUS_MOVE: u8 = 6;
    pub const EQUAL: u8 = 10;
    pub const UNCLEAR: u8 = 13;
    pub const SLIGHT_ADVANTAGE_WHITE: u8 = 14;
    pub const SLIGHT_ADVANTAGE_BLACK: u8 = 15;
    pub const CLEAR_ADVANTAGE_WHITE: u8 = 16;
    pub const CLEAR_ADVANTAGE_BLACK: u8 = 17;
    pub const WINNING_WHITE: u8 = 18;
    pub const WINNING_BLACK: u8 = 19;

    /// Convert NAG to PGN symbol (for common NAGs) or $N notation
    pub fn to_pgn_string(nag: u8) -> String {
        match nag {
            1 => "!".to_string(),
            2 => "?".to_string(),
            3 => "!!".to_string(),
            4 => "??".to_string(),
            5 => "!?".to_string(),
            6 => "?!".to_string(),
            _ => format!("${}", nag),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Move, Role, Square};

    #[test]
    fn test_game_tree_creation() {
        let mut tree = GameTree::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        let node = tree.root.add_continuation(e4);

        let e5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E5,
            capture: None,
            promotion: None,
        };
        node.add_continuation(e5);

        assert_eq!(tree.main_line_moves().len(), 2);
        assert_eq!(tree.total_move_count(), 2);
    }

    #[test]
    fn test_variation_creation() {
        let mut tree = GameTree::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        let node = tree.root.add_continuation(e4);

        let d4 = Move::Normal {
            role: Role::Pawn,
            from: Square::D2,
            to: Square::D4,
            capture: None,
            promotion: None,
        };
        tree.root.add_variation(d4);

        assert_eq!(tree.main_line_moves().len(), 1);
        assert_eq!(tree.total_move_count(), 2);
        assert_eq!(tree.root.variations.len(), 1);
    }

    #[test]
    fn test_variation_parse_state_position_restore() {
        let start = shakmaty::Chess::default();
        let mut state = VariationParseState::new(start.clone());

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        state.apply_move(&e4).unwrap();

        let position_before_var = state.position().clone();

        state.start_variation();
        assert_eq!(state.variation_depth(), 1);

        let d5 = Move::Normal {
            role: Role::Pawn,
            from: Square::D7,
            to: Square::D5,
            capture: None,
            promotion: None,
        };
        state.apply_move(&d5).unwrap();

        assert_ne!(state.position().board(), position_before_var.board());

        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 0);

        assert_eq!(state.position().board(), position_before_var.board());
    }

    #[test]
    fn test_variation_parse_state_nested_variations() {
        let start = shakmaty::Chess::default();
        let mut state = VariationParseState::new(start);

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        state.apply_move(&e4).unwrap();

        state.start_variation();
        assert_eq!(state.variation_depth(), 1);

        state.start_variation();
        assert_eq!(state.variation_depth(), 2);

        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 1);

        assert!(state.end_variation());
        assert_eq!(state.variation_depth(), 0);

        assert!(!state.end_variation());
    }

    #[test]
    fn test_nag_constants() {
        assert_eq!(nag::GOOD_MOVE, 1);
        assert_eq!(nag::MISTAKE, 2);
        assert_eq!(nag::BRILLIANT_MOVE, 3);
        assert_eq!(nag::BLUNDER, 4);
        assert_eq!(nag::EQUAL, 10);
        assert_eq!(nag::UNCLEAR, 13);
    }

    #[test]
    fn test_nag_to_pgn_string() {
        assert_eq!(nag::to_pgn_string(1), "!");
        assert_eq!(nag::to_pgn_string(2), "?");
        assert_eq!(nag::to_pgn_string(3), "!!");
        assert_eq!(nag::to_pgn_string(4), "??");
        assert_eq!(nag::to_pgn_string(5), "!?");
        assert_eq!(nag::to_pgn_string(6), "?!");

        assert_eq!(nag::to_pgn_string(1), "!");
        assert_eq!(nag::to_pgn_string(99), "$99");
    }

    #[test]
    fn test_with_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string();
        let tree = GameTree::with_fen(fen);

        assert_eq!(tree.start_fen, Some(fen));
        assert_eq!(tree.root.chess_move, None);
    }

    #[test]
    fn test_main_line_iterator() {
        let mut tree = GameTree::new();

        let e4 = Move::Normal {
            role: Role::Pawn,
            from: Square::E2,
            to: Square::E4,
            capture: None,
            promotion: None,
        };
        let node = tree.root.add_continuation(e4);

        let e5 = Move::Normal {
            role: Role::Pawn,
            from: Square::E7,
            to: Square::E5,
            capture: None,
            promotion: None,
        };
        node.add_continuation(e5);

        let n3 = Move::Normal {
            role: Role::Knight,
            from: Square::G1,
            to: Square::F3,
            capture: None,
            promotion: None,
        };
        node.add_continuation(n3);

        let moves: Vec<&Move> = tree.main_line_moves().clone();
        assert_eq!(moves.len(), 3);
        assert_eq!(moves[0].from(), &Square::E2);
        assert_eq!(moves[1].from(), &Square::E7);
        assert_eq!(moves[2].from(), &Square::G1);
    }
}
