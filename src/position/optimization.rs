// Memory usage optimization for position tracking
// Phase 5 Step 5.2 from POSITION_TRACKING_IMPLEMENTATION_PLAN.md

use crate::position::{ScidMove, ScidPosition};
use std::collections::VecDeque;

/// Memory-optimized position tracker for large games (ScidPosition-based)
#[derive(Debug)]
#[allow(dead_code)]
pub struct ScidOptimizedPositionTracker {
    current_position: ScidPosition,
    // Only keep recent move history to save memory
    recent_moves: VecDeque<ScidMove>, // Limited to last N moves
    move_count: usize,
    failed_move_count: usize,
    max_recent_moves: usize,
}

#[allow(dead_code)]
impl ScidOptimizedPositionTracker {
    const DEFAULT_MAX_RECENT_MOVES: usize = 10;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_MAX_RECENT_MOVES)
    }

    pub fn with_capacity(max_recent_moves: usize) -> Self {
        Self {
            current_position: ScidPosition::new_starting_position(),
            recent_moves: VecDeque::with_capacity(max_recent_moves),
            move_count: 0,
            failed_move_count: 0,
            max_recent_moves,
        }
    }

    pub fn add_move(&mut self, scid_move: ScidMove) -> Result<(), String> {
        // Apply move to position
        self.current_position.do_move(&scid_move)?;

        // Add to recent moves (with circular buffer behavior)
        if self.recent_moves.len() >= self.max_recent_moves {
            self.recent_moves.pop_front();
        }
        self.recent_moves.push_back(scid_move);

        self.move_count += 1;
        Ok(())
    }

    pub fn record_failed_move(&mut self) {
        self.failed_move_count += 1;
    }

    pub fn get_current_position(&self) -> &ScidPosition {
        &self.current_position
    }

    pub fn get_recent_moves(&self) -> &VecDeque<ScidMove> {
        &self.recent_moves
    }

    pub fn get_move_count(&self) -> usize {
        self.move_count
    }

    pub fn get_failed_move_count(&self) -> usize {
        self.failed_move_count
    }

    pub fn get_success_rate(&self) -> f64 {
        let total = self.move_count + self.failed_move_count;
        if total == 0 {
            0.0
        } else {
            (self.move_count as f64 / total as f64) * 100.0
        }
    }

    pub fn get_memory_usage_estimate(&self) -> usize {
        // Estimate memory usage in bytes
        std::mem::size_of::<ScidPosition>()
            + self.recent_moves.capacity() * std::mem::size_of::<ScidMove>()
            + std::mem::size_of::<Self>()
            + 64 // Additional overhead
    }

    pub fn get_memory_usage_kb(&self) -> usize {
        (self.get_memory_usage_estimate() + 512) / 1024 // Round up to KB
    }

    pub fn clear_recent_moves(&mut self) {
        self.recent_moves.clear();
    }

    pub fn set_max_recent_moves(&mut self, max_moves: usize) {
        self.max_recent_moves = max_moves;

        // Shrink recent moves if necessary
        while self.recent_moves.len() > max_moves {
            self.recent_moves.pop_front();
        }

        // Resize the VecDeque capacity if needed
        self.recent_moves.shrink_to_fit();
        if self.recent_moves.capacity() < max_moves {
            self.recent_moves
                .reserve(max_moves - self.recent_moves.len());
        }
    }

    pub fn reset(&mut self) {
        self.current_position = ScidPosition::new_starting_position();
        self.recent_moves.clear();
        self.move_count = 0;
        self.failed_move_count = 0;
    }

    pub fn generate_memory_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== MEMORY OPTIMIZATION REPORT ===\n");
        report.push_str(&format!("📊 Statistics:\n"));
        report.push_str(&format!("   Total Moves: {}\n", self.move_count));
        report.push_str(&format!("   Failed Moves: {}\n", self.failed_move_count));
        report.push_str(&format!(
            "   Success Rate: {:.1}%\n",
            self.get_success_rate()
        ));
        report.push_str(&format!(
            "   Recent Moves Stored: {}/{}\n",
            self.recent_moves.len(),
            self.max_recent_moves
        ));

        report.push_str(&format!("\n💾 Memory Usage:\n"));
        report.push_str(&format!(
            "   Position Size: {} bytes\n",
            std::mem::size_of::<ScidPosition>()
        ));
        report.push_str(&format!(
            "   Recent Moves Size: {} bytes\n",
            self.recent_moves.capacity() * std::mem::size_of::<ScidMove>()
        ));
        report.push_str(&format!(
            "   Tracker Overhead: {} bytes\n",
            std::mem::size_of::<Self>()
        ));
        report.push_str(&format!(
            "   Total Estimated: {} bytes ({} KB)\n",
            self.get_memory_usage_estimate(),
            self.get_memory_usage_kb()
        ));

        // Memory efficiency analysis
        let efficiency_ratio = if self.move_count > 0 {
            self.recent_moves.len() as f64 / self.move_count as f64
        } else {
            0.0
        };

        report.push_str(&format!("\n🎯 Memory Efficiency:\n"));
        report.push_str(&format!(
            "   Storage Ratio: {:.1}% (recent/total moves)\n",
            efficiency_ratio * 100.0
        ));

        if self.get_memory_usage_kb() < 100 {
            report.push_str("   ✅ Memory usage: EXCELLENT (< 100KB)\n");
        } else if self.get_memory_usage_kb() < 1000 {
            report.push_str("   📈 Memory usage: GOOD (< 1MB)\n");
        } else {
            report.push_str("   ⚠️  Memory usage: HIGH (> 1MB)\n");
        }

        if efficiency_ratio < 0.1 {
            report.push_str("   ✅ Storage efficiency: EXCELLENT (< 10% stored)\n");
        } else if efficiency_ratio < 0.5 {
            report.push_str("   📈 Storage efficiency: GOOD (< 50% stored)\n");
        } else {
            report.push_str("   ⚠️  Storage efficiency: LOW (> 50% stored)\n");
        }

        report.push_str("===================================\n");

        report
    }
}

