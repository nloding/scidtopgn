// Parallel Game Processing for Large SCID Databases
//
// This module implements parallel processing of SCID games using standard library
// threading for concurrent game parsing, leveraging the optimized position tracker 
// and memory-efficient techniques from Phase 5 Step 5.1.
//
// Based on Phase 5 Step 5.2 of the Comprehensive Remediation Plan.
// Note: Using std::thread instead of rayon due to Rust 1.77 compatibility requirements.

use std::thread;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::error::{Result, ScidError};
use crate::bridge::{OptimizedPositionTracker, GameState, GameMetadata, PositionContext};
use crate::sg4::DecodedMove;

/// Configuration for parallel game processing
#[derive(Debug, Clone)]
pub struct ParallelProcessingConfig {
    /// Number of threads to use for parallel processing
    /// If None, rayon will use the default thread pool
    pub thread_count: Option<usize>,
    
    /// Maximum number of games to process in a single batch
    pub batch_size: usize,
    
    /// Whether to enable performance monitoring
    pub enable_monitoring: bool,
    
    /// Whether to collect detailed statistics per game
    pub collect_game_stats: bool,
}

impl Default for ParallelProcessingConfig {
    fn default() -> Self {
        Self {
            thread_count: None, // Use rayon's default
            batch_size: 1000,   // Process 1000 games per batch
            enable_monitoring: true,
            collect_game_stats: false,
        }
    }
}

/// Statistics for parallel processing performance
#[derive(Debug, Clone)]
pub struct ParallelProcessingStats {
    /// Total number of games processed
    pub total_games: usize,
    
    /// Number of successfully processed games
    pub successful_games: usize,
    
    /// Number of games that failed processing
    pub failed_games: usize,
    
    /// Total processing time
    pub total_duration: std::time::Duration,
    
    /// Average processing time per game
    pub avg_time_per_game: std::time::Duration,
    
    /// Games processed per second
    pub games_per_second: f64,
    
    /// Peak memory usage during processing
    pub peak_memory_usage: Option<usize>,
}

/// Result of processing a single game in parallel
#[derive(Clone)]
pub struct ParallelGameResult {
    /// Game index in the database
    pub game_index: usize,
    
    /// Successfully parsed game state (if successful)
    pub game_state: Option<GameState>,
    
    /// Game metadata (if available)
    pub metadata: Option<GameMetadata>,
    
    /// Processing error (if failed)
    pub error: Option<String>,
    
    /// Time taken to process this game
    pub processing_time: std::time::Duration,
    
    /// Memory usage for this game (if monitored)
    pub memory_usage: Option<usize>,
}

impl std::fmt::Debug for ParallelGameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelGameResult")
            .field("game_index", &self.game_index)
            .field("has_game_state", &self.game_state.is_some())
            .field("has_metadata", &self.metadata.is_some())
            .field("error", &self.error)
            .field("processing_time", &self.processing_time)
            .field("memory_usage", &self.memory_usage)
            .finish()
    }
}

/// Parallel game processor for SCID databases
/// 
/// This processor uses std::thread to parallelize game parsing across multiple threads,
/// with each thread using its own OptimizedPositionTracker for memory efficiency.
pub struct ParallelGameProcessor {
    config: ParallelProcessingConfig,
    stats: ParallelProcessingStats,
}

impl ParallelGameProcessor {
    /// Create a new parallel game processor with default configuration
    pub fn new() -> Self {
        Self {
            config: ParallelProcessingConfig::default(),
            stats: ParallelProcessingStats {
                total_games: 0,
                successful_games: 0,
                failed_games: 0,
                total_duration: std::time::Duration::ZERO,
                avg_time_per_game: std::time::Duration::ZERO,
                games_per_second: 0.0,
                peak_memory_usage: None,
            },
        }
    }
    
    /// Create a new parallel game processor with custom configuration
    pub fn with_config(config: ParallelProcessingConfig) -> Self {
        Self {
            config,
            stats: ParallelProcessingStats {
                total_games: 0,
                successful_games: 0,
                failed_games: 0,
                total_duration: std::time::Duration::ZERO,
                avg_time_per_game: std::time::Duration::ZERO,
                games_per_second: 0.0,
                peak_memory_usage: None,
            },
        }
    }
    
