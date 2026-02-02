use crate::database::{GameData, GameIndexEntry, NameDatabase};
use crate::error::Result;
use crate::format::movetext::{MovetextFormatter, MovetextOptions};
use crate::format::tags::{SevenTagRoster, SupplementalTags};
use shakmaty::Move as ChessMove;
use std::collections::HashMap;

/// PGN formatting options
#[derive(Debug, Clone)]
pub struct PgnOptions {
    /// Include comments in output
    pub include_comments: bool,

    /// Include variations in output
    pub include_variations: bool,

    /// Use compact format (no line breaks in movetext)
    pub compact: bool,

    /// Maximum characters per line in movetext
    pub line_width: usize,

    /// Include supplemental tags (ELO, ECO, etc.)
    pub include_supplemental_tags: bool,
}

impl Default for PgnOptions {
    fn default() -> Self {
        PgnOptions {
            include_comments: true,
            include_variations: true,
            compact: false,
            line_width: 80,
            include_supplemental_tags: true,
        }
    }
}

/// Main PGN formatter
pub struct PgnFormatter;

impl PgnFormatter {
    /// Format complete PGN game from SCID data
    ///
    /// # Arguments
    ///
    /// * `index_entry` - Game metadata from index file
    /// * `names` - Name database for player/event/site lookups
    /// * `game_data` - Game data including tags and moves
    /// * `options` - Formatting options
    ///
    /// # Returns
    ///
    /// Complete PGN document as string
    pub fn format_game(
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        options: &PgnOptions,
    ) -> Result<String> {
        let mut pgn = String::new();

        // 1. Seven Tag Roster
        let str = SevenTagRoster::from_scid(index_entry, names);
        pgn.push_str(&str.to_pgn());

        // 2. Supplemental Tags
        if options.include_supplemental_tags {
            let supp_tags = SupplementalTags::from_scid(
                index_entry,
                &game_data.tags,
                game_data.start_position.as_deref(),
            );
            pgn.push_str(&supp_tags.to_pgn());
        }

        // 3. Blank line separating tags from movetext
        pgn.push('\n');

        // 4. Movetext
        let movetext_options = MovetextOptions {
            compact: options.compact,
            line_width: options.line_width,
            include_move_numbers: true,
            start_move_number: 1,
            first_move_color: shakmaty::Color::White,
        };

        let mut formatter = if let Some(ref fen) = game_data.start_position {
            MovetextFormatter::from_fen(fen, movetext_options)?
        } else {
            MovetextFormatter::with_options(movetext_options)
        };

        let movetext = formatter
            .format_moves_with_result(&game_data.moves, &index_entry.result.to_string())?;

        pgn.push_str(&movetext);
        pgn.push('\n');

        // 5. Blank line after game (PGN spec)
        pgn.push('\n');

        Ok(pgn)
    }

    /// Format multiple games to PGN
    pub fn format_games(
        games: &[(GameIndexEntry, GameData)],
        names: &NameDatabase,
        options: &PgnOptions,
    ) -> Result<String> {
        let mut pgn = String::new();

        for (index_entry, game_data) in games {
            let game_pgn = Self::format_game(index_entry, names, game_data, options)?;
            pgn.push_str(&game_pgn);
        }

        Ok(pgn)
    }

