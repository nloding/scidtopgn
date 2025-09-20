use anyhow::Result;
use clap::Parser;
use scidtopgn::api::ScidDatabase;
use scidtopgn::cli::{Cli, Commands, utils, ProgressReporter, OutputFormat};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

mod cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize library with CLI settings
    let config = scidtopgn::config::Config::with_settings(
        cli.max_games.unwrap_or(1000),
        8192,
        cli.verbose,
        true,
    );
    scidtopgn::init_with_config(config)?;
    
    match cli.command {
        Commands::Parse {
            database,
            output,
            max_games,
            start_game,
            end_game,
            include_annotations,
            include_optional_tags,
            validate_moves,
            format: format_str,
        } => {
            if cli.verbose {
                eprintln!("Parsing database: {}", database.display());
            }
            
            // Check if database exists
            if !utils::check_database_exists(&database)? {
                return Err(anyhow::anyhow!("Database not found: {}", database.display()));
            }
            
            // Open the SCID database
            let db = ScidDatabase::open(&database)?;
            
            if cli.verbose {
                let stats = db.statistics();
                eprintln!("Database contains {} games", stats.num_games);
            }
            
            // Determine output file path
            let output_path = output.unwrap_or_else(|| {
                let mut path = database.clone();
                path.set_extension("pgn");
                path
            });
            
            // Check if output file exists and handle overwrite
            if !cli.force && output_path.exists() {
                return Err(anyhow::anyhow!("Output file already exists: {}. Use --force to overwrite.", output_path.display()));
            }
            
            // Create output directory if it doesn't exist
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Create output file
            let mut output_file = fs::File::create(&output_path)?;
            
            // Parse output format
            let output_format = format_str.parse::<OutputFormat>()
                .map_err(|e| anyhow::anyhow!("Invalid output format: {}", e))?;
            
            // Set up progress reporting
            let total_games = db.num_games() as usize;
            let start_index = start_game.unwrap_or(0);
            let end_index = end_game.unwrap_or(total_games);
            let actual_max_games = end_index - start_index;
            
            if cli.verbose {
                eprintln!("Processing {} games ({} to {})", actual_max_games, start_index, end_index - 1);
            }
            
            // Set up progress reporter
            let mut progress = ProgressReporter::new(actual_max_games, cli.verbose);
            
            // Process games with optional limit
            let mut game_count = 0;
            let mut successful_exports = 0;
            let mut failed_exports = 0;
            
            // Iterate through games using the public API
            for (index, game_result) in db.games().skip(start_index).take(actual_max_games).enumerate() {
                match game_result {
                    Ok(game) => {
                        game_count += 1;
                        
                        // Export game to PGN
                        match export_game(&game, &output_format, include_annotations, include_optional_tags) {
                            Ok(pgn_content) => {
                                if let Err(e) = write_game(&mut output_file, index + start_index, &pgn_content) {
                                    if cli.verbose {
                                        eprintln!("Error writing game {}: {}", index + start_index, e);
                                    }
                                    failed_exports += 1;
                                } else {
                                    successful_exports += 1;
                                }
                            }
                            Err(e) => {
                                if cli.verbose {
                                    eprintln!("Error exporting game {}: {}", index + start_index, e);
                                }
                                failed_exports += 1;
                            }
                        }
                        
                        progress.increment();
                    }
                    Err(e) => {
                        if cli.verbose {
                            eprintln!("Error accessing game {}: {}", index + start_index, e);
                        }
                        failed_exports += 1;
                    }
                }
                
                // Check for early termination if too many failures
                if failed_exports > 10 && failed_exports > successful_exports {
                    if cli.verbose {
                        eprintln!("Too many failures encountered. Stopping early.");
                    }
                    break;
                }
            }
            
            progress.finish();
            
            if cli.verbose {
                eprintln!("Successfully exported {} games to {}", successful_exports, output_path.display());
                if failed_exports > 0 {
                    eprintln!("Failed to export {} games", failed_exports);
                }
            }
            
            Ok(())
        }
        
        Commands::Info { database, detailed } => {
            match utils::get_database_info(&database, detailed) {
                Ok(info) => {
                    println!("{}", info);
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error getting database info: {}", e);
                    Err(e.into())
                }
            }
        }
        
        Commands::Validate { database, thorough, json } => {
            match utils::validate_database(&database, thorough) {
                Ok(result) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("Database validation result:");
                        println!("  Valid: {}", result.is_valid);
                        println!("  Errors: {}", result.errors.len());
                        println!("  Warnings: {}", result.warnings.len());
                        println!("  Total games: {}", result.stats.total_games);
                        println!("  Validation time: {:?}", result.stats.validation_time);
                        
                        if !result.errors.is_empty() {
                            println!("Errors:");
                            for error in result.errors {
                                println!("  - {}", error);
                            }
                        }
                        
                        if !result.warnings.is_empty() {
                            println!("Warnings:");
                            for warning in result.warnings {
                                println!("  - {}", warning);
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error validating database: {}", e);
                    Err(e.into())
                }
            }
        }
        
        Commands::List { database, max_games, detailed, json } => {
            match utils::list_games(&database, max_games, detailed) {
                Ok(games) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&games)?);
                    } else {
                        println!("Found {} games:", games.len());
                        for game in games {
                            println!("  Game {}: {} vs {} ({})", 
                                     game.index, game.white, game.black, game.result);
                            
                            if detailed {
                                println!("    Event: {}", game.event);
                                println!("    Site: {}", game.site);
                                println!("    Date: {}", game.date);
                                if let Some(elo) = game.white_elo {
                                    println!("    White ELO: {}", elo);
                                }
                                if let Some(elo) = game.black_elo {
                                    println!("    Black ELO: {}", elo);
                                }
                                if let Some(ref eco) = game.eco {
                                    println!("    ECO: {}", eco);
                                }
                            }
                        }
                    }
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error listing games: {}", e);
                    Err(e.into())
                }
            }
        }
        
        Commands::Search {
            database,
            player,
            event,
            eco,
            result,
            min_elo,
            max_elo,
            year_range,
            max_results,
        } => {
            match utils::search_games(
                &database,
                player.as_deref(),
                event.as_deref(),
                eco.as_deref(),
                result.as_deref(),
                min_elo,
                max_elo,
                year_range.as_deref(),
                max_results,
            ) {
                Ok(search_result) => {
                    println!("Found {} matching games (search time: {:?}):", 
                             search_result.total_matches, search_result.search_time);
                    
                    for (i, game) in search_result.games.iter().enumerate() {
                        println!("  {}. {} vs {} ({})", i + 1, game.white, game.black, game.result);
                        
                        // Show additional details for first few results
                        if i < 5 {
                            println!("     Event: {}", game.event);
                            println!("     Site: {}", game.site);
                            println!("     Date: {}", game.date);
                            if let Some(elo) = game.white_elo {
                                println!("     White ELO: {}", elo);
                            }
                            if let Some(elo) = game.black_elo {
                                println!("     Black ELO: {}", elo);
                            }
                            if let Some(ref eco) = game.eco {
                                println!("     ECO: {}", eco);
                            }
                        }
                    }
                    
                    if search_result.total_matches > search_result.games.len() {
                        println!("  ... and {} more matches", search_result.total_matches - search_result.games.len());
                    }
                    
                    Ok(())
                }
                Err(e) => {
                    eprintln!("Error searching games: {}", e);
                    Err(e.into())
                }
            }
        }
    }
    
    // Development and testing commands (only available in debug builds)
    #[cfg(debug_assertions)]
    cli::Commands::Dev { benchmark, memory, test_db, debug } => {
        println!("SCIDtoPGN Development Mode");
        
        if debug {
            println!("Debug mode enabled - detailed output will be shown");
        }
        
        if benchmark {
            println!("Running performance benchmarks...");
            
            // Run benchmarks on test database or specified database
            let test_database = test_db.unwrap_or_else(|| {
                let mut path = std::env::current_dir();
                path.push("test_database");
                path
            });
            
            match utils::check_database_exists(&test_database) {
                Ok(true) => {
                    run_benchmarks(&test_database)?;
                }
                Err(_) => {
                    eprintln!("Test database not found. Creating sample database for benchmarks...");
                    create_sample_database(&test_database)?;
                    run_benchmarks(&test_database)?;
                }
            }
        }
        
        if memory {
            println!("Running memory analysis...");
            
            let test_database = test_db.unwrap_or_else(|| {
                let mut path = std::env::current_dir();
                path.push("test_database");
                path
            });
            
            match utils::check_database_exists(&test_database) {
                Ok(true) => {
                    run_memory_analysis(&test_database)?;
                }
                Err(_) => {
                    eprintln!("Test database not found. Creating sample database for memory analysis...");
                    create_sample_database(&test_database)?;
                    run_memory_analysis(&test_database)?;
                }
            }
        }
        
        if !benchmark && !memory {
            println!("No development tasks specified. Use --benchmark or --memory");
        }
        
        Ok(())
    }
}

