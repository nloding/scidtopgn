//! SG4 format parser module
//!
//! Migration notice: SG4Parser is now the default decoding path.
//! It maintains a shakmaty::Chess position and synchronized SCID piece list,
//! deriving the actual piece roles from board state to avoid dual numbering conflicts.
//!
//! Deprecated: Sg4File and GameIterator remain for backward compatibility only.
//! They are annotated with #[deprecated] and will be removed in a future release.
//! Use SG4Parser for indexed single-byte moves and queen diagonal multi-byte moves.

use crate::core::error::{Result, ScidError, EnhancedDecodeError};
use memmap2::Mmap;
use shakmaty::{Chess, Position, Role, Color, Square};
use std::fs::File;
use std::path::Path;

/// Represents a decoded SCID move with full context information.
/// 
/// This struct bridges the gap between SG4 move encoding and shakmaty move representation.
/// It preserves both the original SG4 encoding information and the interpreted piece type
/// to handle dual numbering systems correctly.
/// 
/// # Fields
/// 
/// * `raw_bytes` - The original byte(s) from SG4 file
/// * `piece_num` - SG4 piece number from move encoding (1=King, 2=Queen, ..., 6=Pawn)
/// * `move_value` - SG4 move value from move encoding (0-15 range)
/// * `interpretation` - High-level interpretation of the move (castling, promotion, etc.)
/// * `from_square_index` - Optional from square index from SG4
/// * `to_square_index` - Optional to square index from SG4  
/// * `promotion_piece` - Optional promotion piece as string
/// * `piece_type` - **CRITICAL**: Preserved actual piece type for correct routing
/// 
/// # Dual Numbering Systems
/// 
/// SCID uses two different piece numbering systems:
/// 1. **Move Encoding**: piece_num 1-6 maps to piece types (1=King, 2=Queen, etc.)
/// 2. **Board Tracking**: position list indices 0-11 map to specific piece locations
/// 
/// The `piece_type` field preserves the actual piece type from move interpretation
/// to ensure correct routing through the pipeline and avoid dual numbering conflicts.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedMove {
    pub raw_bytes: Vec<u8>,
    pub piece_num: u8,
    pub move_value: u8,
    pub interpretation: MoveInterpretation,
    pub from_square_index: Option<u8>,
    pub to_square_index: Option<u8>,
    pub promotion_piece: Option<String>,
    /// **CRITICAL FIELD**: Preserved actual piece type from interpretation
    /// This prevents dual numbering system conflicts by maintaining piece type context
    /// throughout the entire parsing pipeline. Without this field, the bridge layer
    /// would incorrectly route pawn promotion moves (e.g., 0x6C) to the knight decoder
    /// instead of the pawn decoder.
    pub piece_type: Option<Role>,
}

/// SG4Parser maintains decoding state: shakmaty position and SCID piece list
/// Parser maintaining shakmaty board and SCID indices for role-driven decoding
/// Index semantics: piece_num (0..15) indexes current_piece_list; routing uses role_at_square.
pub struct SG4Parser {
    pub data: Vec<u8>,
    pub offset: usize,
    pub current_shakmaty_position: Option<Chess>,
    pub current_piece_list: Option<[u8; 16]>,
    pub position_history: Vec<Chess>,
}

impl SG4Parser {
    pub fn new(data: Vec<u8>) -> Self {
        let mut parser = Self {
            data,
            offset: 0,
            current_shakmaty_position: None,
            current_piece_list: None,
            position_history: Vec::new(),
        };
        parser.initialize_from_starting_position();
        parser
    }

    fn role_at_square(&self, sq: Square) -> std::result::Result<Role, EnhancedDecodeError> {
        let pos = self.current_shakmaty_position.as_ref().ok_or(EnhancedDecodeError::MissingPosition)?;
        let piece = pos.board().piece_at(sq).ok_or_else(|| EnhancedDecodeError::EmptySquareAtPosition(format!("{:?}", sq)))?;
        Ok(piece.role)
    }

    pub fn decode_single_byte_move_indexed(&mut self, byte: u8) -> std::result::Result<DecodedMove, EnhancedDecodeError> {
        let piece_index = (byte >> 4) & 0x0F;
        let move_value = byte & 0x0F;
        let from_sq = self.get_from_square_by_index(piece_index)?;
        let role = self.role_at_square(from_sq)?;
        let interpretation = match role {
            Role::King => decode_king_move(move_value),
            Role::Queen => decode_queen_move(move_value),
            Role::Rook => decode_rook_move(move_value),
            Role::Bishop => decode_bishop_move(move_value),
            Role::Knight => decode_knight_move(move_value),
            Role::Pawn => decode_pawn_move(move_value),
        };
        Ok(DecodedMove {
            raw_bytes: vec![byte],
            piece_num: piece_index,
            move_value,
            interpretation,
            from_square_index: Some(u32::from(from_sq) as u8),
            to_square_index: None,
            promotion_piece: None,
            piece_type: Some(role),
        })
    }

    /// Role-based single-byte decoder: derives role at from-square and advances offset by 1.
    /// Rationale: avoids dual numbering conflicts by using board state for routing.
    pub fn decode_single_byte_move(&mut self, byte: u8) -> std::result::Result<DecodedMove, EnhancedDecodeError> {
        let piece_num = (byte >> 4) & 0x0F;
        let move_value = byte & 0x0F;
        if piece_num >= 16 { return Err(EnhancedDecodeError::IndexOutOfBounds(piece_num)); }
        let from = self.get_from_square_by_index(piece_num)?;
        let role = self.role_at_square(from)?;
        if role == Role::Queen && self.is_queen_diagonal_move(move_value) {
            // delegate to multi-byte handler; it will consume the next byte
            let result = self.decode_queen_diagonal_start(byte)?;
            return Ok(result);
        }
        let to = self.calculate_target_square(from, move_value, role)?;
        // advance offset for single-byte move
        self.offset += 1;
        Ok(DecodedMove {
            raw_bytes: vec![byte],
            piece_num,
            move_value,
            interpretation: match role {
                Role::King => decode_king_move(move_value),
                Role::Queen => decode_queen_move(move_value),
                Role::Rook => decode_rook_move(move_value),
                Role::Bishop => decode_bishop_move(move_value),
                Role::Knight => decode_knight_move(move_value),
                Role::Pawn => decode_pawn_move(move_value),
            },
            from_square_index: Some(u32::from(from) as u8),
            to_square_index: Some(u32::from(to) as u8),
            promotion_piece: None,
            piece_type: Some(role),
        })
    }

    fn initialize_from_starting_position(&mut self) {
        self.current_shakmaty_position = Some(Chess::default());
        self.current_piece_list = Some(self.get_starting_piece_list());
        self.position_history.clear();
    }

