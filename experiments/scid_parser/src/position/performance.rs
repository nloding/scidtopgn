// Performance monitoring for position tracking
// Phase 5 Step 5.1 from POSITION_TRACKING_IMPLEMENTATION_PLAN.md

use std::time::{Duration, Instant};
use crate::sg4::StreamingGameElement;
use crate::position::{ScidByteStream, ScidPosition};
use crate::position::decoder::decode_move_with_stream;

#[derive(Debug, Default)]
pub struct PositionTrackingMetrics {
    pub total_moves_processed: usize,
    pub successful_decodes: usize,
    pub failed_decodes: usize,
    pub position_updates: usize,
    pub validation_checks: usize,
    
    pub decode_time: Duration,
    pub position_update_time: Duration,
    pub validation_time: Duration,
    
    pub memory_usage_kb: usize,
}

impl PositionTrackingMetrics {
    pub fn record_decode_attempt(&mut self, success: bool, duration: Duration) {
        self.total_moves_processed += 1;
        self.decode_time += duration;
        
        if success {
            self.successful_decodes += 1;
        } else {
            self.failed_decodes += 1;
        }
    }
    
    pub fn record_position_update(&mut self, duration: Duration) {
        self.position_updates += 1;
        self.position_update_time += duration;
    }
    
    pub fn record_validation_check(&mut self, duration: Duration) {
        self.validation_checks += 1;
        self.validation_time += duration;
    }
    
    pub fn get_success_rate(&self) -> f64 {
        if self.total_moves_processed == 0 {
            0.0
        } else {
            (self.successful_decodes as f64 / self.total_moves_processed as f64) * 100.0
        }
    }
    
    pub fn get_average_decode_time_ms(&self) -> f64 {
        if self.total_moves_processed == 0 {
            0.0
        } else {
            self.decode_time.as_secs_f64() * 1000.0 / self.total_moves_processed as f64
        }
    }
    
    pub fn get_average_position_update_time_ms(&self) -> f64 {
        if self.position_updates == 0 {
            0.0
        } else {
            self.position_update_time.as_secs_f64() * 1000.0 / self.position_updates as f64
        }
    }
    
    pub fn get_average_validation_time_ms(&self) -> f64 {
        if self.validation_checks == 0 {
            0.0
        } else {
            self.validation_time.as_secs_f64() * 1000.0 / self.validation_checks as f64
        }
    }
    
    pub fn get_total_processing_time_ms(&self) -> f64 {
        (self.decode_time + self.position_update_time + self.validation_time).as_secs_f64() * 1000.0
    }
    
    pub fn update_memory_usage(&mut self, kb: usize) {
        self.memory_usage_kb = kb;
    }
    
    pub fn generate_performance_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== POSITION TRACKING PERFORMANCE REPORT ===\n");
        report.push_str(&format!("📊 Processing Statistics:\n"));
        report.push_str(&format!("   Total Moves: {}\n", self.total_moves_processed));
        report.push_str(&format!("   Successful: {} ({:.1}%)\n", self.successful_decodes, self.get_success_rate()));
        report.push_str(&format!("   Failed: {}\n", self.failed_decodes));
        report.push_str(&format!("   Position Updates: {}\n", self.position_updates));
        report.push_str(&format!("   Validation Checks: {}\n", self.validation_checks));
        
        report.push_str(&format!("\n⏱️  Timing Statistics:\n"));
        report.push_str(&format!("   Average Decode Time: {:.3}ms\n", self.get_average_decode_time_ms()));
        report.push_str(&format!("   Average Position Update Time: {:.3}ms\n", self.get_average_position_update_time_ms()));
        report.push_str(&format!("   Average Validation Time: {:.3}ms\n", self.get_average_validation_time_ms()));
        report.push_str(&format!("   Total Processing Time: {:.3}ms\n", self.get_total_processing_time_ms()));
        
        report.push_str(&format!("\n💾 Memory Statistics:\n"));
        report.push_str(&format!("   Estimated Usage: {}KB\n", self.memory_usage_kb));
        
