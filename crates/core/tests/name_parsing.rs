//! Integration tests for SN4 name file parsing
//!
//! These tests validate parsing against real SCID database files.

use scidtopgn_core::database::index::parse_si4_file;
use scidtopgn_core::database::names::parse_name_database;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_sn4() {
    let path = test_data_path("five.sn4");

    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let names = parse_name_database(&path).expect("Failed to parse five.sn4");

    println!("Name Database Contents:");
    println!("  Players: {}", names.players.len());
    println!("  Events: {}", names.events.len());
    println!("  Sites: {}", names.sites.len());
    println!("  Rounds: {}", names.rounds.len());

    // Print all names for debugging
    println!("\nPlayer names:");
    for (i, name) in names.players.iter().enumerate() {
        println!("  [{}] '{}'", i, name);
    }

    println!("\nEvent names:");
    for (i, name) in names.events.iter().enumerate() {
        println!("  [{}] '{}'", i, name);
    }

    println!("\nSite names:");
    for (i, name) in names.sites.iter().enumerate() {
        println!("  [{}] '{}'", i, name);
    }

    println!("\nRound names:");
    for (i, name) in names.rounds.iter().enumerate() {
        println!("  [{}] '{}'", i, name);
    }
}

#[test]
fn test_parse_one_sn4() {
    let path = test_data_path("one.sn4");

    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let names = parse_name_database(&path).expect("Failed to parse one.sn4");

    println!("one.sn4 Name Database Contents:");
    println!("  Players: {}", names.players.len());
    println!("  Events: {}", names.events.len());
    println!("  Sites: {}", names.sites.len());
    println!("  Rounds: {}", names.rounds.len());
}

#[test]
fn test_names_with_index_entries() {
    let si4_path = test_data_path("five.si4");
    let sn4_path = test_data_path("five.sn4");

    if !si4_path.exists() || !sn4_path.exists() {
        eprintln!("Skipping test: five.si4 or five.sn4 not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).expect("Failed to parse five.si4");
    let names = parse_name_database(&sn4_path).expect("Failed to parse five.sn4");

    println!("\nGames with player names:");
    for (i, entry) in entries.iter().enumerate() {
        let white = entry.get_white_name(&names).unwrap_or("?");
        let black = entry.get_black_name(&names).unwrap_or("?");
        let event = entry.get_event_name(&names).unwrap_or("?");
        let site = entry.get_site_name(&names).unwrap_or("?");
        let round = entry.get_round_name(&names).unwrap_or("?");

        println!("\nGame {}:", i + 1);
        println!("  White: {} ({})", white, entry.white_elo);
        println!("  Black: {} ({})", black, entry.black_elo);
        println!("  Event: {}", event);
        println!("  Site: {}", site);
        println!("  Round: {}", round);
        println!("  Date: {}", entry.game_date.to_pgn_string());
        println!("  Result: {}", entry.result);
    }
}