    fn get_starting_piece_list(&self) -> [u8; 16] {
        // Placeholder mapping: indices map to initial pawn/king positions; verify ordering later
        // White: indices 0..7, Black: 8..15
        // Using board index encoding (0..63) where a1=0, h8=63.
        [
            4,  // White King e1
            3,  // White Queen d1
            7,  // White Rook h1
            0,  // White Rook a1
            2,  // White Bishop c1
            5,  // White Bishop f1
            1,  // White Knight b1
            6,  // White Knight g1
            60, // Black King e8
            59, // Black Queen d8
            63, // Black Rook h8
            56, // Black Rook a8
            58, // Black Bishop c8
            61, // Black Bishop f8
            57, // Black Knight b8
            62, // Black Knight g8
        ]
    }

    fn sync_piece_list_with_shakmaty(&mut self) -> std::result::Result<(), EnhancedDecodeError> {
        if let Some(pos) = &self.current_shakmaty_position {
            let mut list = [0u8; 16];
            let mut wi = 0usize;
            let mut bi = 8usize;
            for i in 0u32..64u32 {
                let sq = Square::new(i);
                if let Some(piece) = pos.board().piece_at(sq) {
                    let idx = i as u8;
                    match piece.color { // field access
                        Color::White => {
                            if wi < 8 { list[wi] = idx; wi += 1; }
                        }
                        Color::Black => {
                            if bi < 16 { list[bi] = idx; bi += 1; }
                        }
                    }
                }
            }
            self.current_piece_list = Some(list);
            Ok(())
        } else {
            Err(EnhancedDecodeError::MissingPosition)
        }
    }

    fn get_from_square_by_index(&self, piece_num: u8) -> std::result::Result<Square, EnhancedDecodeError> {
        if piece_num >= 16 {
            return Err(EnhancedDecodeError::IndexOutOfBounds(piece_num));
        }
        let list = self.current_piece_list.as_ref().ok_or(EnhancedDecodeError::MissingPieceList)?;
        let idx = list[piece_num as usize] as u32;
        Ok(Square::new(idx))
    }

    fn is_queen_diagonal_move(&self, move_value: u8) -> bool {
        move_value >= 8
    }

