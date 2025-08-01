/// SCID .sn4 name file parsing
use std::io::{self, Read};
use crate::utils::*;

/// SCID namebase header structure - based on nameBaseHeaderT in namebase.h
#[derive(Debug)]
pub struct Sn4Header {
    pub magic: [u8; 8],                    // "Scid.sn" magic identifier
    pub timestamp: u32,                    // 4-byte timestamp
    pub num_names_player: u32,             // Number of player names (3 bytes)
    pub num_names_event: u32,              // Number of event names (3 bytes)
    pub num_names_site: u32,               // Number of site names (3 bytes)
    pub num_names_round: u32,              // Number of round names (3 bytes)
    pub max_frequency_player: u32,         // Max frequency for players (3 bytes)
    pub max_frequency_event: u32,          // Max frequency for events (3 bytes)
    pub max_frequency_site: u32,           // Max frequency for sites (3 bytes)
    pub max_frequency_round: u32,          // Max frequency for rounds (3 bytes)
}

/// Parse sn4 header - based on NameBase::OpenNameFile() in namebase.cpp
pub fn parse_sn4_header(reader: &mut impl Read) -> io::Result<Sn4Header> {
    // Read magic (8 bytes)
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic)?;
    
    // Read timestamp (4 bytes, big-endian) - uses readFourBytes()
    let timestamp = read_u32_be(reader)?;
    
    // Read numNames for each type (3 bytes each, big-endian) - uses readThreeBytes()
    let num_names_player = read_u24_be(reader)?;
    let num_names_event = read_u24_be(reader)?;
    let num_names_site = read_u24_be(reader)?;
    let num_names_round = read_u24_be(reader)?;
    
    // Read maxFrequency for each type (3 bytes each, big-endian) - uses readThreeBytes()
    let max_frequency_player = read_u24_be(reader)?;
    let max_frequency_event = read_u24_be(reader)?;
    let max_frequency_site = read_u24_be(reader)?;
    let max_frequency_round = read_u24_be(reader)?;
    
    Ok(Sn4Header {
        magic,
        timestamp,
        num_names_player,
        num_names_event,
        num_names_site,
        num_names_round,
        max_frequency_player,
        max_frequency_event,
        max_frequency_site,
        max_frequency_round,
    })
}

/// Display the structure of SCID namebase header (like the si4 structure table)
pub fn display_sn4_header_structure() {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                       SCID NAMEBASE HEADER STRUCTURE                       │");
    println!("│                             (36 bytes total)                               │");
    println!("├────────┬──────────┬─────────┬─────────────────────────────────────────────┤");
    println!("│ Offset │ Size     │ Format  │ Field Description                           │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────┤");
    println!("│   0-7  │ 8 bytes  │ string  │ Magic identifier (\"Scid.sn\")                │");
    println!("│   8-11 │ 4 bytes  │ BE uint │ Timestamp                                   │");
    println!("│  12-14 │ 3 bytes  │ BE uint │ Number of Player Names                      │");
    println!("│  15-17 │ 3 bytes  │ BE uint │ Number of Event Names                       │");
    println!("│  18-20 │ 3 bytes  │ BE uint │ Number of Site Names                        │");
    println!("│  21-23 │ 3 bytes  │ BE uint │ Number of Round Names                       │");
    println!("│  24-26 │ 3 bytes  │ BE uint │ Max Frequency for Players                   │");
    println!("│  27-29 │ 3 bytes  │ BE uint │ Max Frequency for Events                    │");
    println!("│  30-32 │ 3 bytes  │ BE uint │ Max Frequency for Sites                     │");
    println!("│  33-35 │ 3 bytes  │ BE uint │ Max Frequency for Rounds                    │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────┘");
    println!();
    println!("Notes:");
    println!("• Based on nameBaseHeaderT structure in namebase.h");
    println!("• All multi-byte integers use big-endian byte order (consistent with .si4)");
    println!("• Uses readThreeBytes() and readFourBytes() functions from mfile.cpp");
    println!("• Name data follows immediately after this 36-byte header");
    println!();
}

/// Display parsed sn4 header values
pub fn display_sn4_header_values(header: &Sn4Header) {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                      PARSED NAMEBASE HEADER VALUES                         │");
    println!("├─────────────────────────┬───────────────────────────────────────────────────┤");
    println!("│ Field                   │ Value                                             │");
    println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
    println!("│ Magic                   │ {:<49} │", std::str::from_utf8(&header.magic).unwrap_or("invalid"));
    println!("│ Timestamp               │ {:<49} │", header.timestamp);
    println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
    println!("│ Player Names Count      │ {:<49} │", header.num_names_player);
    println!("│ Event Names Count       │ {:<49} │", header.num_names_event);
    println!("│ Site Names Count        │ {:<49} │", header.num_names_site);
    println!("│ Round Names Count       │ {:<49} │", header.num_names_round);
    println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
    println!("│ Player Max Frequency    │ {:<49} │", header.max_frequency_player);
    println!("│ Event Max Frequency     │ {:<49} │", header.max_frequency_event);
    println!("│ Site Max Frequency      │ {:<49} │", header.max_frequency_site);
    println!("│ Round Max Frequency     │ {:<49} │", header.max_frequency_round);
    println!("└─────────────────────────┴───────────────────────────────────────────────────┘");
    println!();
}

