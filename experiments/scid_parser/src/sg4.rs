use crate::utils::*;
use crate::position::*;
use std::fs::File;
use std::io::Read;

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
#[derive(Debug, Clone)]
pub struct DecodedMove {
    pub piece_num: u8,
    pub move_value: u8,
    pub raw_byte: u8,
    pub interpretation: MoveInterpretation,
}

/// Move interpretation based on SCID source code analysis
#[derive(Debug, Clone)]
pub enum MoveInterpretation {
    King {
        direction_code: u8,  // 0-10: directions and castling
        description: String, // Human-readable description
    },
    Queen {
        move_type: String,   // "rook-like" or "diagonal"
        description: String, // Human-readable description
    },
    Rook {
        target_info: String, // File or rank target
        description: String, // Human-readable description
    },
    Bishop {
        direction: String,   // Direction of diagonal move
        description: String, // Human-readable description
    },
    Knight {
        l_shape_code: u8,    // 1-8: L-shaped move patterns
        description: String, // Human-readable description
    },
    Pawn {
        direction: String,   // Forward/capture direction
        promotion: Option<String>, // Promotion piece if any
        description: String, // Human-readable description
    },
    Unknown {
        reason: String,      // Why we couldn't decode it
    },
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

pub fn parse_sg4_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📖 Reading SG4 file: {}", file_path);
    
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    println!("📊 File size: {} bytes", buffer.len());
    println!("📊 Expected blocks: {}", (buffer.len() + BLOCK_SIZE - 1) / BLOCK_SIZE);
    
    // Display first 64 bytes in hex for initial analysis
    println!("\n🔍 First 64 bytes (hex):");
    print_hex_dump(&buffer, 0, 64.min(buffer.len()));
    
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
                Ok(game_state) => {
                    display_pgn_tags(&game_state.tags);
                    display_game_flags(&game_state.flags, game_state.flags_offset + start);
                    display_game_elements(&game_state.elements, *start);
                }
                Err(e) => {
                    println!("   ❌ Parsing failed: {}", e);
                    // Show first 32 bytes for debugging
                    let sample_len = (end - start).min(32);
                    println!("   First {} bytes for debugging:", sample_len);
                    print_hex_dump(buffer, *start, sample_len);
                }
            }
        }
    }
}