    /// Queen diagonal multi-byte start: consumes second byte, computes direction+distance.
    /// Rationale: centralizes multi-byte handling to ensure correct offset and target square.
    pub fn decode_queen_diagonal_start(&mut self, first_byte: u8) -> std::result::Result<DecodedMove, EnhancedDecodeError> {
        if self.offset + 1 >= self.data.len() {
            return Err(EnhancedDecodeError::InvalidMove("Queen diagonal move extends beyond data bounds".to_string()));
        }
        let second_byte = self.data[self.offset + 1];
        self.offset += 2;
        let piece_index = (first_byte >> 4) & 0x0F;
        let move_value = first_byte & 0x0F;
        let from_sq = self.get_from_square_by_index(piece_index)?;
        let role = self.role_at_square(from_sq)?;
        // Derive diagonal direction from move_value (8..15) -> 4 diagonals
        let dir_code = (move_value - 8) & 0x03;
        let distance = (second_byte & 0x0F) as i32;
        let from_idx = u32::from(from_sq) as i32;
        let file = from_idx % 8;
        let rank = from_idx / 8;
        let (df, dr) = match dir_code {
            0 => (1, 1),   // up-right
            1 => (1, -1),  // down-right
            2 => (-1, -1), // down-left
            3 => (-1, 1),  // up-left
            _ => (0, 0),
        };
        let nf = file + df * distance;
        let nr = rank + dr * distance;
        let to_idx = if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
            Some((nr * 8 + nf) as u8)
        } else {
            None
        };
        Ok(DecodedMove {
            raw_bytes: vec![first_byte, second_byte],
            piece_num: piece_index,
            move_value,
            interpretation: MoveInterpretation::Queen,
            from_square_index: Some(u32::from(from_sq) as u8),
            to_square_index: to_idx,
            promotion_piece: None,
            piece_type: Some(role),
        })
    }

    fn calculate_target_square(&self, from: Square, move_value: u8, piece_role: Role) -> std::result::Result<Square, EnhancedDecodeError> {
        let from_idx = u32::from(from) as i32;
        let file = from_idx % 8;
        let rank = from_idx / 8;
        let mut to_idx: Option<i32> = None;
        match piece_role {
            Role::King => {
                // 0: up, 1: up-right, 2: right, 3: castle kingside, 4: down-right, 5: down,
                // 6: down-left, 7: castle queenside, 8: left, 9: up-left
                let (df, dr) = match move_value {
                    0 => (0, 1),
                    1 => (1, 1),
                    2 => (1, 0),
                    4 => (1, -1),
                    5 => (0, -1),
                    6 => (-1, -1),
                    8 => (-1, 0),
                    9 => (-1, 1),
                    // Treat castles as horizontal king moves for target computation
                    3 => (2, 0),
                    7 => (-2, 0),
                    _ => (0, 0),
                };
                let nf = file + df;
                let nr = rank + dr;
                if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
            }
            Role::Queen => {
                // Single-byte queen moves cover rook-like rays: files or ranks
                if move_value >= 8 {
                    // to specific rank (0..7) same file
                    let target_rank = (move_value - 8) as i32;
                    let nf = file;
                    let nr = target_rank;
                    if nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                } else {
                    // to specific file (0..7) same rank
                    let target_file = move_value as i32;
                    let nf = target_file;
                    let nr = rank;
                    if nf >= 0 && nf < 8 { to_idx = Some(nr * 8 + nf); }
                }
            }
            Role::Rook => {
                if move_value >= 8 {
                    let target_rank = (move_value - 8) as i32;
                    let nf = file;
                    let nr = target_rank;
                    if nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                } else {
                    let target_file = move_value as i32;
                    let nf = target_file;
                    let nr = rank;
                    if nf >= 0 && nf < 8 { to_idx = Some(nr * 8 + nf); }
                }
            }
            Role::Bishop => {
                // move_value: lower 3 bits = target file (0..7); bit3 selects diagonal direction group
                let target_file = (move_value & 0x07) as i32;
                let dir_bit = ((move_value >> 3) & 0x01) as i32;
                let df = target_file - file;
                // Choose dr so that target lies on the selected diagonal direction
                // If dir_bit==0 use up-left/down-right (dr = -df), else up-right/down-left (dr = df)
                let dr = if dir_bit == 0 { -df } else { df };
                let nf = target_file;
                let nr = rank + dr;
                if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
            }
            Role::Knight => {
                const KNIGHT_DIFFS: &[i32] = &[-17, -15, -10, -6, 6, 10, 15, 17];
                let code = if move_value < 8 { move_value } else { move_value - 8 } as usize;
                if code < KNIGHT_DIFFS.len() {
                    let idx = from_idx + KNIGHT_DIFFS[code];
                    let nf = idx % 8;
                    let nr = idx / 8;
                    if idx >= 0 && idx < 64 && nf >= 0 && nf < 8 && nr >= 0 && nr < 8 {
                        to_idx = Some(idx);
                    }
                }
            }
            Role::Pawn => {
                // Determine color from current position
                let pos = self.current_shakmaty_position.as_ref().ok_or(EnhancedDecodeError::MissingPosition)?;
                let color = pos.board().piece_at(from).ok_or_else(|| EnhancedDecodeError::EmptySquareAtPosition(format!("{:?}", from)))?.color;
                let forward = if color == Color::White { 1 } else { -1 };
                match move_value {
                    0 | 3 | 6 | 9 | 12 => { // forward or forward with promotion type encoded
                        let nf = file;
                        let nr = rank + forward;
                        if nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                    }
                    1 | 4 | 7 | 10 | 13 => { // capture-left
                        let nf = file - 1;
                        let nr = rank + forward;
                        if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                    }
                    2 | 5 | 8 | 11 | 14 => { // capture-right
                        let nf = file + 1;
                        let nr = rank + forward;
                        if nf >= 0 && nf < 8 && nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                    }
                    15 => { // double forward (starting rank only)
                        let nf = file;
                        let nr = rank + 2 * forward;
                        if nr >= 0 && nr < 8 { to_idx = Some(nr * 8 + nf); }
                    }
                    _ => {}
                }
            }
        }
        if let Some(idx) = to_idx { Ok(Square::new(idx as u32)) } else { Err(EnhancedDecodeError::InvalidMove("Target square out of bounds or unsupported move".to_string())) }
    }

    pub fn update_position(&mut self, decoded: &DecodedMove) -> std::result::Result<(), EnhancedDecodeError> {
        // Update SCID piece list based on decoded move indices
        if let Some(list) = self.current_piece_list.as_mut() {
            if let (Some(to_idx), Some(piece_num)) = (decoded.to_square_index, Some(decoded.piece_num)) {
                if piece_num < 16 {
                    list[piece_num as usize] = to_idx;
                } else {
                    return Err(EnhancedDecodeError::IndexOutOfBounds(piece_num));
                }
            }
        } else {
            return Err(EnhancedDecodeError::MissingPieceList);
        }
        // Position history tracking (placeholder)
        if let Some(pos) = &self.current_shakmaty_position {
            self.position_history.push(pos.clone());
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PgnTag {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GameFlags {
    pub non_standard_start: bool,
    pub has_promotions: bool,
    pub has_under_promotions: bool,
    pub raw_value: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MoveInterpretation {
    King {
        direction_code: u8,
        is_castle: bool,
    },
    Queen,
    Rook,
    Bishop,
    Knight {
        l_shape_code: u8,
    },
    Pawn {
        direction: String,
        promotion: Option<String>,
        is_en_passant: Option<bool>,
    },
    Decoded {
        from_square: Option<String>,
        to_square: Option<String>,
        piece_type: Option<String>,
        is_capture: bool,
        is_promotion: bool,
    },
    Unknown {
        reason: String,
    },
}

#[deprecated(note = "Use SG4Parser-based decoding with shakmaty position context")]
pub struct Sg4File {
    #[allow(dead_code)]
    mmap: Mmap,
}

// Movement decoder helpers available to both SG4Parser and legacy Sg4File
fn decode_king_move(move_value: u8) -> MoveInterpretation {
    let (direction_code, is_castle, _description) = match move_value {
        0 => (0, false, "King up"),
        1 => (1, false, "King up-right"),
        2 => (2, false, "King right"),
        3 => (3, true, "King castle kingside"),
        4 => (4, false, "King down-right"),
        5 => (5, false, "King down"),
        6 => (6, false, "King down-left"),
        7 => (7, true, "King castle queenside"),
        8 => (8, false, "King left"),
        9 => (9, false, "King up-left"),
        _ => (move_value, false, "Unknown king move"),
    };
    MoveInterpretation::King { direction_code, is_castle }
}

fn decode_queen_move(move_value: u8) -> MoveInterpretation {
    let _description = match move_value {
        0 => "Queen up",
        1 => "Queen up-right",
        2 => "Queen right",
        3 => "Queen down-right",
        4 => "Queen down",
        5 => "Queen down-left",
        6 => "Queen left",
        7 => "Queen up-left",
        _ => "Unknown queen move",
    };
    MoveInterpretation::Queen
}

fn decode_rook_move(move_value: u8) -> MoveInterpretation {
    let (_target_info, _description) = if move_value >= 8 {
        let rank = move_value - 8;
        (format!("rank {}", rank + 1), format!("Rook to rank {}", rank + 1))
    } else {
        let file = ('a' as u8 + move_value) as char;
        (format!("file {}", file), format!("Rook to file {}", file))
    };
    MoveInterpretation::Rook
}

fn decode_bishop_move(move_value: u8) -> MoveInterpretation {
    let file = move_value & 0x07;
    let direction_bit = (move_value >> 3) & 0x01;
    let _direction = if direction_bit == 0 { "up-left/down-right diagonal" } else { "up-right/down-left diagonal" };
    let _target_file = ('a' as u8 + file) as char;
    MoveInterpretation::Bishop
}

fn decode_knight_move(move_value: u8) -> MoveInterpretation {
    const KNIGHT_MOVES: &[i8] = &[-17, -15, -10, -6, 6, 10, 15, 17];
    let l_shape_code = if move_value < 8 { move_value } else { move_value - 8 };
    let _description = if l_shape_code < KNIGHT_MOVES.len() as u8 {
        format!("Knight L-shaped move pattern {}", l_shape_code + 1)
    } else {
        format!("Unknown knight move: {}", move_value)
    };
    MoveInterpretation::Knight { l_shape_code }
}

fn decode_pawn_move(move_value: u8) -> MoveInterpretation {
    let (direction, promotion, is_en_passant, _description) = match move_value {
        0 => ("forward", None, None, "Pawn forward 1 square"),
        1 => ("capture-left", None, None, "Pawn capture left"),
        2 => ("capture-right", None, None, "Pawn capture right"),
        3 => ("forward", Some("Queen".to_string()), None, "Pawn forward 1, promote to Queen"),
        4 => ("capture-left", Some("Queen".to_string()), None, "Pawn capture left, promote to Queen"),
        5 => ("capture-right", Some("Queen".to_string()), None, "Pawn capture right, promote to Queen"),
        6 => ("forward", Some("Rook".to_string()), None, "Pawn forward 1, promote to Rook"),
        7 => ("capture-left", Some("Rook".to_string()), None, "Pawn capture left, promote to Rook"),
        8 => ("capture-right", Some("Rook".to_string()), None, "Pawn capture right, promote to Rook"),
        9 => ("forward", Some("Bishop".to_string()), None, "Pawn forward 1, promote to Bishop"),
        10 => ("capture-left", Some("Bishop".to_string()), None, "Pawn capture left, promote to Bishop"),
        11 => ("capture-right", Some("Bishop".to_string()), None, "Pawn capture right, promote to Bishop"),
        12 => ("forward", Some("Knight".to_string()), None, "Pawn forward 1, promote to Knight"),
        13 => ("capture-left", Some("Knight".to_string()), None, "Pawn capture left, promote to Knight"),
        14 => ("capture-right", Some("Knight".to_string()), None, "Pawn capture right, promote to Knight"),
        15 => ("double-forward", None, Some(true), "Pawn double forward (en passant possible)"),
        _ => ("unknown", None, None, "Unknown pawn move"),
    };
    MoveInterpretation::Pawn { direction: direction.to_string(), promotion, is_en_passant }
}

impl Sg4File {
    #[deprecated(note = "Use SG4Parser-based decoding with shakmaty position context")]
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| ScidError::FileOpen(e, path.to_path_buf()))?;
        let mmap =
            unsafe { Mmap::map(&file) }.map_err(|e| ScidError::Mmap(e, path.to_path_buf()))?;
        Ok(Self { mmap })
    }
    
    #[allow(dead_code)]
    #[deprecated(note = "Use SG4Parser-based GameIterator for decoding; legacy iterator is obsolete")]
    pub fn iter_games(&self) -> GameIterator<'_> {
        GameIterator {
            sg4: self,
            current_offset: 0,
        }
    }
    
    #[allow(dead_code)]
    #[deprecated(note = "Use SG4Parser-driven parsing via GameIterator; get_game is obsolete")]
    pub fn get_game(&self, game_index: usize) -> Result<GameRecord> {
        let mut iterator = self.iter_games();
        
        for (i, result) in iterator.by_ref().enumerate() {
            if i == game_index {
                return result;
            }
        }
        
        Err(ScidError::invalid_format("Game index out of bounds"))
    }
    
    /// Get the number of games in the SG4 file
    #[allow(dead_code)]
    pub fn num_games(&self) -> usize {
        find_game_boundaries(&self.mmap).len()
    }
    
    /// Get the raw data for debugging
    #[allow(dead_code)]
    pub fn data(&self) -> &[u8] {
        &self.mmap
    }
    
    #[allow(dead_code)]
    fn decode_pawn_promotion(&self, move_value: u8) -> Option<String> {
        match move_value {
            8 => Some("Queen".to_string()),
            9 => Some("Rook".to_string()),
            10 => Some("Bishop".to_string()),
            11 => Some("Knight".to_string()),
            _ => None,
        }
    }
    
    #[cfg(debug_assertions)]
    fn log_move_interpretation(&self, byte: u8, piece_num: u8, move_value: u8, interpretation: &MoveInterpretation) {
        let piece_type_str = match interpretation {
            MoveInterpretation::King { .. } => "King",
            MoveInterpretation::Queen { .. } => "Queen",
            MoveInterpretation::Rook { .. } => "Rook",
            MoveInterpretation::Bishop { .. } => "Bishop",
            MoveInterpretation::Knight { .. } => "Knight",
            MoveInterpretation::Pawn { .. } => "Pawn",
            MoveInterpretation::Decoded { piece_type, .. } => piece_type.as_deref().unwrap_or("Unknown"),
            MoveInterpretation::Unknown { .. } => "Unknown",
        };
        
        eprintln!("[SG4] byte=0x{:02X}, piece_num={}, move_value={}, piece_type={}", 
                 byte, piece_num, move_value, piece_type_str);
    }
}

pub struct GameRecord {
    #[allow(dead_code)]
    pub tags: Vec<PgnTag>,
    #[allow(dead_code)]
    pub flags: GameFlags,
    #[allow(dead_code)]
    pub moves: Vec<DecodedMove>,
    #[allow(dead_code)]
    pub comments: Vec<String>,
    #[allow(dead_code)]
    pub nags: Vec<u8>,
    #[allow(dead_code)]
    pub result: Option<u8>,
    pub next_offset: usize,
}

#[deprecated(note = "Use SG4Parser-based decoding with shakmaty position context")]
pub struct GameIterator<'a> {
    sg4: &'a Sg4File,
    current_offset: usize,
}

