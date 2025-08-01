use std::io::{self, Read};
use crate::utils::*;

#[derive(Debug)]
pub struct ScidHeader {
    pub magic: [u8; 8],
    pub version: u16,
    pub base_type: u32,
    pub num_games: u32,
    pub auto_load: u32,
    pub description: String,
    pub custom_flags: Vec<String>,
}

#[derive(Debug)]
pub struct GameIndex {
    pub offset: u32,
    pub length: u32,
    pub white_id: u32,
    pub black_id: u32,
    pub event_id: u32,
    pub site_id: u32,
    pub round_id: u32,
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub result: u8,
    pub eco: u16,
    pub white_elo: u16,
    pub black_elo: u16,
    pub flags: u16,
    pub parsed_flags: GameFlags,
    pub num_half_moves: u16,
}

#[derive(Debug)]
pub struct GameFlags {
    pub start: bool,           // Game has own start position
    pub promotions: bool,      // Game contains promotion(s)
    pub under_promotions: bool,// Game contains under-promotion(s)
    pub delete: bool,          // Game marked for deletion
    pub white_opening: bool,   // White openings flag
    pub black_opening: bool,   // Black openings flag
    pub middlegame: bool,      // Middlegames flag
    pub endgame: bool,         // Endgames flag
    pub novelty: bool,         // Novelty flag
    pub pawn_structure: bool,  // Pawn structure flag
    pub tactics: bool,         // Tactics flag
    pub kingside: bool,        // Kingside play flag
    pub queenside: bool,       // Queenside play flag
    pub brilliancy: bool,      // Brilliancy or good play
    pub blunder: bool,         // Blunder or bad play
    pub user: bool,            // User-defined flag
}

/// Parse SCID .si4 header based on Index::Open() from index.cpp
pub fn parse_header(reader: &mut impl Read) -> io::Result<ScidHeader> {
    // Read magic header (8 bytes)
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    
    // Verify magic header
    let expected_magic = b"Scid.si\0";
    if magic != *expected_magic {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid magic header: expected {:?}, got {:?}", expected_magic, magic)
        ));
    }
    
    // Read version (2 bytes) - SCID uses big-endian for 2-byte fields
    let version = read_u16_be(reader)?;
    
    // Read base type (4 bytes) - SCID uses big-endian
    let base_type = read_u32_be(reader)?;
    
    // Read num games (3 bytes) - SCID uses big-endian for 3-byte fields
    let num_games = read_u24_be(reader)?;
    
    // Read auto load (3 bytes) - SCID uses big-endian for 3-byte fields
    let auto_load = read_u24_be(reader)?;
    
    // Read description (108 bytes)
    let description = read_string(reader, 108)?;
    
    // Read custom flag descriptions (6 * 9 bytes each)
    let mut custom_flags = Vec::new();
    for _ in 0..6 {
        let flag_desc = read_string(reader, 9)?;
        custom_flags.push(flag_desc);
    }
    
    Ok(ScidHeader {
        magic,
        version,
        base_type,
        num_games,
        auto_load,
        description,
        custom_flags,
    })
}

/// Display SCID header in a nice table format
pub fn display_header_table(header: &ScidHeader) {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                           SCID DATABASE HEADER                             │");
    println!("├─────────────────────────┬───────────────────────────────────────────────────┤");
    println!("│ Field                   │ Value                                             │");
    println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
    println!("│ Magic                   │ {:<49} │", std::str::from_utf8(&header.magic).unwrap_or("invalid"));
    println!("│ Version                 │ {:<49} │", header.version);
    println!("│ Base Type               │ {:<49} │", header.base_type);
    println!("│ Number of Games         │ {:<49} │", header.num_games);
    println!("│ Auto Load Game          │ {:<49} │", header.auto_load);
    
    // Split long description into multiple lines if needed
    let desc = if header.description.len() > 45 {
        format!("{}...", &header.description[..42])
    } else {
        header.description.clone()
    };
    println!("│ Description             │ {:<49} │", desc);
    
    // Show custom flags if any are non-empty
    let non_empty_flags: Vec<_> = header.custom_flags.iter().filter(|s| !s.is_empty()).collect();
    if non_empty_flags.is_empty() {
        println!("│ Custom Flags            │ {:<49} │", "(none)");
    } else {
        println!("│ Custom Flags            │ {:<49} │", format!("{} flags set", non_empty_flags.len()));
    }
    
    println!("└─────────────────────────┴───────────────────────────────────────────────────┘");
    println!();
}

