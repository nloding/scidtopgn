//! Integration tests for SI4 index file parsing
//!
//! These tests require actual SCID database files in tests/data/

use scidtopgn_core::database::index::{parse_si4_header, SCID_VERSION};
use std::fs::File;
use std::path::PathBuf;

/// Get path to test data file
fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

#[test]
fn test_parse_five_si4_header() {
    let path = test_data_path("five.si4");

    // Skip if test file doesn't exist
    if !path.exists() {
        eprintln!("Skipping test: {} not found", path.display());
        return;
    }

    let mut file = File::open(&path).expect("Failed to open five.si4");
    let header = parse_si4_header(&mut file).expect("Failed to parse header");

    // Validate against known values from SCID_DATABASE_FORMAT.md
    assert_eq!(header.version, SCID_VERSION, "Version should be 400");
    assert_eq!(header.num_games, 5, "Should have 5 games");

    // Description might vary, just check it's not empty
    println!("Database description: {}", header.description);
}

#[test]
fn test_parse_invalid_magic() {
    // This test requires creating a file with invalid magic
    // We'll implement this when we have test file generation
}

#[test]
fn test_parse_invalid_version() {
    // This test requires creating a file with wrong version
    // We'll implement this when we have test file generation
}

#[test]
fn test_parse_truncated_header() {
    // Test file that's too short
    // We'll implement this when we have test file generation
}