impl<'a> Iterator for GameIterator<'a> {
    type Item = Result<GameRecord>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_offset >= self.sg4.mmap.len() {
            return None;
        }
        
        match self.parse_game_at_offset(self.current_offset) {
            Ok(game_record) => {
                self.current_offset = game_record.next_offset;
                Some(Ok(game_record))
            }
            Err(e) => {
                self.current_offset = self.sg4.mmap.len(); // Stop on error
                Some(Err(e))
            }
        }
    }
}

impl<'a> GameIterator<'a> {
    #[deprecated(note = "Legacy parsing path; SG4Parser-based pipeline should be used")]
    fn parse_game_at_offset(&self, offset: usize) -> Result<GameRecord> {
        let mut current_offset = offset;
        let mut tags = Vec::new();
        let mut moves = Vec::new();
        let mut comments = Vec::new();
        let mut nags = Vec::new();
        #[allow(unused_assignments)]
        let mut result = None;
        let mut flags = GameFlags {
            non_standard_start: false,
            has_promotions: false,
            has_under_promotions: false,
            raw_value: 0,
        };
        
        // Parse PGN tags first
        let _tags_end_offset = self.parse_pgn_tags(&mut current_offset, &mut tags, &mut flags)?;
        
        // Then parse moves and special bytes
        let mut parser = SG4Parser::new(self.sg4.mmap.to_vec());
        loop {
            if current_offset >= self.sg4.mmap.len() {
                return Err(ScidError::invalid_format("Game extends beyond file bounds"));
            }
            
            let byte = self.sg4.mmap[current_offset];
            
            match byte {
                ENCODE_END_GAME => {
                    if current_offset + 1 >= self.sg4.mmap.len() {
                        return Err(ScidError::invalid_format("Game end extends beyond file bounds"));
                    }
                    result = Some(self.sg4.mmap[current_offset + 1]);
                    current_offset += 2;
                    break;
                }
                ENCODE_NAG => {
                    if current_offset + 1 >= self.sg4.mmap.len() {
                        return Err(ScidError::invalid_format("NAG extends beyond file bounds"));
                    }
                    nags.push(self.sg4.mmap[current_offset + 1]);
                    current_offset += 2;
                }
                ENCODE_COMMENT => {
                    let comment = self.parse_string_at(&mut current_offset)?;
                    comments.push(comment);
                }
                ENCODE_START_MARKER => {
                    // Skip variation start marker
                    current_offset += 1;
                }
                ENCODE_END_MARKER => {
                    // Skip variation end marker
                    current_offset += 1;
                }
                _ => {
                    // Regular move via SG4Parser
                    parser.offset = current_offset;
                    let piece_num = (byte >> 4) & 0x0F;
                    let move_value = byte & 0x0F;
                    let decoded_move = if piece_num == 2 && move_value >= 8 {
                        let dm = parser.decode_queen_diagonal_start(byte).map_err(ScidError::from)?;
                        current_offset += 2;
                        dm
                    } else {
                        let dm = parser.decode_single_byte_move(byte).map_err(ScidError::from)?;
                        current_offset += 1;
                        parser.offset = current_offset;
                        dm
                    };
                    
                    // Check for promotions in flags
                    if let MoveInterpretation::Pawn { promotion: Some(promo), .. } = &decoded_move.interpretation {
                        flags.has_promotions = true;
                        if promo != "Queen" {
                            flags.has_under_promotions = true;
                        }
                    }
                    
                    parser.update_position(&decoded_move).map_err(ScidError::from)?;
                    moves.push(decoded_move);
                }
            }
        }
        
        Ok(GameRecord {
            tags,
            flags,
            moves,
            comments,
            nags,
            result,
            next_offset: current_offset,
        })
    }
    