/// Parse game length from Length_Low (2 bytes) and Length_High (1 byte)
/// Based on SCID's IndexEntry::GetLength() in index.h
/// Formula: length = Length_Low + ((Length_High & 0x80) << 9)
/// This gives 17 bits total (16 + 1), supporting lengths up to 131,071 bytes
pub fn parse_game_length(length_low: u16, length_high: u8) -> u32 {
    let base_length = length_low as u32;
    let extended_bit = (length_high as u32 & 0x80) << 9;
    let total_length = base_length + extended_bit;
    
    println!("DEBUG: Game length parsing:");
    println!("  Length_Low (2 bytes): {} (0x{:04x})", length_low, length_low);
    println!("  Length_High (1 byte): {} (0x{:02x})", length_high, length_high);
    println!("  Extended bit (bit 7): {} (adds {} to length)", 
        (length_high & 0x80) != 0, extended_bit); 
    println!("  Total length: {} bytes", total_length);
    
    total_length
}

/// Parse game flags from the Flags field (2 bytes)
/// Based on SCID flag definitions in index.h
/// Each bit represents a different game attribute or classification
pub fn parse_game_flags(flags: u16) -> GameFlags {
    println!("DEBUG: Game flags parsing:");
    println!("  Flags (2 bytes): {} (0x{:04x} = 0b{:016b})", flags, flags, flags);
    
    let parsed_flags = GameFlags {
        start:           (flags & (1 << 0)) != 0,   // IDX_FLAG_START = 0
        promotions:      (flags & (1 << 1)) != 0,   // IDX_FLAG_PROMO = 1
        under_promotions:(flags & (1 << 2)) != 0,   // IDX_FLAG_UPROMO = 2
        delete:          (flags & (1 << 3)) != 0,   // IDX_FLAG_DELETE = 3
        white_opening:   (flags & (1 << 4)) != 0,   // IDX_FLAG_WHITE_OP = 4
        black_opening:   (flags & (1 << 5)) != 0,   // IDX_FLAG_BLACK_OP = 5
        middlegame:      (flags & (1 << 6)) != 0,   // IDX_FLAG_MIDDLEGAME = 6
        endgame:         (flags & (1 << 7)) != 0,   // IDX_FLAG_ENDGAME = 7
        novelty:         (flags & (1 << 8)) != 0,   // IDX_FLAG_NOVELTY = 8
        pawn_structure:  (flags & (1 << 9)) != 0,   // IDX_FLAG_PAWN = 9
        tactics:         (flags & (1 << 10)) != 0,  // IDX_FLAG_TACTICS = 10
        kingside:        (flags & (1 << 11)) != 0,  // IDX_FLAG_KSIDE = 11
        queenside:       (flags & (1 << 12)) != 0,  // IDX_FLAG_QSIDE = 12
        brilliancy:      (flags & (1 << 13)) != 0,  // IDX_FLAG_BRILLIANCY = 13
        blunder:         (flags & (1 << 14)) != 0,  // IDX_FLAG_BLUNDER = 14
        user:            (flags & (1 << 15)) != 0,  // IDX_FLAG_USER = 15
    };
    
    println!("  Active flags:");
    if parsed_flags.start { println!("    - Start position"); }
    if parsed_flags.promotions { println!("    - Promotions"); }
    if parsed_flags.under_promotions { println!("    - Under-promotions"); }
    if parsed_flags.delete { println!("    - Marked for deletion"); }
    if parsed_flags.white_opening { println!("    - White opening"); }
    if parsed_flags.black_opening { println!("    - Black opening"); }
    if parsed_flags.middlegame { println!("    - Middlegame"); }
    if parsed_flags.endgame { println!("    - Endgame"); }
    if parsed_flags.novelty { println!("    - Novelty"); }
    if parsed_flags.pawn_structure { println!("    - Pawn structure"); }
    if parsed_flags.tactics { println!("    - Tactics"); }
    if parsed_flags.kingside { println!("    - Kingside play"); }
    if parsed_flags.queenside { println!("    - Queenside play"); }
    if parsed_flags.brilliancy { println!("    - Brilliancy"); }
    if parsed_flags.blunder { println!("    - Blunder"); }
    if parsed_flags.user { println!("    - User flag"); }
    
    if flags == 0 {
        println!("    - No flags set");
    }
    
    parsed_flags
}

