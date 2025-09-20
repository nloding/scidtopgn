/// Complete NAG (Numeric Annotation Glyph) processor
pub struct NagProcessor;

impl NagProcessor {
	/// Convert NAG number to PGN symbol
	pub fn nag_to_symbol(nag: u8) -> &'static str {
		match nag {
			// Move evaluation
			1 => "!",      // Good move
			2 => "?",      // Poor move
			3 => "!!",     // Excellent move
			4 => "??",     // Blunder
			5 => "!?",     // Interesting move
			6 => "?!",     // Questionable move
			7 => "□",      // Forced move
            
			// Position evaluation
			10 => "=",     // Equal position
			11 => "=",     // Equal position, alternate
			12 => "=",     // Equal position, alternate
			13 => "∞",     // Unclear position
			14 => "⩲",     // White is slightly better
			15 => "⩱",     // Black is slightly better
			16 => "±",     // White is better
			17 => "∓",     // Black is better
			18 => "+-",    // White is winning
			19 => "-+",    // Black is winning
			20 => "+-",    // White is winning (alternate)
			21 => "-+",    // Black is winning (alternate)
            
			// Time pressure / other
			22 => "⨀",     // White in zugzwang
			23 => "⨀",     // Black in zugzwang
			24 => "○",     // White slight space
			25 => "○",     // Black slight space
			26 => "○",     // White space advantage
			27 => "○",     // Black space advantage
			28 => "○",     // White decisive space
			29 => "○",     // Black decisive space
            
			// Tactical themes
			30 => "→",     // Initiative
			31 => "→",     // Development
			32 => "⇆",     // Counterplay
			33 => "△",     // Better is
			34 => "△",     // Worse is
			35 => "△",     // Equivalent is
			36 => "↑",     // Editorial comment
			37 => "↑",     // Editorial comment
			38 => "↑",     // Editorial comment
			39 => "↑",     // Editorial comment
            
			// Opening/endgame
			40 => "∆",     // With the idea
			41 => "∆",     // Aimed against
			42 => "⌐",     // Better is
			43 => "⌐",     // Worse is
			44 => "=∞",    // With compensation
			45 => "=/∞",   // With slight compensation
            
			// Default for unknown NAGs
			_ => "",
		}
	}
    
	/// Convert NAG number to descriptive text (for comments)
	pub fn nag_to_description(nag: u8) -> Option<&'static str> {
		match nag {
			1 => Some("Good move"),
			2 => Some("Poor move"),
			3 => Some("Excellent move"),
			4 => Some("Blunder"),
			5 => Some("Interesting move"),
			6 => Some("Questionable move"),
			7 => Some("Forced move"),
			10 => Some("Equal position"),
			13 => Some("Unclear position"),
			14 => Some("White is slightly better"),
			15 => Some("Black is slightly better"),
			16 => Some("White is better"),
			17 => Some("Black is better"),
			18 => Some("White is winning"),
			19 => Some("Black is winning"),
			22 => Some("White in zugzwang"),
			23 => Some("Black in zugzwang"),
			30 => Some("Initiative"),
			32 => Some("Counterplay"),
			44 => Some("With compensation"),
			_ => None,
		}
	}
    
	/// Check if NAG should be displayed as symbol or in comment
	pub fn is_symbol_nag(nag: u8) -> bool {
		matches!(nag, 1..=6 | 10..=21)
	}
    
	/// Check if NAG is a move evaluation (as opposed to position evaluation)
	pub fn is_move_evaluation_nag(nag: u8) -> bool {
		matches!(nag, 1..=7)
	}
    
	/// Check if NAG is a position evaluation
	pub fn is_position_evaluation_nag(nag: u8) -> bool {
		matches!(nag, 10..=21)
	}
    
	/// Get NAG category for better formatting
	pub fn get_nag_category(nag: u8) -> NagCategory {
		match nag {
			1..=7 => NagCategory::MoveEvaluation,
			10..=21 => NagCategory::PositionEvaluation,
			22..=29 => NagCategory::TimeAndSpace,
			30..=39 => NagCategory::Tactical,
			40..=45 => NagCategory::Strategic,
			_ => NagCategory::Unknown,
		}
	}
}

/// Categories of NAG annotations for better organization
#[derive(Debug, PartialEq, Clone)]
pub enum NagCategory {
	MoveEvaluation,     // !, ?, !!, ??, etc.
	PositionEvaluation, // =, ±, ∓, +-, etc.
	TimeAndSpace,       // Zugzwang, space advantage
	Tactical,           // Initiative, counterplay
	Strategic,          // Compensation, ideas
	Unknown,            // Unrecognized NAGs
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_move_evaluation_nags() {
		assert_eq!(NagProcessor::nag_to_symbol(1), "!");
		assert_eq!(NagProcessor::nag_to_symbol(2), "?");
		assert_eq!(NagProcessor::nag_to_symbol(3), "!!");
		assert_eq!(NagProcessor::nag_to_symbol(4), "??");
		assert_eq!(NagProcessor::nag_to_symbol(5), "!?");
		assert_eq!(NagProcessor::nag_to_symbol(6), "?!");
		assert_eq!(NagProcessor::nag_to_symbol(7), "□");
	}
    
