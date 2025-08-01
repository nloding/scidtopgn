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
        eprintln!("Usage:");
        eprintln!("  {} encode <date>           - Encode date using SCID format (e.g., 2022.12.19)", args[0]);
        eprintln!("  {} test-position           - Test chess position tracking implementation", args[0]);
        eprintln!("  {} test-one-move           - Test single move decoding with position awareness", args[0]);
        eprintln!("  {} test-moves <base_path>  - Test position-aware move parsing on one game", args[0]);
        eprintln!("  {} parse <base_path>       - Parse SCID database files (both .si4 and .sn4)", args[0]);
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
        "parse" => {
            if args.len() != 3 {
                eprintln!("Usage: {} parse <base_path>", args[0]);
                eprintln!("Example: {} parse /path/to/database (will read database.si4 and database.sn4)", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            
            // Try to parse SI4 file
            let si4_path = format!("{}.si4", base_path);
            println!("Reading SCID index file: {}", si4_path);
            
            match File::open(&si4_path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    match parse_header(&mut reader) {
                        Ok(header) => {
                            display_header_table(&header);
                            
                            // Show the structure of game index entries that follow the header
                            println!("The SI4 file contains game index entries after the header:");
                            display_game_index_structure();
                            
                            // Try to parse and display the first game index entry
                            if header.num_games > 0 {
                                println!("Parsing first game index entry:");
                                if let Err(e) = parse_and_display_first_game_index(&mut reader) {
                                    println!("Could not parse first game index entry: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error parsing SI4 header: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Could not open SI4 file: {}", e);
                }
            }
            
            // Try to parse SN4 file
            let sn4_path = format!("{}.sn4", base_path);
            println!("Reading SCID namebase file: {}", sn4_path);
            
            match File::open(&sn4_path) {
                Ok(file) => {
                    let mut reader = BufReader::new(file);
                    
                    // Show the structure of namebase header
                    println!("The SN4 file contains a namebase header followed by name data:");
                    display_sn4_header_structure();
                    
                    match parse_sn4_header(&mut reader) {
                        Ok(header) => {
                            display_sn4_header_values(&header);
                            
                            // Show name record structure
                            println!("Name records follow the header in this order:");
                            display_name_record_structure();
                            
                            // Parse name records sequentially (first few of each type)
                            println!("Parsing name records sequentially (IDs only for now):");
                            
                            let mut previous_name = String::new();
                            
                            // Player records - parse sequentially through all players  
                            let player_count = header.num_names_player.min(3);
                            for i in 0..player_count {
                                let record = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_player,
                                    header.max_frequency_player,
                                    &previous_name
                                ).ok();
                                display_name_record_values(i as usize, "Player", record.as_ref());
                                if let Some(ref rec) = record {
                                    previous_name = rec.name.clone();
                                }
                            }
                            
                            // Skip remaining player records to get to events
                            for i in player_count..header.num_names_player {
                                let _ = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_player,
                                    header.max_frequency_player,
                                    &previous_name
                                );
                            }
                            
                            // Event records - reset previous_name for new section
                            previous_name.clear();
                            let event_count = header.num_names_event.min(2);
                            for i in 0..event_count {
                                let record = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_event,
                                    header.max_frequency_event,
                                    &previous_name
                                ).ok();
                                display_name_record_values(i as usize, "Event", record.as_ref());
                                if let Some(ref rec) = record {
                                    previous_name = rec.name.clone();
                                }
                            }
                            
                            // Skip remaining event records
                            for i in event_count..header.num_names_event {
                                let _ = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_event,
                                    header.max_frequency_event,
                                    &previous_name
                                );
                            }
                            
                            // Site records - reset previous_name for new section
                            previous_name.clear();
                            let site_count = header.num_names_site.min(2);
                            for i in 0..site_count {
                                let record = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_site,
                                    header.max_frequency_site,
                                    &previous_name
                                ).ok();
                                display_name_record_values(i as usize, "Site", record.as_ref());
                                if let Some(ref rec) = record {
                                    previous_name = rec.name.clone();
                                }
                            }
                            
                            // Skip remaining site records  
                            for i in site_count..header.num_names_site {
                                let _ = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_site,
                                    header.max_frequency_site,
                                    &previous_name
                                );
                            }
                            
                            // Round records - reset previous_name for new section
                            previous_name.clear();
                            let round_count = header.num_names_round.min(2);
                            for i in 0..round_count {
                                let record = parse_name_record_sequential(
                                    &mut reader, 
                                    i, 
                                    header.num_names_round,
                                    header.max_frequency_round,
                                    &previous_name
                                ).ok();
                                display_name_record_values(i as usize, "Round", record.as_ref());
                                if let Some(ref rec) = record {
                                    previous_name = rec.name.clone();
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error parsing SN4 header: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Could not open SN4 file: {}", e);
                }
            }
            
            // Try to parse SG4 file
            let sg4_path = format!("{}.sg4", base_path);
            println!("Reading SCID game file: {}", sg4_path);
            
            match parse_sg4_file(&sg4_path) {
                Ok(_) => {
                    println!("✅ SG4 file analysis completed successfully");
                }
                Err(e) => {
                    eprintln!("Could not analyze SG4 file: {}", e);
                }
            }
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'encode', 'test-position', 'test-one-move', 'test-moves', or 'parse'");
            std::process::exit(1);
        }
    }
    
    Ok(())
}