/// Memory pool for reusing ScidMove objects
#[derive(Debug)]
#[allow(dead_code)]
pub struct MovePool {
    available_moves: Vec<ScidMove>,
    allocated_moves: usize,
    max_pool_size: usize,
}

#[allow(dead_code)]
impl MovePool {
    const DEFAULT_MAX_POOL_SIZE: usize = 100;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_MAX_POOL_SIZE)
    }

    pub fn with_capacity(max_pool_size: usize) -> Self {
        Self {
            available_moves: Vec::with_capacity(max_pool_size),
            allocated_moves: 0,
            max_pool_size,
        }
    }

    pub fn get_move(&mut self) -> ScidMove {
        self.allocated_moves += 1;

        if let Some(reused_move) = self.available_moves.pop() {
            reused_move
        } else {
            // Create new move with default values
            ScidMove {
                from: crate::position::Square(0),
                to: crate::position::Square(0),
                moving_piece: crate::position::PieceType::Empty,
                captured_piece: crate::position::PieceType::Empty,
                promote: crate::position::PieceType::Empty,
                piece_num: 0,
            }
        }
    }

    pub fn return_move(&mut self, scid_move: ScidMove) {
        if self.available_moves.len() < self.max_pool_size {
            self.available_moves.push(scid_move);
        }
        // If pool is full, just drop the move (let it be garbage collected)
    }

    pub fn get_pool_statistics(&self) -> PoolStatistics {
        PoolStatistics {
            available_moves: self.available_moves.len(),
            allocated_moves: self.allocated_moves,
            pool_capacity: self.max_pool_size,
            memory_usage_bytes: self.available_moves.capacity() * std::mem::size_of::<ScidMove>(),
        }
    }

    pub fn clear(&mut self) {
        self.available_moves.clear();
        self.allocated_moves = 0;
    }

    pub fn shrink_to_fit(&mut self) {
        self.available_moves.shrink_to_fit();
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PoolStatistics {
    #[allow(dead_code)]
    pub available_moves: usize,
    #[allow(dead_code)]
    pub allocated_moves: usize,
    #[allow(dead_code)]
    pub pool_capacity: usize,
    #[allow(dead_code)]
    pub memory_usage_bytes: usize,
}

/// Compact position tracker with minimal memory footprint
/// Trades some functionality for memory efficiency
#[derive(Debug)]
#[allow(dead_code)]
pub struct CompactPositionTracker {
    // Only essential data
    move_count: u32,       // 32-bit counter (4 bytes vs 8 bytes for usize)
    successful_moves: u32, // 32-bit counter
    position_hash: u64,    // Current position hash instead of full position
    last_move_hash: u64,   // Hash of last move for validation
}

#[allow(dead_code)]
impl CompactPositionTracker {
    pub fn new() -> Self {
        Self {
            move_count: 0,
            successful_moves: 0,
            position_hash: 0,
            last_move_hash: 0,
        }
    }

    pub fn record_successful_move(&mut self, move_hash: u64, position_hash: u64) {
        self.move_count += 1;
        self.successful_moves += 1;
        self.last_move_hash = move_hash;
        self.position_hash = position_hash;
    }

    pub fn record_failed_move(&mut self) {
        self.move_count += 1;
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.move_count == 0 {
            0.0
        } else {
            (self.successful_moves as f64 / self.move_count as f64) * 100.0
        }
    }

    pub fn get_memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    pub fn get_statistics(&self) -> CompactStatistics {
        CompactStatistics {
            total_moves: self.move_count as usize,
            successful_moves: self.successful_moves as usize,
            success_rate: self.get_success_rate(),
            position_hash: self.position_hash,
            memory_footprint: self.get_memory_footprint(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct CompactStatistics {
    #[allow(dead_code)]
    pub total_moves: usize,
    #[allow(dead_code)]
    pub successful_moves: usize,
    #[allow(dead_code)]
    pub success_rate: f64,
    #[allow(dead_code)]
    pub position_hash: u64,
    #[allow(dead_code)]
    pub memory_footprint: usize,
}

/// Memory usage analyzer for position tracking components
#[allow(dead_code)]
pub struct MemoryAnalyzer;

#[allow(dead_code)]
impl MemoryAnalyzer {
    pub fn analyze_position_memory() -> MemoryAnalysis {
        MemoryAnalysis {
            position_size: std::mem::size_of::<ScidPosition>(),
            move_size: std::mem::size_of::<ScidMove>(),
            tracker_base_size: std::mem::size_of::<ScidOptimizedPositionTracker>(),
            compact_tracker_size: std::mem::size_of::<CompactPositionTracker>(),
        }
    }

    pub fn estimate_game_memory_usage(
        move_count: usize,
        recent_moves_limit: usize,
    ) -> GameMemoryEstimate {
        let analysis = Self::analyze_position_memory();

        let position_memory = analysis.position_size;
        let recent_moves_memory = recent_moves_limit * analysis.move_size;
        let tracker_overhead = analysis.tracker_base_size;

        let total_bytes = position_memory + recent_moves_memory + tracker_overhead;
        let total_kb = (total_bytes + 512) / 1024;
        let total_mb = (total_kb + 512) / 1024;

        GameMemoryEstimate {
            move_count,
            recent_moves_limit,
            position_memory_bytes: position_memory,
            recent_moves_memory_bytes: recent_moves_memory,
            tracker_overhead_bytes: tracker_overhead,
            total_bytes,
            total_kb,
            total_mb,
        }
    }

    pub fn recommend_optimization(
        move_count: usize,
        memory_limit_mb: usize,
    ) -> OptimizationRecommendation {
        let mut recommendations = Vec::new();

        // Test different recent moves limits
        let test_limits = [5, 10, 20, 50, 100];

        for &limit in &test_limits {
            let estimate = Self::estimate_game_memory_usage(move_count, limit);
            if estimate.total_mb <= memory_limit_mb {
                recommendations.push(format!(
                    "✅ Recent moves limit {}: {}MB (within {}MB limit)",
                    limit, estimate.total_mb, memory_limit_mb
                ));
            } else {
                recommendations.push(format!(
                    "❌ Recent moves limit {}: {}MB (exceeds {}MB limit)",
                    limit, estimate.total_mb, memory_limit_mb
                ));
            }
        }

        // Recommend compact tracker for very tight memory constraints
        let compact_estimate = std::mem::size_of::<CompactPositionTracker>() / 1024;
        if memory_limit_mb < 1 && compact_estimate < memory_limit_mb * 1024 {
            recommendations
                .push("💡 Consider CompactPositionTracker for minimal memory usage".to_string());
        }

        OptimizationRecommendation {
            move_count,
            memory_limit_mb,
            recommendations,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct MemoryAnalysis {
    #[allow(dead_code)]
    pub position_size: usize,
    #[allow(dead_code)]
    pub move_size: usize,
    #[allow(dead_code)]
    pub tracker_base_size: usize,
    #[allow(dead_code)]
    pub compact_tracker_size: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct GameMemoryEstimate {
    #[allow(dead_code)]
    pub move_count: usize,
    #[allow(dead_code)]
    pub recent_moves_limit: usize,
    #[allow(dead_code)]
    pub position_memory_bytes: usize,
    #[allow(dead_code)]
    pub recent_moves_memory_bytes: usize,
    #[allow(dead_code)]
    pub tracker_overhead_bytes: usize,
    #[allow(dead_code)]
    pub total_bytes: usize,
    #[allow(dead_code)]
    pub total_kb: usize,
    #[allow(dead_code)]
    pub total_mb: usize,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OptimizationRecommendation {
    #[allow(dead_code)]
    pub move_count: usize,
    #[allow(dead_code)]
    pub memory_limit_mb: usize,
    #[allow(dead_code)]
    pub recommendations: Vec<String>,
}
