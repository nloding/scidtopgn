use crate::bridge::GameState;
use crate::core::error::{Result, ScidError};
use std::collections::HashMap;

/// PGN header structure with validation and formatting support
#[derive(Debug, Clone, PartialEq)]
pub struct PgnHeader {
    /// Header fields stored as key-value pairs
    fields: HashMap<String, String>,
    /// Whether the headers have been validated
    validated: bool,
}

impl PgnHeader {
    /// Create a new empty PgnHeader
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            validated: false,
        }
    }
    
    /// Create PgnHeader from game state metadata
    pub fn from_game_state(game_state: &GameState) -> Result<Self> {
        let mut fields = HashMap::new();
        
        if let Some(metadata) = game_state.metadata() {
            // Seven Tag Roster (mandatory headers)
            fields.insert("Event".to_string(), metadata.event.clone());
            fields.insert("Site".to_string(), metadata.site.clone());
            fields.insert("Date".to_string(), format_pgn_date(&metadata.date));
            fields.insert(
                "Round".to_string(),
                metadata.round.clone().unwrap_or("?".to_string()),
            );
            fields.insert("White".to_string(), metadata.white.clone());
            fields.insert("Black".to_string(), metadata.black.clone());
            fields.insert("Result".to_string(), metadata.result.clone());
            
            // Optional headers
            if let Some(white_elo) = metadata.white_elo {
                fields.insert("WhiteElo".to_string(), white_elo.to_string());
            }
            if let Some(black_elo) = metadata.black_elo {
                fields.insert("BlackElo".to_string(), black_elo.to_string());
            }
            if let Some(ref eco) = metadata.eco {
                fields.insert("ECO".to_string(), eco.clone());
            }
        } else {
            // Add default headers if no metadata is present
            fields.insert("Event".to_string(), "?".to_string());
            fields.insert("Site".to_string(), "?".to_string());
            fields.insert("Date".to_string(), "????.??.??".to_string());
            fields.insert("Round".to_string(), "?".to_string());
            fields.insert("White".to_string(), "?".to_string());
            fields.insert("Black".to_string(), "?".to_string());
            fields.insert("Result".to_string(), "*".to_string());
        }
        
        Ok(Self { fields, validated: false })
    }
    
    /// Create PgnHeader from a hash map of fields
    pub fn from_fields(fields: HashMap<String, String>) -> Result<Self> {
        let mut header = Self {
            fields,
            validated: false,
        };
        
        // Validate the headers
        header.validate()?;
        header.validated = true;
        
        Ok(header)
    }
    
    /// Add a custom header
    pub fn add_header(&mut self, key: String, value: String) -> Result<()> {
        // Validate the header key and value
        self.validate_header_key(&key)?;
        self.validate_header_value(&value)?;
        
        self.fields.insert(key, value);
        self.validated = false; // Mark as unvalidated after modification
        Ok(())
    }
    
    /// Get a header value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.fields.get(key)
    }
    
    /// Get a header value by key, returning a default if not present
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.fields.get(key).cloned().unwrap_or_else(|| default.to_string())
    }
    
    /// Check if a header exists
    pub fn contains(&self, key: &str) -> bool {
        self.fields.contains_key(key)
    }
    
    /// Remove a header
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.fields.remove(key)
    }
    
    /// Get all header fields as a reference
    pub fn fields(&self) -> &HashMap<String, String> {
        &self.fields
    }
    
    /// Get all header fields as a mutable reference
    pub fn fields_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.fields
    }
    
    /// Validate all headers according to PGN standard
    pub fn validate(&self) -> Result<()> {
        // Check for Seven Tag Roster (mandatory headers)
        let mandatory_headers = ["Event", "Site", "Date", "Round", "White", "Black", "Result"];
        
        for header in &mandatory_headers {
            if !self.fields.contains_key(*header) {
                return Err(ScidError::invalid_format(
                    format!("Missing mandatory PGN header: {}", header)
                ));
            }
        }
        
        // Validate all header keys and values
        for (key, value) in &self.fields {
            self.validate_header_key(key)?;
            self.validate_header_value(value)?;
        }
        
        // Validate specific header formats
        self.validate_result_header()?;
        self.validate_date_header()?;
        self.validate_elo_headers()?;
        
        Ok(())
    }
    
    /// Validate a header key according to PGN standard
    fn validate_header_key(&self, key: &str) -> Result<()> {
        // Header keys must start with uppercase letter
        if key.is_empty() {
            return Err(ScidError::invalid_format("Header key cannot be empty".to_string()));
        }
        
        if !key.chars().next().map_or(false, |c| c.is_uppercase()) {
            return Err(ScidError::invalid_format(
                format!("Header key must start with uppercase letter: {}", key)
            ));
        }
        
        // Header keys should only contain letters, numbers, and underscores
        if !key.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(ScidError::invalid_format(
                format!("Header key contains invalid characters: {}", key)
            ));
        }
        
        Ok(())
    }
    
    /// Validate a header value according to PGN standard
    fn validate_header_value(&self, value: &str) -> Result<()> {
        // Header values must be properly escaped
        if value.contains('"') && !value.starts_with('\\') {
            return Err(ScidError::invalid_format(
                format!("Header value contains unescaped quotes: {}", value)
            ));
        }
        
        // Check for balanced quotes if present
        if value.starts_with('"') && value.ends_with('"') {
            let content = &value[1..value.len() - 1];
            if content.contains('"') && !content.ends_with("\\\"") {
                return Err(ScidError::invalid_format(
                    format!("Header value contains unescaped quotes: {}", value)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate result header format
    fn validate_result_header(&self) -> Result<()> {
        if let Some(result) = self.fields.get("Result") {
            match result.as_str() {
                "1-0" | "0-1" | "1/2-1/2" | "*" => Ok(()),
                _ => Err(ScidError::invalid_format(
                    format!("Invalid result format: {}. Must be 1-0, 0-1, 1/2-1/2, or *", result)
                )),
            }
        } else {
            Ok(()) // Result header is optional for validation
        }
    }
    
    /// Validate date header format
    fn validate_date_header(&self) -> Result<()> {
        if let Some(date) = self.fields.get("Date") {
            if date == "????.??.??" {
                return Ok(()); // Placeholder date is acceptable
            }
            
            // Check YYYY.MM.DD format
            let parts: Vec<&str> = date.split('.').collect();
            if parts.len() != 3 {
                return Err(ScidError::invalid_format(
                    format!("Invalid date format: {}. Expected YYYY.MM.DD", date)
                ));
            }
            
            // Validate year, month, day
            if parts[0].len() != 4 || parts[1].len() != 2 || parts[2].len() != 2 {
                return Err(ScidError::invalid_format(
                    format!("Invalid date format: {}. Expected YYYY.MM.DD", date)
                ));
            }
            
            // Try to parse as numbers
            let year = parts[0].parse::<u32>().map_err(|_| {
                ScidError::invalid_format(format!("Invalid year in date: {}", date))
            })?;
            
            let month = parts[1].parse::<u32>().map_err(|_| {
                ScidError::invalid_format(format!("Invalid month in date: {}", date))
            })?;
            
            let day = parts[2].parse::<u32>().map_err(|_| {
                ScidError::invalid_format(format!("Invalid day in date: {}", date))
            })?;
            
            // Validate ranges
            if year < 1000 || year > 9999 {
                return Err(ScidError::invalid_format(
                    format!("Year out of range: {}", year)
                ));
            }
            
            if month < 1 || month > 12 {
                return Err(ScidError::invalid_format(
                    format!("Month out of range: {}", month)
                ));
            }
            
            if day < 1 || day > 31 {
                return Err(ScidError::invalid_format(
                    format!("Day out of range: {}", day)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Validate ELO header formats
    fn validate_elo_headers(&self) -> Result<()> {
        if let Some(white_elo) = self.fields.get("WhiteElo") {
            self.validate_elo_value(white_elo)?;
        }
        
        if let Some(black_elo) = self.fields.get("BlackElo") {
            self.validate_elo_value(black_elo)?;
        }
        
        Ok(())
    }
    
    /// Validate ELO value format
    fn validate_elo_value(&self, elo: &str) -> Result<()> {
        let elo_num = elo.parse::<u16>().map_err(|_| {
            ScidError::invalid_format(format!("Invalid ELO value: {}", elo))
        })?;
        
        if elo_num > 4000 {
            return Err(ScidError::invalid_format(
                format!("ELO value out of range: {}", elo_num)
            ));
        }
        
        Ok(())
    }
    
    /// Format headers for PGN output
    pub fn format_headers(&self) -> String {
        let mut output = String::new();
        
        // Sort headers: Seven Tag Roster first, then alphabetical
        let seven_tag_roster = ["Event", "Site", "Date", "Round", "White", "Black", "Result"];
        
        // Add Seven Tag Roster headers in order
        for header in &seven_tag_roster {
            if let Some(value) = self.fields.get(*header) {
                output.push_str(&format!("[{} \"{}\"]\n", header, value));
            }
        }
        
        // Add remaining headers in alphabetical order
        let mut remaining_headers: Vec<_> = self.fields
            .iter()
            .filter(|(k, _)| !seven_tag_roster.contains(&k.as_str()))
            .collect();
        
        remaining_headers.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
        
        for (key, value) in remaining_headers {
            output.push_str(&format!("[{} \"{}\"]\n", key, value));
        }
        
        output
    }
    
    /// Check if headers are validated
    pub fn is_validated(&self) -> bool {
        self.validated
    }
    
    /// Mark headers as validated
    pub fn mark_validated(&mut self) {
        self.validated = true;
    }
}

/// Format date for PGN output (YYYY.MM.DD format)
fn format_pgn_date(date: &str) -> String {
    // If date is already in YYYY.MM.DD format, return as-is
    if date.chars().filter(|&c| c == '.').count() == 2 && date.len() == 10 {
        if let Ok(_) = date.parse::<u32>() {
            return date.to_string();
        }
    }
    
    // Otherwise, try to parse and reformat
    // This is a simplified implementation - in practice, you'd use a proper date parsing library
    date.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::GameState;
    use std::collections::HashMap;

    #[test]
    fn test_pgn_header_creation() {
        let header = PgnHeader::new();
        assert!(header.fields.is_empty());
        assert!(!header.is_validated());
    }

    #[test]
    fn test_from_game_state() {
        let mut game_state = GameState::new();
        game_state.set_metadata(GameMetadata {
            white: "WhitePlayer".to_string(),
            black: "BlackPlayer".to_string(),
            event: "Test Event".to_string(),
            site: "Test Site".to_string(),
            date: "2025.08.29".to_string(),
            result: "1-0".to_string(),
            white_elo: Some(1500),
            black_elo: Some(1600),
            round: Some("1".to_string()),
            eco: Some("A00".to_string()),
        });
        
        let header = PgnHeader::from_game_state(&game_state).unwrap();
        assert_eq!(header.get("Event"), Some(&"Test Event".to_string()));
        assert_eq!(header.get("Result"), Some(&"1-0".to_string()));
        assert_eq!(header.get("WhiteElo"), Some(&"1500".to_string()));
    }

    #[test]
    fn test_add_custom_header() {
        let mut header = PgnHeader::new();
        header.add_header("Annotator".to_string(), "Test Annotator".to_string()).unwrap();
        assert_eq!(header.get("Annotator"), Some(&"Test Annotator".to_string()));
    }

    #[test]
    fn test_header_validation() {
        let mut header = PgnHeader::new();
        
        // Should fail validation - missing mandatory headers
        assert!(header.validate().is_err());
        
        // Add mandatory headers
        header.add_header("Event".to_string(), "Test".to_string()).unwrap();
        header.add_header("Site".to_string(), "Test".to_string()).unwrap();
        header.add_header("Date".to_string(), "2025.08.29".to_string()).unwrap();
        header.add_header("Round".to_string(), "1".to_string()).unwrap();
        header.add_header("White".to_string(), "Player".to_string()).unwrap();
        header.add_header("Black".to_string(), "Player".to_string()).unwrap();
        header.add_header("Result".to_string(), "1-0".to_string()).unwrap();
        
        // Should now pass validation
        assert!(header.validate().is_ok());
        assert!(header.is_validated());
    }

    #[test]
    fn test_invalid_header_key() {
        let mut header = PgnHeader::new();
        
        // Empty key should fail
        assert!(header.add_header("".to_string(), "value".to_string()).is_err());
        
        // Key starting with lowercase should fail
        assert!(header.add_header("event".to_string(), "value".to_string()).is_err());
        
        // Key with special characters should fail
        assert!(header.add_header("Event-Name".to_string(), "value".to_string()).is_err());
    }

    #[test]
    fn test_invalid_result() {
        let mut header = PgnHeader::new();
        header.add_header("Result".to_string(), "invalid".to_string()).unwrap();
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_invalid_date() {
        let mut header = PgnHeader::new();
        header.add_header("Date".to_string(), "invalid".to_string()).unwrap();
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_invalid_elo() {
        let mut header = PgnHeader::new();
        header.add_header("WhiteElo".to_string(), "9999".to_string()).unwrap();
        assert!(header.validate().is_err());
    }

    #[test]
    fn test_format_headers() {
        let mut header = PgnHeader::new();
        
        // Add headers in non-alphabetical order
        header.add_header("Result".to_string(), "1-0".to_string()).unwrap();
        header.add_header("Event".to_string(), "Test Event".to_string()).unwrap();
        header.add_header("Site".to_string(), "Test Site".to_string()).unwrap();
        header.add_header("Date".to_string(), "2025.08.29".to_string()).unwrap();
        header.add_header("Round".to_string(), "1".to_string()).unwrap();
        header.add_header("White".to_string(), "WhitePlayer".to_string()).unwrap();
        header.add_header("Black".to_string(), "BlackPlayer".to_string()).unwrap();
        header.add_header("WhiteElo".to_string(), "1500".to_string()).unwrap();
        header.add_header("ECO".to_string(), "A00".to_string()).unwrap();
        
        let formatted = header.format_headers();
        
        // Seven Tag Roster should come first in order
        assert!(formatted.starts_with("[Event \"Test Event\"]"));
        assert!(formatted.contains("[Site \"Test Site\"]"));
        assert!(formatted.contains("[Date \"2025.08.29\"]"));
        assert!(formatted.contains("[Round \"1\"]"));
        assert!(formatted.contains("[White \"WhitePlayer\"]"));
        assert!(formatted.contains("[Black \"BlackPlayer\"]"));
        assert!(formatted.contains("[Result \"1-0\"]"));
        
        // Optional headers should come after, in alphabetical order
        let lines: Vec<&str> = formatted.lines().collect();
        let optional_start = lines.iter().position(|line| line.starts_with("[ECO")).unwrap();
        let elo_start = lines.iter().position(|line| line.starts_with("[WhiteElo")).unwrap();
        
        assert!(elo_start < optional_start); // WhiteElo comes before ECO alphabetically
    }

    #[test]
    fn test_get_or_default() {
        let header = PgnHeader::new();
        assert_eq!(header.get_or_default("Event", "Unknown"), "Unknown");
        
        let mut header = PgnHeader::new();
        header.add_header("Event".to_string(), "Known".to_string()).unwrap();
        assert_eq!(header.get_or_default("Event", "Unknown"), "Known");
    }

    #[test]
    fn test_contains() {
        let mut header = PgnHeader::new();
        assert!(!header.contains("Event"));
        
        header.add_header("Event".to_string(), "Test".to_string()).unwrap();
        assert!(header.contains("Event"));
    }

    #[test]
    fn test_remove() {
        let mut header = PgnHeader::new();
        header.add_header("Event".to_string(), "Test".to_string()).unwrap();
        assert!(header.contains("Event"));
        
        assert_eq!(header.remove("Event"), Some("Test".to_string()));
        assert!(!header.contains("Event"));
    }

    #[test]
    fn test_from_fields() {
        let mut fields = HashMap::new();
        fields.insert("Event".to_string(), "Test Event".to_string());
        fields.insert("Site".to_string(), "Test Site".to_string());
        fields.insert("Result".to_string(), "1-0".to_string());
        
        let header = PgnHeader::from_fields(fields).unwrap();
        assert!(header.is_validated());
        assert_eq!(header.get("Event"), Some(&"Test Event".to_string()));
    }
}
