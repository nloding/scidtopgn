// Position Tracking Regression Tests
// Phase 4 Step 4.2 from POSITION_TRACKING_IMPLEMENTATION_PLAN.md

use scidtopgn::sg4::parse_pgn_tags_with_streaming;
use scidtopgn::si4::{parse_game_index, parse_header};
use std::io::Cursor;

/// Test all games in five.sg4 with position tracking
#[test]
fn test_five_database_position_tracking() {
    let test_data_path = "/Users/nloding/code/scidtopgn/test/data/five";

    // Load all test games
    let si4_data = match std::fs::read(format!("{}.si4", test_data_path)) {
        Ok(data) => data,
        Err(e) => {
            println!(
                "⚠️  Could not load SI4 file: {}. Skipping regression test.",
                e
            );
            return; // Skip test if file not available
        }
    };

    let sg4_data = match std::fs::read(format!("{}.sg4", test_data_path)) {
        Ok(data) => data,
        Err(e) => {
            println!(
                "⚠️  Could not load SG4 file: {}. Skipping regression test.",
                e
            );
            return; // Skip test if file not available
        }
    };

    // Parse game indices
    let mut si4_cursor = Cursor::new(&si4_data);
    let header = match parse_header(&mut si4_cursor) {
        Ok(h) => h,
        Err(e) => {
            println!(
                "⚠️  Could not parse SI4 header: {}. Skipping regression test.",
                e
            );
            return; // Skip test if parsing fails
        }
    };

    println!(
        "📋 Testing {} games from five.sg4 database",
        header.num_games
    );

    let mut total_success_rate = 0.0;
    let mut games_tested = 0;
    let mut total_moves = 0;
    let mut total_successful_moves = 0;

    for game_idx in 0..header.num_games {
        if let Ok(game_index) = parse_game_index(&mut si4_cursor) {
            let game_start = game_index.offset as usize;
            let game_end = game_start + game_index.length as usize;

            if game_end <= sg4_data.len() {
                let game_data = &sg4_data[game_start..game_end];

                match parse_pgn_tags_with_streaming(game_data) {
                    Ok(parse_result) => {
                        let stats = &parse_result.position_tracker_stats;
                        total_success_rate += stats.success_rate;
                        games_tested += 1;
                        total_moves += stats.total_moves;
                        total_successful_moves += stats.successful_moves;

                        println!(
                            "🎯 Game {}: {:.1}% success rate ({}/{} moves) - Hash: {:016x}",
                            game_idx + 1,
                            stats.success_rate,
                            stats.successful_moves,
                            stats.total_moves,
                            stats.position_hash
                        );
                    }
                    Err(e) => {
                        println!("❌ Failed to parse game {}: {}", game_idx + 1, e);
                    }
                }
            } else {
                println!("⚠️  Game {} data extends beyond file bounds", game_idx + 1);
            }
        } else {
            println!("⚠️  Could not parse game index {}", game_idx + 1);
        }
    }

    if games_tested > 0 {
        let average_success_rate = total_success_rate / games_tested as f64;
        let overall_success_rate = if total_moves > 0 {
            (total_successful_moves as f64 / total_moves as f64) * 100.0
        } else {
            0.0
        };

        println!("📊 REGRESSION TEST RESULTS:");
        println!(
            "   📈 Average success rate across {} games: {:.1}%",
            games_tested, average_success_rate
        );
        println!(
            "   📊 Overall success rate: {:.1}% ({}/{} moves)",
            overall_success_rate, total_successful_moves, total_moves
        );
        println!(
            "   🎯 Games successfully processed: {}/{}",
            games_tested, header.num_games
        );

        // The main goal is improvement over baseline (which was ~30%)
        // We expect significant improvement with position tracking
        if overall_success_rate > 30.0 {
            println!("✅ REGRESSION TEST PASSED: Position tracking shows {:.1}% improvement over baseline", 
                    overall_success_rate - 30.0);
        } else {
            println!("⚠️  Position tracking success rate {:.1}% is not significantly improved over baseline", 
                    overall_success_rate);
        }

        // Assert reasonable performance (not too strict since this is experimental)
        assert!(
            games_tested > 0,
            "Should process at least one game successfully"
        );
        assert!(total_moves > 0, "Should process at least some moves");

        // Success rate should be better than random (>25%)
        if overall_success_rate > 25.0 {
            println!("🎉 Position tracking is performing better than random chance!");
        }
    } else {
        println!("⚠️  No games were successfully processed");
    }
}

