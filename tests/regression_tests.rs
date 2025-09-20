use scidtopgn::pgn::{EnhancedPgnExporter, ExportOptions, PgnStandardsChecker};
/// Regression Testing Framework for SCID Parser
///
/// This module provides comprehensive regression testing to ensure that
/// functionality doesn't break as improvements are made to the codebase.
/// It tests critical parsing paths, data structures, and output formats.
use std::collections::HashMap;

/// Regression test framework that validates core functionality
pub struct RegressionTestFramework {
    test_results: HashMap<String, RegressionTestResult>,
    known_good_outputs: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct RegressionTestResult {
    pub test_name: String,
    pub passed: bool,
    pub expected_result: String,
    pub actual_result: String,
    pub error_message: Option<String>,
}

impl RegressionTestFramework {
    pub fn new() -> Self {
        Self {
            test_results: HashMap::new(),
            known_good_outputs: HashMap::new(),
        }
    }

    /// Initialize with known good outputs (golden standards)
    pub fn initialize_known_good_outputs(&mut self) {
        // PGN Export - Standard headers test
        self.known_good_outputs.insert(
            "pgn_export_standard_headers".to_string(),
            r#"[Event "Test Tournament"]
[Site "Test City"]
[Date "2023.12.25"]
[Round "1"]
[White "Player White"]
[Black "Player Black"]
[Result "1-0"]
[WhiteElo "1800"]
[BlackElo "1750"]
[ECO "B13"]
[PlyCount "40"]

1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 1-0"#
                .to_string(),
        );

        // PGN Standards - Line length formatting test
        self.known_good_outputs.insert(
            "pgn_standards_line_formatting".to_string(),
            "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6\n8.c3 O-O 9.h3 Nb8"
                .to_string(),
        );

        // Date formatting test
        self.known_good_outputs.insert(
            "date_formatting_standard".to_string(),
            "2023.12.25".to_string(),
        );

        // ECO formatting test
        self.known_good_outputs
            .insert("eco_formatting_b13".to_string(), "B13".to_string());

        // Result formatting test
        self.known_good_outputs.insert(
            "result_formatting_white_wins".to_string(),
            "1-0".to_string(),
        );
    }

    /// Run all regression tests
    pub fn run_all_tests(&mut self) -> bool {
        let mut all_passed = true;

        // Test PGN export functionality
        all_passed &= self.test_pgn_export_standard_headers();
        all_passed &= self.test_pgn_standards_compliance();
        all_passed &= self.test_date_formatting();
        all_passed &= self.test_eco_formatting();
        all_passed &= self.test_result_formatting();
        all_passed &= self.test_line_length_formatting();
        all_passed &= self.test_header_validation();
        all_passed &= self.test_special_character_handling();

        all_passed
    }

    /// Test PGN export with standard headers
    fn test_pgn_export_standard_headers(&mut self) -> bool {
        let test_name = "pgn_export_standard_headers";

        // Create test data
        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        let moves = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7";

        match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
            Ok(actual_pgn) => {
                let expected = self.known_good_outputs.get(test_name).unwrap();

                // Normalize whitespace for comparison
                let actual_normalized = normalize_pgn_for_comparison(&actual_pgn);
                let expected_normalized = normalize_pgn_for_comparison(expected);

                let passed = actual_normalized == expected_normalized;

                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed,
                        expected_result: expected_normalized,
                        actual_result: actual_normalized,
                        error_message: if !passed {
                            Some("PGN output doesn't match expected format".to_string())
                        } else {
                            None
                        },
                    },
                );

