use std::env;
use std::fs::File;
use std::io::{self, BufReader};

mod utils;
mod date;
mod si4;
mod sg4;
mod sn4;

use date::*;
use si4::*;
use sn4::*;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} encode <date>           - Encode date using SCID format (e.g., 2022.12.19)", args[0]);
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
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'encode' or 'parse'");
            std::process::exit(1);
        }
    }
    
    Ok(())
}