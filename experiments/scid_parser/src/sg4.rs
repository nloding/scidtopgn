// ===============================
// Step 9: SCID Move Decoding Documentation for Shakmaty Migration
// This file contains the core logic for parsing SCID .sg4 game files and decoding moves.
// The following documentation outlines how each field and structure in this file will be mapped to shakmaty types:
//
// - DecodedMove: Represents a parsed SCID move. Fields:
//     - piece_num: Maps to shakmaty::Role (King, Queen, Rook, Bishop, Knight, Pawn)
//     - move_value: Encodes target square, promotion, castling, etc. Used to determine shakmaty::Move variant
//     - raw_byte: For debugging, not used in conversion
//     - interpretation: Human-readable and conversion mapping info
// - MoveInterpretation: Used to describe how a move should be converted to shakmaty::Move
// - GameFlags: Used for game-level metadata, e.g., custom start position, promotions
// - Parsing logic: Will be used by bridge/moves.rs to implement ScidToShakmaty trait
// - All square encodings (0-63) will be mapped to shakmaty::Square
// - Promotion types will be mapped to shakmaty::Role
// - Special moves (castling, en passant) will be mapped to appropriate shakmaty::Move variants
// - Move validation will be performed using shakmaty position context
// ===============================
// use crate::utils::*;  // Commented out - unused import
use crate::bridge::{GameState, PositionContext};
use crate::position::{ScidByteStream, decode_move_with_stream, ScidPosition};
use shakmaty::Move as ShakmMove;
// use std::fs::File;  // Commented out - unused import
// use std::io::Read;  // Commented out - unused import

/// SG4 Game File Structure Analysis
/// Based on analysis of scidvspc/src/gfile.cpp, game.cpp, and bytebuf.cpp
/// 
/// CRITICAL: All multi-byte values use BIG-ENDIAN byte order
/// CRITICAL: NO fixed header - games are stored as variable-length records
/// 
/// Block Structure:
/// - 131,072-byte blocks (GF_BLOCKSIZE = 131072)
/// - Games can span blocks but most fit within single blocks
/// - Each game is a variable-length encoded sequence
///
/// Game Record Format (variable length):
/// 1. Non-standard PGN tags (if any) - variable length strings
/// 2. Game moves - encoded using piece-specific 1-3 byte encodings
/// 3. Special encoding bytes for annotations/variations:
///    - ENCODE_NAG (11) + NAG value
///    - ENCODE_COMMENT (12) + variable-length comment
///    - ENCODE_START_MARKER (13) - begin variation
///    - ENCODE_END_MARKER (14) - end variation  
///    - ENCODE_END_GAME (15) - end of game record
///
/// Move Encoding (1-3 bytes per move):
/// - Byte 1: Piece number (4 bits) + piece-specific encoding (4 bits)
/// - King: 1 byte - direction/castling encoded in 4 bits
/// - Queen: 1-2 bytes - rooklike moves in 1 byte, diagonal in 2 bytes
/// - Rook: 1 byte - target rank/file encoded in 4 bits
/// - Bishop: 1 byte - target file + direction bit in 4 bits
/// - Knight: 1 byte - target square difference encoded in 4 bits  
/// - Pawn: 1 byte - direction + promotion encoded in 4 bits

/// Constants from SCID source code (game.cpp)
#[allow(dead_code)]
const ENCODE_NAG: u8 = 11;
#[allow(dead_code)]
const ENCODE_COMMENT: u8 = 12;
#[allow(dead_code)]
const ENCODE_START_MARKER: u8 = 13;
#[allow(dead_code)]
const ENCODE_END_MARKER: u8 = 14;
#[allow(dead_code)]
const ENCODE_END_GAME: u8 = 15;

#[allow(dead_code)]
const ENCODE_FIRST: u8 = 11;
#[allow(dead_code)]
const ENCODE_LAST: u8 = 15;

/// Block size from SCID source code (gfile.h: GF_BLOCKSIZE)
#[allow(dead_code)]
const BLOCK_SIZE: usize = 131072;

/// Maximum tag length from SCID source code (game.h: MAX_TAG_LEN)
const MAX_TAG_LEN: u8 = 240;

/// Common tags encoding threshold - values 241+ are common tags
const COMMON_TAG_THRESHOLD: u8 = MAX_TAG_LEN + 1;

/// Common PGN tag names from SCID source code (game.cpp: commonTags array)
/// These are encoded as single bytes 241-255 instead of full strings
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

/// Parsed PGN tag
#[derive(Debug, Clone)]
pub struct PgnTag {
    pub name: String,
    pub value: String,
}

/// Game flags from SCID source code (game.cpp)
/// Reference: Game::Encode() and Game::Decode() functions
#[derive(Debug, Clone)]
pub struct GameFlags {
    pub non_standard_start: bool,  // Bit 0: Game has custom starting position (FEN)
    pub has_promotions: bool,      // Bit 1: Game contains pawn promotions
    pub has_under_promotions: bool, // Bit 2: Game has under-promotions (not to Queen)
    pub raw_value: u8,
}

/// Chess square representation (0-63: a1=0, b1=1, ..., h8=63)
// type Square = u8;  // Disabled to avoid conflict with position::Square

/// Basic move information decoded from SCID binary format
/// 
/// This struct represents a fully decoded SCID move that contains all the information
/// needed for conversion to shakmaty chess moves. The piece_num identifies which piece
/// is moving (0-15 for pieces on the board), and the move_value contains the piece-specific
/// encoding for the target square or move type.
/// 
/// SHAKMATY CONVERSION PROCESS:
/// 1. Extract piece role from piece_num (King=0-1, Queen=2-3, Rook=4-7, Bishop=8-11, Knight=12-15, Pawn=varies)
/// 2. Get current piece position from chess position using piece tracking
/// 3. Use move_value + interpretation to calculate target square
/// 4. Detect captures by checking if target square contains opponent piece
/// 5. Create appropriate shakmaty::Move variant (Normal, Castle, EnPassant)
/// 
/// CONVERSION REQUIREMENTS:
/// - Position context: Needed to find piece locations and detect captures
/// - Piece tracking: Map piece_num to current board squares
/// - Move validation: Ensure generated moves are legal in current position
#[derive(Debug, Clone)]
pub struct DecodedMove {
    /// SCID piece number (0-15): identifies which specific piece is moving
    /// Used to locate the piece on the current board position
    pub piece_num: u8,
    
    /// SCID move value (0-15): piece-specific encoding of target square/move type
    /// Combined with piece type to determine exact move destination
    pub move_value: u8,
    
    /// Original raw byte from SCID file: piece_num (high 4 bits) + move_value (low 4 bits)
    /// Preserved for debugging and validation purposes
    pub raw_byte: u8,
    
    /// Interpreted move information based on piece type and structured move data
    /// Contains human-readable description and shakmaty conversion mapping information
    pub interpretation: MoveInterpretation,
}

/// Move interpretation based on SCID source code analysis
/// 
/// This enum represents the current understanding of SCID move encoding and will be
/// used as the source data for conversion to shakmaty chess moves. Each variant
/// contains the information needed to reconstruct the actual chess move.
/// 
/// SHAKMATY CONVERSION MAPPING:
/// - King moves map to shakmaty::Move::Normal or shakmaty::Move::Castle
/// - Castling (codes 9,10) maps to shakmaty::Move::Castle with king/rook squares
/// - All other pieces map to shakmaty::Move::Normal with from/to squares
/// - Pawn promotions map to shakmaty::Move::Normal with promotion field
/// - En passant captures map to shakmaty::Move::EnPassant
#[derive(Debug, Clone)]
pub enum MoveInterpretation {
    /// King moves: directions 0-8 are normal moves, 9-10 are castling
    /// SHAKMATY MAPPING: 
    /// - Normal moves → Move::Normal{role: King, from, to, capture, promotion: None}
    /// - Castling → Move::Castle{king: from_square, rook: rook_square}
    King {
        direction_code: u8,  // 0-10: directions and castling
        description: String, // Human-readable description
    },
    /// Queen moves: can be rook-like (rank/file) or diagonal
    /// SHAKMATY MAPPING: Move::Normal{role: Queen, from, to, capture, promotion: None}
    /// - Requires position context to determine exact target square
    /// - May capture opponent pieces (detected from position)
    Queen {
        move_type: String,   // "rook-like" or "diagonal"
        description: String, // Human-readable description
    },
    /// Rook moves: strictly rank or file movement
    /// SHAKMATY MAPPING: Move::Normal{role: Rook, from, to, capture, promotion: None}
    /// - move_value >= 8: vertical move to rank (move_value - 8)
    /// - move_value < 8: horizontal move to file (move_value)
    Rook {
        target_info: String, // File or rank target
        description: String, // Human-readable description
    },
    /// Bishop moves: diagonal movement only
    /// SHAKMATY MAPPING: Move::Normal{role: Bishop, from, to, capture, promotion: None}
    /// - Target file encoded in lower 3 bits
    /// - Direction (up-left/down-right vs up-right/down-left) in bit 3
    Bishop {
        direction: String,   // Direction of diagonal move
        description: String, // Human-readable description
    },
    /// Knight moves: L-shaped patterns
    /// SHAKMATY MAPPING: Move::Normal{role: Knight, from, to, capture, promotion: None}
    /// - Uses lookup table of square differences: [-17,-15,-10,-6,6,10,15,17]
    /// - Extended codes 9-15 for edge cases and special positions
    Knight {
        l_shape_code: u8,    // 1-8: L-shaped move patterns
        description: String, // Human-readable description
    },
    /// Pawn moves: forward movement, captures, and promotions
    /// SHAKMATY MAPPING: 
    /// - Normal move → Move::Normal{role: Pawn, from, to, capture, promotion}
    /// - En passant → Move::EnPassant{from, to}
    /// - Promotion specified in promotion field
    Pawn {
        direction: String,   // Forward/capture direction
        promotion: Option<String>, // Promotion piece if any
        description: String, // Human-readable description
        is_en_passant: Option<bool>, // True if this is an en passant capture
    },
    /// Position-aware decoded move with complete information
    /// SHAKMATY MAPPING: Move::Normal{role, from, to, capture, promotion}
    /// This variant is used by the stream-aware decoder for multi-byte moves
    Decoded {
        description: String,           // Human-readable move description
        from_square: Option<String>,   // Source square in algebraic notation
        to_square: Option<String>,     // Target square in algebraic notation  
        piece_type: Option<String>,    // Moving piece type
        is_capture: bool,              // True if this move captures a piece
        is_promotion: bool,            // True if this is a pawn promotion
        bytes_consumed: usize,         // Number of bytes consumed from stream
    },
    /// Unknown move encoding - conversion will fail with error
    /// SHAKMATY MAPPING: Returns ConversionError
    Unknown {
        reason: String,      // Why we couldn't decode it
    },
}

impl MoveInterpretation {
    /// Get the description string for this move interpretation
    pub fn description(&self) -> &str {
        match self {
            MoveInterpretation::King { description, .. } => description,
            MoveInterpretation::Queen { description, .. } => description,
            MoveInterpretation::Rook { description, .. } => description,
            MoveInterpretation::Bishop { description, .. } => description,
            MoveInterpretation::Knight { description, .. } => description,
            MoveInterpretation::Pawn { description, .. } => description,
            MoveInterpretation::Decoded { description, .. } => description,
            MoveInterpretation::Unknown { reason } => reason,
        }
    }
}

/// Move/annotation data element from SCID source analysis
#[derive(Debug, Clone)]
pub enum GameElement {
    Move {
        piece_num: u8,     // Bits 4-7: piece number (0-15)
        move_value: u8,    // Bits 0-3: piece-specific move encoding
        raw_byte: u8,      // Original byte value
        offset: usize,     // File offset
        decoded: Option<DecodedMove>, // Decoded move information
    },
    Nag {
        nag_value: u8,     // NAG annotation value
        offset: usize,     // File offset of NAG marker
    },
    Comment {
        text: String,      // Comment text (placeholder - not implemented yet)
        offset: usize,     // File offset of comment marker
    },
    VariationStart {
        offset: usize,     // File offset of variation start marker
    },
    VariationEnd {
        offset: usize,     // File offset of variation end marker
    },
    GameEnd {
        offset: usize,     // File offset of game end marker
    },
}

/// Variation tree structure for complex game analysis
/// Based on SCID's variation handling approach from game.cpp
#[derive(Debug, Clone)]
pub struct VariationTree {
    pub main_line: Vec<GameNode>,
    pub current_depth: usize,
    pub variation_stack: Vec<Vec<GameNode>>, // Stack for nested variations
}

/// Individual node in the game tree
#[derive(Debug, Clone)]
pub struct GameNode {
    pub element: GameElement,
    pub variations: Vec<VariationTree>,
    pub parent: Option<usize>,
    pub move_number: Option<usize>,
}