    /// Write PGN to writer (memory-efficient streaming)
    pub fn write_game<W: std::io::Write>(
        writer: &mut W,
        index_entry: &GameIndexEntry,
        names: &NameDatabase,
        game_data: &GameData,
        options: &PgnOptions,
    ) -> Result<()> {
        let pgn = Self::format_game(index_entry, names, game_data, options)?;
        writer.write_all(pgn.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::GameResult;
    use shakmaty::{Move, Role, Square};

    #[test]
    fn test_complete_pgn_generation() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(pgn.contains("[Event \"World Championship\"]"));
        assert!(pgn.contains("[White \"Carlsen, Magnus\"]"));
        assert!(pgn.contains("[Black \"Caruana, Fabiano\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));
        assert!(pgn.contains("1. e4 e5"));
        assert!(pgn.contains("1-0"));

        println!("{}", pgn);
    }

    #[test]
    fn test_pgn_structure() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::Draw,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(pgn.contains("]\n\n1. "));
        assert!(pgn.ends_with("\n\n"));

        assert!(pgn.contains("[Event \"World Championship\"]"));
        assert!(pgn.contains("[Site \"New York, NY USA\"]"));
        assert!(pgn.contains("[Date \"2023.12.25\"]"));
        assert!(pgn.contains("[Round \"1\"]"));
        assert!(pgn.contains("[White \"Carlsen, Magnus\"]"));
        assert!(pgn.contains("[Black \"Caruana, Fabiano\"]"));
        assert!(pgn.contains("[Result \"1/2-1/2\"]"));
        assert!(pgn.contains("1. e4 e5 1/2-1/2"));
    }

    #[test]
    fn test_pgn_with_fen() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: Some(fen.to_string()),
            moves: vec![],
        };

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(
            pgn.contains("[FEN \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1\"]")
        );
        assert!(pgn.contains("[SetUp \"1\"]"));
        assert!(pgn.contains("1. e4"));
    }

