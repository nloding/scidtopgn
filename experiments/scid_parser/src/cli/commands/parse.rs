// Parse command implementation
use std::fs::File;
use std::io::BufReader;
use std::collections::HashMap;
use crate::si4::*;
use crate::sn4::*;
use crate::sg4::*;
use crate::position::{ScidByteStream, ScidPosition, decode_move_with_stream};
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
    
    // Load name lookup tables from SN4 file
    let name_lookup = match load_name_lookup_tables(si4_path) {
        Ok(lookup) => lookup,
        Err(e) => {
            println!("❌ Could not load name lookup tables: {}", e);
            return;
        }
    };
    
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
        
        // White Player with name lookup
        let white_name = name_lookup.players.get(&game_index.white_id)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        println!("│ White Player ID         │ {}{} │", game_index.white_id, white_name);
        
        // Black Player with name lookup
        let black_name = name_lookup.players.get(&game_index.black_id)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        println!("│ Black Player ID         │ {}{} │", game_index.black_id, black_name);
        
        // Event with name lookup
        let event_name = name_lookup.events.get(&game_index.event_id)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        println!("│ Event ID                │ {}{} │", game_index.event_id, event_name);
        
        // Site with name lookup
        let site_name = name_lookup.sites.get(&game_index.site_id)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        println!("│ Site ID                 │ {}{} │", game_index.site_id, site_name);
        
        // Round with name lookup
        let round_name = name_lookup.rounds.get(&game_index.round_id)
            .map(|name| format!(" ({})", name))
            .unwrap_or_default();
        println!("│ Round ID                │ {}{} │", game_index.round_id, round_name);
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
        
    // Extract and display moves with position-aware algebraic notation (decoder-based)
    let mut decode_position = ScidPosition::new_starting_position();
    let mut move_count = 0;
    let mut _position_errors = 0;
        
        for element in &parsed_game.elements {
            if let StreamingGameElement::Move { raw_bytes, offset: _, .. } = element {
                move_count += 1;
                
                // Create stream from raw bytes for this move
                let mut move_stream = ScidByteStream::new(raw_bytes);
                let start_pos = move_stream.position();
                match decode_move_with_stream(&decode_position, &mut move_stream) {
                    Ok(scid_move) => {
                        // Render algebraic notation and apply to local position
                        let desc = scid_move.to_algebraic(&decode_position);
                        if let Err(e) = decode_position.do_move(&scid_move) {
                            _position_errors += 1;
                            println!("│ Move {} ERROR           │ ❌ {} │", move_count, truncate_name(&e, 70));
                            continue;
                        }
                        let consumed = move_stream.position().saturating_sub(start_pos);
                        let byte_info = if consumed > 1 { format!(" ({} bytes)", consumed) } else { String::new() };
                        println!("│ Move {}{}               │ ✅ {} │", move_count, byte_info, truncate_name(&desc, 65));
                    },
                    Err(e) => {
                        _position_errors += 1;
                        // Fallback for undecoded moves
                        if !raw_bytes.is_empty() {
                            println!(
                                "│ Move {} ERROR           │ ❌ {} (raw 0x{:02X}) │",
                                move_count,
                                truncate_name(&e, 58),
                                raw_bytes[0]
                            );
                        } else {
                            println!("│ Move {} ERROR           │ ❌ {} │", move_count, truncate_name(&e, 70));
                        }
                    }
                }
            }
        }
        
        println!("└─────────────────────────┴──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────┘");
        println!();
    }
}

/// Name lookup tables for resolving IDs to names
#[derive(Debug)]
struct NameLookup {
    players: HashMap<u32, String>,
    events: HashMap<u32, String>,
    sites: HashMap<u32, String>,
    rounds: HashMap<u32, String>,
}

/// Load all name lookup tables from the SN4 file
fn load_name_lookup_tables(si4_path: &str) -> std::io::Result<NameLookup> {
    // Construct SN4 path from SI4 path
    let sn4_path = si4_path.replace(".si4", ".sn4");
    
    let file = File::open(&sn4_path)?;
    let mut reader = BufReader::new(file);
    
    // Parse SN4 header
    let header = parse_sn4_header(&mut reader)?;
    
    let mut players = HashMap::new();
    let mut events = HashMap::new();
    let mut sites = HashMap::new();
    let mut rounds = HashMap::new();
    
    // Load all players
    let mut previous_name = String::new();
    for i in 0..header.num_names_player {
        if let Ok(record) = parse_name_record_sequential(&mut reader, i, header.num_names_player, header.max_frequency_player, &previous_name) {
            players.insert(i, record.name.clone());
            previous_name = record.name;
        } else {
            break;
        }
    }
    
    // Load all events
    previous_name.clear();
    for i in 0..header.num_names_event {
        if let Ok(record) = parse_name_record_sequential(&mut reader, i, header.num_names_event, header.max_frequency_event, &previous_name) {
            events.insert(i, record.name.clone());
            previous_name = record.name;
        } else {
            break;
        }
    }
    
    // Load all sites
    previous_name.clear();
    for i in 0..header.num_names_site {
        if let Ok(record) = parse_name_record_sequential(&mut reader, i, header.num_names_site, header.max_frequency_site, &previous_name) {
            sites.insert(i, record.name.clone());
            previous_name = record.name;
        } else {
            break;
        }
    }
    
    // Load all rounds
    previous_name.clear();
    for i in 0..header.num_names_round {
        if let Ok(record) = parse_name_record_sequential(&mut reader, i, header.num_names_round, header.max_frequency_round, &previous_name) {
            rounds.insert(i, record.name.clone());
            previous_name = record.name;
        } else {
            break;
        }
    }
    
    Ok(NameLookup {
        players,
        events,
        sites,
        rounds,
    })
}