/// Parse White and Black player IDs from packed format
/// Based on SCID's IndexEntry::GetWhite() and GetBlack() in index.h
/// 
/// Format: 3 bytes total
/// - WhiteBlack_High (1 byte): bits 4-7 = White high, bits 0-3 = Black high
/// - WhiteID_Low (2 bytes): White player ID low 16 bits
/// - BlackID_Low (2 bytes): Black player ID low 16 bits
/// 
/// This gives 20-bit player IDs (4 + 16 bits), supporting 1,048,575 unique players
pub fn parse_player_ids(white_black_high: u8, white_id_low: u16, black_id_low: u16) -> (u32, u32) {
    println!("DEBUG: Player ID parsing:");
    println!("  WhiteBlack_High (1 byte): {} (0x{:02x} = 0b{:08b})", 
        white_black_high, white_black_high, white_black_high);
    println!("  WhiteID_Low (2 bytes): {} (0x{:04x})", white_id_low, white_id_low);
    println!("  BlackID_Low (2 bytes): {} (0x{:04x})", black_id_low, black_id_low);
    
    // White player ID: high 4 bits from bits 4-7 of WhiteBlack_High + low 16 bits
    let white_high = (white_black_high >> 4) as u32;    // Extract bits 4-7
    let white_id = (white_high << 16) | (white_id_low as u32);
    
    // Black player ID: high 4 bits from bits 0-3 of WhiteBlack_High + low 16 bits  
    let black_high = (white_black_high & 0xF) as u32;   // Extract bits 0-3
    let black_id = (black_high << 16) | (black_id_low as u32);
    
    println!("  White player ID reconstruction:");
    println!("    High 4 bits: {} (from bits 4-7)", white_high);
    println!("    Combined: ({} << 16) | {} = {}", white_high, white_id_low, white_id);
    
    println!("  Black player ID reconstruction:");
    println!("    High 4 bits: {} (from bits 0-3)", black_high);
    println!("    Combined: ({} << 16) | {} = {}", black_high, black_id_low, black_id);
    
    (white_id, black_id)
}

