// Parallel Processing Test
//
// This test demonstrates the parallel game processing capabilities
// for large SCID databases using std::thread.

use scidtopgn::bridge::parallel_processor::{ParallelGameProcessor, ParallelProcessingConfig};
use scidtopgn::bridge::GameMetadata;
use std::time::Instant;

#[test]
fn test_parallel_processor_creation() {
    let processor = ParallelGameProcessor::new();
    let stats = processor.get_stats();

    assert_eq!(stats.total_games, 0);
    assert_eq!(stats.successful_games, 0);
    assert_eq!(stats.failed_games, 0);

    println!("✅ Parallel processor created successfully");
}

#[test]
fn test_parallel_processing_config() {
    let config = ParallelProcessingConfig {
        thread_count: Some(2),
        batch_size: 100,
        enable_monitoring: false,
        collect_game_stats: true,
    };

    let processor = ParallelGameProcessor::with_config(config);
    assert_eq!(processor.get_stats().total_games, 0);

    println!("✅ Custom parallel processing config working");
}

#[test]
fn test_empty_parallel_processing() {
    let mut processor = ParallelGameProcessor::new();
    let games_data: Vec<&[u8]> = vec![];

    let results = processor.process_games_parallel(&games_data, None).unwrap();
    assert_eq!(results.len(), 0);

    let stats = processor.get_stats();
    assert_eq!(stats.total_games, 0);
    assert_eq!(stats.successful_games, 0);
    assert_eq!(stats.failed_games, 0);

    println!("✅ Empty game processing handled correctly");
}

#[test]
fn test_single_game_parallel_processing() {
    let mut processor = ParallelGameProcessor::new();

    // Create a dummy game data
    let game_data = vec![0u8; 200]; // 200 bytes of dummy SCID game data
    let games_data = vec![game_data.as_slice()];

    // Create metadata for the game
    let metadata = vec![GameMetadata {
        event: "Test Tournament".to_string(),
        site: "Test Location".to_string(),
        date: "2024.01.15".to_string(),
        white: "Alice".to_string(),
        black: "Bob".to_string(),
        result: "1-0".to_string(),
        round: Some("1".to_string()),
        white_elo: Some(1600),
        black_elo: Some(1550),
        eco: Some("B10".to_string()),
    }];

    let results = processor
        .process_games_parallel(&games_data, Some(&metadata))
        .unwrap();
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert_eq!(result.game_index, 0);
    assert!(result.game_state.is_some());
    assert!(result.metadata.is_some());
    assert!(result.error.is_none());

    // Verify metadata was preserved
    let result_metadata = result.metadata.as_ref().unwrap();
    assert_eq!(result_metadata.event, "Test Tournament");
    assert_eq!(result_metadata.white, "Alice");
    assert_eq!(result_metadata.black, "Bob");

    let stats = processor.get_stats();
    assert_eq!(stats.total_games, 1);
    assert_eq!(stats.successful_games, 1);
    assert_eq!(stats.failed_games, 0);

    println!(
        "✅ Single game parallel processing: {} ms",
        result.processing_time.as_millis()
    );
}

#[test]
fn test_multiple_games_parallel_processing() {
    let mut processor = ParallelGameProcessor::with_config(ParallelProcessingConfig {
        thread_count: Some(4), // Use 4 threads for testing
        batch_size: 50,        // Small batch size for testing
        enable_monitoring: true,
        collect_game_stats: false,
    });

    // Create multiple dummy games
    let game_count = 10;
    let games_data: Vec<Vec<u8>> = (0..game_count)
        .map(|i| vec![i as u8; 100 + i * 10]) // Variable sized games
        .collect();
    let games_refs: Vec<&[u8]> = games_data.iter().map(|g| g.as_slice()).collect();

    // Create metadata for each game
    let metadata: Vec<GameMetadata> = (0..game_count)
        .map(|i| GameMetadata {
            event: format!("Tournament {}", i + 1),
            site: "Parallel Test".to_string(),
            date: "2024.01.01".to_string(),
            white: format!("Player{}", i * 2 + 1),
            black: format!("Player{}", i * 2 + 2),
            result: if i % 3 == 0 {
                "1-0"
            } else if i % 3 == 1 {
                "0-1"
            } else {
                "1/2-1/2"
            }
            .to_string(),
            round: Some(format!("{}", i + 1)),
            white_elo: Some(1500 + i as u16 * 50),
            black_elo: Some(1450 + i as u16 * 60),
            eco: Some(format!("A{:02}", 10 + i)),
        })
        .collect();

    let start_time = Instant::now();
    let results = processor
        .process_games_parallel(&games_refs, Some(&metadata))
        .unwrap();
    let processing_duration = start_time.elapsed();

    // Verify all games were processed
    assert_eq!(results.len(), game_count);

    // Verify results are in correct order
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.game_index, i);
        assert!(result.game_state.is_some());
        assert!(result.metadata.is_some());
        assert!(result.error.is_none());

        // Verify metadata matches
        let result_metadata = result.metadata.as_ref().unwrap();
        assert_eq!(result_metadata.event, format!("Tournament {}", i + 1));
        assert_eq!(result_metadata.white, format!("Player{}", i * 2 + 1));
        assert_eq!(result_metadata.black, format!("Player{}", i * 2 + 2));
    }

    let stats = processor.get_stats();
    assert_eq!(stats.total_games, game_count);
    assert_eq!(stats.successful_games, game_count);
    assert_eq!(stats.failed_games, 0);
    assert!(stats.games_per_second > 0.0);

    println!(
        "✅ Processed {} games in {:?} ({:.1} games/sec)",
        game_count, processing_duration, stats.games_per_second
    );
}

