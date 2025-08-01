/// SCID date encoding and decoding functions
/// Based on SCID source code from scidvspc/src/index.cpp and date.h

// SCID date encoding functions (recreated from scidvspc source)
pub fn date_make(year: u32, month: u32, day: u32) -> u32 {
    (year << 9) | (month << 5) | day
}

pub fn date_get_year(date: u32) -> u32 {
    date >> 9
}

pub fn date_get_month(date: u32) -> u32 {
    (date >> 5) & 15
}

pub fn date_get_day(date: u32) -> u32 {
    date & 31
}

pub fn u32_set_low_20(u: u32, x: u32) -> u32 {
    (u & 0xFFF00000) | (x & 0x000FFFFF)
}

pub fn u32_set_high_12(u: u32, x: u32) -> u32 {
    (u & 0x000FFFFF) | ((x & 0xFFF) << 20)
}

// Recreate IndexEntry::SetDate() from SCID source
pub fn scid_set_date(existing_dates: u32, year: u32, month: u32, day: u32) -> u32 {
    let date = date_make(year, month, day);
    u32_set_low_20(existing_dates, date)
}

// Recreate IndexEntry::SetEventDate() from SCID source - EXACT C++ implementation
// C++ signature: void IndexEntry::SetEventDate(dateT edate)
// This method takes a single encoded date parameter and extracts month/day using helper functions
pub fn scid_set_event_date(existing_dates: u32, edate: u32) -> u32 {
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
        coded_date |= ((eyear + 4 - dyear) & 7) << 9;
    }
    
    u32_set_high_12(existing_dates, coded_date)
}

// Recreate IndexEntry::GetEventDate() from SCID source - EXACT C++ implementation
pub fn scid_get_event_date(dates_field: u32) -> Option<(u32, u32, u32)> {
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

pub fn encode_date_command(date_string: &str) {
    // Parse date string in format YYYY.MM.DD
    let parts: Vec<&str> = date_string.split('.').collect();
    if parts.len() != 3 {
        eprintln!("Error: Date must be in format YYYY.MM.DD (e.g., 2022.12.19)");
        return;
    }
    
    let year: u32 = match parts[0].parse() {
        Ok(y) => y,
        Err(_) => {
            eprintln!("Error: Invalid year '{}'", parts[0]);
            return;
        }
    };
    
    let month: u32 = match parts[1].parse() {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Error: Invalid month '{}'", parts[1]);
            return;
        }
    };
    
    let day: u32 = match parts[2].parse() {
        Ok(d) => d,
        Err(_) => {
            eprintln!("Error: Invalid day '{}'", parts[2]);
            return;
        }
    };
    
    // Validate ranges
    if year < 1000 || year > 2047 {
        eprintln!("Error: Year must be between 1000 and 2047 (SCID 11-bit limit)");
        return;
    }
    if month < 1 || month > 12 {
        eprintln!("Error: Month must be between 1 and 12");
        return;
    }
    if day < 1 || day > 31 {
        eprintln!("Error: Day must be between 1 and 31");
        return;
    }
    
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
    
    // Verify round-trip
    if decoded_year == year && decoded_month == month && decoded_day == day {
        println!("✅ SUCCESS: Date encoding/decoding works correctly!");
    } else {
        println!("❌ ERROR: Date encoding/decoding failed!");
    }
    
    println!();
}

pub fn test_set_event_date_command(game_year: u32, game_month: u32, game_day: u32, event_year: u32, event_month: u32, event_day: u32) {
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