    fn parse_pgn_tags(&self, offset: &mut usize, tags: &mut Vec<PgnTag>, _flags: &mut GameFlags) -> Result<usize> {
        let _start_offset = *offset;
        
        while *offset < self.sg4.mmap.len() {
            let byte = self.sg4.mmap[*offset];
            
            // Check if we've reached the end of tags (start of moves)
            if byte >= ENCODE_FIRST && byte <= ENCODE_LAST {
                break;
            }
            
            // Parse tag name and value
            if let Ok((name, value)) = self.parse_tag_at(offset) {
                tags.push(PgnTag { name, value });
            } else {
                break;
            }
        }
        
        Ok(*offset)
    }
    
    fn parse_tag_at(&self, offset: &mut usize) -> Result<(String, String)> {
        if *offset >= self.sg4.mmap.len() {
            return Err(ScidError::invalid_format("Tag extends beyond file bounds"));
        }
        
        let tag_byte = self.sg4.mmap[*offset];
        *offset += 1;
        
        let (name, value) = if tag_byte >= COMMON_TAG_THRESHOLD {
            // Common tag encoded as single byte
            let tag_index = (tag_byte - COMMON_TAG_THRESHOLD) as usize;
            if tag_index < COMMON_TAGS.len() {
                let name = COMMON_TAGS[tag_index].to_string();
                let value = self.parse_string_at(offset)?;
                (name, value)
            } else {
                return Err(ScidError::invalid_format("Invalid common tag byte"));
            }
        } else {
            // Regular tag name
            let name = self.parse_string_at(offset)?;
            let value = self.parse_string_at(offset)?;
            (name, value)
        };
        
        Ok((name, value))
    }
    
    fn parse_string_at(&self, offset: &mut usize) -> Result<String> {
        if *offset >= self.sg4.mmap.len() {
            return Err(ScidError::invalid_format("String extends beyond file bounds"));
        }
        
        let length = self.sg4.mmap[*offset] as usize;
        *offset += 1;
        
        if *offset + length > self.sg4.mmap.len() {
            return Err(ScidError::invalid_format("String extends beyond file bounds"));
        }
        
        let string_bytes = &self.sg4.mmap[*offset..*offset + length];
        *offset += length;
        
        Ok(String::from_utf8_lossy(string_bytes).trim_end_matches('\0').to_string())
    }
    
    #[deprecated(note = "Use SG4Parser-based decoding; this legacy function is obsolete")]
    fn decode_move_at(&self, offset: &mut usize) -> Result<DecodedMove> {
        if *offset >= self.sg4.mmap.len() {
            return Err(ScidError::invalid_format("Move extends beyond file bounds"));
        }
        
        // Legacy path deprecated: use GameIterator with SG4Parser for decoding.
        // This function now simply returns an error to prevent per-move parser instantiation.
        Err(ScidError::invalid_format("decode_move_at is deprecated; use GameIterator parser-driven decoding"))
    }
    
    fn decode_single_byte_move(&self, byte: u8, piece_num: u8, move_value: u8, _offset: &mut usize) -> Result<DecodedMove> {
        let interpretation = match piece_num {
            1 => self.decode_king_move(move_value),
            2 => self.decode_queen_move(move_value),
            3 => self.decode_rook_move(move_value),
            4 => self.decode_bishop_move(move_value),
            5 => {
                eprintln!("DEBUG: SG4 piece_num=5 routing to decode_knight_move, move_value={}", move_value);
                eprintln!("DEBUG: Algebraic notation: Knight move {}", move_value);
                self.decode_knight_move(move_value)
            },
            6 => {
                eprintln!("DEBUG: SG4 piece_num=6 routing to decode_pawn_move, move_value={}", move_value);
                // Special case: move_value 12 for pawn is promotion (like 0x6C)
                let move_desc = if move_value == 12 {
                    "Pawn promotion (move_value 12)"
                } else {
                    &format!("Pawn move {}", move_value)
                };
                eprintln!("DEBUG: Algebraic notation: {}", move_desc);
                self.decode_pawn_move(move_value)
            },
            _ => MoveInterpretation::Unknown {
                reason: format!("Invalid piece number: {}", piece_num),
            },
        };
        
        #[cfg(debug_assertions)]
        self.sg4.log_move_interpretation(byte, piece_num, move_value, &interpretation);
        
        // Preserve piece type for correct routing (FIX: extract from interpretation)
        let piece_type = match &interpretation {
            MoveInterpretation::King { .. } => Some(Role::King),
            MoveInterpretation::Queen { .. } => Some(Role::Queen),
            MoveInterpretation::Rook { .. } => Some(Role::Rook),
            MoveInterpretation::Bishop { .. } => Some(Role::Bishop),
            MoveInterpretation::Knight { .. } => Some(Role::Knight),
            MoveInterpretation::Pawn { .. } => Some(Role::Pawn),
            MoveInterpretation::Decoded { .. } => None, // This case has separate piece_type field
            MoveInterpretation::Unknown { .. } => None,
        };
        
        // VALIDATION: Check consistency between piece_num and interpretation
        if let (Some(ptype), Some(expected_role)) = (piece_type.as_ref(), Self::piece_num_to_role(piece_num)) {
            if *ptype != expected_role {
                eprintln!("DEBUG: piece_num/interpretation mismatch - piece_num={} (expected {:?}), interpretation={:?}", 
                         piece_num, expected_role, ptype);
            }
        }
        
        // VALIDATION: Check move_value is within valid range for piece type
        if let (Some(ptype), Some(valid_range)) = (piece_type.as_ref(), Self::valid_move_range(piece_num)) {
            if move_value > valid_range {
                eprintln!("DEBUG: move_value {} exceeds valid range {} for piece_type {:?} (piece_num={})", 
                         move_value, valid_range, ptype, piece_num);
            }
        }
        
        Ok(DecodedMove {
            raw_bytes: vec![byte],
            piece_num,
            move_value,
            interpretation,
            from_square_index: None,
            to_square_index: None,
            promotion_piece: None,
            piece_type,  // Preserved for correct routing
        })
    }
    
