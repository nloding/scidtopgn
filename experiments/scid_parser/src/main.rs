use std::env;
use std::fs::File;
use std::io::{self, Read, BufReader};
use std::path::Path;

#[derive(Debug)]
struct ScidHeader {
    magic: [u8; 8],
    version: u16,
    base_type: u32,
    num_games: u32,
    auto_load: u32,
    description: String,
    custom_flags: Vec<String>,
}

#[derive(Debug)]
struct GameIndex {
    offset: u32,
    length: u32,
    white_id: u32,
    black_id: u32,
    event_id: u32,
    site_id: u32,
    round_id: u32,
    year: u16,
    month: u8,
    day: u8,
    result: u8,
    eco: u16,
    white_elo: u16,
    black_elo: u16,
    flags: u16,
    num_half_moves: u16,
}

fn read_u8(reader: &mut impl Read) -> io::Result<u8> {
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn read_u16_le(reader: &mut impl Read) -> io::Result<u16> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u24_le(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 3];
    reader.read_exact(&mut buf)?;
    Ok((buf[0] as u32) | ((buf[1] as u32) << 8) | ((buf[2] as u32) << 16))
}

fn read_u32_le(reader: &mut impl Read) -> io::Result<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

fn read_string(reader: &mut impl Read, len: usize) -> io::Result<String> {
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    // Find first null byte and truncate there
    if let Some(null_pos) = buf.iter().position(|&b| b == 0) {
        buf.truncate(null_pos);
    }
    Ok(String::from_utf8_lossy(&buf).to_string())
}

fn parse_header(reader: &mut impl Read) -> io::Result<ScidHeader> {
    // Read magic header (8 bytes)
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    
    println!("Raw magic bytes: {:02x?}", magic);
    
    // Verify magic header
    let expected_magic = b"Scid.si\0";
    if magic != *expected_magic {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid magic header: {:?}", magic)
        ));
    }
    
    // Read version (2 bytes) - SCID: Header.version = FilePtr->ReadTwoBytes();
    let version = read_u16_le(reader)?;
    println!("Version: {} (0x{:04x})", version, version);
    
    // Read base type (4 bytes) - SCID: Header.baseType = FilePtr->ReadFourBytes();
    let base_type = read_u32_le(reader)?;
    println!("BaseType: {} (0x{:08x})", base_type, base_type);
    
    // Read num games (3 bytes) - SCID: Header.numGames = FilePtr->ReadThreeBytes();
    let num_games = read_u24_le(reader)?;
    println!("NumGames: {} (0x{:06x})", num_games, num_games);
    
    // Read auto load (3 bytes) - SCID: Header.autoLoad = FilePtr->ReadThreeBytes();
    let auto_load = read_u24_le(reader)?;
    println!("AutoLoad: {} (0x{:06x})", auto_load, auto_load);
    
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