impl VariationTree {
    pub fn new() -> Self {
        VariationTree {
            main_line: Vec::new(),
            current_depth: 0,
            variation_stack: Vec::new(),
        }
    }
    
    /// Add a move to the current line (main line or variation)
    pub fn add_move(&mut self, element: GameElement, move_number: Option<usize>) {
        let node = GameNode {
            element,
            variations: Vec::new(),
            parent: None,
            move_number,
        };
        
        if self.current_depth == 0 {
            // Add to main line
            self.main_line.push(node);
        } else {
            // Add to current variation
            if let Some(current_variation) = self.variation_stack.last_mut() {
                current_variation.push(node);
            }
        }
    }
    
    /// Start a new variation - corresponds to ENCODE_START_MARKER(13)
    pub fn start_variation(&mut self) -> Result<(), String> {
        self.current_depth += 1;
        self.variation_stack.push(Vec::new());
        Ok(())
    }
    
    /// End current variation - corresponds to ENCODE_END_MARKER(14)
    pub fn end_variation(&mut self) -> Result<(), String> {
        if self.current_depth == 0 {
            return Err("Cannot end variation - not in a variation".to_string());
        }
        
        // Pop the completed variation and attach it to the parent move
        if let Some(variation_moves) = self.variation_stack.pop() {
            let variation = VariationTree {
                main_line: variation_moves,
                current_depth: 0,
                variation_stack: Vec::new(),
            };
            
            // Attach variation to the last move in the parent line
            if self.current_depth == 1 {
                // Attaching to main line
                if let Some(last_move) = self.main_line.last_mut() {
                    last_move.variations.push(variation);
                }
            } else {
                // Attaching to parent variation
                if let Some(parent_variation) = self.variation_stack.last_mut() {
                    if let Some(last_move) = parent_variation.last_mut() {
                        last_move.variations.push(variation);
                    }
                }
            }
        }
        
        self.current_depth -= 1;
        Ok(())
    }
    
    pub fn is_in_variation(&self) -> bool {
        self.current_depth > 0
    }
    
    /// Generate PGN-style variation notation
    pub fn to_pgn_with_variations(&self) -> String {
        let mut result = String::new();
        self.append_moves_to_pgn(&self.main_line, &mut result, 1, false);
        result
    }
    
    fn append_moves_to_pgn(&self, moves: &[GameNode], result: &mut String, mut move_num: usize, in_variation: bool) {
        for (i, node) in moves.iter().enumerate() {
            if let GameElement::Move { .. } = node.element {
                // Add move number for white moves or at start of variations
                if move_num % 2 == 1 || (in_variation && i == 0) {
                    if !result.is_empty() && !result.ends_with(' ') {
                        result.push(' ');
                    }
                    result.push_str(&format!("{}.", (move_num + 1) / 2));
                    if move_num % 2 == 0 {
                        result.push_str("..");
                    }
                }
                
                result.push(' ');
                // For now, add placeholder notation - will be replaced with actual algebraic notation
                result.push_str("move");
                
                // Add variations for this move
                for variation in &node.variations {
                    result.push_str(" (");
                    self.append_moves_to_pgn(&variation.main_line, result, move_num, true);
                    result.push(')');
                }
                
                move_num += 1;
            }
        }
    }
}

/// Game parsing state after tag and move parsing
#[derive(Debug)]
pub struct GameParseState {
    pub tags: Vec<PgnTag>,
    pub flags: GameFlags,
    pub elements: Vec<GameElement>,
    pub tags_end_offset: usize,
    pub flags_offset: usize,
    pub moves_start_offset: usize,
}

/// Updated game element for streaming parser
/// Used by parse_pgn_tags_with_streaming for variable-length move support
#[derive(Debug, Clone)]
pub enum StreamingGameElement {
    Move {
        piece_num: u8,
        move_value: u8,        // May not be meaningful for multi-byte moves
        raw_bytes: Vec<u8>,    // All bytes consumed by this move
        offset: usize,
        bytes_consumed: usize, // How many bytes this move used
    },
    Nag {
        nag_value: u8,         // NAG annotation value
        offset: usize,         // File offset of NAG marker
    },
    Comment {
        text: String,          // Comment text
        offset: usize,         // File offset of comment marker
    },
    VariationStart {
        offset: usize,         // File offset of variation start marker
    },
    VariationEnd {
        offset: usize,         // File offset of variation end marker
    },
    GameEnd {
        offset: usize,         // File offset of game end marker
    },
    Unknown {
        byte_value: u8,        // Original byte that couldn't be parsed
        offset: usize,         // File offset
        error: String,         // Error description
    },
}

/// Updated game parse state for streaming
/// Used by parse_pgn_tags_with_streaming function
#[derive(Debug)]
pub struct StreamingGameParseState {
    pub elements: Vec<StreamingGameElement>,
    pub tags: Vec<PgnTag>,
    pub flags: GameFlags,
    pub tags_end_offset: usize,
    pub flags_offset: usize,
    pub moves_start_offset: usize,
}

#[allow(dead_code)]
pub fn display_sg4_structure() {
    println!("\n🎯 SCID .sg4 Game File Structure Analysis");
    println!("========================================");
    println!();
    println!("Based on analysis of scidvspc source code:");
    println!("- gfile.cpp: Block-based file management");
    println!("- game.cpp: Game encoding/decoding functions");  
    println!("- bytebuf.cpp: Big-endian multi-byte value handling");
    println!();
    
    println!("📁 File Organization:");
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Block 0 (131,072 bytes)                                        │");
    println!("│ ┌─ Game 1 (variable length) ─┐ ┌─ Game 2 ─┐ ┌─ Game 3... ─┐   │");
    println!("│ │ Tags│Moves│Annotations│END │ │ Game data │ │ Game data  │   │");
    println!("│ └─────────────────────────────┘ └───────────┘ └────────────┘   │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Block 1 (131,072 bytes)                                        │");
    println!("│ ┌─ Game N ─┐ ┌─ Game N+1 ─┐ ...                               │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("🎮 Individual Game Record Structure:");
    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ Section          │ Format              │ Description            │");
    println!("├──────────────────┼─────────────────────┼────────────────────────┤");
    println!("│ PGN Tags         │ Variable length     │ Non-standard tags only │");
    println!("│ (optional)       │ String pairs        │ Standard tags in .si4  │");
    println!("├──────────────────┼─────────────────────┼────────────────────────┤");
    println!("│ Move Sequence    │ 1-3 bytes per move  │ Piece-specific encoding│");
    println!("│                  │ Big-endian values   │ See move encoding below│");
    println!("├──────────────────┼─────────────────────┼────────────────────────┤");
    println!("│ Annotations      │ Special bytes 11-14 │ NAGs, comments, vars   │");
    println!("│ (interspersed)   │ + variable data     │ Mixed with moves       │");
    println!("├──────────────────┼─────────────────────┼────────────────────────┤");
    println!("│ Game End         │ ENCODE_END_GAME(15) │ Marks end of record    │");
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    println!("♟️  Move Encoding Format (1-3 bytes per move):");
    println!("┌──────────────────────────────────────────────────────────────────┐");
    println!("│ Piece   │ Bytes │ Bit Layout               │ Encoding Strategy    │");
    println!("├─────────┼───────┼──────────────────────────┼──────────────────────┤");
    println!("│ King    │   1   │ [PieceNum:4][Direction:4]│ 8 directions+castling│");
    println!("│ Queen   │  1-2  │ Rooklike: 1 byte         │ Diagonal: 2 bytes    │");
    println!("│ Rook    │   1   │ [PieceNum:4][RankFile:4] │ Target rank or file  │");
    println!("│ Bishop  │   1   │ [PieceNum:4][File+Dir:4] │ Target file+direction│");
    println!("│ Knight  │   1   │ [PieceNum:4][LShape:4]   │ L-shaped move code   │");
    println!("│ Pawn    │   1   │ [PieceNum:4][Dir+Promo:4]│ Direction+promotion  │");
    println!("└──────────────────────────────────────────────────────────────────┘");
    println!();

    println!("🏷️  Special Encoding Bytes (interspersed with moves):");
    println!("┌─────────────────────────────────────────────────────────────────┐");
    println!("│ Value │ Name                │ Purpose                           │");
    println!("├───────┼─────────────────────┼───────────────────────────────────┤");
    println!("│  11   │ ENCODE_NAG          │ Followed by NAG annotation value  │");
    println!("│  12   │ ENCODE_COMMENT      │ Followed by variable-length text  │");
    println!("│  13   │ ENCODE_START_MARKER │ Begin variation                   │");    
    println!("│  14   │ ENCODE_END_MARKER   │ End variation                     │");
    println!("│  15   │ ENCODE_END_GAME     │ End of game record                │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    println!();

    println!("⚠️  CRITICAL IMPLEMENTATION NOTES:");
    println!("   • NO fixed header - games start immediately with data");
    println!("   • BIG-ENDIAN byte order for all multi-byte values");
    println!("   • Game boundaries determined by ENCODE_END_GAME (15)");
    println!("   • Move vs annotation distinguished by value range:");
    println!("     - Values 0-10: Move data");
    println!("     - Values 11-15: Special encoding markers");
    println!("   • Game length from .si4 index determines read boundaries");
    println!();
}

// COMMENTED OUT: Unused debug function that calls removed debug utilities
/*
pub fn parse_sg4_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 Reading SG4 file: {}", file_path);
    
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    println!("📊 File size: {} bytes", buffer.len());
    println!("📊 Expected blocks: {}", (buffer.len() + BLOCK_SIZE - 1) / BLOCK_SIZE);
    
    // Display first 64 bytes in hex for initial analysis
    println!("\n🔍 First 64 bytes (hex):");
    // print_hex_dump(&buffer, 0, 64.min(buffer.len()));  // Function removed
    
    // PHASE 1: Parse game boundaries using ENCODE_END_GAME markers
    println!("\n🎯 PHASE 1: Detecting Game Boundaries");
    println!("====================================");
    let game_boundaries = find_game_boundaries(&buffer);
    display_game_boundaries(&game_boundaries, &buffer);
    
    // Display structure table
    display_sg4_structure();
    
    println!("✅ SG4 structure analysis complete");
    println!("📝 Next step: Parse individual game fields within each game record");
    
    Ok(())
}
*/

/// Find game boundaries by scanning for ENCODE_END_GAME (15) markers
pub fn find_game_boundaries(buffer: &[u8]) -> Vec<(usize, usize)> {
    let mut boundaries = Vec::new();
    let mut game_start = 0;
    
    for i in 0..buffer.len() {
        if buffer[i] == ENCODE_END_GAME {
            // Found end of game marker
            let game_end = i + 1; // Include the END_GAME marker
            boundaries.push((game_start, game_end));
            
            // Next game starts immediately after this one
            game_start = game_end;
        }
    }
    
    // Handle case where file doesn't end with ENCODE_END_GAME
    if game_start < buffer.len() {
        boundaries.push((game_start, buffer.len()));
    }
    
    boundaries
}

#[allow(dead_code)]
fn display_game_boundaries(boundaries: &[(usize, usize)], buffer: &[u8]) {
    println!("🔍 Game Boundary Detection Results:");
    println!("┌──────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Game #   │ Start Offset│ End Offset  │ Length      │");
    println!("├──────────┼─────────────┼─────────────┼─────────────┤");
    
    for (i, (start, end)) in boundaries.iter().enumerate() {
        println!("│ {:8} │ {:11} │ {:11} │ {:11} │", 
                 i + 1, start, end, end - start);
    }
    
    println!("└──────────┴─────────────┴─────────────┴─────────────┘");
    println!("📊 Total games detected: {}", boundaries.len());
    
    if boundaries.len() > 0 {
        println!("\n🔍 Game Record Analysis:");
        for (i, (start, end)) in boundaries.iter().enumerate().take(3) {
            println!("\n🎮 Game {} ({} to {}, {} bytes):", i + 1, start, end, end - start);
            
            // Look for ENCODE_END_GAME marker at the end
            let game_data = &boundaries[i];
            if game_data.1 > game_data.0 {
                let last_byte_pos = game_data.1 - 1;
                if last_byte_pos < buffer.len() {
                    println!("   Last byte at offset {}: 0x{:02x} ({})", 
                             last_byte_pos, 
                             buffer[last_byte_pos],
                             if buffer[last_byte_pos] == ENCODE_END_GAME { "ENCODE_END_GAME" } else { "other" });
                }
            }
            
            // PHASE 2: Parse PGN tags, flags, and move data for this game
            println!("   📝 Parsing PGN tags, flags, and game elements:");
            let game_data = &buffer[*start..*end];
            match parse_pgn_tags(game_data) {
                Ok(_game_state) => {
                    // display_pgn_tags(&game_state.tags);  // Function removed
                    // display_game_flags(&game_state.flags, game_state.flags_offset + start);  // Function removed
                    // display_game_elements(&game_state.elements, *start);  // Function removed
                }
                Err(e) => {
                    println!("   ❌ Parsing failed: {}", e);
                    // Show first 32 bytes for debugging
                    let sample_len = (end - start).min(32);
                    println!("   First {} bytes for debugging:", sample_len);
                    // print_hex_dump(buffer, *start, sample_len);  // Function removed
                }
            }
        }
    }
}

/// Helper function: Check if a move requires three-byte encoding (extremely rare)
fn is_three_byte_move(piece_num: u8, move_value: u8, move_bytes: &[u8]) -> bool {
    piece_num == 1 && move_value == 15 && move_bytes.len() >= 2 && 
    move_bytes[1] == 0xFF  // Special marker for 3-byte encoding
}

/// Helper function: Validate that a byte could be a valid second move byte
fn is_valid_second_move_byte(byte: u8) -> bool {
    byte < ENCODE_FIRST || byte > ENCODE_LAST
}

/// Helper function: Validate that a byte could be a valid third move byte
fn is_valid_third_move_byte(byte: u8) -> bool {
    // Third byte validation based on SCID source analysis
    // Three-byte moves are extremely rare and use extended encoding
    byte < ENCODE_FIRST || byte > ENCODE_LAST || byte == 0xFF  // 0xFF is special marker
}

/// Parse multi-byte move sequences based on SCID encoding
fn parse_multi_byte_move(game_data: &[u8], start_pos: usize, piece_num: u8, move_value: u8) -> Result<(usize, Vec<u8>), String> {
    if start_pos >= game_data.len() {
        return Err("Invalid start position for multi-byte move parsing".to_string());
    }
    
    let first_byte = game_data[start_pos];
    
    // Check if this move requires multiple bytes based on SCID logic
    let needs_multi_byte = match piece_num {
        0..=1 => move_value > 10,  // King/Queen complex moves
        2..=3 => move_value > 15,  // Rook extended moves  
        4..=5 => move_value > 7,   // Bishop diagonal extensions
        6..=7 => move_value > 8,   // Knight L-shape extensions
        _ => move_value > 14,      // Pawn promotion/en passant
    };
    
    if !needs_multi_byte || start_pos + 1 >= game_data.len() {
        return Ok((1, vec![first_byte]));
    }
    
    // Read additional bytes based on move complexity
    let mut bytes_consumed = 1;
    let mut move_bytes = vec![first_byte];
    
    // Read second byte for complex moves
    let second_byte = game_data[start_pos + 1];
    if is_valid_second_move_byte(second_byte) {
        move_bytes.push(second_byte);
        bytes_consumed += 1;
        
        // Check if third byte needed (extremely rare)
        if is_three_byte_move(piece_num, move_value, &move_bytes) && start_pos + 2 < game_data.len() {
            let third_byte = game_data[start_pos + 2];
            if is_valid_third_move_byte(third_byte) {
                move_bytes.push(third_byte);
                bytes_consumed += 1;
            }
        }
    }
    
    Ok((bytes_consumed, move_bytes))
}

/// Decode multi-byte move sequences  
fn try_decode_multi_byte_move(piece_num: u8, move_value: u8, move_bytes: &[u8]) -> Option<DecodedMove> {
    if move_bytes.len() < 2 {
        return None;
    }
    
    // Check for three-byte moves first (Phase 5 Step 5.1: Three-Byte Move Support)
    if move_bytes.len() >= 3 && is_three_byte_move(piece_num, move_value, move_bytes) {
        return decode_rare_three_byte_move(piece_num, move_value, move_bytes);
    }
    
    // Enhanced multi-byte decoder according to Edge Case Remediation Plan Phase 1 Step 1.2
    match piece_num {
        0 => decode_king_multi_byte(move_value, move_bytes[0], move_bytes[1]),
        1 => decode_queen_multi_byte(move_value, move_bytes[0], move_bytes[1]),
        2..=3 => decode_rook_multi_byte(move_value, move_bytes[0], move_bytes[1]),
        4..=5 => decode_bishop_multi_byte(move_value, move_bytes[0], move_bytes[1]),
        6..=7 => decode_knight_multi_byte(move_value, move_bytes[0], move_bytes[1]),
        _ => decode_pawn_multi_byte(move_value, move_bytes[0], move_bytes[1]),
    }
}

/// Decode King multi-byte moves (complex castling scenarios)
/// Based on SCID decodeKing algorithm for 2-byte encodings
fn decode_king_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // King multi-byte moves typically involve complex castling or special king moves
    // Based on SCID source analysis: complex castling verification or unusual king moves
    Some(DecodedMove {
        piece_num: 0, // King is always piece 0 in SCID
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::King {
            direction_code: move_value,
            description: format!("Multi-byte King move: val={}, bytes=[{:02X}, {:02X}]", 
                              move_value, first_byte, second_byte),
        },
    })
}

/// Decode Queen multi-byte moves (complex diagonal moves)
/// Based on SCID decodeQueen algorithm for extended diagonal movements
fn decode_queen_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // Queen multi-byte moves handle complex diagonal moves that require extended encoding
    // According to SCID analysis: diagonal moves to distant squares or complex queen maneuvers
    
    // Extract extended target information from both bytes
    let extended_encoding = ((first_byte as u16) << 8) | (second_byte as u16);
    let target_file = (extended_encoding & 0x07) as u8;  // Lower 3 bits for file
    let target_rank = ((extended_encoding >> 3) & 0x07) as u8;  // Next 3 bits for rank
    
    let move_type = if move_value >= 8 {
        "diagonal_long_range"
    } else {
        "diagonal_complex"
    };
    
    let description = if target_file < 8 && target_rank < 8 {
        format!("Queen {} to {}{} (encoding: 0x{:04X})", 
                move_type,
                char::from(b'a' + target_file), 
                target_rank + 1,
                extended_encoding)
    } else {
        format!("Queen {} (complex encoding: 0x{:04X})", move_type, extended_encoding)
    };
    
    Some(DecodedMove {
        piece_num: 1, // Queen is typically piece 1 in SCID
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::Queen {
            move_type: move_type.to_string(),
            description,
        },
    })
}