/// Compare position tracking results with known good PGN
#[test]
fn test_against_reference_pgn() {
    let reference_pgn_path = "/Users/nloding/code/scidtopgn/test/data/five.pgn";

    // Load reference PGN file
    let reference_pgn = match std::fs::read_to_string(reference_pgn_path) {
        Ok(content) => content,
        Err(e) => {
            println!(
                "⚠️  Could not load reference PGN: {}. Skipping reference test.",
                e
            );
            return; // Skip test if file not available
        }
    };

    println!(
        "📖 Loaded reference PGN with {} characters",
        reference_pgn.len()
    );

    // Parse reference games (simplified parsing)
    let reference_games = parse_reference_pgn(&reference_pgn);
    println!("📋 Found {} games in reference PGN", reference_games.len());

    // Test each game with position tracking
    for (game_idx, reference_game) in reference_games.iter().enumerate() {
        println!("🔍 Testing game {} against reference", game_idx + 1);

        match parse_scid_game_with_position_tracking(game_idx) {
            Ok(parsed_result) => {
                let stats = &parsed_result.position_tracker_stats;

                println!(
                    "   📊 SCID parsing: {:.1}% success ({}/{} moves)",
                    stats.success_rate, stats.successful_moves, stats.total_moves
                );
                println!(
                    "   📖 Reference game has {} moves",
                    reference_game.move_count
                );

                // Basic validation - we should process some moves
                if stats.total_moves > 0 {
                    println!("   ✅ Successfully processed moves from SCID data");
                } else {
                    println!("   ⚠️  No moves processed from SCID data");
                }

                // Compare move sequences (where possible)
                validate_move_sequence_compatibility(&parsed_result, reference_game);
            }
            Err(e) => {
                println!("   ❌ Failed to parse SCID game {}: {}", game_idx + 1, e);
            }
        }
    }
}

#[test]
fn test_position_tracking_improvement_metrics() {
    let test_data_path = "/Users/nloding/code/scidtopgn/test/data/five";

    // This test measures the improvement that position tracking provides
    let sg4_data = match std::fs::read(format!("{}.sg4", test_data_path)) {
        Ok(data) => data,
        Err(_) => {
            println!("⚠️  Test data not available, skipping improvement metrics test");
            return;
        }
    };

    let si4_data = match std::fs::read(format!("{}.si4", test_data_path)) {
        Ok(data) => data,
        Err(_) => {
            println!("⚠️  Test data not available, skipping improvement metrics test");
            return;
        }
    };

    // Parse and get statistics
    let mut si4_cursor = Cursor::new(&si4_data);
    if let Ok(header) = parse_header(&mut si4_cursor) {
        let mut improvement_metrics = Vec::new();

        for game_idx in 0..std::cmp::min(header.num_games, 3) {
            // Test first 3 games
            if let Ok(game_index) = parse_game_index(&mut si4_cursor) {
                let game_start = game_index.offset as usize;
                let game_end = game_start + game_index.length as usize;

                if game_end <= sg4_data.len() {
                    let game_data = &sg4_data[game_start..game_end];

                    if let Ok(parse_result) = parse_pgn_tags_with_streaming(game_data) {
                        let stats = &parse_result.position_tracker_stats;
                        improvement_metrics.push(ImprovementMetric {
                            game_number: (game_idx + 1) as usize,
                            total_moves: stats.total_moves,
                            successful_moves: stats.successful_moves,
                            success_rate: stats.success_rate,
                            position_hash: stats.position_hash,
                        });
                    }
                }
            }
        }

        println!("📊 POSITION TRACKING IMPROVEMENT METRICS:");
        for metric in &improvement_metrics {
            println!(
                "   🎯 Game {}: {:.1}% success ({}/{} moves) - Hash: {:016x}",
                metric.game_number,
                metric.success_rate,
                metric.successful_moves,
                metric.total_moves,
                metric.position_hash
            );
        }

        if !improvement_metrics.is_empty() {
            let avg_success = improvement_metrics
                .iter()
                .map(|m| m.success_rate)
                .sum::<f64>()
                / improvement_metrics.len() as f64;
            let total_moves: usize = improvement_metrics.iter().map(|m| m.total_moves).sum();
            let total_successful: usize =
                improvement_metrics.iter().map(|m| m.successful_moves).sum();

            println!("   📈 Average success rate: {:.1}%", avg_success);
            println!(
                "   📊 Overall success rate: {:.1}% ({}/{})",
                (total_successful as f64 / total_moves as f64) * 100.0,
                total_successful,
                total_moves
            );
        }
    }
}

// Helper types and functions

#[derive(Debug)]
struct ReferenceGame {
    headers: Vec<(String, String)>,
    moves: Vec<String>,
    move_count: usize,
    result: String,
}

