use clap::Parser;
use scidtopgn::api::ScidDatabase;
use scidtopgn::cli::{Cli, Commands};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Parse { database, output: _, max_games, start_game, .. } => {
            use scidtopgn::cli::table_display;
            use scidtopgn::api::NameType;
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            // 1. Display SI4 header
            table_display::display_si4_header(&db)?;
            println!();
            
            // 2. Display SN4 header
            table_display::display_sn4_header(&db)?;
            println!();
            
            // 3. Display all names
            table_display::display_names_table(&db, NameType::Player)?;
            println!();
            table_display::display_names_table(&db, NameType::Event)?;
            println!();
            table_display::display_names_table(&db, NameType::Site)?;
            println!();
            table_display::display_names_table(&db, NameType::Round)?;
            println!();
            
            // 4. Display SG4 statistics
            table_display::display_sg4_stats(&db)?;
            println!();
            
            // 5. Display individual games
            let num_games = db.num_games() as usize;
            let limit = max_games.unwrap_or(num_games);
            let end_game = std::cmp::min(start_game + limit, num_games);
            
            for game_num in start_game..end_game {
                match table_display::display_game(&db, game_num as u32, true) {
                    Ok(()) => {},
                    Err(e) => eprintln!("Error displaying game {}: {}", game_num + 1, e),
                }
            }
            
            Ok(())
        }
        Commands::Info { database, .. } => {
            println!("Getting info for database: {}", database.display());
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            // Get basic info
            let stats = db.statistics();
            println!("Database contains {} games", stats.num_games);
            println!("File sizes: SI4={} bytes, SN4={} bytes, SG4={} bytes", 
                stats.file_size_si4, stats.file_size_sn4, stats.file_size_sg4);
            println!("Database is validated: {}", stats.is_validated);
            
            println!("Info command not fully implemented yet");
            Ok(())
        }
        Commands::Validate { database, .. } => {
            println!("Validating database: {}", database.display());
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            // Get basic info as a simple validation
            let stats = db.statistics();
            println!("✓ Database structure is valid");
            println!("✓ Contains {} games", stats.num_games);
            println!("✓ File sizes: SI4={} bytes, SN4={} bytes, SG4={} bytes", 
                stats.file_size_si4, stats.file_size_sn4, stats.file_size_sg4);
            
            println!("Validate command not fully implemented yet");
            Ok(())
        }
        Commands::List { database, max_games, .. } => {
            println!("Listing games from database: {}", database.display());
            println!("Max games to list: {}", max_games);
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            // List some games (basic implementation)
            let mut count = 0;
            for (index, game_result) in db.games().take(max_games).enumerate() {
                match game_result {
                    Ok(game) => {
                        count += 1;
                        println!("Game {}: Index={}", index + 1, game.index.offset);
                    }
                    Err(e) => {
                        eprintln!("Error loading game {}: {}", index + 1, e);
                    }
                }
            }
            
            println!("Listed {} games", count);
            println!("List command not fully implemented yet");
            Ok(())
        }
        Commands::Search { database, player, event, max_results, .. } => {
            println!("Searching in database: {}", database.display());
            if let Some(p) = &player {
                println!("Player filter: {}", p);
            }
            if let Some(e) = &event {
                println!("Event filter: {}", e);
            }
            println!("Max results: {}", max_results);
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            // Simple search implementation (basic)
            let mut count = 0;
            for game_result in db.games().take(max_results) {
                if let Ok(game) = game_result {
                    // For now, just list all games since we don't have name resolution
                    count += 1;
                    println!("Game {}: Index={}", count, game.index.offset);
                }
            }
            
            println!("Found {} matching games", count);
            println!("Search command not fully implemented yet");
            Ok(())
        }
        #[cfg(debug_assertions)]
        Commands::Dev { benchmark, memory, test_db: _, debug } => {
            println!("SCIDtoPGN Development Mode");
            
            if debug {
                println!("Debug mode enabled - detailed output will be shown");
            }
            
            if benchmark {
                println!("Running performance benchmarks...");
                println!("Benchmark not implemented yet");
            }
            
            if memory {
                println!("Running memory analysis...");
                println!("Memory analysis not implemented yet");
            }
            
            if !benchmark && !memory {
                println!("No development action specified. Use --benchmark or --memory");
            }
            
            Ok(())
        }
        #[cfg(not(debug_assertions))]
        Commands::Dev { .. } => {
            return Err("Development commands are only available in debug builds".into());
        }
    }
}