/// Parse Event, Site, and Round IDs from packed format
/// Based on SCID's IndexEntry::GetEvent(), GetSite(), GetRound() in index.h
/// 
/// Format: 5 bytes total
/// - EventSiteRnd_High (1 byte): bits 5-7 = Event high (3 bits), bits 2-4 = Site high (3 bits), bits 0-1 = Round high (2 bits)
/// - EventID_Low (2 bytes): Event ID low 16 bits
/// - SiteID_Low (2 bytes): Site ID low 16 bits  
/// - RoundID_Low (2 bytes): Round ID low 16 bits
/// 
/// This gives Event/Site IDs with 19 bits each (3+16), Round IDs with 18 bits (2+16)
pub fn parse_event_site_round_ids(event_site_rnd_high: u8, event_id_low: u16, site_id_low: u16, round_id_low: u16) -> (u32, u32, u32) {
    println!("DEBUG: Event/Site/Round ID parsing:");
    println!("  EventSiteRnd_High (1 byte): {} (0x{:02x} = 0b{:08b})", 
        event_site_rnd_high, event_site_rnd_high, event_site_rnd_high);
    println!("  EventID_Low (2 bytes): {} (0x{:04x})", event_id_low, event_id_low);
    println!("  SiteID_Low (2 bytes): {} (0x{:04x})", site_id_low, site_id_low);
    println!("  RoundID_Low (2 bytes): {} (0x{:04x})", round_id_low, round_id_low);
    
    // Event ID: high 3 bits from bits 5-7 of EventSiteRnd_High + low 16 bits
    let event_high = (event_site_rnd_high >> 5) as u32;           // Extract bits 5-7
    let event_id = (event_high << 16) | (event_id_low as u32);
    
    // Site ID: high 3 bits from bits 2-4 of EventSiteRnd_High + low 16 bits
    let site_high = ((event_site_rnd_high >> 2) & 0x7) as u32;    // Extract bits 2-4, mask to 3 bits
    let site_id = (site_high << 16) | (site_id_low as u32);
    
    // Round ID: high 2 bits from bits 0-1 of EventSiteRnd_High + low 16 bits
    let round_high = (event_site_rnd_high & 0x3) as u32;          // Extract bits 0-1, mask to 2 bits
    let round_id = (round_high << 16) | (round_id_low as u32);
    
    println!("  Event ID reconstruction:");
    println!("    High 3 bits: {} (from bits 5-7)", event_high);
    println!("    Combined: ({} << 16) | {} = {}", event_high, event_id_low, event_id);
    
    println!("  Site ID reconstruction:");
    println!("    High 3 bits: {} (from bits 2-4)", site_high);
    println!("    Combined: ({} << 16) | {} = {}", site_high, site_id_low, site_id);
    
    println!("  Round ID reconstruction:");
    println!("    High 2 bits: {} (from bits 0-1)", round_high);
    println!("    Combined: ({} << 16) | {} = {}", round_high, round_id_low, round_id);
    
    (event_id, site_id, round_id)
}

/// Decode game result from numeric value to human-readable string
/// Based on SCID result constants in common.h
/// 
/// Result values:
/// - 0 = RESULT_None  = "*" (no result/ongoing)
/// - 1 = RESULT_White = "1-0" (White wins)
/// - 2 = RESULT_Black = "0-1" (Black wins) 
/// - 3 = RESULT_Draw  = "1/2-1/2" (Draw)
pub fn decode_result(result: u8) -> &'static str {
    match result {
        0 => "*",         // RESULT_None
        1 => "1-0",       // RESULT_White
        2 => "0-1",       // RESULT_Black
        3 => "1/2-1/2",   // RESULT_Draw
        _ => "unknown",
    }
}

/// Parse and display the first game index entry (for testing)
pub fn parse_and_display_first_game_index(reader: &mut impl Read) -> io::Result<()> {
    match parse_game_index(reader) {
        Ok(game_index) => {
            println!();
            println!("┌─────────────────────────────────────────────────────────────────────────────┐");
            println!("│                        FIRST GAME INDEX ENTRY                              │");
            println!("├─────────────────────────┬───────────────────────────────────────────────────┤");
            println!("│ Field                   │ Value                                             │");
            println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
            println!("│ Game File Offset        │ {:<49} │", game_index.offset);
            println!("│ Game Length             │ {:<49} │", game_index.length);
            println!("│ Game Date               │ {}.{:02}.{:02}{:<39} │", game_index.year, game_index.month, game_index.day, "");
            println!("│ White Player ID         │ {:<49} │", game_index.white_id);
            println!("│ Black Player ID         │ {:<49} │", game_index.black_id);
            println!("│ Event ID                │ {:<49} │", game_index.event_id);
            println!("│ Site ID                 │ {:<49} │", game_index.site_id);
            println!("│ Round ID                │ {:<49} │", game_index.round_id);
            println!("│ Result                  │ {} ({}){:<38} │", game_index.result, decode_result(game_index.result), "");
            println!("│ ECO Code                │ {:<49} │", game_index.eco);
            println!("│ White ELO               │ {:<49} │", game_index.white_elo);
            println!("│ Black ELO               │ {:<49} │", game_index.black_elo);
            println!("│ Flags (raw)             │ {} (0x{:04x}){:<35} │", game_index.flags, game_index.flags, "");
            println!("│ Half Moves              │ {:<49} │", game_index.num_half_moves);
            println!("└─────────────────────────┴───────────────────────────────────────────────────┘");
            println!();
            Ok(())
        }
        Err(e) => {
            println!("Error parsing game index: {}", e);
            Err(e)
        }
    }
}

