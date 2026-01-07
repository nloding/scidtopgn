//! Integration tests for SI4 index file parsing
//!
//! These tests validate parsing against real SCID database files.

use scidtopgn_core::database::index::{parse_si4_file, parse_si4_header, SCID_VERSION};
use scidtopgn_core::GameResult;
use std::fs::File;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_si4_header() {
    let path = test_data_path("five.si4");

    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let mut file = File::open(&path).expect("Failed to open five.si4");
    let header = parse_si4_header(&mut file).expect("Failed to parse header");

    assert_eq!(header.version, SCID_VERSION, "Version should be 400");
    assert_eq!(header.num_games, 5, "Should have 5 games");

    println!("Database description: '{}'", header.description);
    println!("Games: {}", header.num_games);
}

#[test]
fn test_parse_all_five_games() {
    let path = test_data_path("five.si4");

    if !path.exists() {
        eprintln!("Skipping: five.si4 not found");
        return;
    }

    let (header, entries) = parse_si4_file(&path).expect("Failed to parse five.si4");

    assert_eq!(header.num_games, 5);
    assert_eq!(entries.len(), 5);

    for (i, entry) in entries.iter().enumerate() {
        println!("\nGame {}:", i + 1);
        println!(
            "  Offset: {}, Length: {}",
            entry.game_offset, entry.game_length
        );
        println!("  Date: {}", entry.game_date.to_pgn_string());
        println!("  Result: {}", entry.result);
        println!(
            "  White ELO: {}, Black ELO: {}",
            entry.white_elo, entry.black_elo
        );
        println!("  Half-moves: {}", entry.half_moves);
        println!(
            "  White ID: {}, Black ID: {}",
            entry.white_id, entry.black_id
        );
    }
}

#[test]
fn test_parse_game_1_specific_values() {
    // Test Game 1 against known values from SCID_DATABASE_FORMAT.md
    let path = test_data_path("five.si4");

    if !path.exists() {
        eprintln!("Skipping: five.si4 not found");
        return;
    }

    let (_, entries) = parse_si4_file(&path).expect("Failed to parse five.si4");

    let game1 = &entries[0];

    // Validate against known values
    assert_eq!(
        game1.game_date.to_pgn_string(),
        "2022.12.19",
        "Game 1 date should be 2022.12.19"
    );
    assert_eq!(
        game1.result,
        GameResult::Draw,
        "Game 1 result should be Draw"
    );
    assert_eq!(game1.white_elo, 2372, "Game 1 white ELO should be 2372");

    println!("Game 1 validation passed!");
    println!("  Date: {}", game1.game_date.to_pgn_string());
    println!("  Result: {}", game1.result);
    println!("  White ELO: {}", game1.white_elo);
    println!("  Black ELO: {}", game1.black_elo);
}

#[test]
fn test_parse_one_si4() {
    let path = test_data_path("one.si4");

    if !path.exists() {
        eprintln!("Skipping: one.si4 not found");
        return;
    }

    let (header, entries) = parse_si4_file(&path).expect("Failed to parse one.si4");

    assert_eq!(header.num_games, 1, "Should have 1 game");
    assert_eq!(entries.len(), 1);

    let game = &entries[0];
    println!("one.si4 Game 1:");
    println!("  Date: {}", game.game_date.to_pgn_string());
    println!("  Result: {}", game.result);
    println!(
        "  White ELO: {}, Black ELO: {}",
        game.white_elo, game.black_elo
    );
}