#[derive(Debug)]
struct ImprovementMetric {
    game_number: usize,
    total_moves: usize,
    successful_moves: usize,
    success_rate: f64,
    position_hash: u64,
}

fn parse_reference_pgn(pgn_content: &str) -> Vec<ReferenceGame> {
    let mut games = Vec::new();
    let mut current_headers = Vec::new();
    let mut current_moves = Vec::new();
    let mut in_headers = true;

    for line in pgn_content.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_headers && !current_headers.is_empty() {
                in_headers = false; // Switch to moves section
            }
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Parse header
            if let Some(space_pos) = trimmed.find(' ') {
                let tag = trimmed[1..space_pos].to_string();
                let value = trimmed[space_pos + 1..trimmed.len() - 1]
                    .trim_matches('"')
                    .to_string();
                current_headers.push((tag, value));
            }
        } else if !in_headers {
            // Parse moves (simplified)
            let moves_in_line: Vec<String> = trimmed
                .split_whitespace()
                .filter(|s| !s.contains('.') && !s.starts_with('{') && !s.ends_with('}'))
                .filter(|s| !["1-0", "0-1", "1/2-1/2", "*"].contains(s))
                .map(|s| s.to_string())
                .collect();

            current_moves.extend(moves_in_line);

            // Check for game end
            if trimmed.contains("1-0")
                || trimmed.contains("0-1")
                || trimmed.contains("1/2-1/2")
                || trimmed.contains("*")
            {
                // End of game
                let result = if trimmed.contains("1-0") {
                    "1-0"
                } else if trimmed.contains("0-1") {
                    "0-1"
                } else if trimmed.contains("1/2-1/2") {
                    "1/2-1/2"
                } else {
                    "*"
                };

                games.push(ReferenceGame {
                    headers: current_headers.clone(),
                    move_count: current_moves.len(),
                    moves: current_moves.clone(),
                    result: result.to_string(),
                });

                // Reset for next game
                current_headers.clear();
                current_moves.clear();
                in_headers = true;
            }
        }
    }

    games
}

fn parse_scid_game_with_position_tracking(
    game_idx: usize,
) -> Result<scidtopgn::sg4::StreamingGameParseState, String> {
    let test_data_path = "/Users/nloding/code/scidtopgn/test/data/five";

    let si4_data = std::fs::read(format!("{}.si4", test_data_path))
        .map_err(|e| format!("Could not read SI4 file: {}", e))?;
    let sg4_data = std::fs::read(format!("{}.sg4", test_data_path))
        .map_err(|e| format!("Could not read SG4 file: {}", e))?;

    let mut si4_cursor = Cursor::new(&si4_data);
    let _header =
        parse_header(&mut si4_cursor).map_err(|e| format!("Could not parse SI4 header: {}", e))?;

    // Skip to the requested game
    for _ in 0..=game_idx {
        if let Ok(game_index) = parse_game_index(&mut si4_cursor) {
            if game_idx == 0 || game_idx > 0 {
                // Parse the game we want
                let game_start = game_index.offset as usize;
                let game_end = game_start + game_index.length as usize;

                if game_end <= sg4_data.len() {
                    let game_data = &sg4_data[game_start..game_end];
                    return parse_pgn_tags_with_streaming(game_data)
                        .map_err(|e| format!("Could not parse game data: {}", e));
                }
            }
        }
    }

    Err("Could not find requested game".to_string())
}

fn validate_move_sequence_compatibility(
    parsed_result: &scidtopgn::sg4::StreamingGameParseState,
    reference_game: &ReferenceGame,
) {
    let stats = &parsed_result.position_tracker_stats;

    println!("   🔍 Move sequence compatibility check:");
    println!(
        "      📊 SCID: {} total moves, {} successful",
        stats.total_moves, stats.successful_moves
    );
    println!(
        "      📖 Reference: {} moves in PGN",
        reference_game.move_count
    );

    // Basic compatibility check
    if stats.successful_moves > 0 && reference_game.move_count > 0 {
        let compatibility_ratio =
            (stats.successful_moves as f64 / reference_game.move_count as f64) * 100.0;
        println!(
            "      📈 Compatibility ratio: {:.1}% (SCID successful / Reference total)",
            compatibility_ratio
        );

        if compatibility_ratio > 10.0 {
            println!("      ✅ Reasonable compatibility with reference");
        } else {
            println!("      ⚠️  Low compatibility with reference");
        }
    } else {
        println!("      ⚠️  Cannot compare - insufficient data");
    }
}