/// Decode Rook multi-byte moves (extended rank/file movements)
/// Based on SCID decodeRook algorithm for moves beyond normal single-byte range
fn decode_rook_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // Rook multi-byte moves handle extended range moves across multiple ranks/files
    // According to SCID analysis: long-distance rook moves requiring extended encoding
    
    let move_type = if move_value >= 16 {
        "extended_range"
    } else if move_value >= 8 {
        "rank_extended"
    } else {
        "file_extended"
    };
    
    // Determine which rook based on piece numbering (2 or 3 in SCID)
    let rook_piece_num = if (first_byte & 0x10) == 0 { 2 } else { 3 };
    
    Some(DecodedMove {
        piece_num: rook_piece_num,
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::Rook {
            target_info: move_type.to_string(),
            description: format!("Rook {} move: val={}, bytes=[{:02X}, {:02X}]", 
                              move_type, move_value, first_byte, second_byte),
        },
    })
}

/// Decode Bishop multi-byte moves (complex diagonal extensions)
/// Based on SCID decodeBishop algorithm for extended diagonal calculations
fn decode_bishop_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // Bishop multi-byte moves handle complex diagonal calculations beyond normal encoding
    // According to SCID analysis: extended diagonal moves with complex file/rank calculations
    
    let move_type = if move_value >= 8 {
        "diagonal_extended_up"  // up-left/down-right direction
    } else {
        "diagonal_extended_down"  // up-right/down-left direction
    };
    
    // Determine which bishop based on piece numbering (4 or 5 in SCID)
    let bishop_piece_num = if (first_byte & 0x10) == 0 { 4 } else { 5 };
    
    Some(DecodedMove {
        piece_num: bishop_piece_num,
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::Bishop {
            direction: move_type.to_string(),
            description: format!("Bishop {} move: val={}, bytes=[{:02X}, {:02X}]", 
                              move_type, move_value, first_byte, second_byte),
        },
    })
}

/// Decode Knight multi-byte moves (extended L-shape patterns)
/// Based on SCID decodeKnight algorithm for distant square jumps
fn decode_knight_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // Knight multi-byte moves handle extended L-shape patterns to distant squares
    // According to SCID analysis: unusual knight patterns in complex positions
    
    // Extract extended jump information
    let extended_pattern = second_byte & 0x0F;  // Lower 4 bits for extended pattern
    let move_type = match extended_pattern {
        0..=3 => "extended_up",
        4..=7 => "extended_down", 
        8..=11 => "extended_left",
        12..=15 => "extended_right",
        _ => "complex_jump",
    };
    
    // Determine which knight based on piece numbering (6 or 7 in SCID)
    let knight_piece_num = if (first_byte & 0x10) == 0 { 6 } else { 7 };
    
    Some(DecodedMove {
        piece_num: knight_piece_num,
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::Knight {
            l_shape_code: extended_pattern,
            description: format!("Knight {} jump: val={}, pattern={}, bytes=[{:02X}, {:02X}]", 
                              move_type, move_value, extended_pattern, first_byte, second_byte),
        },
    })
}

/// Decode Pawn multi-byte moves (complex promotions and en passant)
/// Based on SCID decodePawn algorithm for promotion piece selection
fn decode_pawn_multi_byte(move_value: u8, first_byte: u8, second_byte: u8) -> Option<DecodedMove> {
    // Pawn multi-byte moves handle complex promotion scenarios with piece selection
    // According to SCID analysis: promotion with specific piece type encoding
    
    // Extract promotion piece from second byte (lower 2 bits determine piece)
    let promotion_piece = match second_byte & 0x03 {
        0 => Some("Queen".to_string()),
        1 => Some("Rook".to_string()),
        2 => Some("Bishop".to_string()),
        3 => Some("Knight".to_string()),
        _ => None,
    };
    
    // Determine move type based on additional bits
    let is_capture = (second_byte & 0x04) != 0;
    let is_en_passant = (second_byte & 0x08) != 0;
    
    let move_type = if is_en_passant {
        "en_passant"
    } else if is_capture && promotion_piece.is_some() {
        "capture_promotion"
    } else if promotion_piece.is_some() {
        "promotion"
    } else if is_capture {
        "capture_extended"
    } else {
        "forward_extended"
    };
    
    let description = match (&promotion_piece, is_capture, is_en_passant) {
        (Some(piece), true, false) => format!("Pawn capture-promotion to {}: val={}, bytes=[{:02X}, {:02X}]", 
                                            piece, move_value, first_byte, second_byte),
        (Some(piece), false, false) => format!("Pawn promotion to {}: val={}, bytes=[{:02X}, {:02X}]", 
                                             piece, move_value, first_byte, second_byte),
        (None, _, true) => format!("Pawn en passant: val={}, bytes=[{:02X}, {:02X}]", 
                                 move_value, first_byte, second_byte),
        _ => format!("Pawn {}: val={}, bytes=[{:02X}, {:02X}]", 
                   move_type, move_value, first_byte, second_byte),
    };
    
    Some(DecodedMove {
        piece_num: (first_byte >> 4) & 0x0F,  // Extract piece number from first byte
        move_value,
        raw_byte: first_byte,
        interpretation: MoveInterpretation::Pawn {
            direction: move_type.to_string(),
            promotion: promotion_piece,
            description,
            is_en_passant: Some(is_en_passant),
        },
    })
}

/// Decode extremely rare three-byte move encodings
/// Based on SCID source analysis - used for extremely complex positions
/// Usually involving multiple piece interactions or special position encodings
fn decode_rare_three_byte_move(piece_num: u8, move_value: u8, move_bytes: &[u8]) -> Option<DecodedMove> {
    if move_bytes.len() < 3 {
        return None;
    }
    
    let first_byte = move_bytes[0];
    let second_byte = move_bytes[1];
    let third_byte = move_bytes[2];
    
    // Decode based on piece type and complex encoding
    match piece_num {
        1 => decode_three_byte_queen_move(move_value, first_byte, second_byte, third_byte),
        _ => {
            // Very rare - create unknown move with detailed info for debugging
            Some(DecodedMove {
                piece_num,
                move_value,
                raw_byte: first_byte,
                interpretation: MoveInterpretation::Unknown {
                    reason: format!("3-byte move: piece={}, value={}, bytes=[{:02X}, {:02X}, {:02X}]", 
                                  piece_num, move_value, first_byte, second_byte, third_byte),
                },
            })
        }
    }
}