/// Display the structure of SCID game index entries (47 bytes each)
/// Based on IndexEntry::Read() from scidvspc/src/index.cpp
pub fn display_game_index_structure() {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                      SCID GAME INDEX ENTRY STRUCTURE                       │");
    println!("│                          (47 bytes per game)                               │");
    println!("├────────┬──────────┬─────────┬─────────────────────────────────────────────┤");
    println!("│ Offset │ Size     │ Format  │ Field Description                           │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────┤");
    println!("│   0-3  │ 4 bytes  │ BE uint │ Game File Offset (.sg4)                     │");
    println!("│   4-5  │ 2 bytes  │ BE uint │ Game Length (low 16 bits)                   │");
    println!("│   6    │ 1 byte   │ uint8   │ Length High + Flags (bit 7 = length bit 16) │");
    println!("│   7-8  │ 2 bytes  │ BE uint │ Game Flags                                  │");
    println!("│   9    │ 1 byte   │ packed  │ White/Black ID high bits (4+4 bits)        │");
    println!("│  10-11 │ 2 bytes  │ BE uint │ White Player ID (low 16 bits)               │");
    println!("│  12-13 │ 2 bytes  │ BE uint │ Black Player ID (low 16 bits)               │");
    println!("│  14    │ 1 byte   │ packed  │ Event/Site/Round ID high bits (3+3+2 bits) │");
    println!("│  15-16 │ 2 bytes  │ BE uint │ Event ID (low 16 bits)                      │");
    println!("│  17-18 │ 2 bytes  │ BE uint │ Site ID (low 16 bits)                       │");
    println!("│  19-20 │ 2 bytes  │ BE uint │ Round ID (low 16 bits)                      │");
    println!("│  21-22 │ 2 bytes  │ BE uint │ Variation Counts + Result (top 4 bits)      │");
    println!("│  23-24 │ 2 bytes  │ BE uint │ ECO Code                                    │");
    println!("│  25-28 │ 4 bytes  │ BE uint │ Game/Event Dates (packed)                   │");
    println!("│  29-30 │ 2 bytes  │ BE uint │ White ELO (bottom 12 bits) + Rating Type    │");
    println!("│  31-32 │ 2 bytes  │ BE uint │ Black ELO (bottom 12 bits) + Rating Type    │");
    println!("│  33-36 │ 4 bytes  │ BE uint │ Final Material Signature                    │");
    println!("│  37    │ 1 byte   │ uint8   │ Number of Half Moves (low 8 bits)           │");
    println!("│  38-46 │ 9 bytes  │ packed  │ Home Pawn Data + Move Count High Bits       │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────┘");
    println!();
    println!("Notes:");
    println!("• Game/Event Dates (offset 25-28): Lower 20 bits = game date, upper 12 = event date");
    println!("• Date Format: bits 0-4=day, 5-8=month, 9-19=year (no offset, direct values)");
    println!("• Player/Event/Site/Round IDs: 20-bit values split into high bits + low 16 bits");
    println!("• ELO ratings: 12-bit values (0-4095) with 4-bit rating type flags");
    println!("• Result: stored in top 4 bits of VarCounts field (0=*, 1=1-0, 2=0-1, 3=1/2-1/2)");
    println!("• Total half moves: 8 bits at offset 37 + 2 high bits from HomePawnData[0]");
    println!();
}

