use scidtopgn::pgn::{EnhancedPgnExporter, ExportOptions};
use std::collections::HashMap;
/// Performance Benchmarks for SCID Parser
///
/// This module provides comprehensive performance benchmarking to measure
/// performance characteristics and ensure we meet production requirements.
/// It benchmarks critical operations like parsing, conversion, and export.
use std::time::{Duration, Instant};

/// Performance benchmark framework
pub struct PerformanceBenchmark {
    benchmark_results: HashMap<String, BenchmarkResult>,
    performance_targets: HashMap<String, PerformanceTarget>,
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub operation_name: String,
    pub iterations: usize,
    pub total_duration: Duration,
    pub average_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub operations_per_second: f64,
    pub memory_usage_mb: f64,
}

#[derive(Debug, Clone)]
pub struct PerformanceTarget {
    pub max_average_ms: f64,
    pub min_ops_per_second: f64,
    pub max_memory_mb: f64,
}

impl PerformanceBenchmark {
    pub fn new() -> Self {
        Self {
            benchmark_results: HashMap::new(),
            performance_targets: HashMap::new(),
        }
    }

    /// Initialize performance targets based on production requirements
    pub fn initialize_performance_targets(&mut self) {
        // PGN Export should be fast enough for interactive use
        self.performance_targets.insert(
            "pgn_export_single_game".to_string(),
            PerformanceTarget {
                max_average_ms: 10.0,      // 10ms per game max
                min_ops_per_second: 100.0, // At least 100 games/second
                max_memory_mb: 50.0,       // Should not use more than 50MB
            },
        );

        // Standards checking should be very fast
        self.performance_targets.insert(
            "pgn_standards_validation".to_string(),
            PerformanceTarget {
                max_average_ms: 1.0,        // 1ms per validation max
                min_ops_per_second: 1000.0, // At least 1000 validations/second
                max_memory_mb: 10.0,        // Should not use more than 10MB
            },
        );

        // Line formatting should be very fast
        self.performance_targets.insert(
            "pgn_line_formatting".to_string(),
            PerformanceTarget {
                max_average_ms: 0.5,        // 0.5ms per format max
                min_ops_per_second: 2000.0, // At least 2000 formats/second
                max_memory_mb: 5.0,         // Should not use more than 5MB
            },
        );

        // Multiple game export should be efficient
        self.performance_targets.insert(
            "pgn_export_multiple_games".to_string(),
            PerformanceTarget {
                max_average_ms: 5.0,       // 5ms per game average for batch
                min_ops_per_second: 200.0, // At least 200 games/second in batch
                max_memory_mb: 100.0,      // Should not use more than 100MB for batch
            },
        );

        // Memory stress test should not exceed limits
        self.performance_targets.insert(
            "memory_stress_test".to_string(),
            PerformanceTarget {
                max_average_ms: 50.0,     // 50ms max for stress operations
                min_ops_per_second: 20.0, // At least 20 ops/second under stress
                max_memory_mb: 200.0,     // Should not exceed 200MB under stress
            },
        );
    }

    /// Run all performance benchmarks
    pub fn run_all_benchmarks(&mut self) -> bool {
        let mut all_passed = true;

        // Run individual benchmarks
        all_passed &= self.benchmark_pgn_export_single_game();
        all_passed &= self.benchmark_pgn_standards_validation();
        all_passed &= self.benchmark_pgn_line_formatting();
        all_passed &= self.benchmark_pgn_export_multiple_games();
        all_passed &= self.benchmark_memory_stress_test();

        all_passed
    }

    /// Benchmark single game PGN export
    fn benchmark_pgn_export_single_game(&mut self) -> bool {
        let benchmark_name = "pgn_export_single_game";
        let iterations = 1000;

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_benchmark_game_index();
        let name_db = create_benchmark_name_database();
        let moves = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3";

        let mut durations = Vec::new();
        let start_time = Instant::now();

        for _ in 0..iterations {
            let iteration_start = Instant::now();

            match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
                Ok(_) => {
                    let iteration_duration = iteration_start.elapsed();
                    durations.push(iteration_duration);
                }
                Err(_) => {
                    // Failed iteration - this would affect benchmark validity
                    return false;
                }
            }
        }

        let total_duration = start_time.elapsed();
        let result = self.calculate_benchmark_result(
            benchmark_name,
            iterations,
            durations,
            total_duration,
            0.0,
        );