/// Parse PGN tags and game flags from game data based on SCID Decode function
/// Reference: scidvspc/src/game.cpp DecodeTags() and Decode() functions
fn parse_pgn_tags(game_data: &[u8]) -> Result<GameParseState, Box<dyn std::error::Error>> {
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
                
                // Attempt to decode the move
                let decoded = try_decode_move(piece_num, move_value, byte_val);
                
                elements.push(GameElement::Move {
                    piece_num,
                    move_value,
                    raw_byte: byte_val,
                    offset: element_offset,
                    decoded,
                });
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

/// Decode King moves based on SCID source code (game.cpp decodeKing function)
/// Reference: static const int sqdiff[] = { 0, -9, -8, -7, -1, 1, 7, 8, 9, -2, 2 };
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
fn decode_pawn_move(move_value: u8) -> MoveInterpretation {
    // SCID pawn move encoding from decodePawn()
    let directions = [
        "capture left",    // 0: +7/-7 (capture left)
        "forward",         // 1: +8/-8 (straight forward)
        "capture right",   // 2: +9/-9 (capture right)
        "capture left+Q",  // 3: +7/-7 with Queen promotion
        "forward+Q",       // 4: +8/-8 with Queen promotion
        "capture right+Q", // 5: +9/-9 with Queen promotion
        "capture left+R",  // 6: +7/-7 with Rook promotion
        "forward+R",       // 7: +8/-8 with Rook promotion
        "capture right+R", // 8: +9/-9 with Rook promotion
        "capture left+B",  // 9: +7/-7 with Bishop promotion
        "forward+B",       // 10: +8/-8 with Bishop promotion
        "capture right+B", // 11: +9/-9 with Bishop promotion
        "capture left+N",  // 12: +7/-7 with Knight promotion
        "forward+N",       // 13: +8/-8 with Knight promotion
        "capture right+N", // 14: +9/-9 with Knight promotion
        "double forward",  // 15: +16/-16 (double pawn push)
    ];
    
    let promotions = [
        None, None, None,                    // 0-2: no promotion
        Some("Queen"), Some("Queen"), Some("Queen"),   // 3-5: Queen
        Some("Rook"), Some("Rook"), Some("Rook"),      // 6-8: Rook  
        Some("Bishop"), Some("Bishop"), Some("Bishop"), // 9-11: Bishop
        Some("Knight"), Some("Knight"), Some("Knight"), // 12-14: Knight
        None,                                // 15: double push
    ];
    
    if (move_value as usize) < directions.len() {
        let direction = directions[move_value as usize];
        let promotion = promotions[move_value as usize].map(|s| s.to_string());
        
        MoveInterpretation::Pawn {
            direction: direction.to_string(),
            promotion,
            description: direction.to_string(),
        }
    } else {
        MoveInterpretation::Unknown {
            reason: format!("Invalid pawn move value: {}", move_value),
        }
    }
}

/// Decode a move using position awareness - the foundation for accurate chess notation
/// This replaces heuristic guessing with actual position tracking
fn decode_move_with_position(
    piece_num: &u8, 
    move_value: &u8, 
    raw_byte: &u8,
    position: &ChessPosition
) -> Result<(Move, String), String> {
    // Get the actual piece from the position
    let piece = position.get_piece_by_number(*piece_num)
        .ok_or_else(|| format!("Piece #{} not found on board - position tracking error", piece_num))?;
    
    let from_square = position.get_piece_location(*piece_num)
        .ok_or_else(|| format!("Location of piece #{} not tracked", piece_num))?;
    
    // Decode the target square based on piece type and current position
    let to_square = decode_target_square(piece.piece_type, *move_value, from_square, position)?;
    
    // Create the move
    let mut chess_move = Move::new(from_square, to_square, piece);
    
    // Check for captures
    if let Some(captured_piece) = position.get_piece_at(to_square) {
        chess_move.captured_piece = Some(captured_piece);
    }
    
    // Detect special moves
    chess_move.is_castling = is_castling_move(piece.piece_type, from_square, to_square);
    chess_move.is_en_passant = is_en_passant_move(piece.piece_type, from_square, to_square, position);
    
    // Detect promotions from move_value for pawns
    if piece.piece_type == PieceType::Pawn {
        chess_move.promotion = decode_pawn_promotion(*move_value);
    }
    
    // Generate algebraic notation (basic version for now)
    let algebraic_notation = generate_basic_algebraic_notation(&chess_move, position)?;
    
    Ok((chess_move, algebraic_notation))
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
    // SCID knight move lookup table: { 0, -17, -15, -10, -6, 6, 10, 15, 17 }
    let square_diffs = [0, -17, -15, -10, -6, 6, 10, 15, 17];
    
    if move_value >= 1 && move_value <= 8 {
        let diff = square_diffs[move_value as usize];
        let target_square_num = (from_square.0 as i8) + diff;
        
        if target_square_num >= 0 && target_square_num < 64 {
            Ok(Square(target_square_num as u8))
        } else {
            Err(format!("Knight move out of bounds: {} + {} = {}", from_square.0, diff, target_square_num))
        }
    } else {
        Err(format!("Invalid knight move value: {}", move_value))
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

/// Check if move is castling
fn is_castling_move(piece_type: PieceType, from_square: Square, to_square: Square) -> bool {
    piece_type == PieceType::King && 
    ((from_square.file() as i8 - to_square.file() as i8).abs() == 2)
}

/// Check if move is en passant
fn is_en_passant_move(piece_type: PieceType, from_square: Square, to_square: Square, position: &ChessPosition) -> bool {
    piece_type == PieceType::Pawn &&
    position.en_passant_target == Some(to_square) &&
    from_square.file() != to_square.file()
}

/// Decode pawn promotion from move value
fn decode_pawn_promotion(move_value: u8) -> Option<PieceType> {
    match move_value {
        3..=5 => Some(PieceType::Queen),
        6..=8 => Some(PieceType::Rook),
        9..=11 => Some(PieceType::Bishop),
        12..=14 => Some(PieceType::Knight),
        _ => None,
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
fn try_decode_move(piece_num: u8, move_value: u8, raw_byte: u8) -> Option<DecodedMove> {
    // Without position tracking, we use heuristics based on common piece arrangements
    // This is approximate but demonstrates the decoding capability
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

impl std::fmt::Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PieceType::King => write!(f, "King"),
            PieceType::Queen => write!(f, "Queen"),
            PieceType::Rook => write!(f, "Rook"),
            PieceType::Bishop => write!(f, "Bishop"),
            PieceType::Knight => write!(f, "Knight"),
            PieceType::Pawn => write!(f, "Pawn"),
        }
    }
}

/// Parse a single game with position tracking - the core of position-aware move decoding
pub fn parse_game_with_position_tracking(
    game_data: &[u8],
    game_number: usize
) -> Result<(Vec<Move>, Vec<String>), String> {
    // Initialize position
    let mut position = ChessPosition::starting_position();
    let mut moves = Vec::new();
    let mut algebraic_notation = Vec::new();
    
    // Parse the game structure first
    let game_state = parse_pgn_tags(game_data).map_err(|e| e.to_string())?;
    
    println!("🔥 POSITION-AWARE PARSING: Game {}", game_number);
    println!("📍 Starting position:");
    println!("{}", position.display_board());
    println!("📝 Processing {} moves with position tracking...", game_state.elements.len());
    
    let mut move_count = 0;
    
    // Process each game element with position awareness
    for (i, element) in game_state.elements.iter().enumerate() {
        match element {
            GameElement::Move { piece_num, move_value, raw_byte, offset, .. } => {
                match decode_move_with_position(piece_num, move_value, raw_byte, &position) {
                    Ok((chess_move, notation)) => {
                        move_count += 1;
                        
                        // Show first few moves with detailed analysis
                        if move_count <= 5 {
                            println!("  Move {}: P{} V{} -> {}", move_count, piece_num, move_value, notation);
                            println!("    From: {} To: {}", chess_move.from, chess_move.to);
                            println!("    Piece: {:?} {:?}", chess_move.piece.color, chess_move.piece.piece_type);
                        }
                        
                        // Apply move to position
                        match position.apply_move(&chess_move) {
                            Ok(()) => {
                                moves.push(chess_move);
                                algebraic_notation.push(notation);
                            }
                            Err(e) => {
                                return Err(format!("Failed to apply move {}: {}", move_count, e));
                            }
                        }
                    }
                    Err(e) => {
                        // For now, continue on errors but log them
                        println!("  ⚠️  Move {}: P{} V{} - Error: {}", move_count + 1, piece_num, move_value, e);
                        move_count += 1;
                        
                        // Skip this move but continue parsing
                        continue;
                    }
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
    
    println!("✅ Successfully processed {} moves", moves.len());
    println!("📍 Final position:");
    println!("{}", position.display_board());
    
    Ok((moves, algebraic_notation))
}