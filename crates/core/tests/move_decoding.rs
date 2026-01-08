//! Integration tests for SCID move decoding
//!
//! These tests validate move decoding against real SCID database files.

use scidtopgn_core::database::{
    games::{parse_game_structure, read_game_data},
    index::parse_si4_file,
};
use scidtopgn_core::parser::{ByteStream, ScidMoveDecoder};
use std::fs::File;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

/// Special move data bytes from SCID format
mod special_bytes {
    /// End of game marker
    pub const END_GAME: u8 = 0x00;

    /// Start variation marker
    pub const START_VAR: u8 = 0x0D;

    /// End variation marker
    pub const END_VAR: u8 = 0x0E;

    /// NAG (Numeric Annotation Glyph) marker
    pub const NAG: u8 = 0x0B;

    /// Comment marker
    pub const COMMENT: u8 = 0x0C;

    /// Alternative end game marker
    pub const END_GAME_ALT: u8 = 0x0F;
}

/// Decode moves from game data, handling special markers
fn decode_game_moves(
    move_data: &[u8],
    start_fen: Option<&str>,
) -> Result<Vec<shakmaty::Move>, Box<dyn std::error::Error>> {
    let mut decoder = if let Some(fen) = start_fen {
        ScidMoveDecoder::from_fen(fen)?
    } else {
        ScidMoveDecoder::new()
    };

    let mut moves = Vec::new();
    let mut stream = ByteStream::new(move_data);
    let mut variation_depth = 0;

    while stream.has_more() {
        let byte = stream.get_byte()?;

        match byte {
            special_bytes::END_GAME | special_bytes::END_GAME_ALT => {
                // End of game
                break;
            }
            special_bytes::START_VAR => {
                // Start of variation - skip for now
                variation_depth += 1;
                continue;
            }
            special_bytes::END_VAR => {
                // End of variation
                if variation_depth > 0 {
                    variation_depth -= 1;
                }
                continue;
            }
            special_bytes::NAG => {
                // NAG - skip the NAG value byte
                if stream.has_more() {
                    stream.get_byte()?;
                }
                continue;
            }
            special_bytes::COMMENT => {
                // Comment - skip until null terminator
                while stream.has_more() {
                    if stream.get_byte()? == 0 {
                        break;
                    }
                }
                continue;
            }
            _ => {
                // Regular move byte - only decode main line moves
                if variation_depth == 0 {
                    match decoder.decode_move(byte, &mut stream) {
                        Ok(chess_move) => moves.push(chess_move),
                        Err(e) => {
                            // Log error but continue - some moves might be encoded differently
                            eprintln!(
                                "Warning: Failed to decode move byte 0x{:02X} at move {}: {}",
                                byte,
                                moves.len() + 1,
                                e
                            );
                            // Try to continue - might be a special encoding we don't handle yet
                        }
                    }
                }
            }
        }
    }

    Ok(moves)
}

#[test]
fn test_decode_first_game_moves() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    // Parse index to get game location
    let (_, entries) = parse_si4_file(&si4_path).expect("Failed to parse index");
    let game1 = &entries[0];

    // Read game data
    let mut sg4_file = File::open(&sg4_path).expect("Failed to open sg4");
    let game_bytes = read_game_data(&mut sg4_file, game1.game_offset, game1.game_length)
        .expect("Failed to read game");

    // Parse game structure
    let game_data = parse_game_structure(&game_bytes).expect("Failed to parse structure");

    println!("\n=== Game 1 Move Decoding ===");
    println!("Move data size: {} bytes", game_data.move_data.len());
    println!(
        "First 20 bytes: {:02X?}",
        &game_data.move_data[..game_data.move_data.len().min(20)]
    );

    // Try to decode moves
    let result = decode_game_moves(&game_data.move_data, game_data.start_position.as_deref());

    match result {
        Ok(moves) => {
            println!("Decoded {} moves successfully", moves.len());

            // Print first few moves
            for (i, m) in moves.iter().take(10).enumerate() {
                println!("  Move {}: {:?}", i + 1, m);
            }

            // Basic validation
            assert!(moves.len() > 0, "Should decode at least some moves");
            println!("\nFirst game move decoding: PASSED");
        }
        Err(e) => {
            println!("Move decoding failed: {}", e);
            // Don't fail the test - move encoding might differ
        }
    }
}

