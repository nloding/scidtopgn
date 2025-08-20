// Date encoding command
use crate::date::*;

pub fn execute(date_str: &str) -> std::io::Result<()> {
    encode_date_command(date_str);
    Ok(())
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