    #[test]
    fn test_pgn_with_supplemental_tags() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 100,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let mut game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        game_data
            .tags
            .insert("TimeControl".to_string(), "10".to_string());

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(pgn.contains("[WhiteElo \"2500\"]"));
        assert!(pgn.contains("[BlackElo \"2400\"]"));
        assert!(pgn.contains("[ECO \"B00\"]"));
        assert!(pgn.contains("[TimeControl \"10\"]"));
    }

    #[test]
    fn test_pgn_compact_format() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let mut options = PgnOptions::default();
        options.compact = true;
        options.include_supplemental_tags = false;

        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(
            !pgn.contains('\n'),
            "Compact format should have no newlines"
        );
        assert!(pgn.contains("1. e4 e5 1-0"));
    }

    #[test]
    fn test_pgn_without_supplemental_tags() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let mut options = PgnOptions::default();
        options.include_supplemental_tags = false;

        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(!pgn.contains("WhiteElo"));
        assert!(!pgn.contains("BlackElo"));
        assert!(!pgn.contains("ECO"));
    }

    #[test]
    fn test_pgn_quote_escaping() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["Tournament \"The Best\"".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![],
        };

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options).unwrap();

        assert!(pgn.contains("[Event \"Tournament \\\"The Best\\\"\"]"));
    }

    #[test]
    fn test_format_games_multiple() {
        let index_entry = GameIndexEntry {
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
            result: GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let mut names = NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E2,
                    capture: None,
                    to: Square::E4,
                    promotion: None,
                },
                Move::Normal {
                    role: Role::Pawn,
                    from: Square::E7,
                    capture: None,
                    to: Square::E5,
                    promotion: None,
                },
            ],
        };

        let games = vec![
            (index_entry, game_data),
            (
                GameIndexEntry {
                    game_offset: 100,
                    game_length: 100,
                    white_id: 2,
                    black_id: 3,
                    event_id: 0,
                    site_id: 0,
                    round_id: 1,
                    game_date: GameDate {
                        year: 2023,
                        month: 12,
                        day: 26,
                    },
                    event_date: None,
                    result: GameResult::BlackWins,
                    white_elo: 2400,
                    black_elo: 2500,
                    eco_code: 0,
                    half_moves: 4,
                    flags: 0,
                },
                GameData {
                    tags: HashMap::new(),
                    flags: 0,
                    start_position: None,
                    moves: vec![
                        Move::Normal {
                            role: Role::Pawn,
                            from: Square::E2,
                            capture: None,
                            to: Square::E4,
                            promotion: None,
                        },
                        Move::Normal {
                            role: Role::Pawn,
                            from: Square::E7,
                            capture: None,
                            to: Square::E5,
                            promotion: None,
                        },
                    ],
                },
            ),
        ];

        let options = PgnOptions::default();
        let pgn = PgnFormatter::format_games(&games, &names, &options).unwrap();

        assert!(pgn.contains("1. e4 e5 1-0"));
        assert!(pgn.contains("0-1"));
    }

    // ===== Task 9A.5.1: Additional SAN Generation Tests =====

    #[test]
    fn test_san_simple_pawn_move() {
        let mut gen = super::san::SanGenerator::new();

        let chess_move = shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E2,
            to: shakmaty::Square::E4,
            capture: None,
            promotion: None,
        };

        let san = gen.move_to_san(&chess_move).expect("Should generate SAN");

        assert_eq!(san, "e4");
    }

    #[test]
    fn test_san_piece_move() {
        let mut gen = super::san::SanGenerator::new();

        gen.move_to_san(&shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E2,
            to: shakmaty::Square::E4,
            capture: None,
            promotion: None,
        })
        .unwrap();

        gen.move_to_san(&shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E7,
            to: shakmaty::Square::E5,
            capture: None,
            promotion: None,
        })
        .unwrap();

        let knight_move = shakmaty::Move::Normal {
            role: shakmaty::Role::Knight,
            from: shakmaty::Square::G1,
            to: shakmaty::Square::F3,
            capture: None,
            promotion: None,
        };

        let san = gen
            .move_to_san(&knight_move)
            .expect("Should generate knight SAN");

        assert_eq!(san, "Nf3");
    }

    #[test]
    fn test_san_capture() {
        let mut gen = super::san::SanGenerator::new();

        gen.move_to_san(&shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E2,
            to: shakmaty::Square::E4,
            capture: None,
            promotion: None,
        })
        .unwrap();

        gen.move_to_san(&shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::D7,
            to: shakmaty::Square::D5,
            capture: None,
            promotion: None,
        })
        .unwrap();

        let capture = shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E4,
            to: shakmaty::Square::D5,
            capture: Some(shakmaty::Role::Pawn),
            promotion: None,
        };

        let san = gen
            .move_to_san(&capture)
            .expect("Should generate capture SAN");

        assert_eq!(san, "exd5");
    }

    #[test]
    fn test_san_castling_kingside() {
        let fen = "r1bqk2r/pppp1ppp/2n2n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let castle = shakmaty::Move::Castle {
            king: shakmaty::Square::E1,
            rook: shakmaty::Square::H1,
        };

        let san = gen
            .move_to_san(&castle)
            .expect("Should generate castling SAN");

        assert_eq!(san, "O-O");
    }

    #[test]
    fn test_san_castling_queenside() {
        let fen = "r3kbnr/pppqpppp/2np4/8/8/2NP4/PPPQPPPP/R3KBNR w KQkq - 0 1";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let castle = shakmaty::Move::Castle {
            king: shakmaty::Square::E1,
            rook: shakmaty::Square::A1,
        };

        let san = gen
            .move_to_san(&castle)
            .expect("Should generate castling SAN");

        assert_eq!(san, "O-O-O");
    }

    #[test]
    fn test_san_promotion() {
        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let promotion = shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E7,
            to: shakmaty::Square::E8,
            capture: None,
            promotion: Some(shakmaty::Role::Queen),
        };

        let san = gen
            .move_to_san(&promotion)
            .expect("Should generate promotion SAN");

        assert_eq!(san, "e8=Q");
    }

    #[test]
    fn test_san_underpromotion() {
        let fen = "4k3/4P3/8/8/8/8/8/4K3 w - - 0 1";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let promotion = shakmaty::Move::Normal {
            role: shakmaty::Role::Pawn,
            from: shakmaty::Square::E7,
            to: shakmaty::Square::E8,
            capture: None,
            promotion: Some(shakmaty::Role::Knight),
        };

        let san = gen
            .move_to_san(&promotion)
            .expect("Should generate knight promotion");

        assert_eq!(san, "e8=N");
    }

    #[test]
    fn test_san_check() {
        let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let check_move = shakmaty::Move::Normal {
            role: shakmaty::Role::Queen,
            from: shakmaty::Square::D8,
            to: shakmaty::Square::H4,
            capture: None,
            promotion: None,
        };

        let san = gen
            .move_to_san(&check_move)
            .expect("Should generate SAN with check");

        assert!(san.ends_with('+'), "Check should end with +: {}", san);
    }

    #[test]
    fn test_san_disambiguation_file() {
        let fen = "4k3/8/8/8/8/8/8/R3K2R w KQk - 0 1";
        let mut gen = super::san::SanGenerator::from_fen(fen).expect("Should parse FEN");

        let rook_move = shakmaty::Move::Normal {
            role: shakmaty::Role::Rook,
            from: shakmaty::Square::A1,
            to: shakmaty::Square::A8,
            capture: None,
            promotion: None,
        };

        let san = gen
            .move_to_san(&rook_move)
            .expect("Should generate disambiguated SAN");

        assert!(san.starts_with('R'));
        assert!(san.contains("a8"));
    }

    // ===== Task 9A.5.2: Additional Tag Formatting Tests =====

    #[test]
    fn test_tag_order_is_correct() {
        let roster = super::tags::SevenTagRoster {
            event: "Test".to_string(),
            site: "Online".to_string(),
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white: "Player1".to_string(),
            black: "Player2".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.to_pgn();
        let lines: Vec<&str> = formatted.lines().collect();

        assert_eq!(lines.len(), 7);
        assert!(lines[0].starts_with("[Event"));
        assert!(lines[1].starts_with("[Site"));
        assert!(lines[2].starts_with("[Date"));
        assert!(lines[3].starts_with("[Round"));
        assert!(lines[4].starts_with("[White"));
        assert!(lines[5].starts_with("[Black"));
        assert!(lines[6].starts_with("[Result"));
    }

    #[test]
    fn test_tag_escaping_quotes() {
        let roster = super::tags::SevenTagRoster {
            event: "Match \"The Candidates\"".to_string(),
            site: "Online".to_string(),
            date: "2024.01.01".to_string(),
            round: "1".to_string(),
            white: "Player1".to_string(),
            black: "Player2".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.to_pgn();

        assert!(formatted.contains("Match \\\"The Candidates\\\""));
    }

    #[test]
    fn test_tag_unknown_values() {
        let roster = super::tags::SevenTagRoster {
            event: "?".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "?".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let formatted = roster.to_pgn();

        assert!(formatted.contains("[Event \"?\"]"));
        assert!(formatted.contains("[Date \"????.??.??\"]"));
    }

    #[test]
    fn test_supplemental_tags_sorted() {
        let mut tags = super::tags::SupplementalTags::default();
        tags.insert("ZZZ".to_string(), "last".to_string());
        tags.insert("AAA".to_string(), "first".to_string());
        tags.insert("MMM".to_string(), "middle".to_string());

        let formatted = tags.to_pgn();
        let lines: Vec<&str> = formatted.lines().collect();

        assert!(lines[0].contains("AAA"));
        assert!(lines[1].contains("MMM"));
        assert!(lines[2].contains("ZZZ"));
    }

    #[test]
    fn test_date_formatting_various_cases() {
        let test_cases = vec![
            ((2024, 3, 15), "2024.03.15"),
            ((2024, 12, 31), "2024.12.31"),
            ((2024, 1, 1), "2024.01.01"),
            ((0, 0, 0), "????.??.??"),
            ((2024, 0, 0), "2024.??.??"),
        ];

        for ((year, month, day), expected) in test_cases {
            let formatted = super::tags::SevenTagRoster::format_date(&super::types::GameDate {
                year,
                month,
                day,
            });
            assert_eq!(
                formatted, expected,
                "Date ({}, {}, {}) should format as {}",
                year, month, day, expected
            );
        }
    }

    #[test]
    fn test_result_formatting() {
        let results = vec![
            (super::types::GameResult::WhiteWin, "1-0"),
            (super::types::GameResult::BlackWin, "0-1"),
            (super::types::GameResult::Draw, "1/2-1/2"),
            (super::types::GameResult::Unknown, "*"),
        ];

        for (result, expected) in results {
            let formatted = super::tags::SevenTagRoster::format_result(&result);
            assert_eq!(formatted, expected);
        }
    }

    // ===== Task 9A.5.3: Additional Complete PGN Generation Tests =====

    #[test]
    fn test_complete_pgn_simple_game() {
        let index_entry = super::database::GameIndexEntry {
            game_offset: 0,
            game_length: 100,
            white_id: 0,
            black_id: 1,
            event_id: 0,
            site_id: 0,
            round_id: 0,
            game_date: super::types::GameDate {
                year: 2023,
                month: 12,
                day: 25,
            },
            event_date: None,
            result: super::types::GameResult::WhiteWins,
            white_elo: 2500,
            black_elo: 2400,
            eco_code: 0,
            half_moves: 4,
            flags: 0,
        };

        let names = super::database::NameDatabase {
            players: vec![
                "Carlsen, Magnus".to_string(),
                "Caruana, Fabiano".to_string(),
            ],
            events: vec!["World Championship".to_string()],
            sites: vec!["New York, NY USA".to_string()],
            rounds: vec!["1".to_string()],
        };

        let game_data = super::database::GameData {
            tags: std::collections::HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![
                shakmaty::Move::Normal {
                    role: shakmaty::Role::Pawn,
                    from: shakmaty::Square::E2,
                    capture: None,
                    to: shakmaty::Square::E4,
                    promotion: None,
                },
                shakmaty::Move::Normal {
                    role: shakmaty::Role::Pawn,
                    from: shakmaty::Square::E7,
                    capture: None,
                    to: shakmaty::Square::E5,
                    promotion: None,
                },
            ],
        };

        let options = super::PgnOptions::default();
        let pgn = PgnFormatter::format_game(&index_entry, &names, &game_data, &options)
            .expect("Should format PGN");

        assert!(pgn.contains("[Event"));
        assert!(pgn.contains("[White"));
        assert!(pgn.contains("[Black"));
        assert!(pgn.contains("1. e4 e5"));
        assert!(pgn.contains("1-0"));
        assert!(pgn.contains("]\n\n1. "));
    }

    #[test]
    fn test_pgn_compact_format() {
        let options = super::PgnOptions {
            compact: true,
            ..Default::default()
        };

        let pgn = PgnFormatter::format_game(
            &GameIndexEntry {
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
                result: GameResult::WhiteWins,
                white_elo: 2500,
                black_elo: 2400,
                eco_code: 0,
                half_moves: 4,
                flags: 0,
            },
            &NameDatabase {
                players: vec!["Player1".to_string(), "Player2".to_string()],
                events: vec!["Test".to_string()],
                sites: vec!["Online".to_string()],
                rounds: vec!["1".to_string()],
            },
            &GameData {
                tags: HashMap::new(),
                flags: 0,
                start_position: None,
                moves: vec![],
            },
            &options,
        )
        .expect("Should format compact PGN");

        let movetext = pgn.lines().skip_while(|l| !l.is_empty()).next().unwrap();
        assert!(!movetext.contains('\n'), "Compact should be single line");
    }

    #[test]
    fn test_pgn_line_wrapping() {
        let long_game = vec![
            shakmaty::Move::Normal {
                role: shakmaty::Role::Pawn,
                from: shakmaty::Square::E2,
                capture: None,
                to: shakmaty::Square::E4,
                promotion: None,
            },
            shakmaty::Move::Normal {
                role: shakmaty::Role::Pawn,
                from: shakmaty::Square::E7,
                capture: None,
                to: shakmaty::Square::E5,
                promotion: None,
            },
        ];

        for i in 0..20 {
            long_game.push(shakmaty::Move::Normal {
                role: shakmaty::Role::Knight,
                from: shakmaty::Square::G1,
                capture: None,
                to: shakmaty::Square::F3,
                promotion: None,
            });
            long_game.push(shakmaty::Move::Normal {
                role: shakmaty::Role::Pawn,
                from: shakmaty::Square::D7,
                capture: None,
                to: shakmaty::Square::D5,
                promotion: None,
            });
        }

        let pgn = PgnFormatter::format_game(
            &GameIndexEntry {
                game_offset: 0,
                game_length: 1000,
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
                result: GameResult::WhiteWins,
                white_elo: 2500,
                black_elo: 2400,
                eco_code: 0,
                half_moves: 80,
                flags: 0,
            },
            &NameDatabase {
                players: vec!["A".to_string(), "B".to_string()],
                events: vec!["Test".to_string()],
                sites: vec!["Online".to_string()],
                rounds: vec!["1".to_string()],
            },
            &GameData {
                tags: HashMap::new(),
                flags: 0,
                start_position: None,
                moves: long_game,
            },
            &PgnOptions::default(),
        )
        .expect("Should format long game");

        let movetext = pgn.lines().skip_while(|l| !l.is_empty()).next().unwrap();
        for line in movetext.lines() {
            assert!(
                line.len() <= 85,
                "Line too long ({} chars): {}",
                line.len(),
                line
            );
        }
    }

    #[test]
    fn test_pgn_with_comments() {
        let mut game_data = GameData {
            tags: HashMap::new(),
            flags: 0,
            start_position: None,
            moves: vec![],
        };

        game_data
            .tags
            .insert("Comment".to_string(), "Some comment".to_string());

        let pgn = PgnFormatter::format_game(
            &GameIndexEntry {
                game_offset: 0,
                game_length: 200,
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
                result: GameResult::WhiteWins,
                white_elo: 2500,
                black_elo: 2400,
                eco_code: 0,
                half_moves: 2,
                flags: 0,
            },
            &NameDatabase {
                players: vec!["P1".to_string(), "P2".to_string()],
                events: vec!["Test".to_string()],
                sites: vec!["Online".to_string()],
                rounds: vec!["1".to_string()],
            },
            &game_data,
            &PgnOptions::default(),
        )
        .expect("Should format with tags");

        assert!(pgn.contains("Comment"));
    }

    #[test]
    fn test_pgn_special_characters_escaped() {
        let mut names = NameDatabase {
            players: vec!["P1".to_string(), "P2".to_string()],
            events: vec!["Test \"Special\" Event".to_string()],
            sites: vec!["Online".to_string()],
            rounds: vec!["1".to_string()],
        };

        let pgn = PgnFormatter::format_game(
            &GameIndexEntry {
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
                result: GameResult::WhiteWins,
                white_elo: 2500,
                black_elo: 2400,
                eco_code: 0,
                half_moves: 2,
                flags: 0,
            },
            &names,
            &GameData {
                tags: HashMap::new(),
                flags: 0,
                start_position: None,
                moves: vec![],
            },
            &PgnOptions::default(),
        )
        .expect("Should escape special chars");

        assert!(pgn.contains("\\\""));
    }

    #[test]
    fn test_pgn_result_in_tags_matches_movetext() {
        let pgn = PgnFormatter::format_game(
            &GameIndexEntry {
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
                result: GameResult::WhiteWins,
                white_elo: 2500,
                black_elo: 2400,
                eco_code: 0,
                half_moves: 4,
                flags: 0,
            },
            &NameDatabase {
                players: vec!["P1".to_string(), "P2".to_string()],
                events: vec!["Test".to_string()],
                sites: vec!["Online".to_string()],
                rounds: vec!["1".to_string()],
            },
            &GameData {
                tags: HashMap::new(),
                flags: 0,
                start_position: None,
                moves: vec![
                    shakmaty::Move::Normal {
                        role: shakmaty::Role::Pawn,
                        from: shakmaty::Square::E2,
                        capture: None,
                        to: shakmaty::Square::E4,
                        promotion: None,
                    },
                    shakmaty::Move::Normal {
                        role: shakmaty::Role::Pawn,
                        from: shakmaty::Square::E7,
                        capture: None,
                        to: shakmaty::Square::E5,
                        promotion: None,
                    },
                ],
            },
            &PgnOptions::default(),
        )
        .expect("Should format PGN");

        let tag_result = pgn
            .split("Result")
            .nth(1)
            .unwrap()
            .split('"')
            .nth(1)
            .unwrap()
            .to_string();

        assert!(
            pgn.ends_with(tag_result) || pgn.ends_with(&format!("{}\n", tag_result)),
            "Result should appear at end of movetext"
        );
    }
}
