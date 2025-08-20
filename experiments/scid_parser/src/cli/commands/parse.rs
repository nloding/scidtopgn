// Parse command implementation
use std::fs::File;
use std::io::BufReader;
use crate::si4::*;
use crate::sn4::*;
use crate::sg4::*;
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
        }
        Err(e) => {
            println!("❌ Could not read SG4 file: {}", e);
        }
    }
}