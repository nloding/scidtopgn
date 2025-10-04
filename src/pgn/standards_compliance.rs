/// Ensures PGN output complies with official PGN standards
/// Validates and formats PGN content according to official PGN standards
#[allow(dead_code)]
pub struct PgnStandardsChecker {
    /// Line length limit (PGN standard recommends 80 characters)
    max_line_length: usize,
}

#[allow(dead_code)]
impl PgnStandardsChecker {
    pub fn new() -> Self {
        Self {
            max_line_length: 80,
        }
    }

    /// Create with custom line length limit
    pub fn with_max_line_length(max_length: usize) -> Self {
        Self {
            max_line_length: max_length,
        }
    }

    /// Format PGN with proper line breaks and spacing
    pub fn format_pgn(&self, pgn_content: &str) -> String {
        let mut result = String::new();
        let mut current_line_length = 0;

        // Split into tokens (moves, comments, variations)
        let tokens = self.tokenize_pgn(pgn_content);

        for token in tokens {
            // Check if adding this token would exceed line length
            if current_line_length + token.len() + 1 > self.max_line_length
                && current_line_length > 0
            {
                result.push('\n');
                current_line_length = 0;
            }

            // Add token with appropriate spacing
            if current_line_length > 0 {
                result.push(' ');
                current_line_length += 1;
            }

            result.push_str(&token);
            current_line_length += token.len();
        }

        result
    }

