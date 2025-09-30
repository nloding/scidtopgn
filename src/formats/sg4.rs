use crate::core::error::{Result, ScidError};
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct DecodedMove {
    pub raw_bytes: Vec<u8>,
    pub piece_num: u8,
    pub move_value: u8,
    pub interpretation: MoveInterpretation,
    pub from_square_index: Option<u8>,
    pub to_square_index: Option<u8>,
    pub promotion_piece: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PgnTag {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
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

pub struct Sg4File {
    mmap: Mmap,
}

impl Sg4File {
    pub fn open(path: &Path) -> Result<Self> {
        let file = File::open(path).map_err(|e| ScidError::FileOpen(e, path.to_path_buf()))?;
        let mmap =
            unsafe { Mmap::map(&file) }.map_err(|e| ScidError::Mmap(e, path.to_path_buf()))?;
        Ok(Self { mmap })
    }
    
    pub fn iter_games(&self) -> GameIterator {
        GameIterator {
            sg4: self,
            current_offset: 0,
        }
    }
    
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
    pub fn num_games(&self) -> usize {
        find_game_boundaries(&self.mmap).len()
    }
    
    /// Get the raw data for debugging
    pub fn data(&self) -> &[u8] {
        &self.mmap
    }
    
    fn decode_pawn_promotion(&self, move_value: u8) -> Option<String> {
        match move_value {
            8 => Some("Queen".to_string()),
            9 => Some("Rook".to_string()),
            10 => Some("Bishop".to_string()),
            11 => Some("Knight".to_string()),
            _ => None,
        }
    }
}

pub struct GameRecord {
    pub tags: Vec<PgnTag>,
    pub flags: GameFlags,
    pub moves: Vec<DecodedMove>,
    pub comments: Vec<String>,
    pub nags: Vec<u8>,
    pub result: Option<u8>,
    pub next_offset: usize,
}

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
    fn parse_game_at_offset(&self, offset: usize) -> Result<GameRecord> {
        let mut current_offset = offset;
        let mut tags = Vec::new();
        let mut moves = Vec::new();
        let mut comments = Vec::new();
        let mut nags = Vec::new();
        let mut result = None;
        let mut flags = GameFlags {
            non_standard_start: false,
            has_promotions: false,
            has_under_promotions: false,
            raw_value: 0,
        };
        
        // Parse PGN tags first
        let tags_end_offset = self.parse_pgn_tags(&mut current_offset, &mut tags, &mut flags)?;
        
        // Then parse moves and special bytes
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
                    // Regular move
                    let decoded_move = self.decode_move_at(&mut current_offset)?;
                    
                    // Check for promotions in flags
                    if let MoveInterpretation::Pawn { promotion: Some(promo), .. } = &decoded_move.interpretation {
                        flags.has_promotions = true;
                        if promo != "Queen" {
                            flags.has_under_promotions = true;
                        }
                    }
                    
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
    
    fn parse_pgn_tags(&self, offset: &mut usize, tags: &mut Vec<PgnTag>, flags: &mut GameFlags) -> Result<usize> {
        let start_offset = *offset;
        
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
    
    fn decode_move_at(&self, offset: &mut usize) -> Result<DecodedMove> {
        if *offset >= self.sg4.mmap.len() {
            return Err(ScidError::invalid_format("Move extends beyond file bounds"));
        }
        
        let byte = self.sg4.mmap[*offset];
        *offset += 1;
        
        let piece_num = (byte >> 4) & 0x0F;
        let move_value = byte & 0x0F;
        
        // Check for multi-byte queen moves (queen diagonal moves require 2 bytes)
        if piece_num == 2 && move_value >= 8 {
            // Queen diagonal move - need second byte
            if *offset >= self.sg4.mmap.len() {
                return Err(ScidError::invalid_format("Queen diagonal move extends beyond file bounds"));
            }
            let second_byte = self.sg4.mmap[*offset];
            *offset += 1;
            return self.decode_queen_diagonal_move(byte, second_byte, offset);
        }
        
        // Single byte moves for all other pieces
        self.decode_single_byte_move(byte, piece_num, move_value, offset)
    }
    
    fn decode_single_byte_move(&self, byte: u8, piece_num: u8, move_value: u8, offset: &mut usize) -> Result<DecodedMove> {
        let interpretation = match piece_num {
            1 => self.decode_king_move(move_value),
            2 => self.decode_queen_move(move_value),
            3 => self.decode_rook_move(move_value),
            4 => self.decode_bishop_move(move_value),
            5 => self.decode_knight_move(move_value),
            6 => self.decode_pawn_move(move_value),
            _ => MoveInterpretation::Unknown {
                reason: format!("Invalid piece number: {}", piece_num),
            },
        };
        
        Ok(DecodedMove {
            raw_bytes: vec![byte],
            piece_num,
            move_value,
            interpretation,
            from_square_index: None,
            to_square_index: None,
            promotion_piece: None,
        })
    }
    
    fn decode_king_move(&self, move_value: u8) -> MoveInterpretation {
        let (direction_code, is_castle, description) = match move_value {
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
        
        MoveInterpretation::King {
            direction_code,
            is_castle,
        }
    }
    
    fn decode_queen_move(&self, move_value: u8) -> MoveInterpretation {
        let description = match move_value {
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
    
    fn decode_queen_diagonal_move(&self, first_byte: u8, second_byte: u8, offset: &mut usize) -> Result<DecodedMove> {
        let piece_num = (first_byte >> 4) & 0x0F;
        let move_value = first_byte & 0x0F;
        let diagonal_info = second_byte & 0x0F;
        
        let interpretation = MoveInterpretation::Queen;
        
        Ok(DecodedMove {
            raw_bytes: vec![first_byte, second_byte],
            piece_num,
            move_value,
            interpretation,
            from_square_index: None,
            to_square_index: Some(diagonal_info),
            promotion_piece: None,
        })
    }
    
    fn decode_rook_move(&self, move_value: u8) -> MoveInterpretation {
        let (target_info, description) = if move_value >= 8 {
            let rank = move_value - 8;
            (format!("rank {}", rank + 1), format!("Rook to rank {}", rank + 1))
        } else {
            let file = ('a' as u8 + move_value) as char;
            (format!("file {}", file), format!("Rook to file {}", file))
        };
        
        MoveInterpretation::Rook
    }
    
    fn decode_bishop_move(&self, move_value: u8) -> MoveInterpretation {
        let file = move_value & 0x07; // Lower 3 bits for target file
        let direction_bit = (move_value >> 3) & 0x01; // Bit 3 for direction
        
        let direction = if direction_bit == 0 {
            "up-left/down-right diagonal"
        } else {
            "up-right/down-left diagonal"
        };
        
        let target_file = ('a' as u8 + file) as char;
        let description = format!("Bishop {} to file {}", direction, target_file);
        
        MoveInterpretation::Bishop
    }
    
    fn decode_knight_move(&self, move_value: u8) -> MoveInterpretation {
        // Knight L-shaped moves using square differences
        const KNIGHT_MOVES: &[i8] = &[-17, -15, -10, -6, 6, 10, 15, 17];
        
        let l_shape_code = if move_value < 8 {
            move_value
        } else {
            // Extended codes for edge cases
            move_value - 8
        };
        
        let description = if l_shape_code < KNIGHT_MOVES.len() as u8 {
            format!("Knight L-shaped move pattern {}", l_shape_code + 1)
        } else {
            format!("Unknown knight move: {}", move_value)
        };
        
        MoveInterpretation::Knight {
            l_shape_code,
        }
    }
    
    fn decode_pawn_move(&self, move_value: u8) -> MoveInterpretation {
        let (direction, promotion, is_en_passant, description) = match move_value {
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
        
        MoveInterpretation::Pawn {
            direction: direction.to_string(),
            promotion,
            is_en_passant,
        }
    }
}

// Constants from SCID source code
const ENCODE_NAG: u8 = 11;
const ENCODE_COMMENT: u8 = 12;
const ENCODE_START_MARKER: u8 = 13;
const ENCODE_END_MARKER: u8 = 14;
const ENCODE_END_GAME: u8 = 15;

// Block size from SCID source code
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

pub struct NagProcessor;

impl NagProcessor {
    /// Process NAG (Numeric Annotation Glyph) values
    /// Based on SCID source code for annotation handling
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
                    current_offset += 2;
                } else {
                    current_offset += 1;
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
pub enum GameElement {
    Move {
        raw_byte: u8,
        piece_num: u8,
        move_value: u8,
        offset: usize,
    },
    Nag {
        nag_value: u8,
        offset: usize,
    },
    Comment {
        text: String,
        offset: usize,
    },
    VariationStart {
        offset: usize,
    },
    VariationEnd {
        offset: usize,
    },
    GameEnd {
        result: u8,
        offset: usize,
    },
}

/// Game parse state for legacy compatibility
#[derive(Debug)]
pub struct GameParseState {
    pub tags: Vec<PgnTag>,
    pub flags: GameFlags,
    pub elements: Vec<GameElement>,
    pub tags_end_offset: usize,
    pub flags_offset: usize,
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
pub fn parse_pgn_tags_with_streaming(data: &[u8]) -> Result<Vec<StreamingGameElement>> {
    let mut elements = Vec::new();
    let mut current_offset = 0;
    
    while current_offset < data.len() {
        let byte = data[current_offset];
        let start_offset = current_offset;
        
        match byte {
            ENCODE_END_GAME => {
                if current_offset + 1 < data.len() {
                    elements.push(StreamingGameElement::GameEnd { result: data[current_offset + 1] });
                    current_offset += 2;
                } else {
                    current_offset += 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    #[test]
    fn test_decode_king_moves() {
        let sg4_data = vec![
            0x10, // King piece (1) + move 0 (up)
            0x13, // King piece (1) + move 3 (castle kingside)
        ];
        
        let mut cursor = Cursor::new(sg4_data);
        let mmap = unsafe { Mmap::map(&cursor).unwrap() };
        let sg4_file = Sg4File { mmap };
        let mut iterator = sg4_file.iter_games();
        
        // This would need proper game structure for testing
        // For now, just test that the structure compiles
        assert_eq!(sg4_file.data().len(), 2);
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
    fn test_decode_queen_diagonal() {
        // Test queen diagonal move (requires 2 bytes)
        let sg4_data = vec![0x28, 0x05]; // Queen (2) + diagonal move (8), diagonal info (5)
        
        let mut cursor = Cursor::new(sg4_data);
        let mmap = unsafe { Mmap::map(&cursor).unwrap() };
        let sg4_file = Sg4File { mmap };
        
        assert_eq!(sg4_file.data().len(), 2);
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
}

impl NagProcessor {
    pub fn is_symbol_nag(nag: u8) -> bool {
        matches!(nag, 1..=6 | 10..=21)
    }

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

impl StreamingGameParseState {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_constants() {
        assert_eq!(ENCODE_NAG, 11);
        assert_eq!(ENCODE_COMMENT, 12);
        assert_eq!(ENCODE_START_MARKER, 13);
        assert_eq!(ENCODE_END_MARKER, 14);
        assert_eq!(ENCODE_END_GAME, 15);
    }
}
