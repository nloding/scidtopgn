// Phase 5 Performance and Optimization Tests
// Tests for performance monitoring and memory optimization

use scid_parser::position::{
    performance::{MonitoredPositionTracker, PositionTrackingMetrics, PerformanceMonitor, benchmark_position_tracking},
    optimization::{ScidOptimizedPositionTracker as OptimizedPositionTracker, CompactPositionTracker, MovePool, MemoryAnalyzer},
    ScidByteStream, ScidMove, Square, PieceType
};

#[test]
fn test_monitored_position_tracker() {
    let mut monitored_tracker = MonitoredPositionTracker::new();
    
    // Test basic monitoring functionality
    let test_bytes = [0xCF, 0x2C, 0x16, 0x26]; // Some test moves
    
    for &byte in &test_bytes {
        let byte_data = [byte];
        let mut stream = ScidByteStream::new(&byte_data);
        
        // This should record metrics regardless of success/failure
        let _result = monitored_tracker.try_decode_move_with_monitoring(&mut stream);
    }
    
    let metrics = monitored_tracker.get_metrics();
    
    // Validate metrics were recorded
    assert_eq!(metrics.total_moves_processed, test_bytes.len());
    assert!(metrics.decode_time.as_nanos() > 0, "Should record non-zero decode time");
    
    // Test performance report generation
    let report = monitored_tracker.generate_performance_report();
    assert!(report.contains("PERFORMANCE REPORT"));
    assert!(report.contains("Processing Statistics"));
    assert!(report.contains("Timing Statistics"));
    assert!(report.contains("Memory Statistics"));
    
    println!("📊 Monitored Position Tracker Test Results:");
    println!("{}", report);
}

#[test]
fn test_performance_metrics_accuracy() {
    let mut metrics = PositionTrackingMetrics::default();
    
    // Record some test operations
    metrics.record_decode_attempt(true, std::time::Duration::from_millis(5));
    metrics.record_decode_attempt(false, std::time::Duration::from_millis(10));
    metrics.record_decode_attempt(true, std::time::Duration::from_millis(3));
    
    metrics.record_position_update(std::time::Duration::from_millis(2));
    metrics.record_validation_check(std::time::Duration::from_millis(1));
    
    // Validate calculated metrics
    assert_eq!(metrics.total_moves_processed, 3);
    assert_eq!(metrics.successful_decodes, 2);
    assert_eq!(metrics.failed_decodes, 1);
    assert!((metrics.get_success_rate() - 66.67).abs() < 0.1, "Success rate should be ~66.67%");
    assert!((metrics.get_average_decode_time_ms() - 6.0).abs() < 0.1, "Average decode time should be 6ms");
    
    println!("✅ Performance metrics accuracy test passed");
}

#[test]
fn test_optimized_position_tracker() {
    let mut optimized_tracker = OptimizedPositionTracker::with_capacity(5);
    
    // Create some test moves
    let test_moves = create_test_moves_sequence(10);
    
    let mut successful_moves = 0;
    for (i, scid_move) in test_moves.iter().enumerate() {
        match optimized_tracker.add_move(scid_move.clone()) {
            Ok(_) => {
                successful_moves += 1;
                println!("✅ Move {}: Successfully added to optimized tracker", i + 1);
            }
            Err(e) => {
                optimized_tracker.record_failed_move();
                println!("❌ Move {}: Failed to add - {}", i + 1, e);
            }
        }
    }
    
    // Validate memory optimization
    assert!(optimized_tracker.get_recent_moves().len() <= 5, "Should not exceed capacity");
    assert_eq!(optimized_tracker.get_move_count(), successful_moves);
    
    let memory_usage = optimized_tracker.get_memory_usage_kb();
    assert!(memory_usage < 100, "Memory usage should be reasonable (< 100KB), got {}KB", memory_usage);
    
    let memory_report = optimized_tracker.generate_memory_report();
    println!("💾 Memory Optimization Test Results:");
    println!("{}", memory_report);
}

#[test]
fn test_compact_position_tracker() {
    let mut compact_tracker = CompactPositionTracker::new();
    
    // Test the compact tracker with hash-based tracking
    for i in 0..50 {
        if i % 3 == 0 {
            compact_tracker.record_failed_move();
        } else {
            let move_hash = i as u64 * 12345; // Dummy hash
            let position_hash = i as u64 * 67890; // Dummy hash
            compact_tracker.record_successful_move(move_hash, position_hash);
        }
    }
    
    let stats = compact_tracker.get_statistics();
    let expected_success_rate = (33.0 / 50.0) * 100.0; // ~66%
    
    assert_eq!(stats.total_moves, 50);
    assert_eq!(stats.successful_moves, 33);
    assert!((stats.success_rate - expected_success_rate).abs() < 1.0);
    assert!(stats.memory_footprint < 100, "Compact tracker should have minimal memory footprint");
    
    println!("📦 Compact Position Tracker Test Results:");
    println!("   Memory footprint: {} bytes", stats.memory_footprint);
    println!("   Success rate: {:.1}%", stats.success_rate);
    println!("   ✅ Compact tracker validation passed");
}