/// Decode three-byte Queen moves (extremely complex queen positioning)
/// Complex queen move encoding - analyze all three bytes for target square
fn decode_three_byte_queen_move(move_value: u8, byte1: u8, byte2: u8, byte3: u8) -> Option<DecodedMove> {
    let extended_encoding = ((byte1 as u32) << 16) | ((byte2 as u32) << 8) | (byte3 as u32);
    let target_file = (extended_encoding & 0x07) as u8;
    let target_rank = ((extended_encoding >> 3) & 0x07) as u8;
    
    if target_file < 8 && target_rank < 8 {
        Some(DecodedMove {
            piece_num: 1,
            move_value,
            raw_byte: byte1,
            interpretation: MoveInterpretation::Queen {
                move_type: "three_byte_complex".to_string(),
                description: format!("3-byte Queen to {}{}: val={}, bytes=[{:02X}, {:02X}, {:02X}]", 
                                   char::from(b'a' + target_file), 
                                   target_rank + 1,
                                   move_value, byte1, byte2, byte3),
            },
        })
    } else {
        None
    }
}

/// Decode single-byte moves
fn try_decode_move(piece_num: u8, move_value: u8, raw_byte: u8) -> Option<DecodedMove> {
    // Without position tracking, we use heuristics based on common piece arrangements
    let interpretation = match piece_num {
        0 => decode_king_move(move_value),           // King usually piece 0
        1 | 7 => decode_queen_move(move_value),      // Queens often piece 1 or 7
        2 | 9 => decode_rook_move(move_value),       // Rooks often pieces 2, 9
        3 | 10 => decode_bishop_move(move_value),    // Bishops often pieces 3, 10
        4 | 11 => decode_knight_move(move_value),    // Knights often pieces 4, 11
        5 | 6 | 8 | 12..=15 => decode_pawn_move(move_value), // Pawns typically 5,6,8,12-15
        _ => MoveInterpretation::Unknown {
            reason: format!("Piece {} interpretation uncertain", piece_num),
        }
    };
    
    Some(DecodedMove {
        piece_num,
        move_value,
        raw_byte,
        interpretation,
    })
}

/// Parse PGN tags and game flags from game data based on SCID Decode function
/// Reference: scidvspc/src/game.cpp DecodeTags() and Decode() functions
pub fn parse_pgn_tags(game_data: &[u8]) -> Result<GameParseState, Box<dyn std::error::Error>> {
    let mut tags = Vec::new();
    let mut pos = 0;
    
    // Tags are terminated by a zero byte
    while pos < game_data.len() {
        let tag_length_byte = game_data[pos];
        pos += 1;
        
        // Zero byte marks end of tags section
        if tag_length_byte == 0 {
            break;
        }
        
        // Special case: 255 = binary EventDate encoding (3 bytes follow)
        if tag_length_byte == 255 {
            if pos + 3 > game_data.len() {
                return Err("Insufficient data for binary EventDate encoding".into());
            }
            // Skip the 3-byte date for now - we'll implement this later
            pos += 3;
            continue;
        }
        
        let (tag_name, value_length_pos) = if tag_length_byte >= COMMON_TAG_THRESHOLD {
            // Common tag encoded as single byte (241-255)
            let common_tag_index = (tag_length_byte - COMMON_TAG_THRESHOLD) as usize;
            if common_tag_index >= COMMON_TAGS.len() {
                return Err(format!("Invalid common tag index: {}", common_tag_index).into());
            }
            (COMMON_TAGS[common_tag_index].to_string(), pos)
        } else {
            // Regular tag - length byte followed by tag name string
            let tag_len = tag_length_byte as usize;
            if pos + tag_len > game_data.len() {
                return Err("Insufficient data for tag name".into());
            }
            let tag_name = String::from_utf8_lossy(&game_data[pos..pos + tag_len]).to_string();
            pos += tag_len;
            (tag_name, pos)
        };
        
        // Read value length and value
        if value_length_pos >= game_data.len() {
            return Err("Missing value length byte".into());
        }
        let value_len = game_data[value_length_pos] as usize;
        pos = value_length_pos + 1;
        
        if pos + value_len > game_data.len() {
            return Err("Insufficient data for tag value".into());
        }
        let tag_value = String::from_utf8_lossy(&game_data[pos..pos + value_len]).to_string();
        pos += value_len;
        
        tags.push(PgnTag {
            name: tag_name,
            value: tag_value,
        });
    }
    
    let tags_end_offset = pos;
    
    // After tags, there should be a game flags byte
    // Reference: SCID game.cpp Decode() function - "byte gflags = buf->GetByte();"
    if pos >= game_data.len() {
        return Err("Missing game flags byte after tags".into());
    }
    
    let flags_byte = game_data[pos];
    let flags_offset = pos;
    pos += 1;
    
    // Parse flags according to SCID source code:
    // if (gflags & 1) { NonStandardStart = true; }
    // if (gflags & 2) { PromotionsFlag = true; }
    // if (gflags & 4) { UnderPromosFlag = true; }
    let flags = GameFlags {
        non_standard_start: (flags_byte & 1) != 0,
        has_promotions: (flags_byte & 2) != 0,
        has_under_promotions: (flags_byte & 4) != 0,
        raw_value: flags_byte,
    };
    
    let moves_start_offset = pos;
    
    // Parse move/annotation data until ENCODE_END_GAME
    // Reference: SCID game.cpp DecodeVariation() function
    let mut elements = Vec::new();
    
    while pos < game_data.len() {
        let byte_val = game_data[pos];
        let element_offset = pos;
        pos += 1;
        
        match byte_val {
            ENCODE_END_GAME => {
                elements.push(GameElement::GameEnd { offset: element_offset });
                break;
            }
            ENCODE_NAG => {
                // NAG followed by value byte
                if pos >= game_data.len() {
                    return Err("Missing NAG value byte".into());
                }
                let nag_value = game_data[pos];
                pos += 1;
                elements.push(GameElement::Nag { nag_value, offset: element_offset });
            }
            ENCODE_COMMENT => {
                // Comment followed by null-terminated string
                // Reference: SCID bytebuf.cpp GetTerminatedString() function
                let comment_start = pos;
                let mut comment_end = pos;
                
                // Find null terminator
                while comment_end < game_data.len() && game_data[comment_end] != 0 {
                    comment_end += 1;
                }
                
                if comment_end >= game_data.len() {
                    return Err("Unterminated comment string".into());
                }
                
                // Extract comment text (excluding null terminator)
                let comment_text = if comment_end > comment_start {
                    String::from_utf8_lossy(&game_data[comment_start..comment_end]).to_string()
                } else {
                    String::new() // Empty comment
                };
                
                // Skip past null terminator
                pos = comment_end + 1;
                
                elements.push(GameElement::Comment { 
                    text: comment_text, 
                    offset: element_offset 
                });
            }
            ENCODE_START_MARKER => {
                elements.push(GameElement::VariationStart { offset: element_offset });
            }
            ENCODE_END_MARKER => {
                elements.push(GameElement::VariationEnd { offset: element_offset });
            }
            _ => {
                // Regular move byte - decode according to SCID makeMoveByte format
                // Reference: makeMoveByte (byte pieceNum, byte value)
                // return (byte)((pieceNum & 15) << 4) | (byte)(value & 15);
                let piece_num = (byte_val >> 4) & 0x0F;  // Upper 4 bits
                let move_value = byte_val & 0x0F;        // Lower 4 bits
                
                // Check if this might be a multi-byte move sequence
                let (bytes_consumed, multi_byte_data) = parse_multi_byte_move(game_data, pos - 1, piece_num, move_value)
                    .unwrap_or_else(|_| (1, vec![byte_val]));  // Fallback to single-byte on error
                
                // Attempt to decode the move (single or multi-byte)
                let decoded = if multi_byte_data.len() > 1 {
                    try_decode_multi_byte_move(piece_num, move_value, &multi_byte_data)
                } else {
                    try_decode_move(piece_num, move_value, byte_val)
                };
                
                elements.push(GameElement::Move {
                    piece_num,
                    move_value,
                    raw_byte: byte_val,
                    offset: element_offset,
                    decoded,
                });
                
                // Skip additional bytes if this was a multi-byte move
                if bytes_consumed > 1 {
                    pos += bytes_consumed - 1;
                }
            }
        }
    }
    
    Ok(GameParseState {
        tags,
        flags,
        elements,
        tags_end_offset,
        flags_offset,
        moves_start_offset,
    })
}

/// Updated game parsing with streaming move decoder
/// Modified version of existing parse_pgn_tags() function
/// Supports variable-length moves including Queen diagonal moves (2-byte)
pub fn parse_pgn_tags_with_streaming(game_data: &[u8]) -> Result<StreamingGameParseState, Box<dyn std::error::Error>> {
    let mut elements = Vec::new();
    let mut stream = ScidByteStream::new(game_data);
    
    // Parse tags first (same logic as existing parser)
    let mut tags = Vec::new();
    
    // Tags are terminated by a zero byte
    while stream.has_bytes() {
        let tag_length_byte = stream.get_byte()?;
        
        // Zero byte marks end of tags section
        if tag_length_byte == 0 {
            break;
        }
        
        // Special case: 255 = binary EventDate encoding (3 bytes follow)
        if tag_length_byte == 255 {
            if stream.bytes_remaining() < 3 {
                return Err("Insufficient data for binary EventDate encoding".into());
            }
            // Skip the 3-byte date for now - we'll implement this later
            stream.get_byte()?;
            stream.get_byte()?;
            stream.get_byte()?;
            continue;
        }
        
        let (tag_name, value_length_pos) = if tag_length_byte >= COMMON_TAG_THRESHOLD {
            // Common tag encoded as single byte (241-255)
            let common_tag_index = (tag_length_byte - COMMON_TAG_THRESHOLD) as usize;
            if common_tag_index >= COMMON_TAGS.len() {
                return Err(format!("Invalid common tag index: {}", common_tag_index).into());
            }
            (COMMON_TAGS[common_tag_index].to_string(), stream.position())
        } else {
            // Regular tag - length byte followed by tag name string
            let tag_len = tag_length_byte as usize;
            if stream.bytes_remaining() < tag_len {
                return Err("Insufficient data for tag name".into());
            }
            let mut tag_bytes = Vec::new();
            for _ in 0..tag_len {
                tag_bytes.push(stream.get_byte()?);
            }
            let tag_name = String::from_utf8_lossy(&tag_bytes).to_string();
            (tag_name, stream.position())
        };
        
        // Read value length and value
        if !stream.has_bytes() {
            return Err("Missing value length byte".into());
        }
        let value_len = stream.get_byte()? as usize;
        
        if stream.bytes_remaining() < value_len {
            return Err("Insufficient data for tag value".into());
        }
        let mut value_bytes = Vec::new();
        for _ in 0..value_len {
            value_bytes.push(stream.get_byte()?);
        }
        let tag_value = String::from_utf8_lossy(&value_bytes).to_string();
        
        tags.push(PgnTag {
            name: tag_name,
            value: tag_value,
        });
    }
    
    let tags_end_offset = stream.position();
    
    // After tags, there should be a game flags byte
    if !stream.has_bytes() {
        return Err("Missing game flags byte after tags".into());
    }
    
    let flags_byte = stream.get_byte()?;
    let flags_offset = stream.position() - 1;
    
    // Parse flags according to SCID source code
    let flags = GameFlags {
        non_standard_start: (flags_byte & 1) != 0,
        has_promotions: (flags_byte & 2) != 0,
        has_under_promotions: (flags_byte & 4) != 0,
        raw_value: flags_byte,
    };
    
    let moves_start_offset = stream.position();
    
    // Create dummy position for move decoding
    let dummy_position = ScidPosition::new_starting_position();
    
    // 🔥 KEY CHANGE: Process moves using streaming decoder
    while stream.has_bytes() {
        let current_offset = stream.position();
        
        // Read next byte to determine element type
        let byte = stream.get_byte()?;
        
        match byte {
            // Special game elements (same as existing logic)
            ENCODE_NAG => { // 11
                if stream.has_bytes() {
                    let nag_value = stream.get_byte()?;
                    elements.push(StreamingGameElement::Nag { nag_value, offset: current_offset });
                }
            }
            ENCODE_COMMENT => { // 12
                let comment = read_null_terminated_string(&mut stream)?;
                elements.push(StreamingGameElement::Comment { text: comment, offset: current_offset });
            }
            ENCODE_START_MARKER => { // 13
                elements.push(StreamingGameElement::VariationStart { offset: current_offset });
            }
            ENCODE_END_MARKER => { // 14
                elements.push(StreamingGameElement::VariationEnd { offset: current_offset });
            }
            ENCODE_END_GAME => { // 15
                elements.push(StreamingGameElement::GameEnd { offset: current_offset });
                break;
            }
            // 🔥 REGULAR MOVE: Let streaming decoder handle variable bytes
            _ => {
                // Create temporary stream from current position for move parsing
                let move_start_position = current_offset;
                let mut temp_stream = ScidByteStream::new(&game_data[current_offset..]);
                
                // Try to parse as move - decoder will consume appropriate bytes
                match decode_move_with_stream(&dummy_position, &mut temp_stream) {
                    Ok(scid_move) => {
                        let bytes_consumed = temp_stream.position();
                        let piece_num = scid_move.piece_num;
                        let move_value = 0; // Not meaningful for streaming moves
                        
                        elements.push(StreamingGameElement::Move {
                            piece_num,
                            move_value,
                            raw_bytes: game_data[current_offset..current_offset + bytes_consumed].to_vec(),
                            offset: current_offset,
                            bytes_consumed,
                        });
                        
                        // Advance main stream by consumed bytes
                        for _ in 1..bytes_consumed { // Skip first byte (already read)
                            stream.get_byte()?;
                        }
                    }
                    Err(e) => {
                        // Parsing failed - treat as unknown element
                        elements.push(StreamingGameElement::Unknown { 
                            byte_value: byte, 
                            offset: current_offset,
                            error: e,
                        });
                    }
                }
            }
        }
    }
    
    Ok(StreamingGameParseState { 
        elements,
        tags,
        flags,
        tags_end_offset,
        flags_offset,
        moves_start_offset,
    })
}