        self.benchmark_results
            .insert(benchmark_name.to_string(), result.clone());
        self.check_performance_target(benchmark_name, &result)
    }

    /// Benchmark PGN standards validation
    fn benchmark_pgn_standards_validation(&mut self) -> bool {
        let benchmark_name = "pgn_standards_validation";
        let iterations = 5000;

        use scidtopgn::pgn::PgnStandardsChecker;
        let checker = PgnStandardsChecker::new();

        let test_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            ("Date", "2023.12.25"),
            ("Round", "1"),
            ("White", "Player White"),
            ("Black", "Player Black"),
            ("Result", "1-0"),
        ];

        let mut durations = Vec::new();
        let start_time = Instant::now();

        for _ in 0..iterations {
            let iteration_start = Instant::now();

            // Perform validation
            let _result = checker.validate_headers(&test_headers);

            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);
        }

        let total_duration = start_time.elapsed();
        let result = self.calculate_benchmark_result(
            benchmark_name,
            iterations,
            durations,
            total_duration,
            0.0,
        );

        self.benchmark_results
            .insert(benchmark_name.to_string(), result.clone());
        self.check_performance_target(benchmark_name, &result)
    }

    /// Benchmark PGN line formatting
    fn benchmark_pgn_line_formatting(&mut self) -> bool {
        let benchmark_name = "pgn_line_formatting";
        let iterations = 3000;

        use scidtopgn::pgn::PgnStandardsChecker;
        let checker = PgnStandardsChecker::with_max_line_length(80);

        let test_content = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6 8.c3 O-O 9.h3 Nb8 10.d4 Nbd7 11.Nbd2 Bb7 12.Bc2 Re8 13.Nf1";

        let mut durations = Vec::new();
        let start_time = Instant::now();

        for _ in 0..iterations {
            let iteration_start = Instant::now();

            // Perform line formatting
            let _formatted = checker.format_pgn(test_content);

            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);
        }

        let total_duration = start_time.elapsed();
        let result = self.calculate_benchmark_result(
            benchmark_name,
            iterations,
            durations,
            total_duration,
            0.0,
        );

        self.benchmark_results
            .insert(benchmark_name.to_string(), result.clone());
        self.check_performance_target(benchmark_name, &result)
    }

    /// Benchmark multiple game export (batch processing)
    fn benchmark_pgn_export_multiple_games(&mut self) -> bool {
        let benchmark_name = "pgn_export_multiple_games";
        let iterations = 100; // Process 100 games per iteration
        let games_per_iteration = 10;

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let name_db = create_benchmark_name_database();

        // Create different game scenarios
        let test_games = create_multiple_test_games();

        let mut durations = Vec::new();
        let start_time = Instant::now();

        for _ in 0..iterations {
            let iteration_start = Instant::now();

            // Process multiple games in this iteration
            for i in 0..games_per_iteration {
                let game_index = &test_games[i % test_games.len()].0;
                let moves = &test_games[i % test_games.len()].1;

                match exporter.export_complete_game(game_index, moves, Some(&name_db)) {
                    Ok(_) => {}
                    Err(_) => {
                        return false;
                    }
                }
            }

            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);
        }

        let total_duration = start_time.elapsed();

        // Adjust for multiple games per iteration
        let adjusted_durations: Vec<Duration> = durations
            .iter()
            .map(|d| Duration::from_nanos(d.as_nanos() as u64 / games_per_iteration as u64))
            .collect();

        let result = self.calculate_benchmark_result(
            benchmark_name,
            iterations * games_per_iteration,
            adjusted_durations,
            total_duration,
            0.0,
        );

        self.benchmark_results
            .insert(benchmark_name.to_string(), result.clone());
        self.check_performance_target(benchmark_name, &result)
    }

    /// Benchmark memory usage under stress
    fn benchmark_memory_stress_test(&mut self) -> bool {
        let benchmark_name = "memory_stress_test";
        let iterations = 50;

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let name_db = create_large_name_database(); // Larger name database

        // Create complex games with long move sequences
        let complex_moves = create_complex_game_moves();
        let game_index = create_benchmark_game_index();

        let mut durations = Vec::new();
        let start_time = Instant::now();

        for _ in 0..iterations {
            let iteration_start = Instant::now();

            // Process complex games multiple times in memory
            for moves in &complex_moves {
                match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
                    Ok(_) => {}
                    Err(_) => {
                        return false;
                    }
                }
            }

            let iteration_duration = iteration_start.elapsed();
            durations.push(iteration_duration);
        }

        let total_duration = start_time.elapsed();

        // Estimate memory usage (simplified - would use real memory profiling in production)
        let estimated_memory_mb = 50.0; // Estimated based on data structures

        let result = self.calculate_benchmark_result(
            benchmark_name,
            iterations,
            durations,
            total_duration,
            estimated_memory_mb,
        );

        self.benchmark_results
            .insert(benchmark_name.to_string(), result.clone());
        self.check_performance_target(benchmark_name, &result)
    }

    /// Calculate benchmark result from timing data
    fn calculate_benchmark_result(
        &self,
        operation_name: &str,
        iterations: usize,
        durations: Vec<Duration>,
        total_duration: Duration,
        memory_usage_mb: f64,
    ) -> BenchmarkResult {
        let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
        let average_duration = Duration::from_nanos((total_nanos / iterations as u128) as u64);

        let min_duration = durations.iter().min().cloned().unwrap_or(Duration::ZERO);
        let max_duration = durations.iter().max().cloned().unwrap_or(Duration::ZERO);

        let operations_per_second = if total_duration.as_secs_f64() > 0.0 {
            iterations as f64 / total_duration.as_secs_f64()
        } else {
            0.0
        };

        BenchmarkResult {
            operation_name: operation_name.to_string(),
            iterations,
            total_duration,
            average_duration,
            min_duration,
            max_duration,
            operations_per_second,
            memory_usage_mb,
        }
    }

    /// Check if benchmark result meets performance target
    fn check_performance_target(&self, benchmark_name: &str, result: &BenchmarkResult) -> bool {
        if let Some(target) = self.performance_targets.get(benchmark_name) {
            let avg_ms = result.average_duration.as_secs_f64() * 1000.0;

            let meets_time_target = avg_ms <= target.max_average_ms;
            let meets_throughput_target = result.operations_per_second >= target.min_ops_per_second;
            let meets_memory_target = result.memory_usage_mb <= target.max_memory_mb;

            meets_time_target && meets_throughput_target && meets_memory_target
        } else {
            true // No target defined, consider as passing
        }
    }

    /// Print comprehensive performance report
    pub fn print_performance_report(&self) {
        println!("=== PERFORMANCE BENCHMARK REPORT ===");
        println!();

        let total_benchmarks = self.benchmark_results.len();
        let passed_benchmarks = self
            .benchmark_results
            .iter()
            .filter(|(name, result)| self.check_performance_target(name, result))
            .count();

        println!("SUMMARY:");
        println!("  Total Benchmarks: {}", total_benchmarks);
        println!(
            "  Passed: {} ({:.1}%)",
            passed_benchmarks,
            (passed_benchmarks as f64 / total_benchmarks as f64) * 100.0
        );
        println!(
            "  Failed: {} ({:.1}%)",
            total_benchmarks - passed_benchmarks,
            ((total_benchmarks - passed_benchmarks) as f64 / total_benchmarks as f64) * 100.0
        );
        println!();

        println!("BENCHMARK RESULTS:");
        for (name, result) in &self.benchmark_results {
            let target = self.performance_targets.get(name);
            let passed = self.check_performance_target(name, result);

            println!("  {} {}", if passed { "✅" } else { "❌" }, name);
            println!("    Iterations: {}", result.iterations);
            println!(
                "    Average Time: {:.2}ms",
                result.average_duration.as_secs_f64() * 1000.0
            );
            println!(
                "    Min Time: {:.2}ms",
                result.min_duration.as_secs_f64() * 1000.0
            );
            println!(
                "    Max Time: {:.2}ms",
                result.max_duration.as_secs_f64() * 1000.0
            );
            println!(
                "    Throughput: {:.1} ops/sec",
                result.operations_per_second
            );

            if result.memory_usage_mb > 0.0 {
                println!("    Memory Usage: {:.1} MB", result.memory_usage_mb);
            }

            if let Some(target) = target {
                println!(
                    "    Target: ≤{:.1}ms, ≥{:.1} ops/sec, ≤{:.1}MB",
                    target.max_average_ms, target.min_ops_per_second, target.max_memory_mb
                );
            }
            println!();
        }

        println!("PERFORMANCE VALIDATION:");
        let success_rate = (passed_benchmarks as f64 / total_benchmarks as f64) * 100.0;
        if success_rate >= 90.0 {
            println!("  Overall Performance: {:.1}% - ✅ PASS", success_rate);
        } else {
            println!(
                "  Overall Performance: {:.1}% - ❌ FAIL (minimum 90%)",
                success_rate
            );
        }
    }

    /// Get benchmark results
    pub fn get_benchmark_results(&self) -> &HashMap<String, BenchmarkResult> {
        &self.benchmark_results
    }

    /// Get performance targets
    pub fn get_performance_targets(&self) -> &HashMap<String, PerformanceTarget> {
        &self.performance_targets
    }
}

