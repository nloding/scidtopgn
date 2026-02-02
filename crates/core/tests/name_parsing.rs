//! Integration tests for SN4 name file parsing
//!
//! These tests require actual SCID database files in test/data/

use scidtopgn_core::database::names::parse_name_database;
use std::path::PathBuf;

/// Get path to test data file
fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_sn4_complete() {
    let path = test_data_path("five.sn4");

    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let names = parse_name_database(&path).expect("Failed to parse five.sn4");

    assert!(names.players.len() > 0, "Should have player names");
    assert!(names.events.len() > 0, "Should have event names");
    assert!(names.sites.len() > 0, "Should have site names");

    println!("\n=== Players (first 10) ===");
    for (i, name) in names.players.iter().take(10).enumerate() {
        println!("  {}: {}", i, name);
    }

    println!("\n=== Events (first 5) ===");
    for (i, name) in names.events.iter().take(5).enumerate() {
        println!("  {}: {}", i, name);
    }

    println!("\n=== Sites (first 5) ===");
    for (i, name) in names.sites.iter().take(5).enumerate() {
        println!("  {}: {}", i, name);
    }

    println!("\n=== Summary ===");
    println!("Total players: {}", names.players.len());
    println!("Total events: {}", names.events.len());
    println!("Total sites: {}", names.sites.len());
    println!("Total rounds: {}", names.rounds.len());
}

#[test]
fn test_validate_known_names() {
    let path = test_data_path("five.sn4");

    if !path.exists() {
        eprintln!("Skipping test: five.sn4 not found");
        return;
    }

    let names = parse_name_database(&path).unwrap();

    for window in names.players.windows(2) {
        assert!(
            window[0] <= window[1],
            "Players should be alphabetically sorted: '{}' > '{}'",
            window[0],
            window[1]
        );
    }

    for window in names.events.windows(2) {
        assert!(
            window[0] <= window[1],
            "Events should be alphabetically sorted: '{}' > '{}'",
            window[0],
            window[1]
        );
    }

    println!("✓ All names are alphabetically sorted");
}

#[test]
fn test_name_database_accessors() {
    let path = test_data_path("five.sn4");

    if !path.exists() {
        eprintln!("Skipping test: five.sn4 not found");
        return;
    }

    let names = parse_name_database(&path).unwrap();

    if names.players.len() > 0 {
        assert!(names.get_player(0).is_some());
        assert_eq!(names.get_player(0).unwrap(), &names.players[0]);
    }

    assert!(names.get_player(999999).is_none());
}
