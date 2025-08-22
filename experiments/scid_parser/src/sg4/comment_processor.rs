/// Processes SCID comments for PGN export
pub struct CommentProcessor {
    /// Current comment buffer
    buffer: String,
}

impl CommentProcessor {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }
    
    /// Process raw SCID comment data into PGN-formatted comment
    pub fn process_comment(&mut self, raw_comment: &str) -> String {
        // Clean up SCID-specific formatting
        let cleaned = self.clean_scid_formatting(raw_comment);
        
        // Handle special SCID comment markers
        let processed = self.process_scid_markers(&cleaned);
        
        // Format for PGN (wrap in braces, escape special characters)
        self.format_for_pgn(&processed)
    }
    
    /// Clean SCID-specific formatting characters
    fn clean_scid_formatting(&self, comment: &str) -> String {
        let mut result = comment.to_string();
        
        // Remove SCID control characters
        result = result.chars()
            .filter(|c| c.is_ascii_graphic() || c.is_whitespace())
            .collect();
        
        // Normalize whitespace
        result = result.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        
        result
    }
    
    /// Process SCID-specific comment markers
    fn process_scid_markers(&self, comment: &str) -> String {
        let mut result = comment.to_string();
        
        // Handle position evaluation markers first (longer patterns)
        result = result.replace("$10", "=");   // Equal
        result = result.replace("$14", "⩲");   // White slightly better
        result = result.replace("$15", "⩱");   // Black slightly better
        result = result.replace("$16", "±");   // White better
        result = result.replace("$17", "∓");   // Black better
        result = result.replace("$18", "+-");  // White winning
        result = result.replace("$19", "-+");  // Black winning
        
        // Handle SCID evaluation markers (shorter patterns)
        result = result.replace("$1", "!");   // Good move
        result = result.replace("$2", "?");   // Poor move
        result = result.replace("$3", "!!");  // Excellent move
        result = result.replace("$4", "??");  // Blunder
        result = result.replace("$5", "!?");  // Interesting move
        result = result.replace("$6", "?!");  // Questionable move
        
        result
    }
    
    /// Format comment for PGN output
    fn format_for_pgn(&self, comment: &str) -> String {
        if comment.trim().is_empty() {
            return String::new();
        }
        
        // Escape special PGN characters
        let escaped = comment
            .replace("\\", "\\\\")  // Escape backslashes
            .replace("{", "\\{")    // Escape opening braces
            .replace("}", "\\}");   // Escape closing braces
        
        format!("{{{}}}", escaped)
    }
}

impl Default for CommentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_comment_processing() {
        let mut processor = CommentProcessor::new();
        
        let result = processor.process_comment("This is a basic comment");
        assert_eq!(result, "{This is a basic comment}");
    }
    
    #[test]
    fn test_scid_marker_processing() {
        let mut processor = CommentProcessor::new();
        
        // Test move evaluation markers
        let result = processor.process_comment("Excellent move $3");
        assert_eq!(result, "{Excellent move !!}");
        
        let result = processor.process_comment("Poor choice $2");
        assert_eq!(result, "{Poor choice ?}");
        
        let result = processor.process_comment("Good move $1");
        assert_eq!(result, "{Good move !}");
    }
    
    #[test]
    fn test_position_evaluation_markers() {
        let mut processor = CommentProcessor::new();
        
        let result = processor.process_comment("Position is equal $10");
        assert_eq!(result, "{Position is equal =}");
        
        let result = processor.process_comment("White is better $16");
        assert_eq!(result, "{White is better ±}");
        
        let result = processor.process_comment("Black winning $19");
        assert_eq!(result, "{Black winning -+}");
    }
    
    #[test]
    fn test_special_character_escaping() {
        let mut processor = CommentProcessor::new();
        
        // Test brace escaping
        let result = processor.process_comment("A {nested} comment");
        assert_eq!(result, "{A \\{nested\\} comment}");
        
        // Test backslash escaping
        let result = processor.process_comment("Path with \\backslashes");
        assert_eq!(result, "{Path with \\\\backslashes}");
        
        // Test combined escaping
        let result = processor.process_comment("Complex {comment} with \\slashes");
        assert_eq!(result, "{Complex \\{comment\\} with \\\\slashes}");
    }
    
    #[test]
    fn test_whitespace_normalization() {
        let mut processor = CommentProcessor::new();
        
        // Test extra whitespace removal
        let result = processor.process_comment("  Multiple   spaces   here  ");
        assert_eq!(result, "{Multiple spaces here}");
        
        // Test tab and newline normalization
        let result = processor.process_comment("Tab\there\nand\r\nnewlines");
        assert_eq!(result, "{Tab here and newlines}");
    }
    
    #[test]
    fn test_empty_comment_handling() {
        let mut processor = CommentProcessor::new();
        
        // Empty string
        let result = processor.process_comment("");
        assert_eq!(result, "");
        
        // Whitespace only
        let result = processor.process_comment("   \t\n  ");
        assert_eq!(result, "");
    }
    
    #[test]
    fn test_control_character_filtering() {
        let mut processor = CommentProcessor::new();
        
        // Test with control characters (simulate SCID binary data)
        let comment_with_control = format!("Good move{}\x01\x02\x03 continues", "");
        let result = processor.process_comment(&comment_with_control);
        assert_eq!(result, "{Good move continues}");
    }
    
    #[test]
    fn test_multiple_marker_processing() {
        let mut processor = CommentProcessor::new();
        
        // Test multiple markers in one comment
        let result = processor.process_comment("Great move $3 but position is equal $10");
        assert_eq!(result, "{Great move !! but position is equal =}");
    }
}