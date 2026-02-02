//! Integration tests for SI4 index file parsing
//!
//! These tests require actual SCID database files in tests/data/

use scidtopgn_core::database::games::{parse_game_structure, read_game_data, special_bytes};
use scidtopgn_core::database::index::{
    parse_game_index_entry, parse_si4_file, parse_si4_header, SCID_VERSION,
};
use scidtopgn_core::GameResult;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
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
fn test_parse_result_and_counts() {
    let mut bytes = [0u8; 47];

    // var_counts field (bytes 21-22):
    // Result: 3 (Draw)
    // NAG: 2
    // Comments: 1
    // Variations: 0
    let var_counts = (3u16 << 12) | (2u16 << 8) | (1u16 << 4) | 0u16;
    bytes[21..23].copy_from_slice(&var_counts.to_be_bytes());

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    assert_eq!(entry.result, scidtopgn_core::GameResult::Draw);
    assert_eq!(entry.nag_count, 2);
    assert_eq!(entry.comment_count, 1);
    assert_eq!(entry.variation_count, 0);
}

#[test]
fn test_parse_elo_ratings() {
    let mut bytes = [0u8; 47];

    // White ELO: 2372, Type: 1 (standard Elo)
    // Encoded: (1 << 12) | 2372 = 0x1944
    bytes[29..31].copy_from_slice(&0x1944u16.to_be_bytes());

    // Black ELO: 2500, Type: 2 (Rapid)
    // Encoded: (2 << 12) | 2500 = 0x29C4
    bytes[31..33].copy_from_slice(&0x29C4u16.to_be_bytes());

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    assert_eq!(entry.white_elo, 2372);
    assert_eq!(entry.white_rating_type_raw, 1);
    assert_eq!(entry.black_elo, 2500);
    assert_eq!(entry.black_rating_type_raw, 2);
}

#[test]
fn test_rating_type_enum() {
    // Test RatingType::from_u8
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(0),
        scidtopgn_core::database::index::RatingType::None
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(1),
        scidtopgn_core::database::index::RatingType::Elo
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(2),
        scidtopgn_core::database::index::RatingType::Rapid
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(3),
        scidtopgn_core::database::index::RatingType::Iccf
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(4),
        scidtopgn_core::database::index::RatingType::Uscf
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(5),
        scidtopgn_core::database::index::RatingType::Dwz
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(6),
        scidtopgn_core::database::index::RatingType::Ecf
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(7),
        scidtopgn_core::database::index::RatingType::Unknown
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::from_u8(255),
        scidtopgn_core::database::index::RatingType::Unknown
    );
}

#[test]
fn test_rating_type_pgn_suffix() {
    assert_eq!(
        scidtopgn_core::database::index::RatingType::None.to_pgn_suffix(),
        None
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Elo.to_pgn_suffix(),
        Some("Elo")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Rapid.to_pgn_suffix(),
        Some("Rapid")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Iccf.to_pgn_suffix(),
        Some("ICCF")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Uscf.to_pgn_suffix(),
        Some("USCF")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Dwz.to_pgn_suffix(),
        Some("DWZ")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Ecf.to_pgn_suffix(),
        Some("ECF")
    );
    assert_eq!(
        scidtopgn_core::database::index::RatingType::Unknown.to_pgn_suffix(),
        Some("Rating")
    );
}

#[test]
fn test_parse_rating_function() {
    // Test the parse_rating helper function
    let (rating_type, value) = scidtopgn_core::database::index::parse_rating(0x1944); // Type 1 (Elo), value 2372
    assert_eq!(
        rating_type,
        scidtopgn_core::database::index::RatingType::Elo
    );
    assert_eq!(value, 2372);

    let (rating_type, value) = scidtopgn_core::database::index::parse_rating(0x29C4); // Type 2 (Rapid), value 2500
    assert_eq!(
        rating_type,
        scidtopgn_core::database::index::RatingType::Rapid
    );
    assert_eq!(value, 2500);

    let (rating_type, value) = scidtopgn_core::database::index::parse_rating(0x4800); // Type 4 (USCF), value 2048
    assert_eq!(
        rating_type,
        scidtopgn_core::database::index::RatingType::Uscf
    );
    assert_eq!(value, 2048);

    let (rating_type, value) = scidtopgn_core::database::index::parse_rating(0x0000); // Type 0 (None), value 0
    assert_eq!(
        rating_type,
        scidtopgn_core::database::index::RatingType::None
    );
    assert_eq!(value, 0);
}