fn parse_game_index(reader: &mut impl Read) -> io::Result<GameIndex> {
    // Read the 47-byte game index entry
    let mut entry_bytes = [0u8; 47];
    reader.read_exact(&mut entry_bytes)?;
    
    println!("Raw entry bytes (first 32): {:02x?}", &entry_bytes[0..32]);
    println!("Raw entry bytes (last 15): {:02x?}", &entry_bytes[32..47]);
    println!("Dates field bytes at offset 25-28: {:02x?}", &entry_bytes[25..29]);
    
    // Calculate what 2022.12.19 should encode to using different possible formats
    let date_2022_12_19_standard = ((2022u32 << 9) | (12u32 << 5) | 19u32);
    let date_2022_12_19_with_offset = (((2022u32 - 1408) << 9) | (12u32 << 5) | 19u32); // Try reverse offset
    let date_2022_12_19_alt = ((2022u32 << 16) | (12u32 << 8) | 19u32); // Try different bit layout
    
    println!("Expected patterns for 2022.12.19:");
    println!("  Standard SCID: 0x{:08x}", date_2022_12_19_standard);
    println!("  With -1408 offset: 0x{:08x}", date_2022_12_19_with_offset);
    println!("  Alt encoding: 0x{:08x}", date_2022_12_19_alt);
    
    // Search for ANY pattern containing the bytes 19, 12, or components of 2022
    println!("Searching for date components (19, 12, 2022) in all 4-byte windows:");
    for i in 0..=entry_bytes.len()-4 {
        let pattern = u32::from_le_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
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
        let w0 = u16::from_le_bytes([b0, b1]);
        let w1 = u16::from_le_bytes([b2, b3]);
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
        let pattern = u32::from_le_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
        if pattern == target_pattern {
            println!("Found old hardcoded pattern at offset {}: 0x{:08x}", i, pattern);
        }
    }
    
    // Parse using cursor for easier reading
    let mut cursor = std::io::Cursor::new(entry_bytes);
    
    // Offset (4 bytes)
    let offset = read_u32_le(&mut cursor)?;
    
    // Length (2 + 1 bytes combined)
    let length_low = read_u16_le(&mut cursor)?;
    let length_high = read_u8(&mut cursor)?;
    let length = length_low as u32 + ((length_high as u32 & 0x80) << 9);
    
    // Flags (2 bytes)
    let flags = read_u16_le(&mut cursor)?;
    
    // Name IDs - packed format
    let white_black_high = read_u8(&mut cursor)?;
    let white_id_low = read_u16_le(&mut cursor)?;
    let black_id_low = read_u16_le(&mut cursor)?;
    
    let event_site_rnd_high = read_u8(&mut cursor)?;
    let event_id_low = read_u16_le(&mut cursor)?;
    let site_id_low = read_u16_le(&mut cursor)?;
    let round_id_low = read_u16_le(&mut cursor)?;
    
    // Reconstruct packed IDs
    let white_id = ((white_black_high as u32 & 0xF0) << 12) | white_id_low as u32;
    let black_id = ((white_black_high as u32 & 0x0F) << 16) | black_id_low as u32;
    let event_id = ((event_site_rnd_high as u32 & 0xE0) << 11) | event_id_low as u32;
    let site_id = ((event_site_rnd_high as u32 & 0x1C) << 14) | site_id_low as u32;
    let round_id = ((event_site_rnd_high as u32 & 0x03) << 16) | round_id_low as u32;
    
    // VarCounts and ECO (2 + 2 bytes)
    let var_counts = read_u16_le(&mut cursor)?;
    let eco = read_u16_le(&mut cursor)?;
    
    // CORRECT APPROACH: Read date from offset 25-28 as per SCID IndexEntry::Read()
    // Based on IndexEntry::Read() analysis:
    // Offset(4) + Length_Low(2) + Length_High(1) + Flags(2) + WhiteBlack_High(1) + 
    // WhiteID_Low(2) + BlackID_Low(2) + EventSiteRnd_High(1) + EventID_Low(2) + 
    // SiteID_Low(2) + RoundID_Low(2) + VarCounts(2) + EcoCode(2) = 25 bytes
    // Then Dates = fp->ReadFourBytes() at offset 25-28
    
    let dates_field = u32::from_le_bytes([entry_bytes[25], entry_bytes[26], entry_bytes[27], entry_bytes[28]]);
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
        let expected_pattern = ((2022u32 << 9) | (12u32 << 5) | 19u32);
        println!("Expected pattern for 2022.12.19: 0x{:08x}", expected_pattern);
        
        // Search for this pattern in the entire entry
        for i in 0..=entry_bytes.len()-4 {
            let pattern = u32::from_le_bytes([entry_bytes[i], entry_bytes[i+1], entry_bytes[i+2], entry_bytes[i+3]]);
            if (pattern & 0x000FFFFF) == expected_pattern {
                println!("Found 2022.12.19 pattern at offset {}: 0x{:08x}", i, pattern);
            }
        }
    }
    
    // Also read the "official" dates field that cursor is pointing to for comparison
    let dates_at_cursor = read_u32_le(&mut cursor)?;
    println!("Date at cursor pos: 0x{:08x}", dates_at_cursor);
    
    // ELO ratings (2 + 2 bytes)
    let white_elo_raw = read_u16_le(&mut cursor)?;
    let black_elo_raw = read_u16_le(&mut cursor)?;
    let white_elo = white_elo_raw & 0x0FFF;
    let black_elo = black_elo_raw & 0x0FFF;
    
    // Skip remaining fields for now
    let _final_mat_sig = read_u32_le(&mut cursor)?;
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
        num_half_moves,
    })
}

// SCID date encoding functions (recreated from scidvspc source)
fn date_make(year: u32, month: u32, day: u32) -> u32 {
    ((year << 9) | (month << 5) | day)
}

fn date_get_year(date: u32) -> u32 {
    date >> 9
}

fn date_get_month(date: u32) -> u32 {
    (date >> 5) & 15
}

fn date_get_day(date: u32) -> u32 {
    date & 31
}

