//! Convert SCID database to PGN
//!
//! This example demonstrates:
//! - Opening a database
//! - Converting to PGN format
//! - Writing output to file
//! - Handling errors gracefully
//!
//! Run with: cargo run --example convert_to_pgn database.si4 output.pgn

use scidtopgn_core::{PgnOptions, ScidReader};
use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <input.si4> <output.pgn>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!("  {} database.si4 output.pgn", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let output_path = &args[2];

    println!("SCID to PGN Converter");
    println!("{:=<80}", "");
    println!();

    // Open database
    println!("Opening database: {}", input_path);
    let start = Instant::now();

    let reader = ScidReader::open(input_path)?;

    let open_time = start.elapsed();
    println!("✓ Opened in {:?}", open_time);
    println!("  Game count: {}", reader.game_count());
    println!();

    // Create output file
    println!("Creating output file: {}", output_path);
    let output_file = File::create(output_path)?;
    let writer = BufWriter::new(output_file);
    println!("✓ Output file created");
    println!();

    // Convert to PGN
    println!("Converting to PGN...");
    let start = Instant::now();

    let options = PgnOptions::default();
    reader.write_pgn(writer, &options)?;

    let convert_time = start.elapsed();
    println!("✓ Conversion complete in {:?}", convert_time);
    println!();

    // Show statistics
    println!("Statistics:");
    println!("  Games processed: {}", reader.game_count());
    println!(
        "  Time per game:   {:?}",
        convert_time / reader.game_count() as u32
    );

    if reader.game_count() > 0 {
        let games_per_sec = reader.game_count() as f64 / convert_time.as_secs_f64();
        println!("  Throughput:      {:.0} games/second", games_per_sec);
    }

    println!();
    println!("Success! PGN written to: {}", output_path);

    Ok(())
}
