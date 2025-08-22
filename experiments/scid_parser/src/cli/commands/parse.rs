// Parse command implementation
use std::fs::File;
use std::io::BufReader;
use crate::si4::*;
use crate::sn4::*;
use crate::sg4::*;
use crate::position::{PositionTracker, ScidByteStream};
use crate::cli::output::tables::truncate_name;

pub fn execute(base_path: &str) -> std::io::Result<()> {
    parse_scid_database_clean(base_path);
    Ok(())
}


/// Parse SCID database with clean, tabular output
fn parse_scid_database_clean(base_path: &str) {
    // 1. SI4 Header Table
    let si4_path = format!("{}.si4", base_path);
    match File::open(&si4_path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            match parse_header(&mut reader) {
                Ok(header) => {
                    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
                    println!("│ Field                   │ Value                                           │");
                    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
                    println!("│ Magic                   │ {}                                              │", "Scid.si");
                    println!("│ Version                 │ {}                                              │", header.version);
                    println!("│ Base Type               │ {}                                              │", header.base_type);
                    println!("│ Number of Games         │ {}                                              │", header.num_games);
                    println!("│ Auto Load Game          │ {}                                              │", header.auto_load);
                    println!("│ Description             │ {}                                              │", header.description.trim_end_matches('\0'));
                    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
                }
                Err(e) => {
                    println!("Error parsing SI4 file: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("Could not open SI4 file: {}", e);
            return;
        }
    }
    
    println!();
    
    // 2. SN4 Header Table
    let sn4_path = format!("{}.sn4", base_path);
    match File::open(&sn4_path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            match parse_sn4_header(&mut reader) {
                Ok(header) => {
                    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
                    println!("│ Field                   │ Value                                           │");
                    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
                    println!("│ Magic                   │ {}                                              │", "Scid.sn");
                    println!("│ Timestamp               │ {}                                              │", header.timestamp);
                    println!("│ Num Names Player        │ {}                                              │", header.num_names_player);
                    println!("│ Num Names Event         │ {}                                              │", header.num_names_event);
                    println!("│ Num Names Site          │ {}                                              │", header.num_names_site);
                    println!("│ Num Names Round         │ {}                                              │", header.num_names_round);
                    println!("│ Max Frequency Player    │ {}                                              │", header.max_frequency_player);
                    println!("│ Max Frequency Event     │ {}                                              │", header.max_frequency_event);
                    println!("│ Max Frequency Site      │ {}                                              │", header.max_frequency_site);
                    println!("│ Max Frequency Round     │ {}                                              │", header.max_frequency_round);
                    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
                    
                    println!();
                    
                    // 3. SN4 Entries Table - ALL entries
                    println!("┌────────────┬────────┬──────────┬─────────────────────────────────────────────┐");
                    println!("│ Type       │ ID     │ Frequency│ Name                                        │");
                    println!("├────────────┼────────┼──────────┼─────────────────────────────────────────────┤");
                    
                    let mut previous_name = String::new();
                    
                    // All players
                    for i in 0..header.num_names_player {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_player, header.max_frequency_player, &previous_name) {
                            Ok(record) => {
                                println!("│ Player     │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, truncate_name(&record.name, 43));
                                previous_name = record.name.clone();
                            }
                            Err(_) => break,
                        }
                    }
                    
                    // All events
                    previous_name.clear();
                    for i in 0..header.num_names_event {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_event, header.max_frequency_event, &previous_name) {
                            Ok(record) => {
                                println!("│ Event      │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, truncate_name(&record.name, 43));
                                previous_name = record.name.clone();
                            }
                            Err(_) => break,
                        }
                    }
                    
                    // All sites
                    previous_name.clear();
                    for i in 0..header.num_names_site {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_site, header.max_frequency_site, &previous_name) {
                            Ok(record) => {
                                println!("│ Site       │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, truncate_name(&record.name, 43));
                                previous_name = record.name.clone();
                            }
                            Err(_) => break,
                        }
                    }
                    
                    // All rounds
                    previous_name.clear();
                    for i in 0..header.num_names_round {
                        match parse_name_record_sequential(&mut reader, i, header.num_names_round, header.max_frequency_round, &previous_name) {
                            Ok(record) => {
                                println!("│ Round      │ {:>6} │ {:>8} │ {:<43} │", i, record.frequency, truncate_name(&record.name, 43));
                                previous_name = record.name.clone();
                            }
                            Err(_) => break,
                        }
                    }
                    
                    println!("└────────────┴────────┴──────────┴─────────────────────────────────────────────┘");
                }
                Err(e) => {
                    println!("Error parsing SN4 file: {}", e);
                    return;
                }
            }
        }
        Err(e) => {
            println!("Could not open SN4 file: {}", e);
            return;
        }
    }
    
    println!();
    
    // 4. SG4 Header Table
    let sg4_path = format!("{}.sg4", base_path);
    match std::fs::read(&sg4_path) {
        Ok(file_data) => {
            let games = find_game_boundaries(&file_data);
            println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
            println!("│ Field                   │ Value                                           │");
            println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
            println!("│ File Size               │ {} bytes                                        │", file_data.len());
            println!("│ Games Found             │ {}                                              │", games.len());
            if !games.is_empty() {
                println!("│ Average Game Size       │ {} bytes                                        │", file_data.len() / games.len());
                if let Some((start, end)) = games.first() {
                    println!("│ First Game Offset       │ {}                                              │", start);
                    println!("│ First Game Size         │ {} bytes                                        │", end - start);
                }
                if let Some((start, end)) = games.last() {
                    println!("│ Last Game Offset        │ {}                                              │", start);
                    println!("│ Last Game Size          │ {} bytes                                        │", end - start);
                }
            }
            println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
            
            println!();
            
            // 5. Individual Games Display
            display_games(&si4_path, &file_data, games);
        }
        Err(e) => {
            println!("❌ Could not read SG4 file: {}", e);
        }
    }
}

/// Display individual games with metadata and moves
fn display_games(si4_path: &str, sg4_data: &[u8], games: Vec<(usize, usize)>) {
    // Parse SI4 index file to get game metadata
    let mut si4_reader = match File::open(si4_path) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            println!("❌ Could not open SI4 file for game metadata: {}", e);
            return;
        }
    };
    
    // Skip SI4 header
    if let Err(e) = parse_header(&mut si4_reader) {
        println!("❌ Could not parse SI4 header: {}", e);
        return;
    }
    
    // Display each game
    for (game_num, (start_offset, end_offset)) in games.iter().enumerate() {
        println!("GAME {} DETAILS", game_num + 1);
        
        // Get game metadata from SI4 index
        let game_index = match parse_game_index(&mut si4_reader) {
            Ok(index) => index,
            Err(e) => {
                println!("❌ Could not parse game {} index: {}", game_num + 1, e);
                continue;
            }
        };
        
        // Extract game data from SG4
        let game_data = &sg4_data[*start_offset..*end_offset];
        
        // Parse game content using streaming parser for algebraic notation
        let parsed_game = match parse_pgn_tags_with_streaming(game_data) {
            Ok(game) => game,
            Err(e) => {
                println!("❌ Could not parse game {} content: {}", game_num + 1, e);
                continue;
            }
        };
        
        // Display game table with dynamic width
        println!("┌─────────────────────────┬──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┐");
        println!("│ Field                   │ Value                                                                                                                                                                                        │");
        println!("├─────────────────────────┼──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┤");
        
        // Game metadata from SI4
        println!("│ Game Number             │ {} │", game_num + 1);
        println!("│ Offset                  │ {} │", game_index.offset);
        println!("│ Length                  │ {} bytes │", game_index.length);
        println!("│ White Player ID         │ {} │", game_index.white_id);
        println!("│ Black Player ID         │ {} │", game_index.black_id);
        println!("│ Event ID                │ {} │", game_index.event_id);
        println!("│ Site ID                 │ {} │", game_index.site_id);
        println!("│ Round ID                │ {} │", game_index.round_id);
        println!("│ Date                    │ {}.{:02}.{:02} │", game_index.year, game_index.month, game_index.day);
        
        let result_str = match game_index.result {
            0 => "*",
            1 => "1-0",
            2 => "0-1", 
            3 => "1/2-1/2",
            _ => "Unknown",
        };
        println!("│ Result                  │ {} │", result_str);
        println!("│ ECO Code                │ {} │", game_index.eco);
        println!("│ White ELO               │ {} │", game_index.white_elo);
        println!("│ Black ELO               │ {} │", game_index.black_elo);
        println!("│ Flags                   │ {} (0x{:04x}) │", game_index.flags, game_index.flags);
        println!("│ Half Moves              │ {} │", game_index.num_half_moves);
        
        // Game content from SG4
        println!("│ PGN Tags                │ {} │", parsed_game.tags.len());
        println!("│ Game Elements           │ {} │", parsed_game.elements.len());
        
        // Extract and display moves with position-aware algebraic notation
        let mut position_tracker = PositionTracker::new();
        let mut move_count = 0;
        
        for element in &parsed_game.elements {
            if let StreamingGameElement::Move { raw_bytes, offset, .. } = element {
                move_count += 1;
                
                // Create stream from raw bytes for this move
                let mut move_stream = ScidByteStream::new(raw_bytes);
                let move_description = match position_tracker.process_move_from_stream(&mut move_stream, *offset) {
                    Ok(decoded_move) => {
                        // Use algebraic notation from position-aware decoding
                        decoded_move.interpretation.description().to_string()
                    },
                    Err(_) => {
                        // Fallback for undecoded moves
                        if raw_bytes.len() > 0 {
                            format!("Raw byte 0x{:02X} (undecoded)", raw_bytes[0])
                        } else {
                            "Empty move (undecoded)".to_string()
                        }
                    }
                };
                
                println!("│ Move {}                  │ {} │", 
                    move_count, 
                    move_description);
            }
        }
        
        println!("└─────────────────────────┴──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘");
        println!();
    }
}