/// Helper function to read null-terminated string from stream
fn read_null_terminated_string(stream: &mut ScidByteStream) -> Result<String, Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    
    while stream.has_bytes() {
        let byte = stream.get_byte()?;
        if byte == 0 {
            break; // Found null terminator
        }
        bytes.push(byte);
    }
    
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

/// Decode King moves based on SCID source code (game.cpp decodeKing function)
/// Reference: static const int sqdiff[] = { 0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2 };
/// 
/// SHAKMATY CONVERSION NOTES:
/// - move_value 0-8: Normal king moves → shakmaty::Move::Normal
/// - move_value 9: Queenside castling → shakmaty::Move::Castle{king: e1/e8, rook: a1/a8}
/// - move_value 10: Kingside castling → shakmaty::Move::Castle{king: e1/e8, rook: h1/h8}
/// - Square differences: [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2]
// DEPRECATED: Use position-aware decoder in src/position/decoder.rs
fn decode_king_move(move_value: u8) -> MoveInterpretation {
    // SCID King move lookup table from game.cpp decodeKing()
    let descriptions = [
        "null move (stay in place)",     // 0
        "up-left (-9)",                  // 1: -9
        "up (-8)",                       // 2: -8  
        "up-right (-7)",                 // 3: -7
        "left (-1)",                     // 4: -1
        "right (+1)",                    // 5: +1
        "down-left (+7)",                // 6: +7
        "down (+8)",                     // 7: +8
        "down-right (+9)",               // 8: +9
        "queenside castling (-2)",       // 9: -2
        "kingside castling (+2)",        // 10: +2
    ];
    
    if (move_value as usize) < descriptions.len() {
        MoveInterpretation::King {
            direction_code: move_value,
            description: descriptions[move_value as usize].to_string(),
        }
    } else {
        MoveInterpretation::Unknown {
            reason: format!("Invalid king move value: {}", move_value),
        }
    }
}

/// Decode Queen moves based on SCID source code (game.cpp decodeQueen function)
// DEPRECATED: Use position-aware decoder in src/position/decoder.rs
fn decode_queen_move(move_value: u8) -> MoveInterpretation {
    if move_value >= 8 {
        // Rook-vertical move: val - 8 gives target rank
        let target_rank = move_value - 8;
        MoveInterpretation::Queen {
            move_type: "rook-like vertical".to_string(),
            description: format!("vertical to rank {}", target_rank),
        }
    } else {
        // Could be rook-horizontal or diagonal (needs more context)
        MoveInterpretation::Queen {
            move_type: "rook-like horizontal or diagonal".to_string(),
            description: format!("horizontal to file {} or diagonal", move_value),
        }
    }
}

/// Decode Rook moves based on SCID source code (game.cpp decodeRook function)  
// DEPRECATED: Use position-aware decoder in src/position/decoder.rs
fn decode_rook_move(move_value: u8) -> MoveInterpretation {
    if move_value >= 8 {
        // Move along a file to different rank
        let target_rank = move_value - 8;
        MoveInterpretation::Rook {
            target_info: format!("rank {}", target_rank),
            description: format!("vertical to rank {}", target_rank),
        }
    } else {
        // Move along a rank to different file
        MoveInterpretation::Rook {
            target_info: format!("file {}", move_value),
            description: format!("horizontal to file {}", move_value),
        }
    }
}

/// Decode Bishop moves based on SCID source code (game.cpp decodeBishop function)  
// DEPRECATED: Use position-aware decoder in src/position/decoder.rs
fn decode_bishop_move(move_value: u8) -> MoveInterpretation {
    let target_file = move_value & 7; // Lower 3 bits
    let direction = if move_value >= 8 {
        "up-left/down-right"
    } else {
        "up-right/down-left"
    };
    
    MoveInterpretation::Bishop {
        direction: direction.to_string(),
        description: format!("{} diagonal to file {}", direction, target_file),
    }
}

/// Decode Knight moves based on SCID source code (game.cpp decodeKnight function)
/// Reference: static const int sqdiff[] = { 0, -17, -15, -10, -6, 6, 10, 15, 17 };
// DEPRECATED: Use position-aware decoder in src/position/decoder.rs
fn decode_knight_move(move_value: u8) -> MoveInterpretation {
    let descriptions = [
        "invalid (0)",      // 0: invalid
        "L-shape (-17)",    // 1: -17 (2 up, 1 left)
        "L-shape (-15)",    // 2: -15 (2 up, 1 right)  
        "L-shape (-10)",    // 3: -10 (1 up, 2 left)
        "L-shape (-6)",     // 4: -6  (1 up, 2 right)
        "L-shape (+6)",     // 5: +6  (1 down, 2 left)
        "L-shape (+10)",    // 6: +10 (1 down, 2 right)
        "L-shape (+15)",    // 7: +15 (2 down, 1 left)
        "L-shape (+17)",    // 8: +17 (2 down, 1 right)
    ];
    
    if move_value >= 1 && move_value <= 8 {
        MoveInterpretation::Knight {
            l_shape_code: move_value,
            description: descriptions[move_value as usize].to_string(),
        }
    } else {
        MoveInterpretation::Unknown {
            reason: format!("Invalid knight move value: {}", move_value),
        }
    }
}

/// Decode Pawn moves based on SCID source code (game.cpp decodePawn function)
// ============================================================================
// BROKEN: This function incorrectly interprets pawn moves without position
// PROBLEM: Value 15 interpreted as "en_passant" but is actually double push  
// EXAMPLE: CF byte (piece 12, value 15) wrongly decoded as "en passant" not "e4"
// REPLACED BY: Position-aware pawn decoder in src/position/decoder.rs
// ============================================================================
fn decode_pawn_move(move_value: u8) -> MoveInterpretation {
    // WARNING: This static interpretation is INCORRECT
    // It lacks position context to determine actual pawn moves
    let (move_type, promotion) = match move_value {
        0 => ("BROKEN: static en_passant", None),     // WRONG: needs position
        1 => ("forward", None),
        2 => ("BROKEN: static en_passant", None),     // WRONG: needs position
        3 => ("capture left+Q", Some("Queen")),   // +7/-7 with Queen promotion
        4 => ("forward+Q", Some("Queen")),        // +8/-8 with Queen promotion
        5 => ("capture right+Q", Some("Queen")),  // +9/-9 with Queen promotion
        6 => ("capture left+R", Some("Rook")),    // +7/-7 with Rook promotion
        7 => ("forward+R", Some("Rook")),         // +8/-8 with Rook promotion
        8 => ("capture right+R", Some("Rook")),   // +9/-9 with Rook promotion
        9 => ("capture left+B", Some("Bishop")),  // +7/-7 with Bishop promotion
        10 => ("forward+B", Some("Bishop")),      // +8/-8 with Bishop promotion
        11 => ("capture right+B", Some("Bishop")), // +9/-9 with Bishop promotion
        12 => ("capture left+N", Some("Knight")), // +7/-7 with Knight promotion
        13 => ("forward+N", Some("Knight")),      // +8/-8 with Knight promotion
        14 => ("capture right+N", Some("Knight")), // +9/-9 with Knight promotion
        15 => ("BROKEN: should be double push", None), // WRONG: CF byte is e2-e4, not en passant!
        _ => ("unknown", None),
    };
    
    if move_type == "unknown" {
        return MoveInterpretation::Unknown {
            reason: format!("DEPRECATED: Use position-aware decoder for pawn move {}", move_value),
        };
    }
    
    MoveInterpretation::Pawn {
        direction: move_type.to_string(),
        promotion: promotion.map(|s| s.to_string()),
        description: format!("DEPRECATED Pawn {}", move_type),
        is_en_passant: Some(false), // CORRECTED: Don't make static en passant assumptions
    }
}

// COMMENTED OUT: Unused function that depends on removed custom chess types
/*
/// Decode a move using position awareness - the foundation for accurate chess notation
/// This replaces heuristic guessing with actual position tracking
fn decode_move_with_position(
    piece_num: &u8, 
    move_value: &u8, 
    raw_byte: &u8,
    position: &ChessPosition
) -> Result<(Move, String), String> {
    // CRITICAL: SCID piece numbers are relative to the player to move
    // We need to map them to actual pieces on the board for the current player
    let actual_piece_id = map_scid_piece_number_to_actual(*piece_num, position.to_move)?;
    
    // Get the actual piece from the position
    let piece = position.get_piece_by_number(actual_piece_id)
        .ok_or_else(|| format!("Piece #{} (SCID #{}) not found on board - position tracking error", actual_piece_id, piece_num))?;
    
    let from_square = position.get_piece_location(actual_piece_id)
        .ok_or_else(|| format!("Location of piece #{} (SCID #{}) not tracked", actual_piece_id, piece_num))?;
    
    // Decode the target square based on piece type and current position
    let to_square = decode_target_square(piece.piece_type, *move_value, from_square, position)?;
    
    // Create the move
    let mut chess_move = Move::new(from_square, to_square, piece);
    
    // Check for captures
    if let Some(captured_piece) = position.get_piece_at(to_square) {
        chess_move.captured_piece = Some(captured_piece);
    }
    
    // Detect special moves - but validate they're actually legal
    chess_move.is_castling = is_castling_move(piece.piece_type, from_square, to_square) && 
                            is_castling_legal(from_square, to_square, position);
    chess_move.is_en_passant = is_en_passant_move(piece.piece_type, from_square, to_square, position);
    
    // Detect promotions from move_value for pawns
    if piece.piece_type == PieceType::Pawn {
        chess_move.promotion = decode_pawn_promotion(*move_value);
    }
    
    // Generate algebraic notation (basic version for now)
    let algebraic_notation = generate_basic_algebraic_notation(&chess_move, position)?;
    
    Ok((chess_move, algebraic_notation))
}
*/

