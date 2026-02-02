use scidtopgn_core::ScidReader;

#[test]
fn test_five_database_complete() {
    let reader = ScidReader::open("tests/data/five").unwrap();

    println!("Database: {}", reader.metadata().description);
    println!("Total games: {}", reader.metadata().num_games);

    let mut total_moves = 0;
    let mut successful_games = 0;

    for (idx, game_result) in reader.games().enumerate() {
        match game_result {
            Ok(game) => {
                println!("Game {}: {} moves", idx + 1, game.moves.len());
                total_moves += game.moves.len();
                successful_games += 1;

                assert!(game.moves.len() > 0, "Game should have at least one move");
                assert!(game.moves.len() < 500, "Game should have < 500 moves");
            }
            Err(e) => {
                println!("Game {} failed: {:?}", idx + 1, e);
            }
        }
    }

    println!("\n=== SUMMARY ===");
    println!(
        "Successful games: {}/{}",
        successful_games,
        reader.metadata().num_games
    );
    println!("Total moves parsed: {}", total_moves);
    println!(
        "Success rate: {:.1}%",
        (successful_games as f64 / reader.metadata().num_games as f64) * 100.0
    );

    assert!(successful_games as f64 / reader.metadata().num_games as f64 >= 0.9);
}

#[test]
fn test_special_moves() {
    let reader = ScidReader::open("tests/data/five").unwrap();

    let mut castling_count = 0;
    let mut promotion_count = 0;
    let mut en_passant_count = 0;

    for game_result in reader.games() {
        if let Ok(game) = game_result {
            for chess_move in &game.moves {
                if let shakmaty::Move::Castle { .. } = chess_move {
                    castling_count += 1;
                }

                if chess_move.promotion().is_some() {
                    promotion_count += 1;
                }

                if chess_move.is_en_passant() {
                    en_passant_count += 1;
                }
            }
        }
    }

    println!("\n=== Special Moves ===");
    println!("Castling moves: {}", castling_count);
    println!("Promotions: {}", promotion_count);
    println!("En passant: {}", en_passant_count);

    assert!(castling_count > 0, "Should find castling moves");
    assert!(promotion_count > 0, "Should find promotions");
    assert!(en_passant_count > 0, "Should find en passant moves");
}

#[test]
fn test_queen_diagonal_moves() {
    let reader = ScidReader::open("tests/data/five").unwrap();
    let mut queen_diagonal_count = 0;

    for game_result in reader.games() {
        if let Ok(game) = game_result {
            for chess_move in &game.moves {
                if let shakmaty::Move::Normal { role, from, to, .. } = chess_move {
                    if *role == shakmaty::Role::Queen {
                        if let Some(from_sq) = from {
                            let file_diff = (to.file() as i8 - from_sq.file() as i8).abs();
                            let rank_diff = (to.rank() as i8 - from_sq.rank() as i8).abs();

                            if file_diff == rank_diff && file_diff > 0 {
                                queen_diagonal_count += 1;
                                println!("Queen diagonal: {:?} to {:?}", from_sq, to);
                            }
                        }
                    }
                }
            }
        }
    }

    println!("\nTotal Queen diagonal moves: {}", queen_diagonal_count);
    assert!(
        queen_diagonal_count > 0,
        "Should find at least some Queen diagonal moves"
    );
}

#[test]
fn test_game_tags() {
    let mut reader = ScidReader::open("tests/data/five").unwrap();

    for (idx, game_result) in reader.games().enumerate() {
        let game = game_result.unwrap();

        // Verify essential tags exist
        assert!(
            game.tags.contains_key("White"),
            "Game {} missing White tag",
            idx
        );
        assert!(
            game.tags.contains_key("Black"),
            "Game {} missing Black tag",
            idx
        );
        assert!(
            game.tags.contains_key("Result"),
            "Game {} missing Result tag",
            idx
        );

        // Verify moves are valid shakmaty moves
        for move_result in &game.moves {
            assert!(
                move_result.to_string().len() > 0,
                "Game {} has empty move",
                idx
            );
        }
    }
}

#[test]
fn test_move_count_validation() {
    let reader = ScidReader::open("tests/data/five").unwrap();
    let mut total_half_moves = 0;

    for game_result in reader.games() {
        if let Ok(game) = game_result {
            total_half_moves += game.moves.len();

            // SCID stores half-moves in index entry
            // This is just a sanity check
            assert!(game.moves.len() > 0, "Game should have at least one move");
        }
    }

    println!("Total half-moves across all games: {}", total_half_moves);
    assert!(total_half_moves > 0, "Should have total half-moves");
}