fn u32_set_low_20(u: u32, x: u32) -> u32 {
    (u & 0xFFF00000) | (x & 0x000FFFFF)
}

fn u32_set_high_12(u: u32, x: u32) -> u32 {
    (u & 0x000FFFFF) | ((x & 0xFFF) << 20)
}

// Recreate IndexEntry::SetDate() from SCID source
fn scid_set_date(existing_dates: u32, year: u32, month: u32, day: u32) -> u32 {
    let date = date_make(year, month, day);
    u32_set_low_20(existing_dates, date)
}

// Recreate IndexEntry::SetEventDate() from SCID source - EXACT C++ implementation
// C++ signature: void IndexEntry::SetEventDate(dateT edate)
// This method takes a single encoded date parameter and extracts month/day using helper functions
fn scid_set_event_date(existing_dates: u32, edate: u32) -> u32 {
    // Extract game date from existing_dates (lower 20 bits) for comparison
    let game_date = existing_dates & 0x000FFFFF;
    
    // Follow exact C++ implementation from index.cpp:137-151
    let mut coded_date = date_get_month(edate) << 5;
    coded_date |= date_get_day(edate);
    let eyear = date_get_year(edate);
    let dyear = date_get_year(game_date);
    
    let eyear = if eyear < (dyear - 3) || eyear > (dyear + 3) {
        0  // Event year too far from game date, set to 0
    } else {
        eyear
    };
    
    if eyear == 0 {
        coded_date = 0;
    } else {
        coded_date |= (((eyear + 4 - dyear) & 7) << 9);
    }
    
    u32_set_high_12(existing_dates, coded_date)
}

// Recreate IndexEntry::GetEventDate() from SCID source - EXACT C++ implementation
fn scid_get_event_date(dates_field: u32) -> Option<(u32, u32, u32)> {
    let game_date = dates_field & 0x000FFFFF;  // Lower 20 bits
    let dyear = date_get_year(game_date);
    let edate = (dates_field >> 20) & 0xFFF;   // Upper 12 bits (u32_high_12)
    
    if edate == 0 {
        return None;  // ZERO_DATE equivalent
    }
    
    let month = date_get_month(edate);
    let day = date_get_day(edate);
    let year_offset = date_get_year(edate) & 7;  // Extract 3-bit year offset
    
    if year_offset == 0 {
        return None;
    }
    
    let year = dyear + year_offset - 4;  // Convert offset back to actual year
    Some((year, month, day))
}

fn encode_date_command(year: u32, month: u32, day: u32) {
    println!("=== SCID DATE ENCODING TEST ===");
    println!("Input date: {}.{:02}.{:02}", year, month, day);
    
    // Create the date using SCID's DATE_MAKE
    let date_value = date_make(year, month, day);
    println!("DATE_MAKE result: 0x{:08x}", date_value);
    
    // Test decoding
    let decoded_year = date_get_year(date_value);
    let decoded_month = date_get_month(date_value);  
    let decoded_day = date_get_day(date_value);
    println!("Decoded back: {}.{:02}.{:02}", decoded_year, decoded_month, decoded_day);
    
    // Show how it would be stored in the Dates field (lower 20 bits)
    let dates_field = scid_set_date(0, year, month, day);
    println!("Dates field (game date in lower 20 bits): 0x{:08x}", dates_field);
    
    // Show little-endian bytes
    let bytes = dates_field.to_le_bytes();
    println!("Little-endian bytes: [{:02x}, {:02x}, {:02x}, {:02x}]", bytes[0], bytes[1], bytes[2], bytes[3]);
    
    println!();
}

