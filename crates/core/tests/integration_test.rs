//! Integration tests for SCID to PGN conversion
//!
//! These tests use complete SCID databases to verify end-to-end functionality.

use scidtopgn_core::{PgnOptions, ScidReader};
use std::fs;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn expected_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("expected")
        .join(name)
}

fn load_expected_pgn(name: &str) -> String {
    fs::read_to_string(expected_path(name)).expect(&format!("Should load expected PGN: {}", name))
}

fn normalize_pgn(pgn: &str) -> String {
    pgn.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn validate_pgn_structure(pgn: &str, context: &str) {
    assert!(pgn.contains("[Event"), "{}: Missing Event tag", context);
    assert!(pgn.contains("[Site"), "{}: Missing Site tag", context);
    assert!(pgn.contains("[Date"), "{}: Missing Date tag", context);
    assert!(pgn.contains("[Round"), "{}: Missing Round tag", context);
    assert!(pgn.contains("[White"), "{}: Missing White tag", context);
    assert!(pgn.contains("[Black"), "{}: Missing Black tag", context);
    assert!(pgn.contains("[Result"), "{}: Missing Result tag", context);

    assert!(
        pgn.contains("]\n\n") || pgn.contains("]\r\n\r\n"),
        "{}: Missing blank line after tags",
        context
    );
}

fn assert_pgn_matches_expected(actual: &str, expected: &str, context: &str) {
    let actual_normalized = normalize_pgn(actual);
    let expected_normalized = normalize_pgn(expected);

    if actual_normalized != expected_normalized {
        eprintln!("=== PGN Mismatch in {} ===", context);
        eprintln!("Expected:\n{}", expected);
        eprintln!("\nActual:\n{}", actual);
        eprintln!("=========================");
        panic!("PGN output does not match expected for {}", context);
    }
}

#[test]
fn test_open_minimal_database() {
    let db_path = fixture_path("minimal/minimal.si4");

    let reader = ScidReader::open(&db_path).expect("Should open minimal database");

    assert_eq!(reader.game_count(), 1, "Should have 1 game");
}

#[test]
fn test_parse_minimal_game_metadata() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).expect("Should get first game");

    assert_eq!(game.white(), "Carlsen, Magnus");
    assert_eq!(game.black(), "Nakamura, Hikaru");
    assert_eq!(game.event(), "Test Tournament");
    assert_eq!(game.site(), "Online");
    assert_eq!(game.date(), "2024.03.15");
}

#[test]
fn test_parse_minimal_game_moves() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();

    assert_eq!(game.moves().len(), 4, "Should have 4 moves");
}

#[test]
fn test_generate_pgn_minimal() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();
    let pgn = game.to_pgn().expect("Should generate PGN");

    validate_pgn_structure(&pgn, "minimal game");

    assert!(pgn.contains("Carlsen, Magnus"));
    assert!(pgn.contains("Nakamura, Hikaru"));
    assert!(pgn.contains("Test Tournament"));

    assert!(pgn.contains("1. e4 e5"));
    assert!(pgn.contains("2. Nf3 Nc6"));
}

#[test]
fn test_pgn_matches_expected() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game = reader.game(0).unwrap();
    let actual_pgn = game.to_pgn().unwrap();

    let expected_pgn = load_expected_pgn("minimal.pgn");

    assert_pgn_matches_expected(&actual_pgn, &expected_pgn, "minimal game");
}

#[test]
fn test_iterate_all_games() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let mut count = 0;
    for game_result in reader.games() {
        let game = game_result.expect("Should parse game");
        let pgn = game.to_pgn().expect("Should generate PGN");

        validate_pgn_structure(&pgn, &format!("game {}", count));
        count += 1;
    }

    assert_eq!(count, 1, "Should iterate over 1 game");
}

#[test]
fn test_multiple_game_access() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let game1 = reader.game(0).unwrap();
    let game2 = reader.game(0).unwrap();

    assert_eq!(game1.white(), game2.white());
    assert_eq!(game1.black(), game2.black());
    assert_eq!(game1.moves().len(), game2.moves().len());
}

#[test]
fn test_game_out_of_bounds() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let result = reader.game(999);
    assert!(result.is_err(), "Should fail for out-of-bounds index");
}

#[test]
fn test_pgn_options_compact() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let options = PgnOptions {
        compact: true,
        ..Default::default()
    };

    let mut output = Vec::new();
    reader
        .write_pgn(&mut output, &options)
        .expect("Should write compact PGN");

    let pgn = String::from_utf8(output).unwrap();

    let lines: Vec<&str> = pgn
        .lines()
        .skip_while(|line| line.starts_with('[') || line.is_empty())
        .collect();

    assert_eq!(
        lines.len(),
        1,
        "Compact format should have single line of moves"
    );
}

#[test]
fn test_pgn_options_verbose() {
    let db_path = fixture_path("minimal/minimal.si4");
    let reader = ScidReader::open(&db_path).unwrap();

    let options = PgnOptions {
        verbose: true,
        ..Default::default()
    };

    let mut output = Vec::new();
    reader
        .write_pgn(&mut output, &options)
        .expect("Should write verbose PGN");

    let pgn = String::from_utf8(output).unwrap();

    let move_lines: Vec<&str> = pgn
        .lines()
        .skip_while(|line| line.starts_with('[') || line.is_empty())
        .collect();

    assert!(
        move_lines.len() > 1,
        "Verbose format should have multiple lines"
    );
}