/// Display the structure of SCID name records (follows header)
pub fn display_name_record_structure() {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                        SCID NAME RECORD STRUCTURE                          │");
    println!("│                           (variable size per record)                       │");
    println!("├────────┬──────────┬─────────┬─────────────────────────────────────────────┤");
    println!("│ Offset │ Size     │ Format  │ Field Description                           │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────┤");
    println!("│   0-N  │ 2-3 bytes│ BE uint │ Name ID (2 bytes if count<65536, else 3)   │");
    println!("│   N-M  │ 1-3 bytes│ BE uint │ Frequency (1/2/3 bytes based on max freq)  │");
    println!("│   M    │ 1 byte   │ uint8   │ Total string length                         │");
    println!("│   M+1  │ 1 byte   │ uint8   │ Prefix length (front-coding, skip if 1st)  │");
    println!("│   M+2+ │ variable │ string  │ String suffix data (UTF-8)                  │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────┘");
    println!();
    println!("Notes:");
    println!("• Based on NameBase::ReadNameFile() in namebase.cpp lines 181-221");
    println!("• ID size: 2 bytes if numNames < 65536, otherwise 3 bytes");
    println!("• Frequency size: 1 byte if maxFreq < 256, 2 bytes if < 65536, else 3 bytes");
    println!("• First record has no prefix byte (prefix = 0)");
    println!("• Subsequent records use front-coding: prefix + new suffix");
    println!("• Names stored in sections: Players, Events, Sites, Rounds (in that order)");
    println!();
}

/// Placeholder structure for a parsed name record
#[derive(Debug)]
pub struct NameRecord {
    pub id: u32,
    pub frequency: u32,
    pub name: String,
}

/// Parse a complete name record sequentially (based on namebase.cpp lines 181-221)
/// Implements front-coded string reconstruction as per SCID source code
pub fn parse_name_record_sequential(
    reader: &mut impl Read, 
    record_index: u32,
    num_names: u32, 
    max_frequency: u32,
    previous_name: &str
) -> io::Result<NameRecord> {
    // Parse ID field (2 or 3 bytes based on total count)
    let id = if num_names >= 65536 {
        read_u24_be(reader)?
    } else {
        read_u16_be(reader)? as u32
    };
    
    // Parse frequency field (1, 2, or 3 bytes based on max frequency)
    let frequency = if max_frequency >= 65536 {
        read_u24_be(reader)?
    } else if max_frequency >= 256 {
        read_u16_be(reader)? as u32
    } else {
        read_u8(reader)? as u32
    };
    
    // Parse string data using front-coding algorithm from namebase.cpp lines 202-221
    let total_length = read_u8(reader)? as usize;
    
    let prefix_length = if record_index > 0 {
        read_u8(reader)? as usize
    } else {
        0 // First record has no prefix byte (namebase.cpp line 209)
    };
    
    if prefix_length > total_length {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid prefix length {} > total length {}", prefix_length, total_length)
        ));
    }
    
    let suffix_length = total_length - prefix_length;
    
    // Read suffix bytes
    let mut suffix_bytes = vec![0u8; suffix_length];
    reader.read_exact(&mut suffix_bytes)?;
    
    // Reconstruct name using front-coding (namebase.cpp lines 212-221)
    let mut name_bytes = Vec::with_capacity(total_length);
    
    // Copy prefix from previous name (namebase.cpp lines 212-216)
    if prefix_length > 0 {
        let previous_bytes = previous_name.as_bytes();
        if prefix_length > previous_bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Prefix length {} exceeds previous name length {}", 
                    prefix_length, previous_bytes.len())
            ));
        }
        name_bytes.extend_from_slice(&previous_bytes[..prefix_length]);
    }
    
    // Append suffix (namebase.cpp line 218)
    name_bytes.extend_from_slice(&suffix_bytes);
    
    // Convert to string, handling potential UTF-8 issues and control characters
    let name = String::from_utf8_lossy(&name_bytes)
        .trim_end_matches('\0')  // Remove null terminators
        .chars()
        .filter(|&c| c >= ' ' || c == '\t' || c == '\n')  // Filter control chars except tab/newline
        .collect::<String>()
        .trim()
        .to_string();
    
    Ok(NameRecord {
        id,
        frequency,
        name,
    })
}

/// Display parsed name record values 
pub fn display_name_record_values(record_num: usize, name_type: &str, record: Option<&NameRecord>) {
    println!();
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│                   {} RECORD {}                            │", 
        format!("{:<15}", name_type.to_uppercase()),
        record_num + 1);
    println!("├─────────────────────────┬───────────────────────────────────────────────────┤");
    println!("│ Field                   │ Value                                             │");
    println!("├─────────────────────────┼───────────────────────────────────────────────────┤");
    println!("│ ID                      │ {:<49} │", 
        if let Some(rec) = record { 
            format!("{}", rec.id) 
        } else { 
            "[PARSE ERROR]".to_string() 
        });
    println!("│ Frequency               │ {:<49} │", 
        if let Some(rec) = record { 
            format!("{}", rec.frequency)
        } else { 
            "[PARSE ERROR]".to_string() 
        });
    println!("│ Name                    │ {:<49} │", 
        if let Some(rec) = record { 
            format!("{}", rec.name) 
        } else { 
            "[PARSE ERROR]".to_string() 
        });
    println!("│ Parsed successfully     │ {:<49} │", "Front-coded string reconstructed");
    println!("└─────────────────────────┴───────────────────────────────────────────────────┘");
    println!();
}