/// Run performance benchmarks on a database
#[cfg(debug_assertions)]
fn run_benchmarks(database_path: &PathBuf) -> Result<()> {
    use std::time::Instant;
    
    println!("Running benchmarks on: {}", database_path.display());
    
    let start_time = Instant::now();
    
    // Open database
    let db = ScidDatabase::open(database_path)?;
    
    // Benchmark database opening
    let open_time = start_time.elapsed();
    println!("Database open time: {:?}", open_time);
    
    // Benchmark game iteration
    let iter_start = Instant::now();
    let game_count: usize = db.games().count();
    let iter_time = iter_start.elapsed();
    println!("Game iteration time: {:?} ({} games)", iter_time, game_count);
    
    // Benchmark PGN export
    let export_start = Instant::now();
    let mut successful_exports = 0;
    let max_games = 100.min(game_count);
    
    for (index, game_result) in db.games().take(max_games).enumerate() {
        match game_result {
            Ok(game) => {
                match export_game(&game, &OutputFormat::Pgn, true, true) {
                    Ok(_) => successful_exports += 1,
                    Err(e) => {
                        if cli.verbose {
                            eprintln!("Benchmark export error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if cli.verbose {
                    eprintln!("Benchmark access error: {}", e);
                }
            }
        }
    }
    let export_time = export_start.elapsed();
    println!("PGN export time: {:?} ({} games exported)", export_time, successful_exports);
    
    // Calculate performance metrics
    let total_time = start_time.elapsed();
    let games_per_second = game_count as f64 / total_time.as_secs_f64();
    
    println!("\n=== Benchmark Results ===");
    println!("Total games: {}", game_count);
    println!("Total time: {:?}", total_time);
    println!("Games per second: {:.2}", games_per_second);
    println!("Average time per game: {:.2}ms", total_time.as_millis() as f64 / game_count as f64);
    
    Ok(())
}

/// Run memory analysis on a database
#[cfg(debug_assertions)]
fn run_memory_analysis(database_path: &PathBuf) -> Result<()> {
    use scidtopgn::perf::MemoryStats;
    
    println!("Running memory analysis on: {}", database_path.display());
    
    let mut stats = MemoryStats::new();
    let start_time = std::time::Instant::now();
    
    // Open database and measure memory usage
    let db = ScidDatabase::open(database_path)?;
    stats.update(0);
    
    // Process games and track memory usage
    let mut game_count = 0;
    let max_games = 50; // Limit for memory analysis
    
    for game_result in db.games().take(max_games) {
        match game_result {
            Ok(game) => {
                game_count += 1;
                
                // Simulate memory usage for game processing
                let game_size = std::mem::size_of_val(&game);
                stats.update(game_size);
                
                // Simulate PGN export memory usage
                let pgn_content = export_game(&game, &OutputFormat::Pgn, true, true)?;
                let pgn_size = pgn_content.len();
                stats.update(pgn_size);
            }
            Err(e) => {
                eprintln!("Memory analysis error: {}", e);
            }
        }
    }
    
    let analysis_time = start_time.elapsed();
    
    println!("\n=== Memory Analysis Results ===");
    println!("Games processed: {}", game_count);
    println!("Analysis time: {:?}", analysis_time);
    println!("{}", stats.format());
    
    Ok(())
}

/// Create a sample database for testing and development
#[cfg(debug_assertions)]
fn create_sample_database(database_path: &PathBuf) -> Result<()> {
    println!("Creating sample database at: {}", database_path.display());
    
    // Create directory if it doesn't exist
    if let Some(parent) = database_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Create sample SCID files (simplified version)
    let si4_path = database_path.with_extension("si4");
    let sn4_path = database_path.with_extension("sn4");
    let sg4_path = database_path.with_extension("sg4");
    
    // Create empty files with basic structure
    fs::File::create(&si4_path)?;
    fs::File::create(&sn4_path)?;
    fs::File::create(&sg4_path)?;
    
    println!("Sample database created successfully");
    println!("Files created:");
    println!("  {}", si4_path.display());
    println!("  {}", sn4_path.display());
    println!("  {}", sg4_path.display());
    
    Ok(())
}

/// Write a game to the output file with proper formatting
fn write_game(output_file: &mut fs::File, game_index: usize, pgn_content: &str) -> Result<()> {
    // Write game header
    writeln!(output_file, "[Game \"{}\"]", game_index + 1)?;
    
    // Write PGN content
    writeln!(output_file, "{}", pgn_content)?;
    
    // Write game separator
    writeln!(output_file)?;
    
    Ok(())
}
                            "[Event \"Game {}\"]\n[Result \"*\"]\n\n*\n",
                            game_count + 1
                        )?;
                        game_count += 1;
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse game {}: {}", game_count + 1, e);
                    }
                }
            }

            println!(
                "Successfully converted {} games to {}",
                game_count,
                output_path.display()
            );
        }

        cli::Commands::Info { database } => {
            println!("Getting info for database: {}", database.display());

            // Open the SCID database using public API
            let db = ScidDatabase::open(&database)?;

            // Display basic database information
            println!("\n=== SCID Database Information ===");
            println!("Database path: {}", database.display());

            // Check file existence and sizes
            let si4_path = database.with_extension("si4");
            let sn4_path = database.with_extension("sn4");
            let sg4_path = database.with_extension("sg4");

            if let Ok(metadata) = fs::metadata(&si4_path) {
                println!("Index file (.si4): {} bytes", metadata.len());
            }
            if let Ok(metadata) = fs::metadata(&sn4_path) {
                println!("Names file (.sn4): {} bytes", metadata.len());
            }
            if let Ok(metadata) = fs::metadata(&sg4_path) {
                println!("Games file (.sg4): {} bytes", metadata.len());
            }

            // Count games using the public API
            let game_count: usize = db.games().count();
            println!("Total games: {}", game_count);

            println!("\nDatabase opened successfully!");
        }

        cli::Commands::Validate { database } => {
            println!("Validating database: {}", database.display());

            // Open the SCID database using public API
            let db = ScidDatabase::open(&database)?;

            println!("\n=== SCID Database Validation ===");

            // Basic file existence validation
            let si4_path = database.with_extension("si4");
            let sn4_path = database.with_extension("sn4");
            let sg4_path = database.with_extension("sg4");

            let mut validation_errors = 0;

            // Check file existence
            if !si4_path.exists() {
                eprintln!("ERROR: Index file (.si4) not found");
                validation_errors += 1;
            } else {
                println!("✓ Index file (.si4) exists");
            }

            if !sn4_path.exists() {
                eprintln!("ERROR: Names file (.sn4) not found");
                validation_errors += 1;
            } else {
                println!("✓ Names file (.sn4) exists");
            }

            if !sg4_path.exists() {
                eprintln!("ERROR: Games file (.sg4) not found");
                validation_errors += 1;
            } else {
                println!("✓ Games file (.sg4) exists");
            }

            // Try to iterate through games to check basic parsing using public API
            let mut parsed_games = 0;
            let mut parse_errors = 0;

            println!("\nValidating game parsing...");
            for game_result in db.games().take(10) {
                // Check first 10 games
                match game_result {
                    Ok(_) => parsed_games += 1,
                    Err(_) => parse_errors += 1,
                }
            }

            if parse_errors > 0 {
                eprintln!("WARNING: {} games failed to parse", parse_errors);
                validation_errors += parse_errors;
            } else if parsed_games > 0 {
                println!("✓ Sample games parsed successfully");
            } else {
                println!("✓ No games to validate (empty database)");
            }

            // Summary
            if validation_errors == 0 {
                println!("\n✅ Database validation passed!");
            } else {
                println!(
                    "\n❌ Database validation failed with {} errors",
                    validation_errors
                );
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