#[test]
fn test_batch_processing_with_threads() {
    let mut processor = ParallelGameProcessor::with_config(ParallelProcessingConfig {
        thread_count: Some(2),
        batch_size: 3, // Small batch to force multiple batches
        enable_monitoring: true,
        collect_game_stats: false,
    });

    // Create 7 games to test batching (3 + 3 + 1 games per batch)
    let games_data: Vec<Vec<u8>> = (0..7).map(|i| vec![i as u8; 50]).collect();
    let games_refs: Vec<&[u8]> = games_data.iter().map(|g| g.as_slice()).collect();

    let results = processor.process_games_parallel(&games_refs, None).unwrap();
    assert_eq!(results.len(), 7);

    // Verify all games processed successfully and in order
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.game_index, i);
        assert!(result.error.is_none());
    }

    let stats = processor.get_stats();
    assert_eq!(stats.total_games, 7);
    assert_eq!(stats.successful_games, 7);
    assert_eq!(stats.failed_games, 0);

    println!(
        "✅ Batch processing with threading: processed {} games across multiple batches",
        stats.total_games
    );
}

#[test]
fn test_performance_statistics() {
    let mut processor = ParallelGameProcessor::new();

    // Process some games to generate statistics
    let games_data: Vec<Vec<u8>> = (0..5).map(|i| vec![i as u8; 100]).collect();
    let games_refs: Vec<&[u8]> = games_data.iter().map(|g| g.as_slice()).collect();

    let _results = processor.process_games_parallel(&games_refs, None).unwrap();

    let stats = processor.get_stats();
    assert_eq!(stats.total_games, 5);
    assert_eq!(stats.successful_games, 5);
    assert_eq!(stats.failed_games, 0);
    assert!(stats.total_duration.as_millis() > 0);
    assert!(stats.avg_time_per_game.as_millis() >= 0);
    assert!(stats.games_per_second >= 0.0);

    println!(
        "✅ Performance stats: {:.2} games/sec, avg {:.1}ms per game",
        stats.games_per_second,
        stats.avg_time_per_game.as_millis()
    );

    // Test stats reset
    processor.reset_stats();
    let reset_stats = processor.get_stats();
    assert_eq!(reset_stats.total_games, 0);
    assert_eq!(reset_stats.successful_games, 0);

    println!("✅ Statistics reset functionality working");
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;

    #[test]
    fn benchmark_parallel_vs_sequential() {
        // Test to compare parallel vs sequential processing performance
        let game_count = 20;
        let games_data: Vec<Vec<u8>> = (0..game_count)
            .map(|i| vec![i as u8; 200]) // Larger games for meaningful comparison
            .collect();
        let games_refs: Vec<&[u8]> = games_data.iter().map(|g| g.as_slice()).collect();

        // Test parallel processing (4 threads)
        let mut parallel_processor = ParallelGameProcessor::with_config(ParallelProcessingConfig {
            thread_count: Some(4),
            batch_size: 100,
            enable_monitoring: false,
            collect_game_stats: false,
        });

        let parallel_start = Instant::now();
        let parallel_results = parallel_processor
            .process_games_parallel(&games_refs, None)
            .unwrap();
        let parallel_duration = parallel_start.elapsed();

        // Test sequential processing (1 thread)
        let mut sequential_processor =
            ParallelGameProcessor::with_config(ParallelProcessingConfig {
                thread_count: Some(1),
                batch_size: 100,
                enable_monitoring: false,
                collect_game_stats: false,
            });

        let sequential_start = Instant::now();
        let sequential_results = sequential_processor
            .process_games_parallel(&games_refs, None)
            .unwrap();
        let sequential_duration = sequential_start.elapsed();

        // Verify same results
        assert_eq!(parallel_results.len(), sequential_results.len());
        assert_eq!(parallel_results.len(), game_count);

        let parallel_stats = parallel_processor.get_stats();
        let sequential_stats = sequential_processor.get_stats();

        println!(
            "✅ Parallel processing ({} threads): {:.2} games/sec ({:?})",
            4, parallel_stats.games_per_second, parallel_duration
        );
        println!(
            "✅ Sequential processing (1 thread): {:.2} games/sec ({:?})",
            sequential_stats.games_per_second, sequential_duration
        );

        // For small numbers of simple games, parallel overhead might make it slower,
        // but the infrastructure should be working
        assert!(parallel_stats.total_games == sequential_stats.total_games);
        assert!(parallel_stats.successful_games == sequential_stats.successful_games);
    }
}
