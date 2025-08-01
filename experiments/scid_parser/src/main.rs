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

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} encode <date>           - Encode date using SCID format (e.g., 2022.12.19)", args[0]);
        eprintln!("  {} parse <path_to_si4>     - Parse SCID database header only", args[0]);
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
                eprintln!("Usage: {} parse <path_to_si4_file>", args[0]);
                eprintln!("Example: {} parse /path/to/database.si4", args[0]);
                std::process::exit(1);
            }
            
            let file_path = &args[2];
            println!("Reading SCID index file: {}", file_path);
            
            let file = File::open(file_path)?;
            let mut reader = BufReader::new(file);
            
            // Parse header only
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
                    eprintln!("Error parsing header: {}", e);
                    std::process::exit(1);
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