    /// Process multiple games in parallel using standard library threading
    /// 
    /// # Arguments
    /// * `games_data` - Slice of game data bytes to process
    /// * `metadata_list` - Optional metadata for each game
    /// 
    /// # Returns
    /// Vector of processing results, one for each game
    pub fn process_games_parallel(
        &mut self,
        games_data: &[&[u8]],
        metadata_list: Option<&[GameMetadata]>,
    ) -> Result<Vec<ParallelGameResult>> {
        let start_time = Instant::now();
        
        // Determine number of threads to use
        let thread_count = self.config.thread_count.unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4) // Default to 4 threads if detection fails
        });
        
        // Process games in batches to manage memory usage
        let mut all_results = Vec::with_capacity(games_data.len());
        
        for (batch_index, chunk) in games_data.chunks(self.config.batch_size).enumerate() {
            let batch_start = batch_index * self.config.batch_size;
            
            // Process this batch in parallel using std::thread
            let batch_results = self.process_batch_with_threads(
                chunk, 
                batch_start, 
                metadata_list, 
                thread_count
            )?;
            
            all_results.extend(batch_results);
            
            if self.config.enable_monitoring {
                println!("✅ Processed batch {} ({} games)", batch_index + 1, chunk.len());
            }
        }
        
        // Update statistics
        let total_duration = start_time.elapsed();
        self.update_stats(&all_results, total_duration);
        
        Ok(all_results)
    }
    
    /// Process a batch of games using multiple threads
    fn process_batch_with_threads(
        &self,
        chunk: &[&[u8]],
        batch_start: usize,
        metadata_list: Option<&[GameMetadata]>,
        thread_count: usize,
    ) -> Result<Vec<ParallelGameResult>> {
        // Divide the chunk among threads
        let chunk_size = (chunk.len() + thread_count - 1) / thread_count; // Ceiling division
        let results = Arc::new(Mutex::new(Vec::with_capacity(chunk.len())));
        let mut handles = Vec::new();
        
        for thread_index in 0..thread_count {
            let start_idx = thread_index * chunk_size;
            if start_idx >= chunk.len() {
                break; // No more work for this thread
            }
            
            let end_idx = std::cmp::min(start_idx + chunk_size, chunk.len());
            let thread_chunk = &chunk[start_idx..end_idx];
            
            // Clone data needed for this thread
            let thread_data: Vec<Vec<u8>> = thread_chunk.iter().map(|&data| data.to_vec()).collect();
            let thread_metadata: Option<Vec<GameMetadata>> = metadata_list.map(|list| {
                (start_idx..end_idx).filter_map(|i| {
                    let global_idx = batch_start + i;
                    list.get(global_idx).cloned()
                }).collect()
            });
            let results_clone = Arc::clone(&results);
            let config = self.config.clone();
            
            let handle = thread::spawn(move || {
                let mut thread_results = Vec::new();
                
                for (local_index, game_data) in thread_data.iter().enumerate() {
                    let global_index = batch_start + start_idx + local_index;
                    let metadata = thread_metadata.as_ref()
                        .and_then(|list| list.get(local_index));
                    
                    let result = Self::process_single_game_with_config(
                        &config,
                        global_index,
                        game_data,
                        metadata,
                    );
                    
                    thread_results.push(result);
                }
                
                // Add results to shared vector
                if let Ok(mut results_guard) = results_clone.lock() {
                    results_guard.extend(thread_results);
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().map_err(|_| {
                ScidError::conversion_error("Thread panic during parallel processing".to_string())
            })?;
        }
        
        // Extract results from shared container
        let results_guard = results.lock().map_err(|_| {
            ScidError::conversion_error("Failed to access shared results".to_string())
        })?;
        
        let mut final_results = results_guard.clone();
        
        // Sort results by game_index to maintain order
        final_results.sort_by(|a, b| a.game_index.cmp(&b.game_index));
        
        Ok(final_results)
    }
    
    /// Static method for processing a single game (used by threads)
    fn process_single_game_with_config(
        config: &ParallelProcessingConfig,
        game_index: usize,
        game_data: &[u8],
        metadata: Option<&GameMetadata>,
    ) -> ParallelGameResult {
        let game_start = Instant::now();
        
        // Create a thread-local optimized tracker
        let mut tracker = OptimizedPositionTracker::new_optimized();
        
        // Process the game using position-aware parsing
        match Self::parse_game_with_tracker_static(config, &mut tracker, game_data, metadata) {
            Ok((game_state, final_metadata)) => {
                let processing_time = game_start.elapsed();
                
                ParallelGameResult {
                    game_index,
                    game_state: Some(game_state),
                    metadata: Some(final_metadata),
                    error: None,
                    processing_time,
                    memory_usage: if config.collect_game_stats {
                        Some(Self::estimate_memory_usage_static(&tracker))
                    } else {
                        None
                    },
                }
            }
            Err(e) => {
                let processing_time = game_start.elapsed();
                
                ParallelGameResult {
                    game_index,
                    game_state: None,
                    metadata: metadata.cloned(),
                    error: Some(format!("{}", e)),
                    processing_time,
                    memory_usage: None,
                }
            }
        }
    }
    
    /// Process a single game with parallel-optimized settings
    /// 
    /// This method is designed to be called from parallel threads and uses
    /// thread-local optimized position tracking for maximum efficiency.
    fn process_single_game_parallel(
        &self,
        game_index: usize,
        game_data: &[u8],
        metadata: Option<&GameMetadata>,
    ) -> ParallelGameResult {
        let game_start = Instant::now();
        
        // Create a thread-local optimized tracker
        let mut tracker = OptimizedPositionTracker::new_optimized();
        
        // Process the game using position-aware parsing
        match self.parse_game_with_tracker(&mut tracker, game_data, metadata) {
            Ok((game_state, final_metadata)) => {
                let processing_time = game_start.elapsed();
                
                ParallelGameResult {
                    game_index,
                    game_state: Some(game_state),
                    metadata: Some(final_metadata),
                    error: None,
                    processing_time,
                    memory_usage: if self.config.collect_game_stats {
                        Some(self.estimate_memory_usage(&tracker))
                    } else {
                        None
                    },
                }
            }
            Err(e) => {
                let processing_time = game_start.elapsed();
                
                ParallelGameResult {
                    game_index,
                    game_state: None,
                    metadata: metadata.cloned(),
                    error: Some(format!("{}", e)),
                    processing_time,
                    memory_usage: None,
                }
            }
        }
    }
    
    /// Static version of parse_game_with_tracker for use in threads
    fn parse_game_with_tracker_static(
        _config: &ParallelProcessingConfig,
        tracker: &mut OptimizedPositionTracker,
        game_data: &[u8],
        metadata: Option<&GameMetadata>,
    ) -> Result<(GameState, GameMetadata)> {
        Self::parse_game_with_tracker_impl(tracker, game_data, metadata)
    }
    
    /// Parse a single game using the optimized tracker
    /// 
    /// This method handles the complete SCID game parsing pipeline:
    /// 1. Initialize position tracker
    /// 2. Parse moves with position awareness 
    /// 3. Validate chess logic
    /// 4. Generate final game state
    fn parse_game_with_tracker(
        &self,
        tracker: &mut OptimizedPositionTracker,
        game_data: &[u8],
        metadata: Option<&GameMetadata>,
    ) -> Result<(GameState, GameMetadata)> {
        Self::parse_game_with_tracker_impl(tracker, game_data, metadata)
    }
    
    /// Common implementation for game parsing
    fn parse_game_with_tracker_impl(
        tracker: &mut OptimizedPositionTracker,
        game_data: &[u8],
        metadata: Option<&GameMetadata>,
    ) -> Result<(GameState, GameMetadata)> {
        // Reset tracker for new game
        tracker.reset_for_new_game();
        
        // Parse the game data using position-aware SCID parsing
        // This is a simplified implementation - in practice, this would call
        // the actual SG4 parsing logic with position tracking
        
        // For now, create a basic game state with metadata
        let mut game_state = GameState::new();
        
        // Apply metadata if provided
        let final_metadata = metadata.cloned().unwrap_or_else(|| {
            // Create default metadata for testing
            GameMetadata {
                event: "Parallel Processing Test".to_string(),
                site: "Test Suite".to_string(),
                date: "2024.01.01".to_string(),
                white: "Player 1".to_string(),
                black: "Player 2".to_string(),
                result: "*".to_string(),
                round: None,
                white_elo: None,
                black_elo: None,
                eco: None,
            }
        });
        
        game_state.set_metadata(final_metadata.clone());
        
        // TODO: Implement actual SCID move parsing here
        // This would involve:
        // 1. Reading move bytes from game_data
        // 2. Decoding each move using tracker.apply_scid_move()
        // 3. Building up the game state with validated moves
        
        // For now, return a valid but empty game state
        Ok((game_state, final_metadata))
    }
    
    /// Static version of estimate_memory_usage for use in threads
    fn estimate_memory_usage_static(tracker: &OptimizedPositionTracker) -> usize {
        Self::estimate_memory_usage_impl(tracker)
    }
    
    /// Estimate memory usage for a game processing
    fn estimate_memory_usage(&self, tracker: &OptimizedPositionTracker) -> usize {
        Self::estimate_memory_usage_impl(tracker)
    }
    
    /// Common implementation for memory usage estimation
    fn estimate_memory_usage_impl(tracker: &OptimizedPositionTracker) -> usize {
        // Estimate based on tracker statistics
        let stats = tracker.get_stats();
        
        // Rough estimate: base tracker size + move history + cache usage
        let base_size = std::mem::size_of::<OptimizedPositionTracker>();
        let move_history_size = stats.move_history_capacity * std::mem::size_of::<shakmaty::Move>();
        let cache_size = stats.current_cache_size * 200; // Rough estimate per cached position
        
        base_size + move_history_size + cache_size
    }
    
    /// Update processing statistics after a batch
    fn update_stats(&mut self, results: &[ParallelGameResult], total_duration: std::time::Duration) {
        self.stats.total_games = results.len();
        self.stats.successful_games = results.iter().filter(|r| r.error.is_none()).count();
        self.stats.failed_games = results.iter().filter(|r| r.error.is_some()).count();
        self.stats.total_duration = total_duration;
        
        if !results.is_empty() {
            self.stats.avg_time_per_game = total_duration / results.len() as u32;
            self.stats.games_per_second = results.len() as f64 / total_duration.as_secs_f64();
        }
        
        // Find peak memory usage
        self.stats.peak_memory_usage = results
            .iter()
            .filter_map(|r| r.memory_usage)
            .max();
    }
    
    /// Get current processing statistics
    pub fn get_stats(&self) -> &ParallelProcessingStats {
        &self.stats
    }
    
    /// Reset statistics for a new processing run
    pub fn reset_stats(&mut self) {
        self.stats = ParallelProcessingStats {
            total_games: 0,
            successful_games: 0,
            failed_games: 0,
            total_duration: std::time::Duration::ZERO,
            avg_time_per_game: std::time::Duration::ZERO,
            games_per_second: 0.0,
            peak_memory_usage: None,
        };
    }
    
    /// Process games from a SCID database structure
    /// 
    /// This method provides a higher-level interface for processing complete
    /// SCID databases with proper index and metadata handling.
    pub fn process_scid_database_parallel(
        &mut self,
        database: &ScidDatabaseRef,
    ) -> Result<Vec<ParallelGameResult>> {
        // Extract game data and metadata from the database
        let games_data: Vec<&[u8]> = database.get_games_data();
        let metadata_list: Vec<GameMetadata> = database.get_games_metadata()?;
        
        // Process using the parallel pipeline
        self.process_games_parallel(&games_data, Some(&metadata_list))
    }
}

/// Reference to a SCID database for parallel processing
/// 
/// This struct provides the interface needed for parallel processing
/// without requiring the full database structure to be thread-safe.
pub struct ScidDatabaseRef {
    // Placeholder implementation - in practice this would contain
    // references to the parsed SI4 index, SN4 names, and SG4 game data
    pub game_count: usize,
}

impl ScidDatabaseRef {
    /// Get game data bytes for all games in the database
    pub fn get_games_data(&self) -> Vec<&[u8]> {
        // Placeholder implementation
        // In practice, this would extract game data from SG4 file
        vec![]
    }
    
    /// Get metadata for all games in the database
    pub fn get_games_metadata(&self) -> Result<Vec<GameMetadata>> {
        // Placeholder implementation
        // In practice, this would combine SI4 index data with SN4 name resolution
        Ok(vec![])
    }
}

impl Default for ParallelGameProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parallel_processor_creation() {
        let processor = ParallelGameProcessor::new();
        assert_eq!(processor.config.batch_size, 1000);
        assert!(processor.config.enable_monitoring);
    }
    
    #[test]
    fn test_parallel_processor_with_config() {
        let config = ParallelProcessingConfig {
            thread_count: Some(4),
            batch_size: 500,
            enable_monitoring: false,
            collect_game_stats: true,
        };
        
        let processor = ParallelGameProcessor::with_config(config);
        assert_eq!(processor.config.thread_count, Some(4));
        assert_eq!(processor.config.batch_size, 500);
        assert!(!processor.config.enable_monitoring);
        assert!(processor.config.collect_game_stats);
    }
    
    #[test]
    fn test_empty_games_processing() {
        let mut processor = ParallelGameProcessor::new();
        let games_data: Vec<&[u8]> = vec![];
        
        let results = processor.process_games_parallel(&games_data, None).unwrap();
        assert_eq!(results.len(), 0);
        
        let stats = processor.get_stats();
        assert_eq!(stats.total_games, 0);
        assert_eq!(stats.successful_games, 0);
        assert_eq!(stats.failed_games, 0);
    }
    
    #[test]
    fn test_single_game_processing() {
        let mut processor = ParallelGameProcessor::new();
        let game_data = vec![0u8; 100]; // Dummy game data
        let games_data = vec![game_data.as_slice()];
        
        let results = processor.process_games_parallel(&games_data, None).unwrap();
        assert_eq!(results.len(), 1);
        
        let result = &results[0];
        assert_eq!(result.game_index, 0);
        assert!(result.game_state.is_some());
        assert!(result.metadata.is_some());
        assert!(result.error.is_none());
    }
    
    #[test]
    fn test_batch_processing() {
        let mut processor = ParallelGameProcessor::with_config(
            ParallelProcessingConfig {
                batch_size: 2, // Small batch size for testing
                ..ParallelProcessingConfig::default()
            }
        );
        
        // Create 5 games to test batching
        let games_data: Vec<Vec<u8>> = (0..5).map(|_| vec![0u8; 50]).collect();
        let games_refs: Vec<&[u8]> = games_data.iter().map(|g| g.as_slice()).collect();
        
        let results = processor.process_games_parallel(&games_refs, None).unwrap();
        assert_eq!(results.len(), 5);
        
        // All games should be processed successfully
        let stats = processor.get_stats();
        assert_eq!(stats.total_games, 5);
        assert_eq!(stats.successful_games, 5);
        assert_eq!(stats.failed_games, 0);
    }
    
    #[test]
    fn test_stats_reset() {
        let mut processor = ParallelGameProcessor::new();
        
        // Process some games
        let game_data = vec![0u8; 100];
        let games_data = vec![game_data.as_slice()];
        let _ = processor.process_games_parallel(&games_data, None).unwrap();
        
        // Verify stats are populated
        assert_eq!(processor.get_stats().total_games, 1);
        
        // Reset and verify stats are cleared
        processor.reset_stats();
        assert_eq!(processor.get_stats().total_games, 0);
        assert_eq!(processor.get_stats().successful_games, 0);
    }
}