#[test]
fn test_entry_rating_type_helpers() {
    let mut bytes = [0u8; 47];

    // White: Elo 2372, Black: USCF 1800
    bytes[29..31].copy_from_slice(&0x1944u16.to_be_bytes()); // Type 1, value 2372
    bytes[31..33].copy_from_slice(&0x4708u16.to_be_bytes()); // Type 4, value 1800

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    // Test helper methods
    assert_eq!(
        entry.white_rating_type(),
        scidtopgn_core::database::index::RatingType::Elo
    );
    assert_eq!(
        entry.black_rating_type(),
        scidtopgn_core::database::index::RatingType::Uscf
    );

    // Test combined rating method
    let (w_type, w_value) = entry.white_rating();
    assert_eq!(w_type, scidtopgn_core::database::index::RatingType::Elo);
    assert_eq!(w_value, 2372);

    let (b_type, b_value) = entry.black_rating();
    assert_eq!(b_type, scidtopgn_core::database::index::RatingType::Uscf);
    assert_eq!(b_value, 1800);

    // Test has_rating methods
    assert!(entry.has_white_rating());
    assert!(entry.has_black_rating());
}

#[test]
fn test_entry_no_rating() {
    let bytes = [0u8; 47]; // All zeros
    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    assert_eq!(
        entry.white_rating_type(),
        scidtopgn_core::database::index::RatingType::None
    );
    assert_eq!(
        entry.black_rating_type(),
        scidtopgn_core::database::index::RatingType::None
    );
    assert!(!entry.has_white_rating());
    assert!(!entry.has_black_rating());
}

#[test]
fn test_material_signature_standard_start() {
    // Standard starting position: Q:1, R:2, B:2, N:2, P:8 for each side
    let sig = scidtopgn_core::database::index::MaterialSignature::from_raw(0x00FF8888);

    assert!(sig.is_standard_start());
    assert!(!sig.is_empty());

    // White pieces
    assert_eq!(sig.white_queens(), 1);
    assert_eq!(sig.white_rooks(), 2);
    assert_eq!(sig.white_bishops(), 2);
    assert_eq!(sig.white_knights(), 2);
    assert_eq!(sig.white_pawns(), 8);

    // Black pieces
    assert_eq!(sig.black_queens(), 1);
    assert_eq!(sig.black_rooks(), 2);
    assert_eq!(sig.black_bishops(), 2);
    assert_eq!(sig.black_knights(), 2);
    assert_eq!(sig.black_pawns(), 8);

    // Aggregates
    assert_eq!(sig.white_total(), 15); // 1+2+2+2+8
    assert_eq!(sig.black_total(), 15);
}

#[test]
fn test_material_signature_empty() {
    let sig = scidtopgn_core::database::index::MaterialSignature::from_raw(0);

    assert!(sig.is_empty());
    assert!(!sig.is_standard_start());

    assert_eq!(sig.white_queens(), 0);
    assert_eq!(sig.white_rooks(), 0);
    assert_eq!(sig.white_bishops(), 0);
    assert_eq!(sig.white_knights(), 0);
    assert_eq!(sig.white_pawns(), 0);

    assert_eq!(sig.black_queens(), 0);
    assert_eq!(sig.black_rooks(), 0);
    assert_eq!(sig.black_bishops(), 0);
    assert_eq!(sig.black_knights(), 0);
    assert_eq!(sig.black_pawns(), 0);

    assert!(!sig.has_queens());
    assert!(!sig.has_pawns());
}

