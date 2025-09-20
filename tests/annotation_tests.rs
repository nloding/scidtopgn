use scidtopgn::pgn::annotation_formatter::AnnotationFormatter;
use scidtopgn::position::{PieceType, ScidMove, Square};
use scidtopgn::sg4::{
    comment_processor::CommentProcessor, nag_processor::NagProcessor, VariationGameElement,
    VariationMove,
};

#[test]
fn test_comment_processing_comprehensive() {
    let mut processor = CommentProcessor::new();

    // Test basic comment
    let result = processor.process_comment("This is a good move");
    assert_eq!(result, "{This is a good move}");

    // Test comment with SCID markers
    let result = processor.process_comment("Excellent! $3");
    assert_eq!(result, "{Excellent! !!}");

    // Test comment with special characters
    let result = processor.process_comment("A {complex} comment with \\backslashes");
    assert_eq!(result, "{A \\{complex\\} comment with \\\\backslashes}");

    // Test empty comment
    let result = processor.process_comment("");
    assert_eq!(result, "");

    // Test whitespace-only comment
    let result = processor.process_comment("   \t\n  ");
    assert_eq!(result, "");

    // Test multiple markers
    let result = processor.process_comment("Great move $3 but position is equal $10");
    assert_eq!(result, "{Great move !! but position is equal =}");
}

#[test]
fn test_nag_processing_comprehensive() {
    // Test symbol NAGs
    assert_eq!(NagProcessor::nag_to_symbol(1), "!");
    assert_eq!(NagProcessor::nag_to_symbol(2), "?");
    assert_eq!(NagProcessor::nag_to_symbol(3), "!!");
    assert_eq!(NagProcessor::nag_to_symbol(4), "??");

    // Test position evaluation NAGs
    assert_eq!(NagProcessor::nag_to_symbol(14), "⩲");
    assert_eq!(NagProcessor::nag_to_symbol(15), "⩱");
    assert_eq!(NagProcessor::nag_to_symbol(16), "±");

    // Test symbol vs description NAGs
    assert!(NagProcessor::is_symbol_nag(1));
    assert!(NagProcessor::is_symbol_nag(14));
    assert!(!NagProcessor::is_symbol_nag(30));

    // Test descriptions
    assert_eq!(NagProcessor::nag_to_description(1), Some("Good move"));
    assert_eq!(NagProcessor::nag_to_description(30), Some("Initiative"));
    assert_eq!(NagProcessor::nag_to_description(100), None);
}

#[test]
fn test_move_annotation_formatting_comprehensive() {
    // Create test move with multiple types of annotations
    let test_move = create_complex_annotated_move();

    let formatted = AnnotationFormatter::format_move_with_annotations(&test_move);

    // Should include move, NAG symbols, and comments
    assert!(formatted.contains("Nf3")); // The move
    assert!(formatted.contains("!")); // Symbol NAG
    assert!(formatted.contains("⩲")); // Position evaluation NAG
    assert!(formatted.contains("{Developing the knight}")); // Comment
    assert!(formatted.contains("{Initiative}")); // Descriptive NAG as comment

    println!("Complex annotated move formatted as: {}", formatted);
}

#[test]
fn test_complex_annotation_sequence() {
    // Test a complex game with multiple types of annotations
    // Including variations with comments and NAGs

    let moves = create_complex_game_sequence();
    let sequence_formatted = AnnotationFormatter::format_move_sequence(&moves);

    // Verify the sequence contains all expected elements
    assert!(sequence_formatted.contains("1.e4")); // First move with number
    assert!(sequence_formatted.contains("e5")); // Response without number
    assert!(sequence_formatted.contains("!")); // Contains NAG symbols
    assert!(sequence_formatted.contains("{")); // Contains comments

    println!("Complex sequence formatted as: {}", sequence_formatted);
}

#[test]
fn test_nag_categorization_integration() {
    let mixed_nags = vec![1, 2, 14, 16, 30, 44]; // Mix of symbol and descriptive
    let (symbol_nags, descriptive_nags) = AnnotationFormatter::categorize_nags(&mixed_nags);

    // Verify categorization
    assert_eq!(symbol_nags.len(), 4); // 1, 2, 14, 16
    assert_eq!(descriptive_nags.len(), 2); // 30, 44

    // Verify content
    assert!(symbol_nags.contains(&1));
    assert!(symbol_nags.contains(&14));
    assert!(descriptive_nags.contains(&30));
    assert!(descriptive_nags.contains(&44));
}

#[test]
fn test_full_annotation_pipeline() {
    // Test the complete pipeline from raw data to formatted PGN
    let mut processor = CommentProcessor::new();

    // Process a raw comment
    let raw_comment = "Excellent opening! $3 Position looks good $14";
    let processed_comment = processor.process_comment(raw_comment);

    // Create a move with processed comment and NAGs
    let mut test_move = create_basic_move();
    test_move.comments.push(processed_comment);
    test_move.nags = vec![1, 14]; // Good move, White better

    // Format the complete move
    let formatted = AnnotationFormatter::format_move_with_annotations(&test_move);

    // Verify complete processing
    assert!(formatted.contains("e4")); // Move
    assert!(formatted.contains("!")); // NAG symbol from move
    assert!(formatted.contains("⩲")); // NAG symbol from move
    assert!(formatted.contains("!!")); // NAG symbol from comment
    assert!(formatted.contains("Position looks good")); // Processed comment text

    println!("Full pipeline result: {}", formatted);
}