	#[test]
	fn test_position_evaluation_nags() {
		assert_eq!(NagProcessor::nag_to_symbol(10), "=");
		assert_eq!(NagProcessor::nag_to_symbol(13), "∞");
		assert_eq!(NagProcessor::nag_to_symbol(14), "⩲");
		assert_eq!(NagProcessor::nag_to_symbol(15), "⩱");
		assert_eq!(NagProcessor::nag_to_symbol(16), "±");
		assert_eq!(NagProcessor::nag_to_symbol(17), "∓");
		assert_eq!(NagProcessor::nag_to_symbol(18), "+-");
		assert_eq!(NagProcessor::nag_to_symbol(19), "-+");
	}
    
	#[test]
	fn test_tactical_and_strategic_nags() {
		assert_eq!(NagProcessor::nag_to_symbol(30), "→");
		assert_eq!(NagProcessor::nag_to_symbol(32), "⇆");
		assert_eq!(NagProcessor::nag_to_symbol(44), "=∞");
		assert_eq!(NagProcessor::nag_to_symbol(45), "=/∞");
	}
    
	#[test]
	fn test_unknown_nags() {
		assert_eq!(NagProcessor::nag_to_symbol(100), "");
		assert_eq!(NagProcessor::nag_to_symbol(255), "");
		assert_eq!(NagProcessor::nag_to_symbol(0), "");
	}
    
	#[test]
	fn test_nag_descriptions() {
		assert_eq!(NagProcessor::nag_to_description(1), Some("Good move"));
		assert_eq!(NagProcessor::nag_to_description(4), Some("Blunder"));
		assert_eq!(NagProcessor::nag_to_description(14), Some("White is slightly better"));
		assert_eq!(NagProcessor::nag_to_description(19), Some("Black is winning"));
		assert_eq!(NagProcessor::nag_to_description(30), Some("Initiative"));
		assert_eq!(NagProcessor::nag_to_description(100), None);
	}
    
	#[test]
	fn test_symbol_nag_classification() {
		// Move evaluation NAGs should be symbols
		assert!(NagProcessor::is_symbol_nag(1));
		assert!(NagProcessor::is_symbol_nag(2));
		assert!(NagProcessor::is_symbol_nag(6));
        
		// Position evaluation NAGs should be symbols
		assert!(NagProcessor::is_symbol_nag(10));
		assert!(NagProcessor::is_symbol_nag(16));
		assert!(NagProcessor::is_symbol_nag(21));
        
		// Tactical/strategic NAGs should not be symbols (go in comments)
		assert!(!NagProcessor::is_symbol_nag(30));
		assert!(!NagProcessor::is_symbol_nag(44));
	}
    
	#[test]
	fn test_nag_category_classification() {
		assert_eq!(NagProcessor::get_nag_category(1), NagCategory::MoveEvaluation);
		assert_eq!(NagProcessor::get_nag_category(4), NagCategory::MoveEvaluation);
		assert_eq!(NagProcessor::get_nag_category(14), NagCategory::PositionEvaluation);
		assert_eq!(NagProcessor::get_nag_category(22), NagCategory::TimeAndSpace);
		assert_eq!(NagProcessor::get_nag_category(30), NagCategory::Tactical);
		assert_eq!(NagProcessor::get_nag_category(44), NagCategory::Strategic);
		assert_eq!(NagProcessor::get_nag_category(100), NagCategory::Unknown);
	}
    
	#[test]
	fn test_move_vs_position_evaluation() {
		// Move evaluation NAGs
		assert!(NagProcessor::is_move_evaluation_nag(1));
		assert!(NagProcessor::is_move_evaluation_nag(4));
		assert!(NagProcessor::is_move_evaluation_nag(7));
		assert!(!NagProcessor::is_move_evaluation_nag(10));
        
		// Position evaluation NAGs
		assert!(NagProcessor::is_position_evaluation_nag(10));
		assert!(NagProcessor::is_position_evaluation_nag(16));
		assert!(NagProcessor::is_position_evaluation_nag(21));
		assert!(!NagProcessor::is_position_evaluation_nag(1));
		assert!(!NagProcessor::is_position_evaluation_nag(30));
	}
    
	#[test]
	fn test_comprehensive_nag_coverage() {
		// Test that all defined NAGs return non-empty symbols or descriptions
		let test_nags = [1, 2, 3, 4, 5, 6, 7, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21,
						22, 23, 30, 32, 44, 45];
        
		for nag in test_nags.iter() {
			let symbol = NagProcessor::nag_to_symbol(*nag);
			let description = NagProcessor::nag_to_description(*nag);
            
			// Each NAG should have either a symbol or description (or both)
			assert!(
				!symbol.is_empty() || description.is_some(),
				"NAG {} should have either symbol or description", 
				nag
			);
		}
	}
}
