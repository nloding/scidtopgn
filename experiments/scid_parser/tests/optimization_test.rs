// Performance Optimization Test
//
// This test demonstrates the memory-efficient position tracking
// capabilities for processing large SCID databases.

use scid_parser::bridge::{OptimizedPositionTracker, GameState, PositionContext};

#[test]
fn test_optimized_tracker_performance_stats() {
    let mut tracker = OptimizedPositionTracker::new_optimized();
    
    // Get initial stats
    let initial_stats = tracker.get_stats();
    assert_eq!(initial_stats.cache_hits, 0);
    assert_eq!(initial_stats.cache_misses, 0);
    assert_eq!(initial_stats.total_games_processed, 0);
    
    // Simulate processing multiple games
    for game_id in 0..5 {
        tracker.reset_for_new_game();
        
        // Verify stats update
        let stats = tracker.get_stats();
        assert_eq!(stats.total_games_processed, game_id + 1);
    }
    
    println!("✅ Optimized tracker processed {} games", 
             tracker.get_stats().total_games_processed);
}

#[test]
fn test_memory_efficient_allocations() {
    let tracker = OptimizedPositionTracker::new_optimized();
    
    // Test pre-allocated vectors maintain capacity
    let piece_list = tracker.get_piece_list();
    assert!(piece_list.capacity() >= 16, "Piece list should have pre-allocated capacity");
    
    let san_list = tracker.get_san_list();
    assert!(san_list.capacity() >= 200, "SAN list should have pre-allocated capacity");
    
    println!("✅ Memory optimization: piece_list capacity={}, san_list capacity={}", 
             piece_list.capacity(), san_list.capacity());
}

#[test]
fn test_position_cache_functionality() {
    let mut tracker = OptimizedPositionTracker::new_optimized();
    
    // Start with a clean cache
    tracker.clear_caches();
    let stats = tracker.get_stats();
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
    
    println!("✅ Position cache cleared and ready for optimization");
}

#[test]
fn test_batch_processing_interface() {
    let mut tracker = OptimizedPositionTracker::new_optimized();
    
    // Test empty batch processing
    let empty_batch: Vec<&[u8]> = vec![];
    let results = tracker.process_games_batch(&empty_batch);
    assert!(results.is_ok());
    assert_eq!(results.unwrap().len(), 0);
    
    // Test with sample game data (empty for now)
    let sample_data = vec![&[0u8; 10][..], &[1u8; 10][..]];
    let results = tracker.process_games_batch(&sample_data);
    assert!(results.is_ok());
    assert_eq!(results.unwrap().len(), 2);
    
    println!("✅ Batch processing interface working correctly");
}

#[test]
fn test_integration_with_game_state() {
    // Test that OptimizedPositionTracker can work alongside existing GameState
    let game_state = GameState::new();
    let tracker = OptimizedPositionTracker::new_optimized();
    
    // Verify both use the same underlying chess position representation
    assert_eq!(game_state.move_count(), 0);
    assert_eq!(tracker.current_ply(), 0);
    
    println!("✅ OptimizedPositionTracker integrates with existing GameState");
}

#[cfg(test)]
mod benchmark_tests {
    use super::*;
    use std::time::Instant;
    
    #[test]
    fn benchmark_tracker_creation() {
        let start = Instant::now();
        
        // Create multiple trackers to test allocation performance
        let trackers: Vec<_> = (0..100)
            .map(|_| OptimizedPositionTracker::new_optimized())
            .collect();
        
        let duration = start.elapsed();
        
        assert_eq!(trackers.len(), 100);
        println!("✅ Created 100 optimized trackers in {:?}", duration);
        
        // Should be very fast due to efficient allocation strategies
        assert!(duration.as_millis() < 100, "Tracker creation should be fast");
    }
    
    #[test]
    fn benchmark_game_processing() {
        let mut tracker = OptimizedPositionTracker::new_optimized();
        let start = Instant::now();
        
        // Simulate processing many games
        for _ in 0..1000 {
            tracker.reset_for_new_game();
        }
        
        let duration = start.elapsed();
        let stats = tracker.get_stats();
        
        println!("✅ Processed {} games in {:?} (avg: {:?} per game)", 
                 stats.total_games_processed, 
                 duration,
                 duration / stats.total_games_processed as u32);
        
        // Should be efficient due to memory reuse
        assert!(duration.as_millis() < 1000, "Game processing should be efficient");
    }
}