#[test]
fn test_move_pool_efficiency() {
    let mut pool = MovePool::with_capacity(20);
    
    // Test pool allocation and return
    let mut allocated_moves = Vec::new();
    
    // Allocate moves from pool
    for i in 0..30 {
        let mut scid_move = pool.get_move();
        scid_move.piece_num = i as u8;
        allocated_moves.push(scid_move);
    }
    
    // Return moves to pool
    for scid_move in allocated_moves {
        pool.return_move(scid_move);
    }
    
    let pool_stats = pool.get_pool_statistics();
    
    assert!(pool_stats.available_moves <= 20, "Pool should not exceed capacity");
    assert!(pool_stats.memory_usage_bytes > 0, "Pool should report memory usage");
    
    println!("🏊 Move Pool Test Results:");
    println!("   Available moves: {}", pool_stats.available_moves);
    println!("   Pool capacity: {}", pool_stats.pool_capacity);
    println!("   Memory usage: {} bytes", pool_stats.memory_usage_bytes);
    println!("   ✅ Move pool efficiency test passed");
}

#[test]
fn test_memory_analyzer() {
    let analysis = MemoryAnalyzer::analyze_position_memory();
    
    // Validate memory analysis results
    assert!(analysis.position_size > 0, "Position should have non-zero size");
    assert!(analysis.move_size > 0, "Move should have non-zero size");
    assert!(analysis.compact_tracker_size < analysis.tracker_base_size, 
           "Compact tracker should be smaller than base tracker");
    
    println!("🔍 Memory Analysis Results:");
    println!("   Position size: {} bytes", analysis.position_size);
    println!("   Move size: {} bytes", analysis.move_size);
    println!("   Base tracker size: {} bytes", analysis.tracker_base_size);
    println!("   Compact tracker size: {} bytes", analysis.compact_tracker_size);
    
    // Test game memory estimation
    let estimate = MemoryAnalyzer::estimate_game_memory_usage(100, 10);
    assert_eq!(estimate.move_count, 100);
    assert_eq!(estimate.recent_moves_limit, 10);
    assert!(estimate.total_kb > 0, "Should have non-zero memory estimate");
    
    println!("   Game memory estimate (100 moves, 10 recent): {}KB", estimate.total_kb);
    
    // Test optimization recommendations
    let recommendation = MemoryAnalyzer::recommend_optimization(100, 1); // 1MB limit
    assert!(!recommendation.recommendations.is_empty(), "Should provide recommendations");
    
    println!("💡 Optimization Recommendations for 100 moves with 1MB limit:");
    for rec in &recommendation.recommendations {
        println!("   {}", rec);
    }
}

#[test]
fn test_performance_monitor_utility() {
    let mut monitor = PerformanceMonitor::new("test_operation");
    
    monitor.start();
    
    // Simulate some work
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let duration = monitor.stop_and_report();
    assert!(duration.as_millis() >= 10, "Should measure at least 10ms");
    
    println!("✅ Performance monitor utility test passed");
}

#[test]
fn test_benchmarking_utility() {
    let mut counter = 0;
    
    benchmark_position_tracking("test_increment_operation", 100, || {
        counter += 1;
        if counter % 10 == 0 {
            // Simulate occasional failure
            Err("Simulated failure".to_string())
        } else {
            Ok(())
        }
    });
    
    assert!(counter >= 90, "Should have performed most operations successfully");
    println!("✅ Benchmarking utility test completed");
}

#[test]
fn test_phase5_integration() {
    println!("🚀 Phase 5 Integration Test");
    
    // Test complete workflow: monitoring + optimization
    let mut monitored_tracker = MonitoredPositionTracker::new();
    let mut optimized_tracker = OptimizedPositionTracker::with_capacity(3);
    
    let test_bytes = [0xCF, 0x2C, 0x16]; // Test moves
    
    // Test monitored tracking
    for &byte in &test_bytes {
        let byte_data = [byte];
        let mut stream = ScidByteStream::new(&byte_data);
        let _result = monitored_tracker.try_decode_move_with_monitoring(&mut stream);
    }
    
    // Test optimized tracking with the same data
    let test_moves = create_test_moves_sequence(3);
    for scid_move in test_moves {
        match optimized_tracker.add_move(scid_move) {
            Ok(_) => {}
            Err(_) => optimized_tracker.record_failed_move(),
        }
    }
    
    // Generate comprehensive reports
    let performance_report = monitored_tracker.generate_performance_report();
    let memory_report = optimized_tracker.generate_memory_report();
    
    println!("📊 PHASE 5 INTEGRATION TEST RESULTS:");
    println!("{}", performance_report);
    println!("{}", memory_report);
    
    // Validate integration
    assert!(performance_report.contains("PERFORMANCE REPORT"));
    assert!(memory_report.contains("MEMORY OPTIMIZATION"));
    
    println!("✅ Phase 5 integration test completed successfully!");
}

// Helper functions

fn create_test_moves_sequence(count: usize) -> Vec<ScidMove> {
    let mut moves = Vec::new();
    
    for i in 0..count {
        moves.push(ScidMove {
            from: Square((i % 64) as u8),
            to: Square(((i + 10) % 64) as u8),
            moving_piece: match i % 6 {
                0 => PieceType::Pawn,
                1 => PieceType::Knight,
                2 => PieceType::Bishop,
                3 => PieceType::Rook,
                4 => PieceType::Queen,
                5 => PieceType::King,
                _ => PieceType::Pawn,
            },
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: (i % 16) as u8,
        });
    }
    
    moves
}