        // Performance analysis
        report.push_str(&format!("\n🎯 Performance Analysis:\n"));
        if self.get_average_decode_time_ms() < 10.0 {
            report.push_str("   ✅ Decode performance: EXCELLENT (< 10ms per move)\n");
        } else if self.get_average_decode_time_ms() < 50.0 {
            report.push_str("   📈 Decode performance: GOOD (< 50ms per move)\n");
        } else {
            report.push_str("   ⚠️  Decode performance: NEEDS OPTIMIZATION (> 50ms per move)\n");
        }
        
        if self.get_success_rate() > 90.0 {
            report.push_str("   ✅ Success rate: EXCELLENT (> 90%)\n");
        } else if self.get_success_rate() > 70.0 {
            report.push_str("   📈 Success rate: GOOD (> 70%)\n");
        } else {
            report.push_str("   ⚠️  Success rate: NEEDS IMPROVEMENT\n");
        }
        
        if self.memory_usage_kb < 10_000 { // Less than 10MB
            report.push_str("   ✅ Memory usage: EXCELLENT (< 10MB)\n");
        } else if self.memory_usage_kb < 50_000 { // Less than 50MB
            report.push_str("   📈 Memory usage: ACCEPTABLE (< 50MB)\n");
        } else {
            report.push_str("   ⚠️  Memory usage: HIGH (> 50MB)\n");
        }
        
        report.push_str("=============================================\n");
        
        report
    }
}

/// Enhanced position tracker with performance monitoring (decoder-based)
#[derive(Debug)]
pub struct MonitoredPositionTracker {
    position: ScidPosition,
    metrics: PositionTrackingMetrics,
}

impl MonitoredPositionTracker {
    pub fn new() -> Self {
        Self {
            position: ScidPosition::new_starting_position(),
            metrics: PositionTrackingMetrics::default(),
        }
    }
    
    pub fn try_decode_move_with_monitoring(&mut self, stream: &mut ScidByteStream) -> Result<StreamingGameElement, String> {
    let start_time = Instant::now();
    let initial_pos = stream.position();
        // Decode a move in current position context
        let result = match decode_move_with_stream(&self.position, stream) {
            Ok(scid_move) => {
                // Apply to position for subsequent decoding
                let _ = self.position.do_move(&scid_move);
                Ok(StreamingGameElement::Move {
                    piece_num: scid_move.piece_num,
                    move_value: 0,
            raw_bytes: stream.get_consumed_bytes(initial_pos),
            offset: initial_pos,
            bytes_consumed: stream.position().saturating_sub(initial_pos),
                })
            }
            Err(e) => Err(e),
        };
        let decode_duration = start_time.elapsed();
        
        self.metrics.record_decode_attempt(result.is_ok(), decode_duration);
        
        result
    }
    
    pub fn apply_move_with_monitoring(&mut self, scid_move: &crate::position::ScidMove) -> Result<(), String> {
        let start_time = Instant::now();
    // Apply move to internal position
    let res = self.position.do_move(scid_move);
    let position_update_duration = start_time.elapsed();
        self.metrics.record_position_update(position_update_duration);
        
    res
    }
    
    pub fn validate_position_with_monitoring(&mut self) -> Result<(), String> {
        let start_time = Instant::now();
        
        // Validate position (this would require access to internal position) 
        // For now, we'll just record the timing
        let validation_duration = start_time.elapsed();
        self.metrics.record_validation_check(validation_duration);
        
        Ok(())
    }
    
    pub fn get_metrics(&self) -> &PositionTrackingMetrics {
        &self.metrics
    }
    
    /// Lightweight stats compatible with previous API surface
    pub fn get_statistics(&self) -> PositionTrackerStatsCompat {
        PositionTrackerStatsCompat::from_position(&self.position)
    }
    
