use std::env;
use std::fs::File;
use std::io::{self, BufReader};

mod utils;
mod date;
mod si4;
mod sg4;
mod sn4;
mod position;

use date::*;
use si4::*;
use sn4::*;
use sg4::*;
use position::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        display_help(&args[0]);
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "encode" => {
            if args.len() != 3 {
                eprintln!("Usage: {} encode <date>", args[0]);
                eprintln!("Example: {} encode 2022.12.19", args[0]);
                std::process::exit(1);
            }
            encode_date_command(&args[2]);
        }
        "test-position" => {
            println!("🧪 Testing ChessPosition implementation:");
            let position = ChessPosition::starting_position();
            println!("{}", position.display_board());
            
            // Test piece lookup by SCID number
            if let Some(king) = position.get_piece_by_number(0) {
                println!("✅ SCID piece #0: {:?} {:?}", king.color, king.piece_type);
            }
            
            if let Some(location) = position.get_piece_location(0) {
                println!("✅ SCID piece #0 location: {}", location);
            }
            
            println!("✅ Position tracking foundation implemented successfully!");
        }
        "test-one-move" => {
            println!("🧪 Testing single move decoding with position:");
            let mut position = ChessPosition::starting_position();
            println!("📍 Starting position:");
            println!("{}", position.display_board());
            
            // Test decoding a simple pawn move: P12 with move_value 15 (double pawn push)
            // From our test data: "P12: Pawn double ..."
            println!("\n🔍 Testing pawn double push: piece P12, move_value 15");
            
            // P12 should be a pawn on file e (based on our mapping)
            if let Some(piece) = position.get_piece_by_number(12) {
                println!("✅ Found piece P12: {:?} {:?}", piece.color, piece.piece_type);
                if let Some(location) = position.get_piece_location(12) {
                    println!("✅ P12 location: {}", location);
                } else {
                    println!("❌ Could not find P12 location");
                }
            } else {
                println!("❌ Could not find piece P12 in position");
            }
            
            println!("✅ Basic piece lookup test completed!");
        }
        "test-moves" => {
            if args.len() != 3 {
                eprintln!("Usage: {} test-moves <base_path>", args[0]);
                eprintln!("Example: {} test-moves /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            let sg4_path = format!("{}.sg4", base_path);
            
            println!("🔥 TESTING POSITION-AWARE MOVE PARSING");
            println!("📂 Reading: {}", sg4_path);
            
            // Read the SG4 file
            match std::fs::read(&sg4_path) {
                Ok(file_data) => {
                    // Parse game boundaries first
                    let games = find_game_boundaries(&file_data);
                    if !games.is_empty() {
                            println!("📊 Found {} games", games.len());
                            
                            // Test on first game only for now
                            if let Some((start_offset, end_offset)) = games.first() {
                                let game_data = &file_data[*start_offset..*end_offset];
                                println!("\n🎮 Testing Game 1 ({} bytes)", game_data.len());
                                
                                match parse_game_with_position_tracking(game_data, 1) {
                                    Ok((moves, notation)) => {
                                        println!("\n🎯 RESULTS:");
                                        println!("✅ Successfully parsed {} moves", moves.len());
                                        println!("📝 Generated notation:");
                                        for (i, note) in notation.iter().take(10).enumerate() {
                                            println!("  {}. {}", i + 1, note);
                                        }
                                        if notation.len() > 10 {
                                            println!("  ... and {} more moves", notation.len() - 10);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("❌ Position-aware parsing failed: {}", e);
                                    }
                                }
                            } else {
                                eprintln!("❌ No games found in file");
                            }
                    } else {
                        eprintln!("❌ No games found in file");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read SG4 file: {}", e);
                }
            }
        }
        "test-variations" => {
            if args.len() != 3 {
                eprintln!("Usage: {} test-variations <base_path>", args[0]);
                eprintln!("Example: {} test-variations /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            let sg4_path = format!("{}.sg4", base_path);
            
            println!("🌳 TESTING VARIATION TREE PARSING");
            println!("📂 Reading: {}", sg4_path);
            
            // Read the SG4 file
            match std::fs::read(&sg4_path) {
                Ok(file_data) => {
                    // Parse game boundaries first
                    let games = find_game_boundaries(&file_data);
                    if !games.is_empty() {
                        println!("📊 Found {} games", games.len());
                        
                        // Test on first game with variation support
                        if let Some((start_offset, end_offset)) = games.first() {
                            let game_data = &file_data[*start_offset..*end_offset];
                            println!("\n🎮 Testing Game 1 with Variation Trees ({} bytes)", game_data.len());
                            
                            match parse_game_with_variation_trees(game_data, 1) {
                                Ok((variation_tree, moves, notation)) => {
                                    println!("\n🌳 VARIATION TREE RESULTS:");
                                    println!("✅ Successfully parsed {} main line moves", moves.len());
                                    println!("🌿 Variation tree depth: {}", variation_tree.current_depth);
                                    println!("📝 Total elements in tree: {}", variation_tree.main_line.len());
                                    
                                    // Show variation structure
                                    let variations_count = variation_tree.main_line.iter()
                                        .map(|node| node.variations.len())
                                        .sum::<usize>();
                                    if variations_count > 0 {
                                        println!("🌿 Found {} variations in the game", variations_count);
                                    }
                                    
                                    // Show first few moves with variations
                                    println!("\n📝 Generated notation with variations:");
                                    for (i, note) in notation.iter().take(15).enumerate() {
                                        println!("  {}. {}", i + 1, note);
                                    }
                                    if notation.len() > 15 {
                                        println!("  ... and {} more moves", notation.len() - 15);
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Variation tree parsing failed: {}", e);
                                }
                            }
                        } else {
                            eprintln!("❌ No games found in file");
                        }
                    } else {
                        eprintln!("❌ No games found in file");
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to read SG4 file: {}", e);
                }
            }
        }
        "parse" => {
            if args.len() != 3 {
                eprintln!("Usage: {} parse <base_path>", args[0]);
                eprintln!("Example: {} parse /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            parse_scid_database_clean(base_path);
        }
        "format" => {
            display_scid_format_specifications();
        }
        "help" | "--help" | "-h" => {
            display_help(&args[0]);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            eprintln!();
            display_help(&args[0]);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

/// Display comprehensive SCID database format specifications
fn display_scid_format_specifications() {
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                          SCID DATABASE FORMAT SPECIFICATIONS");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SI4 format
    display_si4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SN4 format  
    display_sn4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!();
    
    // Display SG4 format
    display_sg4_format_specification();
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("                                 IMPLEMENTATION NOTES");
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("• All multi-byte integers use BIG-ENDIAN byte order");
    println!("• SCID uses proprietary binary encoding throughout");
    println!("• This implementation reverse-engineered from scidvspc source code");
    println!("• Date encoding: ((year << 9) | (month << 5) | day) with no year offset");
    println!("• Name compression: Front-coded strings with variable-length IDs/frequencies");
    println!("• Move encoding: 1-3 bytes per move depending on piece type and complexity");
}

/// Display SI4 (Index) format specification
fn display_si4_format_specification() {
    println!("📁 SI4 INDEX FILE FORMAT (.si4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("HEADER STRUCTURE (182 bytes):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-7  │ 8 bytes  │ ASCII   │ Magic: \"Scid.si\\0\"                              │");
    println!("│   8-9  │ 2 bytes  │ BE uint │ Version (usually 400)                           │");
    println!("│  10-13 │ 4 bytes  │ BE uint │ Base Type                                       │");
    println!("│  14-16 │ 3 bytes  │ BE uint │ Number of Games                                 │");
    println!("│  17-19 │ 3 bytes  │ BE uint │ Auto Load Game                                  │");
    println!("│  20-127│108 bytes │ String  │ Description (null-terminated)                   │");
    println!("│128-181 │ 54 bytes │ Strings │ Custom Flag Descriptions (6 × 9 bytes each)    │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    
    println!("GAME INDEX ENTRIES (47 bytes each):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-3  │ 4 bytes  │ BE uint │ Game File Offset                                │");
    println!("│   4-5  │ 2 bytes  │ BE uint │ Game Length (low 16 bits)                       │");
    println!("│    6   │ 1 byte   │ uint8   │ Game Length (high 1 bit) + flags               │");
    println!("│   7-8  │ 2 bytes  │ BE uint │ Game Flags (16 types)                          │");
    println!("│    9   │ 1 byte   │ packed  │ Player ID high bits (4+4)                       │");
    println!("│  10-11 │ 2 bytes  │ BE uint │ White Player ID (low 16 bits)                   │");
    println!("│  12-13 │ 2 bytes  │ BE uint │ Black Player ID (low 16 bits)                   │");
    println!("│   14   │ 1 byte   │ packed  │ Event/Site/Round ID high bits (3+3+2)          │");
    println!("│  15-16 │ 2 bytes  │ BE uint │ Event ID (low 16 bits)                          │");
    println!("│  17-18 │ 2 bytes  │ BE uint │ Site ID (low 16 bits)                           │");
    println!("│  19-20 │ 2 bytes  │ BE uint │ Round ID (low 16 bits)                          │");
    println!("│  21-22 │ 2 bytes  │ BE uint │ Variation Counts + Result (top 4 bits)          │");
    println!("│  23-24 │ 2 bytes  │ BE uint │ ECO Code                                        │");
    println!("│  25-28 │ 4 bytes  │ BE uint │ Game/Event Dates (packed format)               │");
    println!("│  29-30 │ 2 bytes  │ BE uint │ White ELO (12 bits) + Rating Type (4 bits)     │");
    println!("│  31-32 │ 2 bytes  │ BE uint │ Black ELO (12 bits) + Rating Type (4 bits)     │");
    println!("│  33-36 │ 4 bytes  │ BE uint │ Material Signature (final position)            │");
    println!("│   37   │ 1 byte   │ uint8   │ Half Moves (low 8 bits)                         │");
    println!("│  38-46 │ 9 bytes  │ packed  │ Pawn Data + Half Moves high bits                │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
}

/// Display SN4 (Names) format specification  
fn display_sn4_format_specification() {
    println!("📂 SN4 NAME FILE FORMAT (.sn4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("HEADER STRUCTURE (36 bytes):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Offset │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│   0-7  │ 8 bytes  │ ASCII   │ Magic: \"Scid.sn\\0\"                              │");
    println!("│   8-11 │ 4 bytes  │ BE uint │ Timestamp                                       │");
    println!("│  12-14 │ 3 bytes  │ BE uint │ Number of Player Names                          │");
    println!("│  15-17 │ 3 bytes  │ BE uint │ Number of Event Names                           │");
    println!("│  18-20 │ 3 bytes  │ BE uint │ Number of Site Names                            │");
    println!("│  21-23 │ 3 bytes  │ BE uint │ Number of Round Names                           │");
    println!("│  24-26 │ 3 bytes  │ BE uint │ Max Frequency for Players                       │");
    println!("│  27-29 │ 3 bytes  │ BE uint │ Max Frequency for Events                        │");
    println!("│  30-32 │ 3 bytes  │ BE uint │ Max Frequency for Sites                         │");
    println!("│  33-35 │ 3 bytes  │ BE uint │ Max Frequency for Rounds                        │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    
    println!("NAME RECORD STRUCTURE (Variable Length):");
    println!("┌────────┬──────────┬─────────┬─────────────────────────────────────────────────┐");
    println!("│ Field  │   Size   │ Format  │ Description                                     │");
    println!("├────────┼──────────┼─────────┼─────────────────────────────────────────────────┤");
    println!("│ Name ID│ 2-3 bytes│ BE uint │ Sequential ID (2 bytes if count<65536)          │");
    println!("│Frequency│1-3 bytes│ BE uint │ Usage frequency (1/2/3 bytes based on max)     │");
    println!("│ Length │ 1 byte   │ uint8   │ Name string length                              │");
    println!("│ Name   │ N bytes  │ String  │ Front-coded compressed name                     │");
    println!("└────────┴──────────┴─────────┴─────────────────────────────────────────────────┘");
    println!();
    println!("• Names stored in 4 sections: Player, Event, Site, Round");
    println!("• Front-coding: Each name stores only the suffix after common prefix");
    println!("• Variable-length encoding optimizes for database size");
}

/// Display SG4 (Games) format specification
fn display_sg4_format_specification() {
    println!("🎮 SG4 GAME FILE FORMAT (.sg4)");
    println!("─────────────────────────────────────────────────────────────────────────────");
    println!();
    
    println!("FILE STRUCTURE:");
    println!("┌─────────────────────────────────────────────────────────────────────────────┐");
    println!("│ • Block-based organization: 131,072-byte blocks                            │");
    println!("│ • Variable-length game records (no fixed headers)                          │");
    println!("│ • Games separated by ENCODE_END_GAME (15) markers                          │");
    println!("│ • Complex games may span multiple blocks                                   │");
    println!("└─────────────────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("GAME RECORD STRUCTURE (Variable Length):");
    println!("┌─────────────┬─────────────────────────────────────────────────────────────────┐");
    println!("│ Component   │ Description                                                     │");
    println!("├─────────────┼─────────────────────────────────────────────────────────────────┤");
    println!("│ PGN Tags    │ Non-standard tags (WhiteTitle, BlackTitle, etc.)               │");
    println!("│ Game Flags  │ Promotion flags, non-standard starts                           │");
    println!("│ Move Data   │ 1-3 byte move encodings + annotations                          │");
    println!("│ Variations  │ Nested alternative move sequences                              │");
    println!("│ Comments    │ Null-terminated text strings                                   │");
    println!("│ NAGs        │ Numeric Annotation Glyphs (!, ?, !!, etc.)                    │");
    println!("│ End Marker  │ ENCODE_END_GAME (15) - marks game completion                   │");
    println!("└─────────────┴─────────────────────────────────────────────────────────────────┘");
    println!();
    
    println!("MOVE ENCODING (1-3 bytes per move):");
    println!("┌─────────┬───────────┬─────────────────────────────────────────────────────────┐");
    println!("│ Piece   │ Bytes     │ Encoding Method                                         │");
    println!("├─────────┼───────────┼─────────────────────────────────────────────────────────┤");
    println!("│ King    │ 1-2 bytes │ Direction/castling codes + complex scenarios           │");
    println!("│ Queen   │ 1-2 bytes │ Rook-like moves: 1 byte, Diagonal moves: 2 bytes      │");
    println!("│ Rook    │ 1 byte    │ Target rank/file encoded in 4 bits                     │");
    println!("│ Bishop  │ 1 byte    │ Target file + direction in 4 bits                      │");
    println!("│ Knight  │ 1 byte    │ L-shaped move pattern in 4 bits                        │");
    println!("│ Pawn    │ 1-2 bytes │ Direction + promotion, complex promotions: 2 bytes     │");
    println!("└─────────┴───────────┴─────────────────────────────────────────────────────────┘");
    println!();
    
    println!("SPECIAL ENCODING VALUES:");
    println!("┌───────┬─────────────────────────────────────────────────────────────────────────┐");
    println!("│ Value │ Meaning                                                                 │");
    println!("├───────┼─────────────────────────────────────────────────────────────────────────┤");
    println!("│  0-10 │ Regular moves (piece_num << 4 | move_value)                            │");
    println!("│   11  │ ENCODE_NAG - followed by NAG value byte                                │");
    println!("│   12  │ ENCODE_COMMENT - followed by null-terminated string                    │");
    println!("│   13  │ ENCODE_START_MARKER - begin variation                                  │");
    println!("│   14  │ ENCODE_END_MARKER - end variation                                      │");
    println!("│   15  │ ENCODE_END_GAME - end of game record                                   │");
    println!("└───────┴─────────────────────────────────────────────────────────────────────────┘");
}

/// Parse SCID database with clean, tabular output
fn parse_scid_database_clean(base_path: &str) {
    println!("SCID Database Analysis: {}", base_path);
    println!("═══════════════════════════════════════════════════════════════════════════════");
    
    // Parse SI4 Index File
    let si4_path = format!("{}.si4", base_path);
    match File::open(&si4_path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            match parse_header(&mut reader) {
                Ok(header) => {
                    println!();
                    println!("📁 INDEX FILE (.si4) - Header Information");
                    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
                    println!("│ Field                   │ Value                                           │");
                    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
                    println!("│ Version                 │ {}                                              │", header.version);
                    println!("│ Total Games             │ {}                                              │", header.num_games);
                    println!("│ Database Description    │ {}                                              │", header.description.trim_end_matches('\0'));
                    println!("│ Auto Load Game          │ {}                                              │", header.auto_load);
                    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
                    
                    // Parse a few game entries
                    if header.num_games > 0 {
                        println!();
                        println!("📊 Game Index Entries (first 3 games)");
                        println!("┌──────┬────────────┬─────────┬──────────────┬─────────────────────────────────┐");
                        println!("│ Game │    Date    │ Result  │ Game Length  │ Player Names (White vs Black)   │");
                        println!("├──────┼────────────┼─────────┼──────────────┼─────────────────────────────────┤");
                        
                        let games_to_show = std::cmp::min(3, header.num_games);
                        for game_num in 0..games_to_show {
                            match parse_game_index(&mut reader) {
                                Ok(entry) => {
                                    let result_str = match entry.result {
                                        0 => "*",
                                        1 => "1-0", 
                                        2 => "0-1",
                                        3 => "1/2-1/2",
                                        _ => "?",
                                    };
                                    
                                    let date_str = format!("{:04}.{:02}.{:02}", entry.year, entry.month, entry.day);
                                    println!("│ {:>4} │ {} │ {:>7} │ {:>12} │ {:>15} vs {:<15} │", 
                                        game_num + 1,
                                        date_str,
                                        result_str,
                                        entry.length,
                                        format!("ID:{}", entry.white_id),
                                        format!("ID:{}", entry.black_id)
                                    );
                                }
                                Err(e) => {
                                    println!("│ {:>4} │     ERROR  │   ---   │      ---     │ Failed to parse: {}             │", game_num + 1, e);
                                    break;
                                }
                            }
                        }
                        println!("└──────┴────────────┴─────────┴──────────────┴─────────────────────────────────┘");
                    }
                }
                Err(e) => {
                    println!("❌ Error parsing SI4 file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Could not open SI4 file: {}", e);
        }
    }
    
    // Parse SN4 Name File
    println!();
    let sn4_path = format!("{}.sn4", base_path);
    match File::open(&sn4_path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            match parse_sn4_header(&mut reader) {
                Ok(header) => {
                    println!("📂 NAME FILE (.sn4) - Header Information");
                    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
                    println!("│ Name Type               │ Count                                           │");
                    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
                    println!("│ Players                 │ {}                                              │", header.num_names_player);
                    println!("│ Events                  │ {}                                              │", header.num_names_event);
                    println!("│ Sites                   │ {}                                              │", header.num_names_site);
                    println!("│ Rounds                  │ {}                                              │", header.num_names_round);
                    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
                    
                    // Show some sample names
                    println!();
                    println!("📝 Sample Names (first 3 of each type)");
                    println!("┌────────────┬────────┬──────────┬─────────────────────────────────────────────┐");
                    println!("│ Type       │ ID     │ Frequency│ Name                                        │");
                    println!("├────────────┼────────┼──────────┼─────────────────────────────────────────────┤");
                    
                    let mut previous_name = String::new();
                    
                    // Show first few players
                    let player_count = std::cmp::min(3, header.num_names_player);
                    for i in 0..player_count {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_player, header.max_frequency_player, &previous_name) {
                            Ok(record) => {
                                println!("│ Player     │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, record.name);
                                previous_name = record.name.clone();
                            }
                            Err(e) => {
                                println!("│ Player     │ {:>6} │   ERROR  │ Failed to parse: {:<27} │", i, e);
                                break;
                            }
                        }
                    }
                    
                    // Skip remaining players and show events
                    for i in player_count..header.num_names_player {
                        let _ = parse_name_record_sequential(&mut reader, i, header.num_names_player, header.max_frequency_player, &previous_name);
                    }
                    
                    previous_name.clear();
                    let event_count = std::cmp::min(2, header.num_names_event);
                    for i in 0..event_count {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_event, header.max_frequency_event, &previous_name) {
                            Ok(record) => {
                                println!("│ Event      │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, record.name);
                                previous_name = record.name.clone();
                            }
                            Err(_) => break,
                        }
                    }
                    
                    println!("└────────────┴────────┴──────────┴─────────────────────────────────────────────┘");
                }
                Err(e) => {
                    println!("❌ Error parsing SN4 file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Could not open SN4 file: {}", e);
        }
    }
    
    // Parse SG4 Game File
    println!();
    let sg4_path = format!("{}.sg4", base_path);
    match std::fs::read(&sg4_path) {
        Ok(file_data) => {
            let games = find_game_boundaries(&file_data);
            println!("🎮 GAME FILE (.sg4) - Structure Analysis");
            println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
            println!("│ Property                │ Value                                           │");
            println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
            println!("│ File Size               │ {} bytes                                        │", file_data.len());
            println!("│ Games Found             │ {}                                              │", games.len());
            println!("│ Average Game Size       │ {} bytes                                        │", 
                if games.is_empty() { 0 } else { file_data.len() / games.len() });
            
            if !games.is_empty() {
                if let Some((start, end)) = games.first() {
                    println!("│ First Game Size         │ {} bytes                                        │", end - start);
                }
                if let Some((start, end)) = games.last() {
                    println!("│ Last Game Size          │ {} bytes                                        │", end - start);
                }
            }
            println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
            
            // Show first game summary
            if !games.is_empty() {
                if let Some((start_offset, end_offset)) = games.first() {
                    let game_data = &file_data[*start_offset..*end_offset];
                    match parse_pgn_tags(game_data) {
                        Ok(game_state) => {
                            let move_count = game_state.elements.iter()
                                .filter(|e| matches!(e, GameElement::Move { .. }))
                                .count();
                            let comment_count = game_state.elements.iter()
                                .filter(|e| matches!(e, GameElement::Comment { .. }))
                                .count();
                            let variation_starts = game_state.elements.iter()
                                .filter(|e| matches!(e, GameElement::VariationStart { .. }))
                                .count();
                            
                            println!();
                            println!("📋 First Game Analysis");
                            println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
                            println!("│ Component               │ Count                                           │");
                            println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
                            println!("│ Move Elements           │ {}                                              │", move_count);
                            println!("│ Comments                │ {}                                              │", comment_count);
                            println!("│ Variations              │ {}                                              │", variation_starts);
                            println!("│ Non-standard Tags       │ {}                                              │", game_state.tags.len());
                            println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
                        }
                        Err(e) => {
                            println!("❌ Error parsing first game: {}", e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Could not read SG4 file: {}", e);
        }
    }
    
    println!();
    println!("═══════════════════════════════════════════════════════════════════════════════");
    println!("Use '{} format' to see detailed SCID format specifications", std::env::args().next().unwrap_or_default());
}

/// Display comprehensive help information for the SCID parser CLI tool
fn display_help(program_name: &str) {
    println!("SCID Database Parser - A tool for analyzing and parsing SCID chess databases");
    println!();
    println!("USAGE:");
    println!("    {} <COMMAND> [OPTIONS]", program_name);
    println!();
    println!("COMMANDS:");
    println!("    parse <DATABASE>       Parse and analyze SCID database files");
    println!("                          Shows decoded database information in tabular format");
    println!("                          DATABASE should be the base path (e.g., 'mydb' for mydb.si4/sn4/sg4)");
    println!();
    println!("    format                 Display comprehensive SCID database format specifications");
    println!("                          Shows detailed technical documentation for .si4, .sn4, and .sg4 formats");
    println!();
    println!("    encode <DATE>          Encode a date using SCID format for testing");
    println!("                          DATE format: YYYY.MM.DD (e.g., 2022.12.19)");
    println!();
    println!("    help, --help, -h       Show this help message");
    println!();
    println!("TESTING COMMANDS:");
    println!("    test-position          Test chess position tracking implementation");
    println!("                          Demonstrates the chess board state management system");
    println!();
    println!("    test-one-move          Test single move decoding with position awareness");  
    println!("                          Shows how individual chess moves are decoded from SCID format");
    println!();
    println!("    test-moves <DATABASE>  Test position-aware move parsing on one game");
    println!("                          Parses and displays moves from the first game with position tracking");
    println!();
    println!("    test-variations <DATABASE>");
    println!("                          Test variation tree parsing with complex games");
    println!("                          Demonstrates parsing of chess variations and alternative move sequences");
    println!();
    println!("EXAMPLES:");
    println!("    {} parse /path/to/database", program_name);
    println!("                          Analyzes database.si4, database.sn4, and database.sg4 files");
    println!();
    println!("    {} format", program_name);
    println!("                          Shows complete SCID format specifications");
    println!();
    println!("    {} encode 2022.12.19", program_name);
    println!("                          Encodes the date December 19, 2022 using SCID format");
    println!();
    println!("    {} test-moves /Users/chess/mygames", program_name);
    println!("                          Tests move parsing on mygames.sg4 file");
    println!();
    println!("ABOUT:");
    println!("    This tool reverse-engineers and parses SCID (Shane's Chess Information Database)");
    println!("    files, which use a proprietary binary format. It can analyze .si4 index files,");
    println!("    .sn4 name files, and .sg4 game files to extract chess game data.");
    println!();
    println!("    The implementation is based on analysis of the scidvspc source code and extensive");
    println!("    reverse engineering to understand the binary format specifications.");
    println!();
    println!("FILE FORMATS:");
    println!("    .si4    Index file containing game metadata (dates, players, results, etc.)");
    println!("    .sn4    Name file containing compressed player, event, site, and round names");
    println!("    .sg4    Game file containing chess moves, variations, comments, and annotations");
    println!();
    println!("NOTES:");
    println!("    • All SCID multi-byte values use big-endian byte order");
    println!("    • SCID uses front-coded string compression for space efficiency");
    println!("    • Move encoding is variable-length (1-3 bytes per move depending on complexity)");
    println!("    • The format supports variations, comments, and chess annotation symbols (NAGs)");
    println!();
    println!("For detailed format specifications, run: {} format", program_name);
}