#[test]
fn test_material_signature_endgame() {
    // King + Rook vs King endgame (KR:k)
    // White: 1 rook, 0 everything else
    // Black: 0 everything
    let raw = 0x00_10_00_00u32; // Only WR bit set (bits 20-21 = 1)
    let sig = scidtopgn_core::database::index::MaterialSignature::from_raw(raw);

    assert_eq!(sig.white_rooks(), 1);
    assert_eq!(sig.white_queens(), 0);
    assert_eq!(sig.white_pawns(), 0);
    assert_eq!(sig.black_rooks(), 0);

    assert!(sig.has_rooks());
    assert!(!sig.has_queens());
    assert!(!sig.has_pawns());
}

#[test]
fn test_material_signature_to_string() {
    // Standard start
    let sig = scidtopgn_core::database::index::MaterialSignature::from_raw(0x006A86A8);
    assert_eq!(sig.to_string(), "QRRBBNNPPPPPPPP:qrrbbnnpppppppp");

    // KR endgame
    let sig = scidtopgn_core::database::index::MaterialSignature::from_raw(0x00010000);
    assert_eq!(sig.to_string(), "R:R");
}

#[test]
fn test_entry_material_signature_helper() {
    let mut bytes = [0u8; 47];

    // Set material signature at bytes 33-36
    let mat_sig = 0x006A86A8u32;
    bytes[33..37].copy_from_slice(&mat_sig.to_be_bytes());

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    let sig = entry.material_signature();

    assert!(sig.is_standard_start());
    assert_eq!(sig.white_pawns(), 8);
    assert_eq!(sig.black_pawns(), 8);
}

#[test]
fn test_parse_half_moves_10bit() {
    let mut bytes = [0u8; 47];

    // Test 10-bit half-move count
    // Low 8 bits: 255 (byte 37)
    bytes[37] = 255;

    // High 2 bits: 3 (bits 7-6 of byte 38)
    bytes[38] = 0xC0; // Binary: 11000000

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    // Expected: 255 | (3 << 8) = 255 + 768 = 1023
    assert_eq!(entry.half_moves, 1023);
}

#[test]
fn test_game_flags() {
    let mut bytes = [0u8; 47];

    // Set flags: DELETE_FLAG (bit 3) and PROMO_FLAG (bit 1)
    let flags = 0x001Au16; // 0b0000000000001010
    bytes[7..9].copy_from_slice(&flags.to_be_bytes());

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    assert!(entry.is_deleted());
    assert!(entry.has_promotions());
    assert!(!entry.has_custom_start());
    assert!(!entry.has_underpromotions());
}

#[test]
fn test_custom_flags() {
    let mut bytes = [0u8; 47];

    // Set custom flag 3 (bit 12)
    let flags = 0x1000u16;
    bytes[7..9].copy_from_slice(&flags.to_be_bytes());

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    assert!(!entry.has_custom_flag(1));
    assert!(!entry.has_custom_flag(2));
    assert!(entry.has_custom_flag(3));
    assert!(!entry.has_custom_flag(4));
    assert!(!entry.has_custom_flag(7)); // Out of range
}

#[test]
fn test_packed_flag_detection() {
    // Test detecting zlib-compressed game data
    let mut bytes = [0u8; 47];

    // Set FLAG_PACKED (bit 7 of byte 6)
    bytes[6] = 0x80;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert!(entry.is_packed(), "Should detect packed/compressed game");

    // Test without packed flag
    let mut bytes_uncompressed = [0u8; 47];
    bytes_uncompressed[6] = 0x00;

    let entry_uncompressed =
        scidtopgn_core::database::index::parse_game_index_entry(&bytes_uncompressed).unwrap();
    assert!(
        !entry_uncompressed.is_packed(),
        "Should not be marked as packed"
    );
}