/// Parse a single game index entry (47 bytes) - currently unused but available for future use
pub fn parse_game_index(reader: &mut impl Read) -> io::Result<GameIndex> {
    // Read the 47-byte game index entry
    let mut entry_bytes = [0u8; 47];
    reader.read_exact(&mut entry_bytes)?;
    
    println!("Raw entry bytes (first 32): {:02x?}", &entry_bytes[0..32]);
    println!("Raw entry bytes (last 15): {:02x?}", &entry_bytes[32..47]);
    println!("Dates field bytes at offset 25-28: {:02x?}", &entry_bytes[25..29]);
    
    // Calculate what 2022.12.19 should encode to using different possible formats
    let date_2022_12_19_standard = (2022u32 << 9) | (12u32 << 5) | 19u32;
    let date_2022_12_19_with_offset = ((2022u32 - 1408) << 9) | (12u32 << 5) | 19u32; // Try reverse offset
    let date_2022_12_19_alt = (2022u32 << 16) | (12u32 << 8) | 19u32; // Try different bit layout
    
    println!("Expected patterns for 2022.12.19:");
    println!("  Standard SCID: 0x{:08x}", date_2022_12_19_standard);
    println!("  With -1408 offset: 0x{:08x}", date_2022_12_19_with_offset);
    println!("  Alt encoding: 0x{:08x}", date_2022_12_19_alt);
    
    // Search for ANY pattern containing the bytes 19, 12, or components of 2022
    println!("Searching for date components (19, 12, 2022) in all 4-byte windows:");
    for i in 0..=entry_bytes.len()-4 {
        let pattern = u32::from_be_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
        let b0 = entry_bytes[i];
        let b1 = entry_bytes[i+1];
        let b2 = entry_bytes[i+2];
        let b3 = entry_bytes[i+3];
        
        // Check if this 4-byte window contains our target values
        if (b0 == 19 || b1 == 19 || b2 == 19 || b3 == 19) &&
           (b0 == 12 || b1 == 12 || b2 == 12 || b3 == 12) {
            println!("  Offset {}: 0x{:08x} (bytes: {} {} {} {}) - contains 19 and 12", 
                i, pattern, b0, b1, b2, b3);
        }
        
        // Check for 2022 components
        let w0 = u16::from_be_bytes([b0, b1]);
        let w1 = u16::from_be_bytes([b2, b3]);
        if w0 == 2022 || w1 == 2022 {
            println!("  Offset {}: 0x{:08x} (words: {} {}) - contains 2022", 
                i, pattern, w0, w1);
        }
        
        // Check against our calculated patterns
        if pattern == date_2022_12_19_standard || pattern == date_2022_12_19_with_offset || pattern == date_2022_12_19_alt {
            println!("  Offset {}: 0x{:08x} - MATCHES calculated pattern!", i, pattern);
        }
    }
    
    // Search for the old hardcoded pattern too
    let target_pattern = 0x0944cd93u32;
    for i in 0..=entry_bytes.len()-4 {
        let pattern = u32::from_be_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
        if pattern == target_pattern {
            println!("Found old hardcoded pattern at offset {}: 0x{:08x}", i, pattern);
        }
    }
    
    // Parse using cursor for easier reading
    let mut cursor = std::io::Cursor::new(entry_bytes);
    
    // Offset (4 bytes) - SCID uses big-endian for all multi-byte values
    let offset = read_u32_be(&mut cursor)?;
    
    // Length (2 + 1 bytes combined) - SCID uses big-endian
    let length_low = read_u16_be(&mut cursor)?;
    let length_high = read_u8(&mut cursor)?;
    let length = parse_game_length(length_low, length_high);
    
    // Flags (2 bytes) - SCID uses big-endian
    let flags = read_u16_be(&mut cursor)?;
    let parsed_flags = parse_game_flags(flags);
    
    // Player IDs - packed format - SCID uses big-endian for 2-byte values
    let white_black_high = read_u8(&mut cursor)?;
    let white_id_low = read_u16_be(&mut cursor)?;
    let black_id_low = read_u16_be(&mut cursor)?;
    let (white_id, black_id) = parse_player_ids(white_black_high, white_id_low, black_id_low);
    
    let event_site_rnd_high = read_u8(&mut cursor)?;
    let event_id_low = read_u16_be(&mut cursor)?;
    let site_id_low = read_u16_be(&mut cursor)?;
    let round_id_low = read_u16_be(&mut cursor)?;
    
    // Parse Event/Site/Round IDs using correct SCID logic
    let (event_id, site_id, round_id) = parse_event_site_round_ids(event_site_rnd_high, event_id_low, site_id_low, round_id_low);
    
    // VarCounts and ECO (2 + 2 bytes) - SCID uses big-endian
    let var_counts = read_u16_be(&mut cursor)?;
    let eco = read_u16_be(&mut cursor)?;
    
    // CORRECT APPROACH: Read date from offset 25-28 as per SCID IndexEntry::Read()
    // Based on IndexEntry::Read() analysis:
    // Offset(4) + Length_Low(2) + Length_High(1) + Flags(2) + WhiteBlack_High(1) + 
    // WhiteID_Low(2) + BlackID_Low(2) + EventSiteRnd_High(1) + EventID_Low(2) + 
    // SiteID_Low(2) + RoundID_Low(2) + VarCounts(2) + EcoCode(2) = 25 bytes
    // Then Dates = fp->ReadFourBytes() at offset 25-28
    
    // Dates field uses big-endian like all SCID multi-byte values
    let dates_field = u32::from_be_bytes([entry_bytes[25], entry_bytes[26], entry_bytes[27], entry_bytes[28]]);
    println!("SCID Dates field at offset 25-28: 0x{:08x}", dates_field);
    
    // Extract game date from lower 20 bits (as per SCID source: u32_low_20)
    let game_date = dates_field & 0x000FFFFF;
    println!("Game date (lower 20 bits): 0x{:05x}", game_date);
    
    // Decode using exact SCID format with NO year offset (as per SCID source)
    let day = (game_date & 31) as u8;                    // Bits 0-4
    let month = ((game_date >> 5) & 15) as u8;           // Bits 5-8  
    let year = ((game_date >> 9) & 0x7FF) as u16;        // Bits 9-19, NO OFFSET
    
    println!("Decoded with NO offset: {}.{:02}.{:02}", year, month, day);
    
    // If this doesn't give 2022.12.19, then we need to look elsewhere
    if year == 2022 && month == 12 && day == 19 {
        println!("SUCCESS! Found correct 2022.12.19 date");
    } else {
        println!("Still wrong date - need to investigate further");
        
        // Let's also check what the expected 2022.12.19 pattern should be
        let expected_pattern = (2022u32 << 9) | (12u32 << 5) | 19u32;
        println!("Expected pattern for 2022.12.19: 0x{:08x}", expected_pattern);
        
        // Search for this pattern in the entire entry
        for i in 0..=entry_bytes.len()-4 {
            let pattern = u32::from_be_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
            if (pattern & 0x000FFFFF) == expected_pattern {
                println!("Found 2022.12.19 pattern at offset {}: 0x{:08x}", i, pattern);
            }
        }
    }
    
    // Also read the "official" dates field that cursor is pointing to for comparison
    let dates_at_cursor = read_u32_be(&mut cursor)?;
    println!("Date at cursor pos: 0x{:08x}", dates_at_cursor);
    
    // ELO ratings (2 + 2 bytes) - SCID uses big-endian
    let white_elo_raw = read_u16_be(&mut cursor)?;
    let black_elo_raw = read_u16_be(&mut cursor)?;
    let white_elo = white_elo_raw & 0x0FFF;
    let black_elo = black_elo_raw & 0x0FFF;
    
    // Skip remaining fields for now - SCID uses big-endian
    let _final_mat_sig = read_u32_be(&mut cursor)?;
    let num_half_moves_low = read_u8(&mut cursor)?;
    
    // Skip home pawn data (9 bytes)
    let mut _home_pawn_data = [0u8; 9];
    cursor.read_exact(&mut _home_pawn_data)?;
    
    // Calculate full num_half_moves (high bits are in home_pawn_data[0])
    let num_half_moves = num_half_moves_low as u16 | (((_home_pawn_data[0] >> 6) as u16) << 8);
    
    // Extract result from VarCounts (top 4 bits)
    let result = (var_counts >> 12) as u8;
    
    Ok(GameIndex {
        offset,
        length,
        white_id,
        black_id,
        event_id,
        site_id,
        round_id,
        year,
        month,
        day,
        result,
        eco,
        white_elo,
        black_elo,
        flags,
        parsed_flags,
        num_half_moves,
    })
}