// COMMENTED OUT: Additional unused functions that depend on removed custom chess types
/*
/// Decode multi-byte move with position awareness
/// Handles 2-byte and 3-byte move sequences for complex positions
fn decode_multi_byte_move_with_position(
    piece_num: &u8,
    move_value: &u8,
    move_bytes: &[u8],
    position: &ChessPosition
) -> Result<(Move, String), String> {
    if move_bytes.len() < 2 {
        // Fall back to single-byte decoding
        return decode_move_with_position(piece_num, move_value, &move_bytes[0], position);
    }
    
    // CRITICAL: SCID piece numbers are relative to the player to move
    let actual_piece_id = map_scid_piece_number_to_actual(*piece_num, position.to_move)?;
    
    // Get the actual piece from the position
    let piece = position.get_piece_by_number(actual_piece_id)
        .ok_or_else(|| format!("Piece #{} (SCID #{}) not found on board", actual_piece_id, piece_num))?;
    
    let from_square = position.get_piece_location(actual_piece_id)
        .ok_or_else(|| format!("Location of piece #{} (SCID #{}) not tracked", actual_piece_id, piece_num))?;
    
    // Decode multi-byte target square based on piece type and move data
    let to_square = decode_multi_byte_target_square(piece.piece_type, *move_value, move_bytes, from_square, position)?;
    
    // Create the move
    let mut chess_move = Move::new(from_square, to_square, piece);
    
    // Check for captures
    if let Some(captured_piece) = position.get_piece_at(to_square) {
        chess_move.captured_piece = Some(captured_piece);
    }
    
    // Detect special moves for multi-byte sequences
    chess_move.is_castling = is_castling_move(piece.piece_type, from_square, to_square) && 
                            is_castling_legal(from_square, to_square, position);
    chess_move.is_en_passant = is_en_passant_move(piece.piece_type, from_square, to_square, position);
    
    // Handle multi-byte promotions
    if piece.piece_type == PieceType::Pawn && move_bytes.len() >= 2 {
        chess_move.promotion = decode_multi_byte_pawn_promotion(move_bytes);
    }
    
    // Generate algebraic notation
    let algebraic_notation = generate_multi_byte_algebraic_notation(&chess_move, move_bytes, position)?;
    
    Ok((chess_move, algebraic_notation))
}

/// Decode target square for multi-byte moves
fn decode_multi_byte_target_square(
    piece_type: PieceType,
    move_value: u8,
    move_bytes: &[u8],
    from_square: Square,
    position: &ChessPosition
) -> Result<Square, String> {
    match piece_type {
        PieceType::Queen => decode_multi_byte_queen_target(move_value, move_bytes, from_square),
        PieceType::King => decode_multi_byte_king_target(move_value, move_bytes, from_square),
        PieceType::Pawn => decode_multi_byte_pawn_target(move_value, move_bytes, from_square, position),
        _ => {
            // For other pieces, fall back to single-byte decoding
            decode_target_square(piece_type, move_value, from_square, position)
        }
    }
}

/// Decode Queen multi-byte target (2-byte diagonal moves)
fn decode_multi_byte_queen_target(move_value: u8, move_bytes: &[u8], from_square: Square) -> Result<Square, String> {
    if move_bytes.len() < 2 {
        return Err("Queen multi-byte move requires at least 2 bytes".to_string());
    }
    
    let first_byte = move_bytes[0];
    let second_byte = move_bytes[1];
    
    // Complex encoding for long diagonal Queen moves
    let extended_target = ((first_byte as u16) << 8) | (second_byte as u16);
    let target_file = (extended_target & 0x0F) as u8;
    let target_rank = ((extended_target >> 4) & 0x0F) as u8;
    
    if target_file < 8 && target_rank < 8 {
        Square::new(target_file, target_rank)
    } else {
        Err(format!("Invalid Queen multi-byte target: file={}, rank={}", target_file, target_rank))
    }
}

/// Decode King multi-byte target (complex castling scenarios)
fn decode_multi_byte_king_target(move_value: u8, move_bytes: &[u8], from_square: Square) -> Result<Square, String> {
    if move_bytes.len() < 2 {
        return Err("King multi-byte move requires at least 2 bytes".to_string());
    }
    
    // For now, treat complex King moves as regular King moves
    // This would need refinement based on SCID source code analysis
    decode_king_target(move_value, from_square)
}

/// Decode Pawn multi-byte target (complex promotions)
fn decode_multi_byte_pawn_target(move_value: u8, move_bytes: &[u8], from_square: Square, position: &ChessPosition) -> Result<Square, String> {
    if move_bytes.len() < 2 {
        return Err("Pawn multi-byte move requires at least 2 bytes".to_string());
    }
    
    // Use standard pawn decoding with additional promotion information in second byte
    decode_pawn_target(move_value, from_square, position)
}

/// Decode promotion from multi-byte pawn moves
fn decode_multi_byte_pawn_promotion(move_bytes: &[u8]) -> Option<String> {
    if move_bytes.len() < 2 {
        return None;
    }
    
    // Second byte encodes promotion piece in multi-byte pawn moves
    let promotion_byte = move_bytes[1];
    match promotion_byte & 0x03 {  // Lower 2 bits determine piece
        0 => Some("Queen".to_string()),
        1 => Some("Rook".to_string()),
        2 => Some("Bishop".to_string()),
        3 => Some("Knight".to_string()),
        _ => None,
    }
}

/// Generate algebraic notation for multi-byte moves
fn generate_multi_byte_algebraic_notation(chess_move: &Move, move_bytes: &[u8], position: &ChessPosition) -> Result<String, String> {
    // For now, use standard notation with multi-byte indicator
    let standard_notation = generate_basic_algebraic_notation(chess_move, position)?;
    
    if move_bytes.len() > 1 {
        Ok(format!("{} [{}B]", standard_notation, move_bytes.len()))
    } else {
        Ok(standard_notation)
    }
}

/// Decode target square based on piece type, move value, and current position
fn decode_target_square(
    piece_type: PieceType,
    move_value: u8,
    from_square: Square,
    position: &ChessPosition
) -> Result<Square, String> {
    match piece_type {
        PieceType::King => decode_king_target(move_value, from_square),
        PieceType::Queen => decode_queen_target(move_value, from_square, position),
        PieceType::Rook => decode_rook_target(move_value, from_square),
        PieceType::Bishop => decode_bishop_target(move_value, from_square),
        PieceType::Knight => decode_knight_target(move_value, from_square),
        PieceType::Pawn => decode_pawn_target(move_value, from_square, position),
    }
}

/// Decode King target square - handles regular moves and castling
fn decode_king_target(move_value: u8, from_square: Square) -> Result<Square, String> {
    // SCID king move lookup table from decode_king_move function
    // Based on SCID source analysis: 10 = kingside castling, 11 = queenside castling (not in basic table)
    match move_value {
        10 => {
            // Kingside castling - king moves to g-file
            match from_square.rank() {
                0 => Square::from_algebraic("g1"), // White kingside
                7 => Square::from_algebraic("g8"), // Black kingside
                _ => Err("King not on home rank for castling".to_string())
            }
        }
        11 => {
            // Queenside castling - king moves to c-file
            match from_square.rank() {
                0 => Square::from_algebraic("c1"), // White queenside
                7 => Square::from_algebraic("c8"), // Black queenside
                _ => Err("King not on home rank for castling".to_string())
            }
        }
        _ => {
            // Regular king moves using square difference table
            let square_diffs = [0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2];
            
            if (move_value as usize) < square_diffs.len() {
                let diff = square_diffs[move_value as usize];
                let target_square_num = (from_square.0 as i8) + diff;
                
                if target_square_num >= 0 && target_square_num < 64 {
                    Ok(Square(target_square_num as u8))
                } else {
                    Err(format!("King move out of bounds: {} + {} = {}", from_square.0, diff, target_square_num))
                }
            } else {
                Err(format!("Invalid king move value: {}", move_value))
            }
        }
    }
}

/// Decode Rook target square  
fn decode_rook_target(move_value: u8, from_square: Square) -> Result<Square, String> {
    if move_value >= 8 {
        // Move along a file to different rank
        let target_rank = move_value - 8;
        Square::new(from_square.file(), target_rank)
    } else {
        // Move along a rank to different file  
        Square::new(move_value, from_square.rank())
    }
}

/// Decode Bishop target square
fn decode_bishop_target(move_value: u8, from_square: Square) -> Result<Square, String> {
    let target_file = move_value & 7; // Lower 3 bits
    // For now, simple file-based decoding - will need refinement
    Square::new(target_file, from_square.rank())
}

/// Decode Knight target square
fn decode_knight_target(move_value: u8, from_square: Square) -> Result<Square, String> {
    // SCID knight move lookup table expanded based on empirical analysis
    // Standard L-shaped moves: { 0, -17, -15, -10, -6, 6, 10, 15, 17 }
    // Extended values observed: 0, 9, 11, 12, 15 from test data
    let square_diffs = match move_value {
        0 => 0,   // Null move or stay in place (special case)
        1 => -17, // Up 2, Left 1
        2 => -15, // Up 2, Right 1
        3 => -10, // Up 1, Left 2
        4 => -6,  // Up 1, Right 2
        5 => 6,   // Down 1, Left 2
        6 => 10,  // Down 1, Right 2
        7 => 15,  // Down 2, Left 1
        8 => 17,  // Down 2, Right 1
        // Extended values - possibly special cases or multi-byte sequences
        9 => -33,  // Extended up-left (2×up + left, for edge cases)
        10 => -31, // Extended up-right (2×up + right) 
        11 => -19, // Extended left-up (left + 2×up)
        12 => -13, // Extended right-up (right + 2×up)
        13 => 13,  // Extended left-down (left + 2×down)
        14 => 19,  // Extended right-down (right + 2×down)
        15 => 33,  // Extended down-right (2×down + right)
        _ => return Err(format!("Knight move value {} exceeds maximum range", move_value))
    };
    
    let target_square_num = (from_square.0 as i8) + square_diffs;
    
    if target_square_num >= 0 && target_square_num < 64 {
        Ok(Square(target_square_num as u8))
    } else {
        Err(format!("Knight move out of bounds: {} + {} = {}", from_square.0, square_diffs, target_square_num))
    }
}

/// Decode Pawn target square  
fn decode_pawn_target(move_value: u8, from_square: Square, position: &ChessPosition) -> Result<Square, String> {
    // Get piece to determine color
    let piece = position.get_piece_at(from_square)
        .ok_or("No piece at from_square for pawn move")?;
    
    let direction = if piece.color == Color::White { 1 } else { -1 };
    
    match move_value {
        0 => { // Capture left: +7/-7
            Square::new(from_square.file().wrapping_sub(1), (from_square.rank() as i8 + direction) as u8)
        }
        1 => { // Forward: +8/-8  
            Square::new(from_square.file(), (from_square.rank() as i8 + direction) as u8)
        }
        2 => { // Capture right: +9/-9
            Square::new(from_square.file() + 1, (from_square.rank() as i8 + direction) as u8)
        }
        3..=5 => { // Capture + Queen promotion (same moves as 0-2)
            decode_pawn_target(move_value - 3, from_square, position)
        }
        6..=8 => { // Capture + Rook promotion  
            decode_pawn_target(move_value - 6, from_square, position)
        }
        9..=11 => { // Capture + Bishop promotion
            decode_pawn_target(move_value - 9, from_square, position)
        }
        12..=14 => { // Capture + Knight promotion
            decode_pawn_target(move_value - 12, from_square, position)  
        }
        15 => { // Double forward: +16/-16
            Square::new(from_square.file(), (from_square.rank() as i8 + 2 * direction) as u8)
        }
        _ => Err(format!("Invalid pawn move value: {}", move_value))
    }
}

/// Decode Queen target square (placeholder - needs more complex logic)
fn decode_queen_target(move_value: u8, from_square: Square, _position: &ChessPosition) -> Result<Square, String> {
    // Simplified - treat like rook for now
    decode_rook_target(move_value, from_square)
}

/// Check if move is castling - based on SCID move values and king movement
fn is_castling_move(piece_type: PieceType, from_square: Square, to_square: Square) -> bool {
    piece_type == PieceType::King && 
    from_square.file() == 4 &&  // King starts on e-file
    (to_square.file() == 6 || to_square.file() == 2)  // Moves to g-file (kingside) or c-file (queenside)
}

/// Check if castling is actually legal (rook present, path clear, etc.)
fn is_castling_legal(from_square: Square, to_square: Square, position: &ChessPosition) -> bool {
    let is_kingside = to_square.file() == 6;
    let color = position.get_piece_at(from_square).map(|p| p.color).unwrap_or(Color::White);
    
    // Check if the required rook is present
    let rook_square = match (color, is_kingside) {
        (Color::White, true) => Square::from_algebraic("h1").ok(),   // White kingside
        (Color::White, false) => Square::from_algebraic("a1").ok(),  // White queenside  
        (Color::Black, true) => Square::from_algebraic("h8").ok(),   // Black kingside
        (Color::Black, false) => Square::from_algebraic("a8").ok(),  // Black queenside
    };
    
    if let Some(rook_pos) = rook_square {
        if let Some(rook) = position.get_piece_at(rook_pos) {
            rook.piece_type == PieceType::Rook && rook.color == color
        } else {
            false  // No rook at expected position
        }
    } else {
        false  // Invalid square calculation
    }
}

/// Check if move is en passant
fn is_en_passant_move(piece_type: PieceType, from_square: Square, to_square: Square, position: &ChessPosition) -> bool {
    piece_type == PieceType::Pawn &&
    position.en_passant_target == Some(to_square) &&
    from_square.file() != to_square.file()
}

/// Decode pawn promotion from move value
fn decode_pawn_promotion(move_value: u8) -> Option<String> {
    // Based on SCID decodePawn source analysis
    match move_value {
        3 | 4 | 5 => Some("Queen".to_string()),   // Forward/capture + Queen
        6 | 7 | 8 => Some("Rook".to_string()),    // Forward/capture + Rook  
        9 | 10 | 11 => Some("Bishop".to_string()), // Forward/capture + Bishop
        12 | 13 | 14 => Some("Knight".to_string()), // Forward/capture + Knight
        _ => None,
    }
}

/// Map SCID piece number to actual piece ID based on current player to move
/// CRITICAL: SCID uses piece numbers 0-15 for the current player, not absolute IDs
fn map_scid_piece_number_to_actual(scid_piece_num: u8, to_move: Color) -> Result<u8, String> {
    // SCID piece number mapping based on analysis of test data:
    // P0 = King, P2 = Rook(a1), P9 = Rook(h1), P3 = Bishop(f1), P10 = Bishop(c1)  
    // P4 = Knight(g1), P11 = Knight(b1), P5-P8/P12-P15 = Pawns
    
    match to_move {
        Color::White => {
            // For White, SCID piece numbers map directly to our white piece IDs
            match scid_piece_num {
                0 => Ok(0),   // King
                1 => Ok(1),   // Queen  
                2 => Ok(2),   // a1 Rook
                3 => Ok(3),   // f1 Bishop
                4 => Ok(4),   // g1 Knight
                5 => Ok(5),   // a2 Pawn
                6 => Ok(6),   // b2 Pawn  
                7 => Ok(7),   // c2 Pawn
                8 => Ok(8),   // d2 Pawn
                9 => Ok(9),   // h1 Rook
                10 => Ok(10), // c1 Bishop
                11 => Ok(11), // b1 Knight
                12 => Ok(12), // e2 Pawn
                13 => Ok(13), // f2 Pawn
                14 => Ok(14), // g2 Pawn
                15 => Ok(15), // h2 Pawn
                _ => Err(format!("Invalid SCID piece number for White: {}", scid_piece_num))
            }
        }
        Color::Black => {
            // For Black, SCID piece numbers map to our black piece IDs (offset by 16)
            match scid_piece_num {
                0 => Ok(16),  // Black King
                1 => Ok(17),  // Black Queen
                2 => Ok(18),  // a8 Rook  
                3 => Ok(19),  // f8 Bishop
                4 => Ok(20),  // g8 Knight
                5 => Ok(21),  // a7 Pawn
                6 => Ok(22),  // b7 Pawn
                7 => Ok(23),  // c7 Pawn
                8 => Ok(24),  // d7 Pawn
                9 => Ok(25),  // h8 Rook
                10 => Ok(26), // c8 Bishop
                11 => Ok(27), // b8 Knight
                12 => Ok(28), // e7 Pawn
                13 => Ok(29), // f7 Pawn
                14 => Ok(30), // g7 Pawn
                15 => Ok(31), // h7 Pawn
                _ => Err(format!("Invalid SCID piece number for Black: {}", scid_piece_num))
            }
        }
    }
}

/// Generate basic algebraic notation from a move and position
fn generate_basic_algebraic_notation(chess_move: &Move, _position: &ChessPosition) -> Result<String, String> {
    // Basic implementation - will be enhanced in next phase
    let piece_symbol = match chess_move.piece.piece_type {
        PieceType::King => "K",
        PieceType::Queen => "Q", 
        PieceType::Rook => "R",
        PieceType::Bishop => "B",
        PieceType::Knight => "N",
        PieceType::Pawn => "",
    };
    
    // Handle special moves
    if chess_move.is_castling {
        return Ok(if chess_move.to.file() > chess_move.from.file() {
            "O-O".to_string()
        } else {
            "O-O-O".to_string()
        });
    }
    
    // Basic move notation
    let capture = if chess_move.captured_piece.is_some() { "x" } else { "" };
    let promotion = if let Some(promo) = chess_move.promotion {
        match promo {
            PieceType::Queen => "=Q",
            PieceType::Rook => "=R", 
            PieceType::Bishop => "=B",
            PieceType::Knight => "=N",
            _ => "",
        }
    } else {
        ""
    };
    
    Ok(format!("{}{}{}{}", piece_symbol, capture, chess_move.to, promotion))
}

/// Attempt to decode a move based on available information (legacy heuristic version)
/// This will be replaced by decode_move_with_position once position tracking is integrated
// ============================================================================
// DEPRECATED: This function uses broken static interpretation without position
// REPLACED BY: Position-aware decoding in src/position/decoder.rs
// STATUS: Kept for backward compatibility but should not be used
// TODO: Remove this function once all callers use position-aware decoding
// ============================================================================
fn try_decode_move(piece_num: u8, move_value: u8, raw_byte: u8) -> Option<DecodedMove> {
    // WARNING: This function provides inaccurate static interpretation
    // Example: CF byte was incorrectly decoded as "en passant" instead of "e4"
    // Use crate::position::decode_move() for accurate position-aware decoding
    
    let interpretation = match piece_num {
        0 => decode_king_move(move_value),           // King usually piece 0
        1 | 7 => decode_queen_move(move_value),      // Queens often piece 1 or 7
        2 | 9 => decode_rook_move(move_value),       // Rooks often pieces 2, 9
        3 | 10 => decode_bishop_move(move_value),    // Bishops often pieces 3, 10
        4 | 11 => decode_knight_move(move_value),    // Knights often pieces 4, 11
        5 | 6 | 8 | 12..=15 => decode_pawn_move(move_value), // Pawns typically 5,6,8,12-15
        _ => MoveInterpretation::Unknown {
            reason: format!("DEPRECATED: Use position-aware decoder for piece {}", piece_num),
        }
    };
    
    Some(DecodedMove {
        piece_num,
        move_value,
        raw_byte,
        interpretation,
    })
}

fn display_pgn_tags(tags: &[PgnTag]) {
    if tags.is_empty() {
        println!("      📋 No non-standard PGN tags found");
        return;
    }
    
    println!("      📋 Non-standard PGN tags found:");
    println!("      ┌──────────────────┬─────────────────────────────────┐");
    println!("      │ Tag Name         │ Value                           │");
    println!("      ├──────────────────┼─────────────────────────────────┤");
    
    for tag in tags {
        println!("      │ {:16} │ {:31} │", 
                 truncate_string(&tag.name, 16),
                 truncate_string(&tag.value, 31));
    }
    
    println!("      └──────────────────┴─────────────────────────────────┘");
    println!("      📊 Total tags: {}", tags.len());
}

fn display_game_flags(flags: &GameFlags, offset: usize) {
    println!("      🚩 Game flags (offset {}, value 0x{:02x}):", offset, flags.raw_value);
    println!("      ┌──────────────────────┬────────────────┐");
    println!("      │ Flag                 │ Value          │");
    println!("      ├──────────────────────┼────────────────┤");
    println!("      │ Non-standard start   │ {}             │", 
             if flags.non_standard_start { "✅ YES" } else { "❌ NO" });
    println!("      │ Has promotions       │ {}             │", 
             if flags.has_promotions { "✅ YES" } else { "❌ NO" });
    println!("      │ Has under-promotions │ {}             │", 
             if flags.has_under_promotions { "✅ YES" } else { "❌ NO" });
    println!("      │ Reserved bits (3-7)  │ {}             │", 
             (flags.raw_value >> 3) & 0x1f);
    println!("      └──────────────────────┴────────────────┘");
    
    if flags.non_standard_start {
        println!("      ⚠️  Note: Non-standard start means FEN string follows flags byte");
    }
}

fn display_game_elements(elements: &[GameElement], base_offset: usize) {
    println!("      ♟️  Game elements (moves and annotations):");
    
    let move_count = elements.iter().filter(|e| matches!(e, GameElement::Move { .. })).count();
    let nag_count = elements.iter().filter(|e| matches!(e, GameElement::Nag { .. })).count();
    let comment_count = elements.iter().filter(|e| matches!(e, GameElement::Comment { .. })).count();
    let variation_count = elements.iter().filter(|e| matches!(e, GameElement::VariationStart { .. })).count();
    
    println!("      📊 Summary: {} moves, {} NAGs, {} comments, {} variations", 
             move_count, nag_count, comment_count, variation_count);
    
    if elements.len() > 10 {
        println!("      📋 First 10 elements (showing element type and raw encoding):");
        println!("      ┌────────┬──────────┬────────────┬─────────────────────────────┐");
        println!("      │ #      │ Offset   │ Type       │ Details                     │");
        println!("      ├────────┼──────────┼────────────┼─────────────────────────────┤");
        
        for (i, element) in elements.iter().take(10).enumerate() {
            match element {
                GameElement::Move { piece_num, move_value, raw_byte, offset, decoded } => {
                    let move_desc = if let Some(decoded_move) = decoded {
                        match &decoded_move.interpretation {
                            MoveInterpretation::King { description, .. } => {
                                format!("P{}: King {}", piece_num, truncate_string(description, 10))
                            }
                            MoveInterpretation::Queen { description, .. } => {
                                format!("P{}: Queen {}", piece_num, truncate_string(description, 9))
                            }
                            MoveInterpretation::Rook { description, .. } => {
                                format!("P{}: Rook {}", piece_num, truncate_string(description, 10))
                            }
                            MoveInterpretation::Bishop { description, .. } => {
                                format!("P{}: Bishop {}", piece_num, truncate_string(description, 8))
                            }
                            MoveInterpretation::Knight { description, .. } => {
                                format!("P{}: Knight {}", piece_num, truncate_string(description, 8))
                            }
                            MoveInterpretation::Pawn { description, promotion, .. } => {
                                let promo_str = if let Some(p) = promotion {
                                    format!("={}", p)
                                } else {
                                    String::new()
                                };
                                format!("P{}: Pawn {}{}", piece_num, truncate_string(description, 5), promo_str)
                            }
                            MoveInterpretation::Unknown { reason } => {
                                format!("P{} V{} ({})", piece_num, move_value, 
                                       truncate_string(reason, 8))
                            }
                        }
                    } else {
                        format!("P{} V{} (raw)", piece_num, move_value)
                    };
                    println!("      │ {:6} │ {:8} │ Move       │ {:27} │", 
                             i + 1, base_offset + offset, move_desc);
                }
                GameElement::Nag { nag_value, offset } => {
                    println!("      │ {:6} │ {:8} │ NAG        │ Value {} (annotation)       │", 
                             i + 1, base_offset + offset, nag_value);
                }
                GameElement::Comment { text, offset } => {
                    let display_text = if text.is_empty() { 
                        "(empty)".to_string() 
                    } else { 
                        truncate_string(text, 25) 
                    };
                    println!("      │ {:6} │ {:8} │ Comment    │ \"{}\"           │", 
                             i + 1, base_offset + offset, display_text);
                }
                GameElement::VariationStart { offset } => {
                    println!("      │ {:6} │ {:8} │ Var Start  │ Begin variation             │", 
                             i + 1, base_offset + offset);
                }
                GameElement::VariationEnd { offset } => {
                    println!("      │ {:6} │ {:8} │ Var End    │ End variation               │", 
                             i + 1, base_offset + offset);
                }
                GameElement::GameEnd { offset } => {
                    println!("      │ {:6} │ {:8} │ Game End   │ ENCODE_END_GAME (15)        │", 
                             i + 1, base_offset + offset);
                }
            }
        }
        
        println!("      └────────┴──────────┴────────────┴─────────────────────────────┘");
        if elements.len() > 10 {
            println!("      ... and {} more elements", elements.len() - 10);
        }
    } else {
        println!("      📋 All {} elements:", elements.len());
        println!("      ┌────────┬──────────┬────────────┬─────────────────────────────┐");
        println!("      │ #      │ Offset   │ Type       │ Details                     │");
        println!("      ├────────┼──────────┼────────────┼─────────────────────────────┤");
        
        for (i, element) in elements.iter().enumerate() {
            match element {
                GameElement::Move { piece_num, move_value, raw_byte, offset, decoded } => {
                    let move_desc = if let Some(decoded_move) = decoded {
                        match &decoded_move.interpretation {
                            MoveInterpretation::King { description, .. } => {
                                format!("P{}: King {}", piece_num, truncate_string(description, 10))
                            }
                            MoveInterpretation::Queen { description, .. } => {
                                format!("P{}: Queen {}", piece_num, truncate_string(description, 9))
                            }
                            MoveInterpretation::Rook { description, .. } => {
                                format!("P{}: Rook {}", piece_num, truncate_string(description, 10))
                            }
                            MoveInterpretation::Bishop { description, .. } => {
                                format!("P{}: Bishop {}", piece_num, truncate_string(description, 8))
                            }
                            MoveInterpretation::Knight { description, .. } => {
                                format!("P{}: Knight {}", piece_num, truncate_string(description, 8))
                            }
                            MoveInterpretation::Pawn { description, promotion, .. } => {
                                let promo_str = if let Some(p) = promotion {
                                    format!("={}", p)
                                } else {
                                    String::new()
                                };
                                format!("P{}: Pawn {}{}", piece_num, truncate_string(description, 5), promo_str)
                            }
                            MoveInterpretation::Unknown { reason } => {
                                format!("P{} V{} ({})", piece_num, move_value, 
                                       truncate_string(reason, 8))
                            }
                        }
                    } else {
                        format!("P{} V{} (raw)", piece_num, move_value)
                    };
                    println!("      │ {:6} │ {:8} │ Move       │ {:27} │", 
                             i + 1, base_offset + offset, move_desc);
                }
                GameElement::Nag { nag_value, offset } => {
                    println!("      │ {:6} │ {:8} │ NAG        │ Value {} (annotation)       │", 
                             i + 1, base_offset + offset, nag_value);
                }
                GameElement::Comment { text, offset } => {
                    let display_text = if text.is_empty() { 
                        "(empty)".to_string() 
                    } else { 
                        truncate_string(text, 25) 
                    };
                    println!("      │ {:6} │ {:8} │ Comment    │ \"{}\"           │", 
                             i + 1, base_offset + offset, display_text);
                }
                GameElement::VariationStart { offset } => {
                    println!("      │ {:6} │ {:8} │ Var Start  │ Begin variation             │", 
                             i + 1, base_offset + offset);
                }
                GameElement::VariationEnd { offset } => {
                    println!("      │ {:6} │ {:8} │ Var End    │ End variation               │", 
                             i + 1, base_offset + offset);
                }
                GameElement::GameEnd { offset } => {
                    println!("      │ {:6} │ {:8} │ Game End   │ ENCODE_END_GAME (15)        │", 
                             i + 1, base_offset + offset);
                }
            }
        }
        
        println!("      └────────┴──────────┴────────────┴─────────────────────────────┘");
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        format!("{:width$}", s, width = max_len)
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn print_hex_dump(data: &[u8], offset: usize, length: usize) {
    for i in (0..length).step_by(16) {
        print!("{:08x}: ", offset + i);
        
        // Print hex bytes
        for j in 0..16 {
            if i + j < length {
                print!("{:02x} ", data[offset + i + j]);
            } else {
                print!("   ");
            }
            if j == 7 { print!(" "); }
        }
        
        print!(" |");
        
        // Print ASCII representation
        for j in 0..16 {
            if i + j < length {
                let byte = data[offset + i + j];
                if byte >= 32 && byte <= 126 {
                    print!("{}", byte as char);
                } else {
                    print!(".");
                }
            } else {
                print!(" ");
            }
        }
        println!("|");
    }
}

/// Test simple move decoding - simplified version for testing
pub fn test_simple_move_decoding(piece_num: u8, move_value: u8) -> Result<String, String> {
    let mut position = ChessPosition::starting_position();
    
    // Get the actual piece from the position
    let piece = position.get_piece_by_number(piece_num)
        .ok_or_else(|| format!("Piece #{} not found on board", piece_num))?;
    
    let from_square = position.get_piece_location(piece_num)
        .ok_or_else(|| format!("Location of piece #{} not tracked", piece_num))?;
    
    // Simple move decoding based on piece type
    let move_description = match piece.piece_type {
        PieceType::King => {
            match move_value {
                10 => "O-O (kingside castling)".to_string(),
                9 => "O-O-O (queenside castling)".to_string(),
                1..=8 => format!("King move (direction {})", move_value),
                _ => format!("Unknown king move ({})", move_value),
            }
        }
        PieceType::Pawn => {
            match move_value {
                15 => format!("{}4 (pawn double push)", 
                    ((from_square.file() as u8) + b'a') as char),
                1 => format!("{}3 (pawn forward)", 
                    ((from_square.file() as u8) + b'a') as char),
                _ => format!("Pawn move {} from {}", move_value, from_square),
            }
        }
        _ => format!("{:?} move {} from {}", piece.piece_type, move_value, from_square),
    };
    
    Ok(format!("P{} ({}): {}", piece_num, piece.piece_type.to_string(), move_description))
}
*/