                passed
            }
            Err(e) => {
                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed: false,
                        expected_result: self.known_good_outputs.get(test_name).unwrap().clone(),
                        actual_result: String::new(),
                        error_message: Some(format!("PGN export failed: {}", e)),
                    },
                );
                false
            }
        }
    }

    /// Test PGN standards compliance
    fn test_pgn_standards_compliance(&mut self) -> bool {
        let test_name = "pgn_standards_compliance";

        let checker = PgnStandardsChecker::new();

        // Test header validation
        let valid_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            ("Date", "2023.12.25"),
            ("Round", "1"),
            ("White", "Player White"),
            ("Black", "Player Black"),
            ("Result", "1-0"),
        ];

        let header_validation_result = checker.validate_headers(&valid_headers).is_ok();

        // Test line length checking
        let test_content = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1";
        let line_length_result = checker.check_line_lengths(test_content).is_ok();

        let passed = header_validation_result && line_length_result;

        self.test_results.insert(
            test_name.to_string(),
            RegressionTestResult {
                test_name: test_name.to_string(),
                passed,
                expected_result: "Headers valid, line lengths OK".to_string(),
                actual_result: format!(
                    "Headers: {}, Lines: {}",
                    header_validation_result, line_length_result
                ),
                error_message: if !passed {
                    Some("PGN standards validation failed".to_string())
                } else {
                    None
                },
            },
        );

        passed
    }

    /// Test date formatting by checking PGN output
    fn test_date_formatting(&mut self) -> bool {
        let test_name = "date_formatting_standard";

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index();
        let name_db = create_test_name_database();
        let moves = "1.e4 e5";

        match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
            Ok(pgn_output) => {
                let has_correct_date = pgn_output.contains("[Date \"2023.12.25\"]");
                let expected = self.known_good_outputs.get(test_name).unwrap();
                let passed = has_correct_date;

                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed,
                        expected_result: expected.clone(),
                        actual_result: if has_correct_date {
                            "2023.12.25".to_string()
                        } else {
                            "Date not found".to_string()
                        },
                        error_message: if !passed {
                            Some("Date formatting doesn't match expected format".to_string())
                        } else {
                            None
                        },
                    },
                );

                passed
            }
            Err(e) => {
                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed: false,
                        expected_result: self.known_good_outputs.get(test_name).unwrap().clone(),
                        actual_result: String::new(),
                        error_message: Some(format!("Export failed: {}", e)),
                    },
                );
                false
            }
        }
    }

    /// Test ECO formatting by checking PGN output
    fn test_eco_formatting(&mut self) -> bool {
        let test_name = "eco_formatting_b13";

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index(); // Has ECO 113 = B13
        let name_db = create_test_name_database();
        let moves = "1.e4 e5";

        match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
            Ok(pgn_output) => {
                let has_correct_eco = pgn_output.contains("[ECO \"B13\"]");
                let expected = self.known_good_outputs.get(test_name).unwrap();
                let passed = has_correct_eco;

                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed,
                        expected_result: expected.clone(),
                        actual_result: if has_correct_eco {
                            "B13".to_string()
                        } else {
                            "ECO not found".to_string()
                        },
                        error_message: if !passed {
                            Some("ECO formatting doesn't match expected format".to_string())
                        } else {
                            None
                        },
                    },
                );

                passed
            }
            Err(e) => {
                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed: false,
                        expected_result: self.known_good_outputs.get(test_name).unwrap().clone(),
                        actual_result: String::new(),
                        error_message: Some(format!("Export failed: {}", e)),
                    },
                );
                false
            }
        }
    }

    /// Test result formatting by checking PGN output
    fn test_result_formatting(&mut self) -> bool {
        let test_name = "result_formatting_white_wins";

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let game_index = create_test_game_index(); // Has result 1 = 1-0
        let name_db = create_test_name_database();
        let moves = "1.e4 e5";

        match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
            Ok(pgn_output) => {
                let has_correct_result =
                    pgn_output.contains("[Result \"1-0\"]") && pgn_output.ends_with("1-0\n");
                let expected = self.known_good_outputs.get(test_name).unwrap();
                let passed = has_correct_result;

                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed,
                        expected_result: expected.clone(),
                        actual_result: if has_correct_result {
                            "1-0".to_string()
                        } else {
                            "Result not found".to_string()
                        },
                        error_message: if !passed {
                            Some("Result formatting doesn't match expected format".to_string())
                        } else {
                            None
                        },
                    },
                );

                passed
            }
            Err(e) => {
                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed: false,
                        expected_result: self.known_good_outputs.get(test_name).unwrap().clone(),
                        actual_result: String::new(),
                        error_message: Some(format!("Export failed: {}", e)),
                    },
                );
                false
            }
        }
    }

    /// Test line length formatting
    fn test_line_length_formatting(&mut self) -> bool {
        let test_name = "pgn_standards_line_formatting";

        let checker = PgnStandardsChecker::with_max_line_length(70);
        let long_input =
            "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5 7.Bb3 d6 8.c3 O-O 9.h3 Nb8";
        let formatted = checker.format_pgn(long_input);

        // Check that lines don't exceed the limit
        let lines_ok = formatted.lines().all(|line| line.len() <= 70);
        let has_line_breaks = formatted.contains('\n');

        let passed = lines_ok && has_line_breaks;

        self.test_results.insert(
            test_name.to_string(),
            RegressionTestResult {
                test_name: test_name.to_string(),
                passed,
                expected_result: "Lines <= 70 chars with line breaks".to_string(),
                actual_result: format!(
                    "Lines OK: {}, Has breaks: {}, Output: {}",
                    lines_ok, has_line_breaks, formatted
                ),
                error_message: if !passed {
                    Some("Line length formatting failed".to_string())
                } else {
                    None
                },
            },
        );

        passed
    }

    /// Test header validation
    fn test_header_validation(&mut self) -> bool {
        let test_name = "header_validation";

        let checker = PgnStandardsChecker::new();

        // Test valid headers
        let valid_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            ("Date", "2023.12.25"),
            ("Round", "1"),
            ("White", "Player White"),
            ("Black", "Player Black"),
            ("Result", "1-0"),
        ];

        let valid_result = checker.validate_headers(&valid_headers).is_ok();

        // Test invalid headers (missing required)
        let invalid_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            // Missing Date, Round, White, Black, Result
        ];

        let invalid_result = checker.validate_headers(&invalid_headers).is_err();

        let passed = valid_result && invalid_result;

        self.test_results.insert(
            test_name.to_string(),
            RegressionTestResult {
                test_name: test_name.to_string(),
                passed,
                expected_result: "Valid headers pass, invalid headers fail".to_string(),
                actual_result: format!(
                    "Valid: {}, Invalid rejected: {}",
                    valid_result, invalid_result
                ),
                error_message: if !passed {
                    Some("Header validation logic incorrect".to_string())
                } else {
                    None
                },
            },
        );

        passed
    }

    /// Test special character handling
    fn test_special_character_handling(&mut self) -> bool {
        let test_name = "special_character_handling";

        let exporter = EnhancedPgnExporter::new(ExportOptions::default());
        let mut game_index = create_test_game_index();
        let mut name_db = create_test_name_database();

        // Add special characters
        name_db.player_names.insert(1, "José García".to_string());
        name_db
            .player_names
            .insert(2, "François Müller".to_string());
        name_db
            .event_names
            .insert(1, "World Championship \"2023\"".to_string());

        let moves = "1.e4 e5";

        match exporter.export_complete_game(&game_index, moves, Some(&name_db)) {
            Ok(pgn_output) => {
                let has_jose = pgn_output.contains("José García");
                let has_francois = pgn_output.contains("François Müller");
                let has_quotes = pgn_output.contains("World Championship \"2023\"");

                let passed = has_jose && has_francois && has_quotes;

                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed,
                        expected_result: "Special characters preserved in output".to_string(),
                        actual_result: format!(
                            "José: {}, François: {}, Quotes: {}",
                            has_jose, has_francois, has_quotes
                        ),
                        error_message: if !passed {
                            Some("Special characters not properly handled".to_string())
                        } else {
                            None
                        },
                    },
                );

                passed
            }
            Err(e) => {
                self.test_results.insert(
                    test_name.to_string(),
                    RegressionTestResult {
                        test_name: test_name.to_string(),
                        passed: false,
                        expected_result: "Special characters preserved".to_string(),
                        actual_result: String::new(),
                        error_message: Some(format!("Export failed: {}", e)),
                    },
                );
                false
            }
        }
    }

    /// Get test results
    pub fn get_test_results(&self) -> &HashMap<String, RegressionTestResult> {
        &self.test_results
    }

    /// Print comprehensive test report
    pub fn print_regression_report(&self) {
        println!("=== REGRESSION TEST REPORT ===");
        println!();

        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.values().filter(|r| r.passed).count();
        let failed_tests = total_tests - passed_tests;

        println!("SUMMARY:");
        println!("  Total Tests: {}", total_tests);
        println!(
            "  Passed: {} ({:.1}%)",
            passed_tests,
            (passed_tests as f64 / total_tests as f64) * 100.0
        );
        println!(
            "  Failed: {} ({:.1}%)",
            failed_tests,
            (failed_tests as f64 / total_tests as f64) * 100.0
        );
        println!();

        if failed_tests > 0 {
            println!("FAILED TESTS:");
            for (test_name, result) in &self.test_results {
                if !result.passed {
                    println!(
                        "  ❌ {}: {}",
                        test_name,
                        result
                            .error_message
                            .as_ref()
                            .unwrap_or(&"Unknown error".to_string())
                    );
                    println!("    Expected: {}", result.expected_result);
                    println!("    Actual:   {}", result.actual_result);
                    println!();
                }
            }
        }

        println!("PASSED TESTS:");
        for (test_name, result) in &self.test_results {
            if result.passed {
                println!("  ✅ {}", test_name);
            }
        }

        println!();
        println!("REGRESSION TEST VALIDATION:");
        let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
        if success_rate >= 95.0 {
            println!("  Overall Success Rate: {:.1}% - ✅ PASS", success_rate);
        } else {
            println!(
                "  Overall Success Rate: {:.1}% - ❌ FAIL (minimum 95%)",
                success_rate
            );
        }
    }
}

