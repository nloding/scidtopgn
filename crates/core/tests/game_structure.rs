//! Integration tests for SG4 game file structure parsing

use scidtopgn_core::database::{
    games::{flags, parse_game_structure, read_game_data},
    index::parse_si4_file,
    names::parse_name_database,
};
use std::fs::File;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_sg4_game_1() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse index to get game location
    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let game1 = &entries[0];

    // Read game data
    let mut sg4_file = File::open(&sg4_path).unwrap();
    let game_bytes = read_game_data(&mut sg4_file, game1.game_offset, game1.game_length).unwrap();

    println!("Game 1 size: {} bytes", game_bytes.len());

    // Parse game structure
    let game_data = parse_game_structure(&game_bytes).unwrap();

    println!("\n=== Game 1 Tags ===");
    for (name, value) in &game_data.tags {
        println!("  {}: {}", name, value);
    }

    println!("\n=== Game 1 Flags ===");
    println!("  Flags byte: 0x{:02X}", game_data.flags);
    println!(
        "  Non-standard start: {}",
        game_data.start_position.is_some()
    );

    println!("\n=== Game 1 Move Data ===");
    println!("  Move data size: {} bytes", game_data.move_data.len());
    println!(
        "  First 10 bytes: {:02X?}",
        &game_data.move_data[..game_data.move_data.len().min(10)]
    );
}

#[test]
fn test_parse_all_five_games() {
    let si4_path = test_data_path("five.si4");
    let sn4_path = test_data_path("five.sn4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sn4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse all three files
    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let names = parse_name_database(&sn4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    println!("\n=== Complete Game Metadata ===\n");

    for (i, entry) in entries.iter().enumerate() {
        // Read game data
        let game_bytes =
            read_game_data(&mut sg4_file, entry.game_offset, entry.game_length).unwrap();

        // Parse structure
        let game_data = parse_game_structure(&game_bytes).unwrap();

        // Get names
        let white = entry.get_white_name(&names);
        let black = entry.get_black_name(&names);
        let event = entry.get_event_name(&names);
        let site = entry.get_site_name(&names);

        println!("Game {}:", i + 1);
        println!("  White: {}", white.unwrap_or("?"));
        println!("  Black: {}", black.unwrap_or("?"));
        println!("  Event: {}", event.unwrap_or("?"));
        println!("  Site: {}", site.unwrap_or("?"));
        println!("  Date: {}", entry.game_date.to_pgn_string());
        println!("  Result: {}", entry.result);
        println!("  Tags: {}", game_data.tags.len());
        println!("  Move data: {} bytes", game_data.move_data.len());
        println!();
    }
}

#[test]
fn test_validate_game_boundaries() {
    // Validate that games don't overlap and cover the file correctly
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();

    println!("\nGame boundaries:");
    for (i, entry) in entries.iter().enumerate() {
        let end = entry.game_offset + entry.game_length;
        println!(
            "  Game {}: offset={}, length={}, end={}",
            i + 1,
            entry.game_offset,
            entry.game_length,
            end
        );
    }

    // Check games are sequential with no gaps
    for i in 0..entries.len() - 1 {
        let current_end = entries[i].game_offset + entries[i].game_length;
        let next_start = entries[i + 1].game_offset;

        assert_eq!(
            current_end,
            next_start,
            "Game {} ends at {}, Game {} starts at {} - gap or overlap!",
            i + 1,
            current_end,
            i + 2,
            next_start
        );
    }

    println!("\nAll games are sequential with no gaps or overlaps");
}

#[test]
fn test_game_flags_parsing() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    println!("\nGame flags analysis:");
    for (i, entry) in entries.iter().enumerate() {
        let game_bytes =
            read_game_data(&mut sg4_file, entry.game_offset, entry.game_length).unwrap();
        let game_data = parse_game_structure(&game_bytes).unwrap();

        println!("Game {} flags: 0x{:02X}", i + 1, game_data.flags);
        if flags::is_set(game_data.flags, flags::NON_STANDARD_START) {
            println!("  - Non-standard start position");
        }
        if flags::is_set(game_data.flags, flags::HAS_PROMOTIONS) {
            println!("  - Has promotions");
        }
        if flags::is_set(game_data.flags, flags::HAS_UNDER_PROMOS) {
            println!("  - Has under-promotions");
        }
    }
}