fn test_set_event_date_command(game_year: u32, game_month: u32, game_day: u32, event_year: u32, event_month: u32, event_day: u32) {
    println!("=== SCID SetEventDate TEST ===");
    println!("Game date: {}.{:02}.{:02}", game_year, game_month, game_day);
    println!("Event date: {}.{:02}.{:02}", event_year, event_month, event_day);
    
    // Create game date using SCID encoding
    let game_date_encoded = date_make(game_year, game_month, game_day);
    println!("Game date encoded: 0x{:08x}", game_date_encoded);
    
    // Create initial Dates field with game date in lower 20 bits
    let initial_dates = scid_set_date(0, game_year, game_month, game_day);
    println!("Initial Dates field: 0x{:08x}", initial_dates);
    
    // Create event date using SCID encoding
    let event_date_encoded = date_make(event_year, event_month, event_day);
    println!("Event date encoded: 0x{:08x}", event_date_encoded);
    
    // Apply SetEventDate method (takes single encoded date parameter)
    let final_dates = scid_set_event_date(initial_dates, event_date_encoded);
    println!("Final Dates field after SetEventDate: 0x{:08x}", final_dates);
    
    // Verify the encoding by extracting event date from upper 12 bits
    let encoded_event_date = (final_dates >> 20) & 0xFFF;
    println!("Encoded event date in upper 12 bits: 0x{:03x}", encoded_event_date);
    
    // Test decoding with GetEventDate to verify round-trip accuracy
    match scid_get_event_date(final_dates) {
        Some((decoded_year, decoded_month, decoded_day)) => {
            println!("Decoded event date: {}.{:02}.{:02}", decoded_year, decoded_month, decoded_day);
            if decoded_year == event_year && decoded_month == event_month && decoded_day == event_day {
                println!("✅ SUCCESS: Event date round-trip is correct!");
            } else {
                println!("❌ ERROR: Event date round-trip failed!");
                println!("  Expected: {}.{:02}.{:02}", event_year, event_month, event_day);
                println!("  Got:      {}.{:02}.{:02}", decoded_year, decoded_month, decoded_day);
            }
        }
        None => {
            println!("❌ Event date decoding returned None (ZERO_DATE or invalid)");
            if (event_year < (game_year - 3)) || (event_year > (game_year + 3)) {
                println!("✅ This is expected because event year {} is too far from game year {}", event_year, game_year);
            }
        }
    }
    
    println!();
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    // Check if this is a date encoding command
    if args.len() == 5 && args[1] == "encode" {
        let year: u32 = args[2].parse().expect("Invalid year");
        let month: u32 = args[3].parse().expect("Invalid month");
        let day: u32 = args[4].parse().expect("Invalid day");
        encode_date_command(year, month, day);
        return Ok(());
    }
    
    // Check if this is a SetEventDate test command
    if args.len() == 8 && args[1] == "test-event-date" {
        let game_year: u32 = args[2].parse().expect("Invalid game year");
        let game_month: u32 = args[3].parse().expect("Invalid game month");
        let game_day: u32 = args[4].parse().expect("Invalid game day");
        let event_year: u32 = args[5].parse().expect("Invalid event year");
        let event_month: u32 = args[6].parse().expect("Invalid event month");
        let event_day: u32 = args[7].parse().expect("Invalid event day");
        test_set_event_date_command(game_year, game_month, game_day, event_year, event_month, event_day);
        return Ok(());
    }
    
    if args.len() != 2 {
        eprintln!("Usage:");
        eprintln!("  {} <path_to_si4_file>   - Parse SCID index file", args[0]);
        eprintln!("  {} encode <year> <month> <day>   - Encode date using SCID format", args[0]);
        eprintln!("  {} test-event-date <game_y> <game_m> <game_d> <event_y> <event_m> <event_d>   - Test SetEventDate method", args[0]);
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    println!("Reading SCID index file: {}", file_path);
    
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);
    
    // Parse header
    let header = parse_header(&mut reader)?;
    
    println!("\n=== SCID DATABASE HEADER ===");
    println!("Magic: {:?}", std::str::from_utf8(&header.magic).unwrap_or("invalid"));
    println!("Version: {}", header.version);
    println!("Base Type: {}", header.base_type);
    println!("Number of Games: {}", header.num_games);
    println!("Auto Load: {}", header.auto_load);
    println!("Description: {}", header.description);
    println!("Custom Flags: {:?}", header.custom_flags);
    
    println!("\n=== GAME ENTRIES ===");
    
    // Parse up to 10 games for testing
    let games_to_parse = std::cmp::min(header.num_games, 10);
    
    for i in 0..games_to_parse {
        match parse_game_index(&mut reader) {
            Ok(game) => {
                println!("Game {}: {}.{:02}.{:02} - White:{} Black:{} Event:{} Site:{} Round:{} - Result:{} ELO:{}vs{} Moves:{}", 
                    i + 1,
                    game.year, game.month, game.day,
                    game.white_id, game.black_id, 
                    game.event_id, game.site_id, game.round_id,
                    game.result, game.white_elo, game.black_elo,
                    game.num_half_moves
                );
            }
            Err(e) => {
                eprintln!("Error parsing game {}: {}", i + 1, e);
                break;
            }
        }
    }
    
    Ok(())
}