    /// Helper function to map SCID piece numbers to roles for validation
    /// Maps move encoding numbers (1-6) to piece types
    /// This is the SG4 piece numbering system, not the board numbering system
    #[cfg(debug_assertions)]
    fn piece_num_to_role(piece_num: u8) -> Option<Role> {
        match piece_num {
            1 => Some(Role::King),
            2 => Some(Role::Queen), 
            3 => Some(Role::Rook),
            4 => Some(Role::Bishop),
            5 => Some(Role::Knight),
            6 => Some(Role::Pawn),
            _ => None, // Invalid piece numbers
        }
    }
    
    /// Helper function to get valid move value range for piece types
    /// Returns maximum valid move value for each piece type
    /// This helps detect dual numbering system conflicts
    #[cfg(debug_assertions)]
    fn valid_move_range(piece_num: u8) -> Option<u8> {
        match piece_num {
            1..=6 => Some(15), // Standard pieces: 0-15
            _ => None, // Invalid piece numbers
        }
    }
    
    fn decode_king_move(&self, move_value: u8) -> MoveInterpretation {
        decode_king_move(move_value)
    }
    
    fn decode_queen_move(&self, move_value: u8) -> MoveInterpretation {
        decode_queen_move(move_value)
    }
    
    fn decode_queen_diagonal_move(&self, first_byte: u8, second_byte: u8, _offset: &mut usize) -> Result<DecodedMove> {
        let piece_num = (first_byte >> 4) & 0x0F;
        let move_value = first_byte & 0x0F;
        let diagonal_info = second_byte & 0x0F;
        
        let interpretation = MoveInterpretation::Queen;
        
        #[cfg(debug_assertions)]
        self.sg4.log_move_interpretation(first_byte, piece_num, move_value, &interpretation);
        
        // Preserve piece type for correct routing (FIX: extract from interpretation)
        let piece_type = Some(Role::Queen);
        
        Ok(DecodedMove {
            raw_bytes: vec![first_byte, second_byte],
            piece_num,
            move_value,
            interpretation,
            from_square_index: None,
            to_square_index: Some(diagonal_info),
            promotion_piece: None,
            piece_type,  // Preserved for correct routing
        })
    }
    
    fn decode_rook_move(&self, move_value: u8) -> MoveInterpretation {
        decode_rook_move(move_value)
    }
    
    fn decode_bishop_move(&self, move_value: u8) -> MoveInterpretation {
        decode_bishop_move(move_value)
    }
    
    fn decode_knight_move(&self, move_value: u8) -> MoveInterpretation {
        decode_knight_move(move_value)
    }
    
    fn decode_pawn_move(&self, move_value: u8) -> MoveInterpretation {
        decode_pawn_move(move_value)
    }
}

// Constants from SCID source code
const ENCODE_NAG: u8 = 11;
const ENCODE_COMMENT: u8 = 12;
const ENCODE_START_MARKER: u8 = 13;
const ENCODE_END_MARKER: u8 = 14;
const ENCODE_END_GAME: u8 = 15;

// Block size from SCID source code
#[allow(dead_code)]
const BLOCK_SIZE: usize = 131072;

// Maximum tag length from SCID source code
const MAX_TAG_LEN: u8 = 240;

// Common tags encoding threshold - values 241+ are common tags
const COMMON_TAG_THRESHOLD: u8 = MAX_TAG_LEN + 1;

// Common PGN tag names from SCID source code
const COMMON_TAGS: &[&str] = &[
    "WhiteCountry",    // 241
    "BlackCountry",    // 242  
    "Annotator",       // 243
    "PlyCount",        // 244
    "EventDate",       // 245
    "Opening",         // 246
    "Variation",       // 247
    "SubVariation",    // 248
    "ECO",             // 249
    "WhiteTitle",      // 250
    "BlackTitle",      // 251
    "WhiteElo",        // 252
    "BlackElo",        // 253
    "WhiteFideId",     // 254
    "BlackFideId",     // 255
];

const ENCODE_FIRST: u8 = 11;
const ENCODE_LAST: u8 = 15;

#[allow(dead_code)]
pub struct NagProcessor;

impl NagProcessor {
    /// Process NAG (Numeric Annotation Glyph) values
    /// Based on SCID source code for annotation handling
    #[allow(dead_code)]
    pub fn process_nag(nag_value: u8) -> Option<String> {
        // Standard NAG values from SCID source code
        match nag_value {
            1 => Some("Good move".to_string()),
            2 => Some("Poor move".to_string()),
            3 => Some("Very good move".to_string()),
            4 => Some("Very poor move".to_string()),
            5 => Some("Speculative move".to_string()),
            6 => Some("Dubious move".to_string()),
            7 => Some("Forced move".to_string()),
            8 => Some("Singular move".to_string()),
            9 => Some("Worst move".to_string()),
            10 => Some("Drawish position".to_string()),
            11 => Some("Equal chances, quiet position".to_string()),
            12 => Some("Equal chances, active position".to_string()),
            13 => Some("Unclear position".to_string()),
            14 => Some("White has a slight advantage".to_string()),
            15 => Some("Black has a slight advantage".to_string()),
            16 => Some("White has a moderate advantage".to_string()),
            17 => Some("Black has a moderate advantage".to_string()),
            18 => Some("White has a decisive advantage".to_string()),
            19 => Some("Black has a decisive advantage".to_string()),
            22 => Some("Zugzwang".to_string()),
            23 => Some("Zwischenzug".to_string()),
            32 => Some("Development advantage".to_string()),
            33 => Some("Initiative".to_string()),
            34 => Some("Attack".to_string()),
            35 => Some("Counterattack".to_string()),
            36 => Some("Time pressure".to_string()),
            40 => Some("With attack".to_string()),
            41 => Some("Without attack".to_string()),
            44 => Some("Compensation".to_string()),
            45 => Some("Counterplay".to_string()),
            132 => Some("Position is drawn".to_string()),
            133 => Some("Position is equal".to_string()),
            134 => Some("Unclear".to_string()),
            146 => Some("White is slightly better".to_string()),
            147 => Some("Black is slightly better".to_string()),
            148 => Some("White is better".to_string()),
            149 => Some("Black is better".to_string()),
            150 => Some("White is much better".to_string()),
            151 => Some("Black is much better".to_string()),
            156 => Some("White has winning advantage".to_string()),
            157 => Some("Black has winning advantage".to_string()),
            _ => None,
        }
    }
}