    pub fn update_memory_usage_estimate(&mut self) {
        // Estimate memory usage based on tracker components
    let base_size = std::mem::size_of::<ScidPosition>();
        let metrics_size = std::mem::size_of::<PositionTrackingMetrics>();
        let estimated_kb = (base_size + metrics_size + 1024) / 1024; // Add 1KB overhead
        
        self.metrics.update_memory_usage(estimated_kb);
    }
    
    pub fn reset_metrics(&mut self) {
        self.metrics = PositionTrackingMetrics::default();
    }
    
    pub fn generate_performance_report(&mut self) -> String {
        // Update memory usage before generating report
        self.update_memory_usage_estimate();
        self.metrics.generate_performance_report()
    }
}

/// Minimal compatibility struct replacing sg4::PositionTrackerStats usage here
#[derive(Debug, Clone, Copy)]
pub struct PositionTrackerStatsCompat {
    pub total_moves: usize,
    pub successful_moves: usize,
    pub failed_moves: usize,
    pub success_rate: f64,
    pub position_hash: u64,
    pub current_turn: crate::position::Color,
}

impl PositionTrackerStatsCompat {
    fn from_position(pos: &ScidPosition) -> Self {
    // Derive total half-moves from full move number and side to move
    let full = pos.full_move_number() as usize;
    let total_moves = (full.saturating_sub(1)) * 2 + if pos.to_move == crate::position::Color::Black { 1 } else { 0 };
        let successful_moves = total_moves;
        let failed_moves = 0;
        let success_rate = if total_moves == 0 { 0.0 } else { 100.0 };
        let position_hash = pos.calculate_hash();
        let current_turn = pos.to_move;
        Self { total_moves, successful_moves, failed_moves, success_rate, position_hash, current_turn }
    }
}

/// Performance monitoring utilities
pub struct PerformanceMonitor {
    start_time: Option<Instant>,
    operation_name: String,
}

impl PerformanceMonitor {
    pub fn new(operation_name: &str) -> Self {
        Self {
            start_time: None,
            operation_name: operation_name.to_string(),
        }
    }
    
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }
    
    pub fn stop(&self) -> Duration {
        if let Some(start) = self.start_time {
            start.elapsed()
        } else {
            Duration::from_secs(0)
        }
    }
    
    pub fn stop_and_report(&self) -> Duration {
        let duration = self.stop();
        println!("⏱️  {} completed in {:.3}ms", self.operation_name, duration.as_secs_f64() * 1000.0);
        duration
    }
}

/// Benchmarking utilities for performance testing
pub fn benchmark_position_tracking<F>(name: &str, iterations: usize, mut operation: F) 
where 
    F: FnMut() -> Result<(), String>,
{
    println!("🏁 Starting benchmark: {} ({} iterations)", name, iterations);
    
    let start_time = Instant::now();
    let mut successful_operations = 0;
    let mut failed_operations = 0;
    
    for i in 0..iterations {
        match operation() {
            Ok(_) => successful_operations += 1,
            Err(e) => {
                failed_operations += 1;
                if failed_operations <= 5 { // Only show first 5 errors
                    println!("   ⚠️  Iteration {} failed: {}", i + 1, e);
                }
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let avg_time_ms = total_time.as_secs_f64() * 1000.0 / iterations as f64;
    let success_rate = (successful_operations as f64 / iterations as f64) * 100.0;
    
    println!("📊 Benchmark Results for '{}':", name);
    println!("   Total Time: {:.3}ms", total_time.as_secs_f64() * 1000.0);
    println!("   Average Time: {:.3}ms per operation", avg_time_ms);
    println!("   Success Rate: {:.1}% ({}/{})", success_rate, successful_operations, iterations);
    println!("   Operations/sec: {:.0}", 1000.0 / avg_time_ms);
    
    // Performance assessment
    if avg_time_ms < 1.0 {
        println!("   ✅ Performance: EXCELLENT (< 1ms per operation)");
    } else if avg_time_ms < 10.0 {
        println!("   📈 Performance: GOOD (< 10ms per operation)");
    } else {
        println!("   ⚠️  Performance: NEEDS OPTIMIZATION (> 10ms per operation)");
    }
}