/*
/// Parse a single game with position tracking and variation tree support
/// TODO: Update this function to use GameState in a future step
#[allow(dead_code)]
fn parse_game_with_variation_trees_old(
    game_data: &[u8],
    game_number: usize
) -> Result<(VariationTree, Vec<Move>, Vec<String>), String> {
    // Initialize GameState and variation tree
    let mut game_state = GameState::new();
    let mut variation_tree = VariationTree::new();
    
    // Parse the game structure first
    let parsed_game = parse_pgn_tags(game_data).map_err(|e| e.to_string())?;
    
    println!("🌳 VARIATION-AWARE PARSING: Game {}", game_number);
    println!("📍 Starting position with ScidPositionTracker");
    println!("📝 Processing {} elements with variation tracking...", parsed_game.elements.len());
    
    let mut move_count = 0;
    let mut in_variation = false;
    
    // Process each game element with variation awareness
    for element in game_state.elements.iter() {
        match element {
            GameElement::VariationStart { offset } => {
                println!("📂 Variation start at offset {}", offset);
                variation_tree.start_variation()?;
                in_variation = true;
            }
            GameElement::VariationEnd { offset } => {
                println!("📁 Variation end at offset {}", offset);  
                variation_tree.end_variation()?;
                in_variation = variation_tree.is_in_variation();
            }
            GameElement::Move { piece_num, move_value, raw_byte, offset, .. } => {
                match decode_move_with_position(piece_num, move_value, raw_byte, &position) {
                    Ok((chess_move, notation)) => {
                        move_count += 1;
                        
                        // Show moves in variations differently
                        let move_prefix = if in_variation { "  ↳ Var" } else { "  Move" };
                        if move_count <= 10 || chess_move.is_castling {
                            println!("{} {}: P{} V{} -> {}", move_prefix, move_count, piece_num, move_value, notation);
                            if chess_move.is_castling {
                                println!("    🏰 CASTLING DETECTED!");
                            }
                        }
                        
                        // Add to variation tree
                        variation_tree.add_move(element.clone(), Some(move_count));
                        
                        // Apply move to position (only for main line to maintain accurate state)
                        if !in_variation {
                            match position.apply_move(&chess_move) {
                                Ok(()) => {
                                    moves.push(chess_move);
                                    algebraic_notation.push(notation);
                                }
                                Err(e) => {
                                    println!("❌ FAILED TO APPLY MOVE {}:", move_count);
                                    println!("   Move: P{} V{} -> {}", piece_num, move_value, notation);
                                    println!("   Error: {}", e);
                                    return Err(format!("Failed to apply move {}: {}", move_count, e));
                                }
                            }
                        } else {
                            // For variations, just track the notation without applying to main position
                            algebraic_notation.push(format!("({})", notation));
                        }
                    }
                    Err(e) => {
                        let actual_piece_id = map_scid_piece_number_to_actual(*piece_num, position.to_move).unwrap_or(*piece_num);
                        let piece_info = position.get_piece_by_number(actual_piece_id)
                            .map(|p| format!("{:?} {:?}", p.color, p.piece_type))
                            .unwrap_or_else(|| "Unknown".to_string());
                        let move_prefix = if in_variation { "  ⚠️  Var" } else { "  ⚠️  Move" };
                        println!("{} {}: P{} V{} (actual piece: {}) - Error: {}", 
                            move_prefix, move_count + 1, piece_num, move_value, piece_info, e);
                        move_count += 1;
                        
                        // Add failed move to variation tree for completeness
                        variation_tree.add_move(element.clone(), Some(move_count));
                        continue;
                    }
                }
            }
            GameElement::Comment { text, offset } => {
                println!("💬 Comment at {}: \"{}\"", offset, text);
                variation_tree.add_move(element.clone(), None);
            }
            GameElement::Nag { nag_value, offset } => {
                println!("📊 NAG {} at offset {}", nag_value, offset);
                variation_tree.add_move(element.clone(), None);
            }
            GameElement::GameEnd { offset } => {
                println!("🏁 Game end at offset {}", offset);
                break;
            }
        }
    }
    
    println!("📍 Final position:");
    println!("{}", position.display_board());
    
    Ok((variation_tree, moves, algebraic_notation))
}
*/