/// Find game boundaries in SG4 file data
/// Returns vector of (start_offset, end_offset) tuples for each game
pub fn find_game_boundaries(data: &[u8]) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::new();
    let mut current_start = 0;
    
    for i in 0..data.len() {
        if data[i] == ENCODE_END_GAME && i + 1 < data.len() {
            // Found end of game, mark the boundary
            boundaries.push((current_start, i + 2));
            current_start = i + 2;
        }
    }
    
    // If we have data left and no end marker, add it as a final game
    if current_start < data.len() && boundaries.is_empty() {
        boundaries.push((current_start, data.len()));
    }
    
    boundaries
}

/// Parse PGN tags and game elements from SG4 data
/// Legacy function for backward compatibility
#[allow(dead_code)]
pub fn parse_pgn_tags(data: &[u8]) -> Result<GameParseState> {
    let mut elements = Vec::new();
    let mut tags = Vec::new();
    let mut current_offset = 0;
    
    // Parse tags first (simplified - in real SG4 files, tags are encoded)
    while current_offset < data.len() && data[current_offset] < ENCODE_FIRST {
        if let Ok((name, value)) = parse_simple_tag(&data[current_offset..], &mut current_offset) {
            tags.push(PgnTag { name, value });
        } else {
            break;
        }
    }
    
    let tags_end_offset = current_offset;
    
    // Parse moves and special elements
    while current_offset < data.len() {
        let byte = data[current_offset];
        
        match byte {
            ENCODE_END_GAME => {
                if current_offset + 1 < data.len() {
                    elements.push(GameElement::GameEnd {
                        result: data[current_offset + 1],
                        offset: current_offset,
                    });
                    let _ = current_offset + 2;
                } else {
                    let _ = current_offset + 1;
                }
                break;
            }
            ENCODE_NAG => {
                if current_offset + 1 < data.len() {
                    elements.push(GameElement::Nag {
                        nag_value: data[current_offset + 1],
                        offset: current_offset,
                    });
                    current_offset += 2;
                } else {
                    current_offset += 1;
                }
            }
            ENCODE_COMMENT => {
                if let Ok(comment) = parse_simple_string(&data[current_offset..], &mut current_offset) {
                    elements.push(GameElement::Comment {
                        text: comment,
                        offset: current_offset,
                    });
                } else {
                    current_offset += 1;
                }
            }
            ENCODE_START_MARKER => {
                elements.push(GameElement::VariationStart { offset: current_offset });
                current_offset += 1;
            }
            ENCODE_END_MARKER => {
                elements.push(GameElement::VariationEnd { offset: current_offset });
                current_offset += 1;
            }
            _ => {
                // Regular move
                let piece_num = (byte >> 4) & 0x0F;
                let move_value = byte & 0x0F;
                elements.push(GameElement::Move {
                    raw_byte: byte,
                    piece_num,
                    move_value,
                    offset: current_offset,
                });
                current_offset += 1;
            }
        }
    }
    
    Ok(GameParseState {
        tags,
        flags: GameFlags {
            non_standard_start: false,
            has_promotions: false,
            has_under_promotions: false,
            raw_value: 0,
        },
        elements,
        tags_end_offset,
        flags_offset: 0,
        moves_start_offset: tags_end_offset,
    })
}

/// Game element for legacy parsing
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GameElement {
    Move {
        #[allow(dead_code)]
        raw_byte: u8,
        #[allow(dead_code)]
        piece_num: u8,
        #[allow(dead_code)]
        move_value: u8,
        #[allow(dead_code)]
        offset: usize,
    },
    Nag {
        #[allow(dead_code)]
        nag_value: u8,
        #[allow(dead_code)]
        offset: usize,
    },
    Comment {
        #[allow(dead_code)]
        text: String,
        #[allow(dead_code)]
        offset: usize,
    },
    VariationStart {
        #[allow(dead_code)]
        offset: usize,
    },
    VariationEnd {
        #[allow(dead_code)]
        offset: usize,
    },
    GameEnd {
        #[allow(dead_code)]
        result: u8,
        #[allow(dead_code)]
        offset: usize,
    },
}

/// Game parse state for legacy compatibility
#[derive(Debug)]
#[allow(dead_code)]
pub struct GameParseState {
    #[allow(dead_code)]
    pub tags: Vec<PgnTag>,
    #[allow(dead_code)]
    pub flags: GameFlags,
    #[allow(dead_code)]
    pub elements: Vec<GameElement>,
    #[allow(dead_code)]
    pub tags_end_offset: usize,
    #[allow(dead_code)]
    pub flags_offset: usize,
    #[allow(dead_code)]
    pub moves_start_offset: usize,
}

/// Parse a simple tag from byte data (simplified implementation)
fn parse_simple_tag(data: &[u8], offset: &mut usize) -> Result<(String, String)> {
    if data.is_empty() {
        return Err(ScidError::invalid_format("Empty tag data"));
    }
    
    // Very simplified tag parsing - in real SG4 this is more complex
    let name = format!("Tag{}", *offset);
    let value = format!("Value{}", *offset);
    *offset += 1;
    
    Ok((name, value))
}

/// Parse PGN tags with streaming support for variable-length moves
/// Enhanced version that supports the new streaming game elements
#[allow(dead_code)]
pub fn parse_pgn_tags_with_streaming(data: &[u8]) -> Result<Vec<StreamingGameElement>> {
    let mut elements = Vec::new();
    let mut current_offset = 0;
    
    while current_offset < data.len() {
        let byte = data[current_offset];
        let _start_offset = current_offset;
        
        match byte {
            ENCODE_END_GAME => {
                if current_offset + 1 < data.len() {
                    elements.push(StreamingGameElement::GameEnd { result: data[current_offset + 1] });
                    let _ = current_offset + 2;
                } else {
                    let _ = current_offset + 1;
                }
                break;
            }
            ENCODE_NAG => {
                if current_offset + 1 < data.len() {
                    elements.push(StreamingGameElement::Nag {
                        nag_code: data[current_offset + 1],
                    });
                    current_offset += 2;
                } else {
                    current_offset += 1;
                }
            }
            ENCODE_COMMENT => {
                if let Ok(comment) = parse_simple_string(&data[current_offset..], &mut current_offset) {
                    elements.push(StreamingGameElement::Comment {
                        text: comment,
                    });
                } else {
                    current_offset += 1;
                }
            }
            ENCODE_START_MARKER => {
                elements.push(StreamingGameElement::VariationStart);
                current_offset += 1;
            }
            ENCODE_END_MARKER => {
                elements.push(StreamingGameElement::VariationEnd);
                current_offset += 1;
            }
            _ => {
                // Regular move - check for multi-byte queen moves
                let piece_num = (byte >> 4) & 0x0F;
                let move_value = byte & 0x0F;
                
                if piece_num == 2 && move_value >= 8 && current_offset + 1 < data.len() {
                    // Queen diagonal move - 2 bytes
                    elements.push(StreamingGameElement::Move {
                        raw: vec![byte, data[current_offset + 1]],
                    });
                    current_offset += 2;
                } else {
                    // Single byte move
                    elements.push(StreamingGameElement::Move {
                        raw: vec![byte],
                    });
                    current_offset += 1;
                }
            }
        }
    }
    
    Ok(elements)
}