#[test]
fn test_annotation_edge_cases() {
    // Test empty annotations
    let empty_move = create_basic_move();
    assert!(!AnnotationFormatter::has_annotations(&empty_move));
    assert_eq!(AnnotationFormatter::annotation_count(&empty_move), 0);

    // Test move with only comments
    let mut comment_only = create_basic_move();
    comment_only.comments.push("{Just a comment}".to_string());
    assert!(AnnotationFormatter::has_annotations(&comment_only));
    assert_eq!(AnnotationFormatter::annotation_count(&comment_only), 1);

    // Test move with only NAGs
    let mut nag_only = create_basic_move();
    nag_only.nags.push(1);
    assert!(AnnotationFormatter::has_annotations(&nag_only));
    assert_eq!(AnnotationFormatter::annotation_count(&nag_only), 1);

    // Test unknown NAG handling
    let unknown_nag_symbol = NagProcessor::nag_to_symbol(255);
    assert_eq!(unknown_nag_symbol, "");

    let unknown_nag_desc = NagProcessor::nag_to_description(255);
    assert_eq!(unknown_nag_desc, None);
}

#[test]
fn test_multiline_comment_handling() {
    let mut processor = CommentProcessor::new();

    // Test comment with line breaks (should be normalized)
    let multiline_comment = "This is a\nmultiline\ncomment with\ttabs";
    let result = processor.process_comment(multiline_comment);
    assert_eq!(result, "{This is a multiline comment with tabs}");

    // Test comment with extra whitespace
    let spaced_comment = "  Too   many   spaces   here  ";
    let result = processor.process_comment(spaced_comment);
    assert_eq!(result, "{Too many spaces here}");
}

#[test]
fn test_nag_combination_handling() {
    // Test move with multiple NAG types
    let mut test_move = create_basic_move();
    test_move.nags = vec![1, 3, 14, 16, 30, 44]; // Mix of all types

    let formatted = AnnotationFormatter::format_move_with_annotations(&test_move);

    // Should contain all symbol NAGs inline
    assert!(formatted.contains("!")); // NAG 1
    assert!(formatted.contains("!!")); // NAG 3
    assert!(formatted.contains("⩲")); // NAG 14
    assert!(formatted.contains("±")); // NAG 16

    // Should contain descriptive NAGs as comments
    assert!(formatted.contains("{Initiative}")); // NAG 30
    assert!(formatted.contains("{With compensation}")); // NAG 44

    println!("Multiple NAG types: {}", formatted);
}

// Helper functions for creating test data

fn create_basic_move() -> VariationMove {
    VariationMove {
        chess_move: ScidMove {
            from: Square(12),
            to: Square(28),
            moving_piece: PieceType::Pawn,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 12,
        },
        move_number: 1,
        is_white_move: true,
        comments: Vec::new(),
        nags: Vec::new(),
        algebraic: "e4".to_string(),
    }
}

fn create_complex_annotated_move() -> VariationMove {
    VariationMove {
        chess_move: ScidMove {
            from: Square(6),
            to: Square(21),
            moving_piece: PieceType::Knight,
            captured_piece: PieceType::Empty,
            promote: PieceType::Empty,
            piece_num: 6,
        },
        move_number: 1,
        is_white_move: true,
        comments: vec!["{Developing the knight}".to_string()],
        nags: vec![1, 14, 30], // Good move, White better, Initiative
        algebraic: "Nf3".to_string(),
    }
}

fn create_complex_game_sequence() -> Vec<VariationMove> {
    vec![
        // 1.e4
        VariationMove {
            chess_move: ScidMove {
                from: Square(12),
                to: Square(28),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: true,
            comments: vec!["{King's pawn opening}".to_string()],
            nags: vec![1], // Good move
            algebraic: "e4".to_string(),
        },
        // 1...e5
        VariationMove {
            chess_move: ScidMove {
                from: Square(52),
                to: Square(36),
                moving_piece: PieceType::Pawn,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 12,
            },
            move_number: 1,
            is_white_move: false,
            comments: vec!["{Symmetrical response}".to_string()],
            nags: vec![1, 10], // Good move, Equal
            algebraic: "e5".to_string(),
        },
        // 2.Nf3
        VariationMove {
            chess_move: ScidMove {
                from: Square(6),
                to: Square(21),
                moving_piece: PieceType::Knight,
                captured_piece: PieceType::Empty,
                promote: PieceType::Empty,
                piece_num: 6,
            },
            move_number: 2,
            is_white_move: true,
            comments: vec!["{Developing}".to_string()],
            nags: vec![1, 14], // Good move, White slightly better
            algebraic: "Nf3".to_string(),
        },
    ]
}