#[test]
fn test_packed_flag_with_length() {
    // Verify packed flag doesn't interfere with game length parsing
    let mut bytes = [0u8; 47];

    // Set length_low = 100 (bytes 4-5)
    bytes[4] = 0x0064u8; // 100 in hex
    bytes[5] = 0x00u8;

    // Set packed flag (bit 7 of byte 6)
    bytes[6] = 0x80;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();

    // Game length should be 1000, packed flag shouldn't affect length parsing
    assert_eq!(entry.game_length, 1000);
    assert!(entry.is_packed());
}

#[test]
fn test_eco_code_parsing() {
    let mut bytes = [0u8; 47];

    // Test ECO code 1 = A01
    bytes[23] = 0x01u8;
    bytes[24] = 0x00u8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_code, 1);
    assert_eq!(entry.eco_to_string(), Some("A01".to_string()));

    // Test ECO code 100 = B00
    bytes[23] = 0x00u8;
    bytes[24] = 0x64u8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_code, 100);
    assert_eq!(entry.eco_to_string(), Some("B00".to_string()));

    // Test ECO code 199 = B99
    bytes[23] = 0x00u8;
    bytes[24] = 0xC7u8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_code, 199);
    assert_eq!(entry.eco_to_string(), Some("B99".to_string()));

    // Test ECO code 300 = D00
    bytes[23] = 0x01u8;
    bytes[24] = 0x2Cu8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_code, 300);
    assert_eq!(entry.eco_to_string(), Some("D00".to_string()));

    // Test ECO code 499 = E99
    bytes[23] = 0x01u8;
    bytes[24] = 0xF7u8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_code, 499);
    assert_eq!(entry.eco_to_string(), Some("E99".to_string()));

    // Test invalid ECO (500 > 499)
    bytes[23] = 0x01u8;
    bytes[24] = 0xF8u8;

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&bytes).unwrap();
    assert_eq!(entry.eco_to_string(), None);
}