/// Helper function to create test game index
fn create_test_game_index() -> scidtopgn::pgn::exporter::SimpleGameIndex {
    scidtopgn::pgn::exporter::SimpleGameIndex {
        event_id: 1,
        site_id: 1,
        white_id: 1,
        black_id: 2,
        round_id: 1,
        year: 2023,
        month: 12,
        day: 25,
        result: 1, // 1-0
        white_elo: 1800,
        black_elo: 1750,
        eco: 113, // B13
        num_half_moves: 40,
    }
}

/// Helper function to create test name database
fn create_test_name_database() -> scidtopgn::pgn::exporter::SimpleNameDatabase {
    let mut name_db = scidtopgn::pgn::exporter::SimpleNameDatabase::new();
    name_db.event_names.insert(1, "Test Tournament".to_string());
    name_db.site_names.insert(1, "Test City".to_string());
    name_db.player_names.insert(1, "Player White".to_string());
    name_db.player_names.insert(2, "Player Black".to_string());
    name_db.round_names.insert(1, "1".to_string());
    name_db
}

/// Normalize PGN for comparison by removing extra whitespace
fn normalize_pgn_for_comparison(pgn: &str) -> String {
    pgn.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_framework_creation() {
        let framework = RegressionTestFramework::new();
        assert_eq!(framework.test_results.len(), 0);
        assert_eq!(framework.known_good_outputs.len(), 0);
    }

    #[test]
    fn test_known_good_outputs_initialization() {
        let mut framework = RegressionTestFramework::new();
        framework.initialize_known_good_outputs();

        assert!(!framework.known_good_outputs.is_empty());
        assert!(framework
            .known_good_outputs
            .contains_key("pgn_export_standard_headers"));
        assert!(framework
            .known_good_outputs
            .contains_key("date_formatting_standard"));
        assert!(framework
            .known_good_outputs
            .contains_key("eco_formatting_b13"));
    }

    #[test]
    fn test_run_all_regression_tests() {
        let mut framework = RegressionTestFramework::new();
        framework.initialize_known_good_outputs();

        let all_passed = framework.run_all_tests();

        // Print report for debugging
        framework.print_regression_report();

        // All regression tests should pass
        assert!(all_passed, "All regression tests should pass");

        // Verify we ran multiple tests
        assert!(
            framework.test_results.len() >= 5,
            "Should run multiple regression tests"
        );

        // Verify all tests passed
        let failed_tests: Vec<_> = framework
            .test_results
            .values()
            .filter(|r| !r.passed)
            .collect();

        if !failed_tests.is_empty() {
            for test in failed_tests {
                println!(
                    "Failed test: {} - {}",
                    test.test_name,
                    test.error_message
                        .as_ref()
                        .unwrap_or(&"Unknown".to_string())
                );
                println!("Expected: {}", test.expected_result);
                println!("Actual: {}", test.actual_result);
            }
        }
    }

    #[test]
    fn test_normalize_pgn_for_comparison() {
        let input = r#"  [Event "Test"]  
        
[Site "City"]  
    
1.e4 e5  "#;

        let expected = "[Event \"Test\"]\n[Site \"City\"]\n1.e4 e5";
        let actual = normalize_pgn_for_comparison(input);

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_helper_functions() {
        let game_index = create_test_game_index();
        assert_eq!(game_index.year, 2023);
        assert_eq!(game_index.month, 12);
        assert_eq!(game_index.day, 25);
        assert_eq!(game_index.result, 1);

        let name_db = create_test_name_database();
        assert!(name_db.player_names.contains_key(&1));
        assert!(name_db.event_names.contains_key(&1));
        assert!(name_db.site_names.contains_key(&1));
    }
}