/// Parse a simple string from byte data
fn parse_simple_string(data: &[u8], offset: &mut usize) -> Result<String> {
    if data.is_empty() {
        return Err(ScidError::invalid_format("Empty string data"));
    }
    
    let length = data[0] as usize;
    if length + 1 > data.len() {
        return Err(ScidError::invalid_format("String extends beyond data bounds"));
    }
    
    let string_data = &data[1..length + 1];
    *offset += length + 1;
    
    Ok(String::from_utf8_lossy(string_data).trim_end_matches('\0').to_string())
}

    #[test]
    fn test_decode_pawn_moves() {
        // Test pawn forward move
        let raw_byte = 0x60; // Pawn (6) + forward (0)
        let piece_num = (raw_byte >> 4) & 0x0F;
        let move_value = raw_byte & 0x0F;
        
        assert_eq!(piece_num, 6);
        assert_eq!(move_value, 0);
    }
    
    
    #[test]
    fn test_constants() {
        assert_eq!(ENCODE_NAG, 11);
        assert_eq!(ENCODE_COMMENT, 12);
        assert_eq!(ENCODE_START_MARKER, 13);
        assert_eq!(ENCODE_END_MARKER, 14);
        assert_eq!(ENCODE_END_GAME, 15);
        assert_eq!(BLOCK_SIZE, 131072);
    }
    
    #[test]
    fn test_find_game_boundaries() {
        let data = vec![
            0x10, 0x20, 0x30, ENCODE_END_GAME, 0x01, // Game 1
            0x40, 0x50, ENCODE_END_GAME, 0x02,             // Game 2
        ];
        
        let boundaries = find_game_boundaries(&data);
        assert_eq!(boundaries.len(), 2);
        assert_eq!(boundaries[0], (0, 5));
        assert_eq!(boundaries[1], (5, 9));
    }
    
    #[test]
    fn test_pawn_promotion_decoding() {
        // Test various pawn promotion scenarios
        let test_cases = vec![
            (0x63, Some("Queen")),   // Pawn + promote to Queen
            (0x64, Some("Rook")),    // Pawn + promote to Rook
            (0x65, Some("Bishop")),  // Pawn + promote to Bishop
            (0x66, Some("Knight")),  // Pawn + promote to Knight
            (0x60, None),            // Pawn + forward (no promotion)
        ];
        
        for (raw_byte, expected_promotion) in test_cases {
            let piece_num = (raw_byte >> 4) & 0x0F;
            let move_value = raw_byte & 0x0F;
            
            assert_eq!(piece_num, 6, "Should be pawn piece");
            
            if let MoveInterpretation::Pawn { promotion, .. } = decode_pawn_move_for_test(move_value) {
                assert_eq!(promotion, expected_promotion.map(|s| s.to_string()));
            } else {
                panic!("Expected pawn move interpretation");
            }
        }
    }
    
    // Helper function for testing pawn moves
    fn decode_pawn_move_for_test(move_value: u8) -> MoveInterpretation {
        match move_value {
            0 => MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: None,
                is_en_passant: None,
            },
            3 => MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: Some("Queen".to_string()),
                is_en_passant: None,
            },
            6 => MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: Some("Rook".to_string()),
                is_en_passant: None,
            },
            9 => MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: Some("Bishop".to_string()),
                is_en_passant: None,
            },
            12 => MoveInterpretation::Pawn {
                direction: "forward".to_string(),
                promotion: Some("Knight".to_string()),
                is_en_passant: None,
            },
            15 => MoveInterpretation::Pawn {
                direction: "double-forward".to_string(),
                promotion: None,
                is_en_passant: Some(true),
            },
            _ => MoveInterpretation::Unknown {
                reason: format!("Unknown pawn move: {}", move_value),
            },
        }
    }

impl NagProcessor {
    #[allow(dead_code)]
    pub fn is_symbol_nag(nag: u8) -> bool {
        matches!(nag, 1..=6 | 10..=21)
    }

    #[allow(dead_code)]
    pub fn nag_to_symbol(nag: u8) -> &'static str {
        match nag {
            1 => "!",   // Good move
            2 => "?",   // Poor move
            3 => "!!",  // Excellent move
            4 => "??",  // Blunder
            5 => "!?",  // Interesting move
            6 => "?!",  // Questionable move
            10 => "=",  // Equal position
            13 => "∞",  // Unclear position
            14 => "⩲",  // White is slightly better
            15 => "⩱",  // Black is slightly better
            16 => "±",  // White is better
            17 => "∓",  // Black is better
            18 => "+-", // White is winning
            19 => "-+", // Black is winning
            _ => "",
        }
    }

    #[allow(dead_code)]
    pub fn nag_to_description(nag: u8) -> Option<&'static str> {
        match nag {
            1 => Some("Good move"),
            2 => Some("Poor move"),
            3 => Some("Excellent move"),
            4 => Some("Blunder"),
            5 => Some("Interesting move"),
            6 => Some("Questionable move"),
            10 => Some("Equal position"),
            13 => Some("Unclear position"),
            14 => Some("White is slightly better"),
            15 => Some("Black is slightly better"),
            16 => Some("White is better"),
            17 => Some("Black is better"),
            18 => Some("White is winning"),
            19 => Some("Black is winning"),
            30 => Some("Initiative"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum StreamingGameElement {
    Move { raw: Vec<u8> },
    Comment { text: String },
    Nag { nag_code: u8 },
    VariationStart,
    VariationEnd,
    GameEnd { result: u8 },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StreamingGameParseState {
    pub elements: Vec<StreamingGameElement>,
}

pub fn parse_streaming_state(data: &[u8]) -> Result<StreamingGameParseState> {
    let elements = parse_pgn_tags_with_streaming(data)?;
    Ok(StreamingGameParseState { elements })
}

impl StreamingGameParseState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(ENCODE_NAG, 11);
        assert_eq!(ENCODE_COMMENT, 12);
        assert_eq!(ENCODE_START_MARKER, 13);
        assert_eq!(ENCODE_END_MARKER, 14);
        assert_eq!(ENCODE_END_GAME, 15);
    }
}