/// Parse a single game with position tracking - the core of position-aware move decoding
pub fn parse_game_with_position_tracking(
    game_data: &[u8],
    game_number: usize
) -> Result<(Vec<ShakmMove>, Vec<String>), String> {
    // Initialize GameState with position-aware parsing
    let mut game_state = GameState::new();
    
    // Parse the game structure first
    let parsed_game = parse_pgn_tags(game_data).map_err(|e| e.to_string())?;
    
    println!("🔥 POSITION-AWARE PARSING: Game {}", game_number);
    println!("📍 Starting position with ScidPositionTracker");
    println!("📝 Processing {} moves with position-aware bridge layer...", parsed_game.elements.len());
    
    let mut move_count = 0;
    
    // Process each game element using position-aware GameState
    for (_i, element) in parsed_game.elements.iter().enumerate() {
        match element {
            GameElement::Move { piece_num, move_value, raw_byte: _, offset: _, decoded } => {
                if let Some(decoded_move) = decoded {
                    match game_state.play_scid_move(decoded_move) {
                        Ok(()) => {
                            move_count += 1;
                            
                            // Show first few moves with position-aware analysis
                            if move_count <= 5 {
                                println!("  Move {}: P{} V{} -> {}", 
                                    move_count, piece_num, move_value, 
                                    decoded_move.interpretation.description());
                                println!("    Position-aware parsing via ScidPositionTracker");
                            }
                        }
                        Err(e) => {
                            println!("❌ FAILED TO APPLY SCID MOVE {}:", move_count + 1);
                            println!("   Move: P{} V{} -> {}", piece_num, move_value, 
                                decoded_move.interpretation.description());
                            return Err(format!("Failed to apply SCID move {}: {}", move_count + 1, e));
                        }
                    }
                } else {
                        // No decoded move available, skip this element
                        println!("  ⚠️  Move {}: P{} V{} - No decoded move available", 
                            move_count + 1, piece_num, move_value);
                        continue;
                    }
                }
                GameElement::GameEnd { .. } => {
                    break; // End of game
                }
                _ => {
                    // Handle other elements (NAGs, comments, variations) later
                    continue;
                }
        }
    }
    
    // Get the move history and SAN notation from GameState
    let moves = game_state.move_history().to_vec();
    let algebraic_notation = game_state.san_moves().to_vec();
    
    println!("✅ Successfully processed {} moves", moves.len());
    println!("📍 Final position: {}", game_state.fen());
    
    Ok((moves, algebraic_notation))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_three_byte_move_parsing() {
        // Test Phase 5 Step 5.1: Three-Byte Move Support
        // Create test data representing a three-byte move scenario
        
        // Test the three-byte move detection function
        let move_bytes_three = vec![0x1F, 0xFF, 0x42];  // piece=1, value=15, marker=0xFF
        assert!(is_three_byte_move(1, 15, &move_bytes_three), "Should detect three-byte move");
        
        let move_bytes_two = vec![0x1F, 0x20];  // Normal two-byte move
        assert!(!is_three_byte_move(1, 15, &move_bytes_two), "Should not detect three-byte move without 0xFF marker");
        
        // Test three-byte queen move decoding
        let decoded = decode_three_byte_queen_move(15, 0x1F, 0xFF, 0x42);
        assert!(decoded.is_some(), "Three-byte queen move should decode successfully");
        
        let decoded_move = decoded.unwrap();
        assert_eq!(decoded_move.piece_num, 1, "Piece number should be 1 (queen)");
        assert_eq!(decoded_move.move_value, 15, "Move value should be preserved");
        
        match &decoded_move.interpretation {
            MoveInterpretation::Queen { move_type, description } => {
                assert_eq!(move_type, "three_byte_complex", "Move type should be three_byte_complex");
                assert!(description.contains("3-byte Queen"), "Description should mention 3-byte Queen");
                assert!(description.contains("[1F, FF, 42]"), "Description should include byte values");
            }
            _ => panic!("Expected Queen move interpretation"),
        }
        
        // Test rare three-byte move for non-queen pieces (should create Unknown move)
        let decoded_unknown = decode_rare_three_byte_move(2, 15, &move_bytes_three);
        assert!(decoded_unknown.is_some(), "Unknown three-byte move should be created");
        
        let unknown_move = decoded_unknown.unwrap();
        match &unknown_move.interpretation {
            MoveInterpretation::Unknown { reason } => {
                assert!(reason.contains("3-byte move"), "Reason should mention 3-byte move");
                assert!(reason.contains("piece=2"), "Reason should include piece number");
                assert!(reason.contains("[1F, FF, 42]"), "Reason should include byte values");
            }
            _ => panic!("Expected Unknown move interpretation for non-queen piece"),
        }
        
        println!("✅ Three-byte move parsing test passed - Phase 5 Step 5.1 implemented correctly");
    }
}