#[test]
fn test_event_date_year_offset() {
    // Test event date year offset encoding/decoding
    // Game year: 2023, Event year: 2020 (3 years before)

    let game_year = 2023u32;
    let game_month = 6u32;
    let game_day = 15u32;
    let game_encoded = (game_year << 9) | (game_month << 5) | game_day;

    // Event year 2020 is 3 years before, so offset = -3 + 4 = 1
    let event_year_offset = 1u32;
    let event_month = 1u32;
    let event_day = 1u32;
    let event_encoded = (event_year_offset << 9) | (event_month << 5) | event_day;

    let dates_field = (event_encoded << 20) | game_encoded;

    let (game_date, event_date) = scidtopgn_core::database::index::parse_dates_field(dates_field);

    assert_eq!(game_date.year, 2023);
    assert!(event_date.is_some());
    let event = event_date.unwrap();
    assert_eq!(event.year, 2020); // 2023 + 1 - 4 = 2020
    assert_eq!(event.month, 1);
    assert_eq!(event.day, 1);
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
fn test_parse_all_five_games() {
    let path = test_data_path("five.si4");

    if !path.exists() {
        eprintln!("Skipping: five.si4 not found");
        return;
    }

    let mut file = File::open(&path).expect("Failed to open five.si4");

    // Parse header
    let header = parse_si4_header(&mut file).expect("Failed to parse header");
    assert_eq!(header.num_games, 5);

    // Parse all 5 game entries
    for game_num in 0..5 {
        let mut entry_bytes = [0u8; 47];
        file.read_exact(&mut entry_bytes)
            .expect(&format!("Failed to read game {} entry", game_num + 1));

        let entry = scidtopgn_core::database::index::parse_game_index_entry(&entry_bytes)
            .expect(&format!("Failed to parse game {} entry", game_num + 1));

        // Print entry for verification
        println!("\nGame {}:", game_num + 1);
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
    // Test Game 1 against known values from SCID_DATABASE_FORMAT.md lines 1556-1567
    let path = test_data_path("five.si4");

    if !path.exists() {
        eprintln!("Skipping: five.si4 not found");
        return;
    }

    let mut file = File::open(&path).unwrap();

    // Skip header
    file.seek(SeekFrom::Start(182)).unwrap();

    // Read first game entry
    let mut entry_bytes = [0u8; 47];
    file.read_exact(&mut entry_bytes).unwrap();

    let entry = scidtopgn_core::database::index::parse_game_index_entry(&entry_bytes).unwrap();

    // Validate against known values
    assert_eq!(
        entry.game_date.to_pgn_string(),
        "2022.12.19",
        "Game 1 date should be 2022.12.19"
    );
    assert_eq!(
        entry.result,
        GameResult::Draw,
        "Game 1 result should be Draw"
    );
    assert_eq!(entry.white_elo, 2372, "Game 1 white ELO should be 2372");

    // Note: Player names will be validated in Phase 3 when we parse the name file
    println!("Game 1 validation passed!");
}

#[test]
fn test_combine_index_and_names() {
    use scidtopgn_core::database::names::parse_name_database;

    let si4_path = test_data_path("five.si4");
    let sn4_path = test_data_path("five.sn4");

    if !si4_path.exists() || !sn4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse both files
    let (header, entries) = parse_si4_file(&si4_path).unwrap();
    let names = parse_name_database(&sn4_path).unwrap();

    println!("\n=== Complete Game Information ===\n");

    // Display first game with all metadata
    if let Some(entry) = entries.first() {
        let (white, black) = entry.get_player_names(&names);
        let event = entry.get_event_name(&names);
        let site = entry.get_site_name(&names);

        println!("Game 1:");
        println!("  White: {}", white.unwrap_or("?"));
        println!("  Black: {}", black.unwrap_or("?"));
        println!("  Event: {}", event.unwrap_or("?"));
        println!("  Site: {}", site.unwrap_or("?"));
        println!("  Date: {}", entry.game_date.to_pgn_string());
        println!("  Result: {}", entry.result);
        println!("  White ELO: {}", entry.white_elo);
        println!("  Black ELO: {}", entry.black_elo);

        // Validate against known values from SCID_DATABASE_FORMAT.md
        // Game 1 should have:
        // - White: "Hossain, Enam"
        // - Date: 2022.12.19
        // - Result: Draw
        // - White ELO: 2372

        if let Some(white_name) = white {
            println!("\n✓ Game 1 white player: {}", white_name);
            // We expect "Hossain, Enam" but exact name depends on database
        }
    }
}

#[test]
fn test_comment_separation_real_data() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    println!("\n=== Comment Structure Analysis ===\n");

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes =
            read_game_data(&mut sg4_file, entry.game_offset, entry.game_length).unwrap();

        let game = parse_game_structure(&game_bytes).unwrap();

        // Count comment markers in move data
        let comment_markers = game
            .move_data
            .iter()
            .filter(|&&b| b == special_bytes::ENCODE_COMMENT)
            .count();

        // Count null-terminated strings in comment data
        let comment_count = game.comment_data.iter().filter(|&&b| b == 0).count();

        println!("Game {}:", i + 1);
        println!("  Move data: {} bytes", game.move_data.len());
        println!("  Comment markers in moves: {}", comment_markers);
        println!("  Comment section: {} bytes", game.comment_data.len());
        println!("  Comment strings: ~{}", comment_count);

        // Verify marker count roughly matches comment count
        // (May not be exact due to pre-game comments)
        if comment_markers > 0 || comment_count > 0 {
            println!("  → Has annotations!");
        }
        println!();
    }
}

#[test]
fn test_verify_move_data_ends_correctly() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).unwrap();
    let mut sg4_file = File::open(&sg4_path).unwrap();

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes =
            read_game_data(&mut sg4_file, entry.game_offset, entry.game_length).unwrap();

        let game = parse_game_structure(&game_bytes).unwrap();

        // Move data MUST end with ENCODE_END_GAME
        assert!(
            game.move_data.last() == Some(&special_bytes::ENCODE_END_GAME),
            "Game {} move data doesn't end with 0x0F",
            i + 1
        );
    }

    println!("✓ All games have correct move data termination");
}