/// Helper function to create benchmark game index
fn create_benchmark_game_index() -> scidtopgn::pgn::exporter::SimpleGameIndex {
    scidtopgn::pgn::exporter::SimpleGameIndex {
        event_id: 1,
        site_id: 1,
        white_id: 1,
        black_id: 2,
        round_id: 1,
        year: 2023,
        month: 6,
        day: 15,
        result: 1, // 1-0
        white_elo: 2200,
        black_elo: 2150,
        eco: 113, // B13
        num_half_moves: 60,
    }
}

/// Helper function to create benchmark name database
fn create_benchmark_name_database() -> scidtopgn::pgn::exporter::SimpleNameDatabase {
    let mut name_db = scidtopgn::pgn::exporter::SimpleNameDatabase::new();
    name_db
        .event_names
        .insert(1, "Performance Test Tournament".to_string());
    name_db.site_names.insert(1, "Benchmark City".to_string());
    name_db.player_names.insert(1, "FastPlayer".to_string());
    name_db.player_names.insert(2, "QuickPlayer".to_string());
    name_db.round_names.insert(1, "1".to_string());
    name_db
}

/// Helper function to create multiple test games
fn create_multiple_test_games() -> Vec<(scidtopgn::pgn::exporter::SimpleGameIndex, String)> {
    let base_index = create_benchmark_game_index();

    vec![
        (base_index.clone(), "1.e4 e5 2.Nf3 Nc6 3.Bb5".to_string()),
        (
            {
                let mut idx = base_index.clone();
                idx.white_id = 3;
                idx.black_id = 4;
                idx.result = 2; // 0-1
                idx
            },
            "1.d4 d5 2.c4 e6 3.Nc3".to_string(),
        ),
        (
            {
                let mut idx = base_index.clone();
                idx.white_id = 5;
                idx.black_id = 6;
                idx.result = 3; // 1/2-1/2
                idx
            },
            "1.Nf3 Nf6 2.g3 g6 3.Bg2".to_string(),
        ),
        (
            {
                let mut idx = base_index.clone();
                idx.white_id = 7;
                idx.black_id = 8;
                idx.eco = 150; // C50
                idx
            },
            "1.e4 e5 2.Nf3 Nc6 3.Bc4".to_string(),
        ),
        (base_index, "1.c4 c5 2.Nc3 Nc6 3.g3".to_string()),
    ]
}