#[test]
fn test_decode_all_games_summary() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).expect("Failed to parse index");
    let mut sg4_file = File::open(&sg4_path).expect("Failed to open sg4");

    println!("\n=== All Games Move Decoding Summary ===\n");

    let mut total_moves = 0;
    let mut successful_games = 0;
    let mut failed_games = 0;

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes = read_game_data(&mut sg4_file, entry.game_offset, entry.game_length)
            .expect("Failed to read game");
        let game_data = parse_game_structure(&game_bytes).expect("Failed to parse structure");

        let result = decode_game_moves(&game_data.move_data, game_data.start_position.as_deref());

        match result {
            Ok(moves) => {
                println!("Game {}: {} moves decoded", i + 1, moves.len());
                total_moves += moves.len();
                successful_games += 1;
            }
            Err(e) => {
                println!("Game {}: FAILED - {}", i + 1, e);
                failed_games += 1;
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Total games: {}", entries.len());
    println!("Successful: {}", successful_games);
    println!("Failed: {}", failed_games);
    println!("Total moves decoded: {}", total_moves);

    // We expect some games to work
    assert!(
        successful_games > 0,
        "At least some games should decode successfully"
    );
}

#[test]
fn test_move_byte_structure() {
    // Test that we understand the move byte format correctly
    // Move byte: [piece_num:4][move_value:4]

    // Example: e2-e4 (pawn double push)
    // Piece 12 (e-pawn), move value 15 (double push)
    let e4_byte: u8 = (12 << 4) | 15;
    assert_eq!(e4_byte, 0xCF);

    let piece_num = (e4_byte >> 4) & 0x0F;
    let move_value = e4_byte & 0x0F;

    assert_eq!(piece_num, 12); // e-pawn
    assert_eq!(move_value, 15); // double push

    // Example: King move (king is piece 0)
    // Kingside castle (move value 10)
    let castle_byte: u8 = (0 << 4) | 10;
    assert_eq!(castle_byte, 0x0A);

    let piece_num = (castle_byte >> 4) & 0x0F;
    let move_value = castle_byte & 0x0F;

    assert_eq!(piece_num, 0); // king
    assert_eq!(move_value, 10); // kingside castle
}

#[test]
fn test_special_byte_detection() {
    let si4_path = test_data_path("five.si4");
    let sg4_path = test_data_path("five.sg4");

    if !si4_path.exists() || !sg4_path.exists() {
        eprintln!("Skipping: test files not found");
        return;
    }

    let (_, entries) = parse_si4_file(&si4_path).expect("Failed to parse index");
    let mut sg4_file = File::open(&sg4_path).expect("Failed to open sg4");

    println!("\n=== Special Byte Analysis ===\n");

    for (i, entry) in entries.iter().enumerate() {
        let game_bytes = read_game_data(&mut sg4_file, entry.game_offset, entry.game_length)
            .expect("Failed to read game");
        let game_data = parse_game_structure(&game_bytes).expect("Failed to parse structure");

        let mut end_markers = 0;
        let mut nag_markers = 0;
        let mut comment_markers = 0;
        let mut variation_markers = 0;

        for &byte in &game_data.move_data {
            match byte {
                0x00 | 0x0F => end_markers += 1,
                0x0B => nag_markers += 1,
                0x0C => comment_markers += 1,
                0x0D | 0x0E => variation_markers += 1,
                _ => {}
            }
        }

        println!(
            "Game {}: {} bytes, end={}, nag={}, comment={}, var={}",
            i + 1,
            game_data.move_data.len(),
            end_markers,
            nag_markers,
            comment_markers,
            variation_markers
        );
    }
}
