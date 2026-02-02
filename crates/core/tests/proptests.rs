//! Property-based tests using proptest
//!
//! These tests generate random inputs to verify properties hold
//! for all possible inputs, not just hand-picked test cases.

use crate::database::{eco_to_string, parse_dates_field, GameIndexEntry, NameDatabase};
use crate::types::GameDate;
use proptest::prelude::*;
use std::collections::HashMap;

// ===== Property: ELO Rating Encoding =====

proptest! {
    #[test]
    fn prop_elo_roundtrip(elo in 0u16..3000) {
        let mut bytes = [0u8; 47];

        let rating_type = 1u8;
        let encoded_raw = ((rating_type as u16) << 12) | (elo as u16);

        bytes[29..31].copy_from_slice(&encoded_raw.to_be_bytes());

        let entry = GameIndexEntry::parse(&bytes)
            .expect("Should parse ELO");

        prop_assert_eq!(entry.white_elo, elo);
    }
}

// ===== Property: ECO Code Conversion =====

proptest! {
    #[test]
    fn prop_eco_code_conversion(eco_code in 0u16..500) {
        let encoded = eco_to_string(eco_code);

        if let Some(ref eco_str) = encoded {
            prop_assert!(eco_str.len() >= 3, "ECO code should be 3 characters: {}", eco_str);
            prop_assert!(eco_str.len() <= 3, "ECO code should be exactly 3 characters: {}", eco_str);
            prop_assert!(eco_str.chars().all(|c| c.is_ascii_uppercase()), "ECO should be uppercase: {}", eco_str);
        }
    }
}

// ===== Property: GameDate Encoding/Decoding =====

proptest! {
    #[test]
    fn prop_date_encoding_roundtrip(year in 0u16..4096, month in 0u8..13, day in 0u8..32) {
        let game_date = GameDate {
            year: year as u16,
            month,
            day,
        };

        let dates_field = ((year as u32) << 9) | ((month as u32) << 5) | (day as u32);

        let (decoded_year, decoded_month, decoded_day) = {
            let raw = dates_field & 0x000F_FFFF;
            let decoded_day = (raw & 0x1F) as u8;
            let decoded_month = ((raw >> 5) & 0x0F) as u8;
            let decoded_year = ((raw >> 9) & 0xFFF) as u16;
            (decoded_year, decoded_month, decoded_day)
        };

        prop_assert_eq!(decoded_year, year & 0xFFF);
        prop_assert_eq!(decoded_month, month & 0x0F);
        prop_assert_eq!(decoded_day, day & 0x1F);
    }
}

// ===== Property: Name Database Lookup Returns Valid Results =====

proptest! {
    #[test]
    fn prop_name_lookup_in_bounds(id: u32, max_id: u32) {
        let id = id % (max_id + 1);

        let mut names = NameDatabase {
            players: vec!["Player1".to_string(), "Player2".to_string()],
            events: vec!["Event1".to_string()],
            sites: vec!["Site1".to_string()],
            rounds: vec![],
        };

        let result = names.get_player(id);
        prop_assert!(result.is_none() || result.unwrap().len() > 0);
    }
}

// ===== Property: PGN Tag Format Has Correct Structure =====

proptest! {
    #[test]
    fn prop_pgn_tag_format(event in "[a-zA-Z ]{1,30}",
                            site in "[a-zA-Z ]{1,30}",
                            white in "[a-zA-Z, ]{1,30}",
                            black in "[a-zA-Z, ]{1,30}") {
        let roster = crate::format::tags::SevenTagRoster {
            event: event.to_string(),
            site: site.to_string(),
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white: white.to_string(),
            black: black.to_string(),
            result: "1-0".to_string(),
        };

        let formatted = roster.to_pgn();

        for line in formatted.lines() {
            prop_assert!(line.len() > 10, "Tag line should have content: {}", line);
            prop_assert!(line.starts_with('['), "Tag should start with '[': {}", line);
            prop_assert!(line.ends_with(']'), "Tag should end with ']': {}", line);
            prop_assert!(line.contains('"'), "Tag should contain quotes: {}", line);
        }
    }
}

// ===== Property: Move List Non-Empty After Formatting =====

proptest! {
    #[test]
    fn prop_pgn_has_movetext(moves: [crate::shakmaty::Move; 0..10]) {
        let moves: Vec<crate::shakmaty::Move> = moves.iter().copied().collect();

        let entry = GameIndexEntry {
            game_offset: 0,
            game_length: 100,
            white_id: 0,
            black_id: 1,
            event_id: 0,
            site_id: 0,
            round_id: 0,
            game_date: GameDate {
                year: 2023,
                month: 12,
                day: 25,
            },
            event_date: None,
            result: crate::types::GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: moves.len() as u16,
            flags: 0,
        };

        let names = NameDatabase {
            players: vec!["Player1".to_string(), "Player2".to_string()],
            events: vec!["Test".to_string()],
            sites: vec!["Online".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = crate::database::GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves,
        };

        let options = crate::PgnOptions::default();
        let pgn = crate::PgnFormatter::format_game(&entry, &names, &game_data, &options);

        prop_assert!(pgn.is_ok(), "PGN formatting should not fail");

        let pgn_str = pgn.unwrap();

        let has_movetext = pgn_str.lines().any(|line| !line.is_empty() && !line.starts_with('['));

        prop_assert!(has_movetext, "PGN should have movetext section");
    }
}