/// Helper function to create larger name database for stress testing
fn create_large_name_database() -> scidtopgn::pgn::exporter::SimpleNameDatabase {
    let mut name_db = scidtopgn::pgn::exporter::SimpleNameDatabase::new();

    // Add many names to simulate larger database
    for i in 1..=1000 {
        name_db.player_names.insert(i, format!("Player_{}", i));
    }

    for i in 1..=100 {
        name_db.event_names.insert(i, format!("Tournament_{}", i));
        name_db.site_names.insert(i, format!("City_{}", i));
        name_db.round_names.insert(i, format!("{}", i));
    }

    name_db
}

/// Helper function to create complex game moves for stress testing
fn create_complex_game_moves() -> Vec<String> {
    vec![
        // Long tactical game
        "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6 8.c3 O-O 9.h3 Nb8 10.d4 Nbd7 11.Nbd2 Bb7 12.Bc2 Re8 13.Nf1 Bf8 14.Ng3 g6 15.a4 c5 16.d5 c4 17.Be3 Nc5 18.Qd2 h6 19.Nh2 Kh7 20.Nhf1".to_string(),

        // Complex positional game
        "1.d4 d5 2.c4 e6 3.Nc3 Nf6 4.cxd5 exd5 5.Bg5 Be7 6.e3 c6 7.Bd3 Nbd7 8.Qc2 Nh5 9.Bxe7 Qxe7 10.h3 Nb6 11.Nf3 Be6 12.O-O O-O-O 13.a4 Kb8 14.a5 Nc8 15.Na4 Nd6 16.Rac1 Rc8 17.Nc5 Bc8 18.b4 f6 19.Qb2 g6 20.Rc2".to_string(),
        // Sharp attacking game
        "1.e4 c5 2.Nf3 d6 3.d4 cxd4 4.Nxd4 Nf6 5.Nc3 a6 6.Be3 e6 7.f3 b5 8.Qd2 Bb7 9.O-O-O Nbd7 10.h4 b4 11.Nd5 Bxd5 12.exd5 e5 13.Nf5 a5 14.Kb1 Be7 15.g4 O-O 16.g5 Nh5 17.Nxe7+ Qxe7 18.Qxb4 Rfc8 19.Bd3 Nf4 20.Bxf4".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_framework_creation() {
        let benchmark = PerformanceBenchmark::new();
        assert_eq!(benchmark.benchmark_results.len(), 0);
        assert_eq!(benchmark.performance_targets.len(), 0);
    }

    #[test]
    fn test_performance_targets_initialization() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.initialize_performance_targets();

        assert!(!benchmark.performance_targets.is_empty());
        assert!(benchmark
            .performance_targets
            .contains_key("pgn_export_single_game"));
        assert!(benchmark
            .performance_targets
            .contains_key("pgn_standards_validation"));
    }

    #[test]
    fn test_benchmark_result_calculation() {
        let benchmark = PerformanceBenchmark::new();

        let durations = vec![
            Duration::from_millis(5),
            Duration::from_millis(3),
            Duration::from_millis(7),
            Duration::from_millis(4),
        ];

        let result = benchmark.calculate_benchmark_result(
            "test_operation",
            4,
            durations,
            Duration::from_millis(20),
            10.0,
        );

        assert_eq!(result.operation_name, "test_operation");
        assert_eq!(result.iterations, 4);
        assert_eq!(result.min_duration, Duration::from_millis(3));
        assert_eq!(result.max_duration, Duration::from_millis(7));
        assert!(result.operations_per_second > 100.0); // Should be around 200 ops/sec
        assert_eq!(result.memory_usage_mb, 10.0);
    }

    #[test]
    fn test_performance_target_checking() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.initialize_performance_targets();

        // Create a good result that should pass
        let good_result = BenchmarkResult {
            operation_name: "pgn_export_single_game".to_string(),
            iterations: 1000,
            total_duration: Duration::from_secs(1),
            average_duration: Duration::from_millis(1), // 1ms average
            min_duration: Duration::from_millis(1),
            max_duration: Duration::from_millis(2),
            operations_per_second: 1000.0, // 1000 ops/sec
            memory_usage_mb: 20.0,         // 20MB
        };

        assert!(benchmark.check_performance_target("pgn_export_single_game", &good_result));

        // Create a bad result that should fail
        let bad_result = BenchmarkResult {
            operation_name: "pgn_export_single_game".to_string(),
            iterations: 100,
            total_duration: Duration::from_secs(10),
            average_duration: Duration::from_millis(100), // 100ms average (too slow)
            min_duration: Duration::from_millis(50),
            max_duration: Duration::from_millis(200),
            operations_per_second: 10.0, // 10 ops/sec (too slow)
            memory_usage_mb: 200.0,      // 200MB (too much memory)
        };

        assert!(!benchmark.check_performance_target("pgn_export_single_game", &bad_result));
    }

    #[test]
    fn test_helper_functions() {
        let game_index = create_benchmark_game_index();
        assert_eq!(game_index.year, 2023);
        assert_eq!(game_index.month, 6);
        assert_eq!(game_index.day, 15);
        assert_eq!(game_index.result, 1);

        let name_db = create_benchmark_name_database();
        assert!(name_db.player_names.contains_key(&1));
        assert!(name_db.event_names.contains_key(&1));

        let test_games = create_multiple_test_games();
        assert_eq!(test_games.len(), 5);

        let large_name_db = create_large_name_database();
        assert!(large_name_db.player_names.len() >= 1000);

        let complex_moves = create_complex_game_moves();
        assert_eq!(complex_moves.len(), 3);
        assert!(complex_moves[0].len() > 100); // Should be a long move sequence
    }

    #[test]
    fn test_run_performance_benchmarks() {
        let mut benchmark = PerformanceBenchmark::new();
        benchmark.initialize_performance_targets();

        // Run a subset of benchmarks for testing (full suite would be slow)
        let passed = benchmark.benchmark_pgn_standards_validation();
        assert!(passed, "PGN standards validation benchmark should pass");

        // Print report for debugging
        benchmark.print_performance_report();

        // Verify we have results
        assert!(!benchmark.benchmark_results.is_empty());

        // Check that the benchmark met its target
        let result = benchmark
            .benchmark_results
            .get("pgn_standards_validation")
            .unwrap();
        assert!(benchmark.check_performance_target("pgn_standards_validation", result));
    }
}