    /// Tokenize PGN content for formatting
    fn tokenize_pgn(&self, content: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_comment = false;
        let mut in_variation = 0;

        for ch in content.chars() {
            match ch {
                '{' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.trim().to_string());
                        current_token.clear();
                    }
                    current_token.push(ch);
                    in_comment = true;
                }
                '}' => {
                    current_token.push(ch);
                    if in_comment {
                        tokens.push(current_token.trim().to_string());
                        current_token.clear();
                        in_comment = false;
                    }
                }
                '(' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.trim().to_string());
                        current_token.clear();
                    }
                    current_token.push(ch);
                    in_variation += 1;
                }
                ')' => {
                    current_token.push(ch);
                    in_variation -= 1;
                    if in_variation == 0 {
                        tokens.push(current_token.trim().to_string());
                        current_token.clear();
                    }
                }
                ' ' | '\t' | '\n' => {
                    if in_comment || in_variation > 0 {
                        current_token.push(ch);
                    } else if !current_token.is_empty() {
                        tokens.push(current_token.trim().to_string());
                        current_token.clear();
                    }
                }
                _ => {
                    current_token.push(ch);
                }
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token.trim().to_string());
        }

        // Filter out empty tokens
        tokens
            .into_iter()
            .filter(|t| !t.trim().is_empty())
            .collect()
    }

    /// Validate PGN headers
    pub fn validate_headers(&self, headers: &[(&str, &str)]) -> Result<(), String> {
        // Check for required headers
        let required_headers = ["Event", "Site", "Date", "Round", "White", "Black", "Result"];

        for required in &required_headers {
            if !headers.iter().any(|(key, _)| key == required) {
                return Err(format!("Missing required header: {}", required));
            }
        }

        // Validate header formats
        for (key, value) in headers {
            match *key {
                "Date" => self.validate_date_format(value)?,
                "Result" => self.validate_result_format(value)?,
                "Round" => self.validate_round_format(value)?,
                _ => {} // Other headers are flexible
            }
        }

        Ok(())
    }

    fn validate_date_format(&self, date: &str) -> Result<(), String> {
        // PGN date format: YYYY.MM.DD or YYYY.MM.?? or YYYY.??.??
        // Simple validation without regex dependency
        let parts: Vec<&str> = date.split('.').collect();

        if parts.len() != 3 {
            return Err(format!(
                "Invalid date format: {} (expected YYYY.MM.DD)",
                date
            ));
        }

        // Check year format (4 digits)
        if parts[0].len() != 4 || !parts[0].chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("Invalid year format: {}", parts[0]));
        }

        // Check month format (2 digits or ??)
        if parts[1].len() != 2
            || (!parts[1].chars().all(|c| c.is_ascii_digit()) && parts[1] != "??")
        {
            return Err(format!("Invalid month format: {}", parts[1]));
        }

        // Check day format (2 digits or ??)
        if parts[2].len() != 2
            || (!parts[2].chars().all(|c| c.is_ascii_digit()) && parts[2] != "??")
        {
            return Err(format!("Invalid day format: {}", parts[2]));
        }

        // Validate month and day ranges if not unknown
        if parts[1] != "??" {
            let month: u8 = parts[1]
                .parse()
                .map_err(|_| format!("Invalid month: {}", parts[1]))?;
            if !(1..=12).contains(&month) {
                return Err(format!("Month out of range: {}", month));
            }
        }

        if parts[2] != "??" {
            let day: u8 = parts[2]
                .parse()
                .map_err(|_| format!("Invalid day: {}", parts[2]))?;
            if !(1..=31).contains(&day) {
                return Err(format!("Day out of range: {}", day));
            }
        }

        Ok(())
    }

    fn validate_result_format(&self, result: &str) -> Result<(), String> {
        if !matches!(result, "1-0" | "0-1" | "1/2-1/2" | "*") {
            return Err(format!("Invalid result format: {}", result));
        }
        Ok(())
    }

    fn validate_round_format(&self, round: &str) -> Result<(), String> {
        // Round can be a number, "?" for unknown, or "-" for not applicable
        if round.is_empty() {
            return Err("Round cannot be empty".to_string());
        }

        // Allow common round formats: numbers, "?", "-"
        if round == "?" || round == "-" {
            return Ok(());
        }

        // Check if it's a valid number
        if round.chars().all(|c| c.is_ascii_digit()) {
            return Ok(());
        }

        // Allow decimal rounds like "1.1"
        let parts: Vec<&str> = round.split('.').collect();
        if parts.len() == 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit())) {
            return Ok(());
        }

        Err(format!("Invalid round format: {}", round))
    }

    /// Check if content follows PGN line length guidelines
    pub fn check_line_lengths(&self, content: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        for (line_num, line) in content.lines().enumerate() {
            // Skip header lines and empty lines
            if line.starts_with('[') || line.trim().is_empty() {
                continue;
            }

            if line.len() > self.max_line_length {
                let preview = if line.len() > 50 {
                    format!("{}...", &line[..50])
                } else {
                    line.to_string()
                };

                errors.push(format!(
                    "Line {} exceeds maximum length ({} > {}): {}",
                    line_num + 1,
                    line.len(),
                    self.max_line_length,
                    preview
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate complete PGN format
    pub fn validate_complete_pgn(&self, pgn_content: &str) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check for header section
        let has_headers = pgn_content.lines().any(|line| line.starts_with('['));
        if !has_headers {
            errors.push("No PGN headers found".to_string());
        }

        // Check for moves section
        let has_moves = pgn_content.lines().any(|line| {
            !line.starts_with('[')
                && !line.trim().is_empty()
                && (line.contains("1.") || line.contains("e4") || line.contains("d4"))
        });
        if !has_moves {
            errors.push("No moves section found".to_string());
        }

        // Check line lengths
        if let Err(length_errors) = self.check_line_lengths(pgn_content) {
            errors.extend(length_errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for PgnStandardsChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pgn_tokenization() {
        let checker = PgnStandardsChecker::new();

        let input = "1.e4 e5 2.Nf3 {A good move} Nc6 (2...Nf6 3.Bb5) 3.Bb5";
        let tokens = checker.tokenize_pgn(input);

        assert!(tokens.contains(&"1.e4".to_string()));
        assert!(tokens.contains(&"e5".to_string()));
        assert!(tokens.contains(&"{A good move}".to_string()));
        assert!(tokens.contains(&"(2...Nf6 3.Bb5)".to_string()));
    }

    #[test]
    fn test_pgn_formatting() {
        let checker = PgnStandardsChecker::with_max_line_length(20);

        let input = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7";
        let formatted = checker.format_pgn(input);

        // Should break lines when they get too long
        assert!(formatted.contains('\n'));

        for line in formatted.lines() {
            assert!(line.len() <= 20, "Line too long: '{}'", line);
        }
    }

    #[test]
    fn test_header_validation() {
        let checker = PgnStandardsChecker::new();

        // Valid headers
        let valid_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            ("Date", "2023.12.25"),
            ("Round", "1"),
            ("White", "Player A"),
            ("Black", "Player B"),
            ("Result", "1-0"),
        ];

        assert!(checker.validate_headers(&valid_headers).is_ok());

        // Missing required header
        let invalid_headers = vec![
            ("Event", "Test Tournament"),
            ("Site", "Test City"),
            // Missing Date
            ("Round", "1"),
            ("White", "Player A"),
            ("Black", "Player B"),
            ("Result", "1-0"),
        ];

        assert!(checker.validate_headers(&invalid_headers).is_err());
    }

    #[test]
    fn test_date_validation() {
        let checker = PgnStandardsChecker::new();

        // Valid dates
        assert!(checker.validate_date_format("2023.12.25").is_ok());
        assert!(checker.validate_date_format("2023.12.??").is_ok());
        assert!(checker.validate_date_format("2023.??.??").is_ok());

        // Invalid dates
        assert!(checker.validate_date_format("23.12.25").is_err()); // Wrong year format
        assert!(checker.validate_date_format("2023.13.25").is_err()); // Invalid month
        assert!(checker.validate_date_format("2023.12.32").is_err()); // Invalid day
        assert!(checker.validate_date_format("2023-12-25").is_err()); // Wrong separator
    }

    #[test]
    fn test_result_validation() {
        let checker = PgnStandardsChecker::new();

        // Valid results
        assert!(checker.validate_result_format("1-0").is_ok());
        assert!(checker.validate_result_format("0-1").is_ok());
        assert!(checker.validate_result_format("1/2-1/2").is_ok());
        assert!(checker.validate_result_format("*").is_ok());

        // Invalid results
        assert!(checker.validate_result_format("1:0").is_err());
        assert!(checker.validate_result_format("draw").is_err());
        assert!(checker.validate_result_format("").is_err());
    }

    #[test]
    fn test_round_validation() {
        let checker = PgnStandardsChecker::new();

        // Valid rounds
        assert!(checker.validate_round_format("1").is_ok());
        assert!(checker.validate_round_format("10").is_ok());
        assert!(checker.validate_round_format("1.1").is_ok());
        assert!(checker.validate_round_format("?").is_ok());
        assert!(checker.validate_round_format("-").is_ok());

        // Invalid rounds
        assert!(checker.validate_round_format("").is_err());
        assert!(checker.validate_round_format("Round 1").is_err());
    }

    #[test]
    fn test_line_length_checking() {
        let checker = PgnStandardsChecker::with_max_line_length(20);

        let short_content = "1.e4 e5\n2.Nf3 Nc6\n";
        assert!(checker.check_line_lengths(short_content).is_ok());

        let long_content = "1.e4 e5 2.Nf3 Nc6 3.Bb5 a6 4.Ba4 Nf6 5.O-O Be7 6.Re1 b5\n";
        assert!(checker.check_line_lengths(long_content).is_err());
    }

    #[test]
    fn test_complete_pgn_validation() {
        let checker = PgnStandardsChecker::new();

        let valid_pgn = r#"[Event "Test"]
[Site "Test City"]
[Date "2023.12.25"]
[Round "1"]
[White "Player A"]
[Black "Player B"]
[Result "1-0"]

1.e4 e5 2.Nf3 Nc6 3.Bb5 1-0
"#;

        assert!(checker.validate_complete_pgn(valid_pgn).is_ok());

        let invalid_pgn = "1.e4 e5 2.Nf3"; // No headers
        assert!(checker.validate_complete_pgn(invalid_pgn).is_err());
    }

    #[test]
    fn test_complex_variation_tokenization() {
        let checker = PgnStandardsChecker::new();

        let complex_input =
            "1.e4 e5 2.Nf3 Nc6 (2...Nf6 3.Nxe5 d6 4.Nf3 Nxe4) 3.Bb5 {Spanish Opening} a6 4.Ba4";
        let tokens = checker.tokenize_pgn(complex_input);

        // Should properly handle nested structures
        assert!(tokens.contains(&"1.e4".to_string()));
        assert!(tokens.contains(&"(2...Nf6 3.Nxe5 d6 4.Nf3 Nxe4)".to_string()));
        assert!(tokens.contains(&"{Spanish Opening}".to_string()));
        assert!(tokens.contains(&"4.Ba4".to_string()));
    }
}
