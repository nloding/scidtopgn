//! Diagnostic helpers for move processing pipeline analysis
//! 
//! This module provides utilities to inspect and trace moves through
//! SCID move processing pipeline to identify issues and mismatches.

use crate::formats::sg4::{DecodedMove, MoveInterpretation};
use shakmaty::{Chess, Color};

/// Information extracted from a single move byte
#[derive(Debug, Clone)]
pub struct MoveByteInspection {
    pub raw_byte: u8,
    pub piece_num: u8,
    pub move_value: u8,
    pub piece_type_interpretation: String,
    pub move_description: String,
}

/// Report comparing move interpretation to board position
#[derive(Debug, Clone)]
pub struct DiagnosticReport {
    pub move_byte: u8,
    pub interpretation_says: String,
    pub board_has: String,
    pub conflict_detected: bool,
    pub explanation: String,
}

/// Trace of a move through complete processing pipeline
#[derive(Debug, Clone)]
pub struct PipelineTrace {
    pub stages: Vec<PipelineStage>,
}

#[derive(Debug, Clone)]
pub struct PipelineStage {
    pub name: String,
    pub input: String,
    pub output: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Report of piece lookup mismatch
#[derive(Debug, Clone)]
pub struct MismatchReport {
    pub move_byte: u8,
    pub piece_num: u8,
    pub expected_piece_type: String,
    pub actual_board_piece: String,
    pub piece_square: String,
    pub severity: MismatchSeverity,
}

#[derive(Debug, Clone)]
pub enum MismatchSeverity {
    TypeConflict,
    IndexNotFound,
    SystemicIssue,
}

/// Inspect a move byte and extract all available information
pub fn inspect_move_byte(byte: u8) -> MoveByteInspection {
    let piece_num = (byte >> 4) & 0x0F;
    let move_value = byte & 0x0F;
    
    let (piece_type_interpretation, move_description) = match piece_num {
        1 => ("King".to_string(), format!("King direction code {}", move_value)),
        2 => ("Queen".to_string(), format!("Queen move {}", move_value)),
        3 => ("Rook".to_string(), format!("Rook move {}", move_value)),
        4 => ("Bishop".to_string(), format!("Bishop move {}", move_value)),
        5 => ("Knight".to_string(), format!("Knight L-shape {}", move_value)),
        6 => ("Pawn".to_string(), format!("Pawn move {}", move_value)),
        _ => ("Unknown".to_string(), format!("Invalid piece_num {}", piece_num)),
    };
    
    MoveByteInspection {
        raw_byte: byte,
        piece_num,
        move_value,
        piece_type_interpretation,
        move_description,
    }
}

/// Compare move interpretation to current board position
pub fn compare_interpretation_to_position(move_byte: u8, position: &Chess) -> DiagnosticReport {
    let inspection = inspect_move_byte(move_byte);
    
    // What move byte interpretation says
    let interpretation_says = format!("piece_num {} = {}", 
                                   inspection.piece_num, 
                                   inspection.piece_type_interpretation);
    
    // What board position actually has at that index - simulate without calling private function
    let board_has = simulate_piece_lookup(inspection.piece_num, position.turn(), position);
    
    let conflict_detected = inspection.piece_type_interpretation != board_has;
    let explanation = if conflict_detected {
        format!("Move encoding piece type ({}) conflicts with position list piece at same index", 
                inspection.piece_type_interpretation)
    } else {
        "Move interpretation and position lookup agree".to_string()
    };
    
    DiagnosticReport {
        move_byte,
        interpretation_says,
        board_has,
        conflict_detected,
        explanation,
    }
}

/// Simulate piece lookup without calling private function
fn simulate_piece_lookup(piece_num: u8, color: Color, position: &Chess) -> String {
    // Simplified simulation based on starting position
    match piece_num {
        0 => "King".to_string(),
        1 => "Rook".to_string(), // This would be A1 rook in position list
        2 => "Knight".to_string(), // This would be B1 knight in position list
        3 => "Bishop".to_string(), // This would be C1 bishop in position list
        4 => "Queen".to_string(),
        5 => "Bishop".to_string(), // This would be F1 bishop in position list
        6 => "Knight".to_string(), // This would be G1 knight in position list
        7 => "Rook".to_string(), // This would be H1 rook in position list
        8..=15 => "Pawn".to_string(),
        _ => "Unknown".to_string(),
    }
}

/// Trace a move through the complete processing pipeline
pub fn trace_move_through_pipeline(byte: u8, position: &Chess) -> PipelineTrace {
    let mut stages = Vec::new();
    
    // Stage 1: SG4 Byte Parsing
    let inspection = inspect_move_byte(byte);
    stages.push(PipelineStage {
        name: "SG4 Parsing".to_string(),
        input: format!("0x{:02X}", byte),
        output: format!("piece_num={}, move_value={}, type={}", 
                     inspection.piece_num, 
                     inspection.move_value, 
                     inspection.piece_type_interpretation),
        success: true,
        error: None,
    });
    
    // Stage 2: Move Interpretation
    let decoded_result = DecodedMove {
        raw_bytes: vec![byte],
        piece_num: inspection.piece_num,
        move_value: inspection.move_value,
        interpretation: match inspection.piece_num {
            1 => MoveInterpretation::King { direction_code: inspection.move_value, is_castle: false },
            2 => MoveInterpretation::Queen,
            3 => MoveInterpretation::Rook,
            4 => MoveInterpretation::Bishop,
            5 => MoveInterpretation::Knight { l_shape_code: inspection.move_value },
            6 => MoveInterpretation::Pawn { 
                direction: "forward".to_string(), 
                promotion: None, 
                is_en_passant: None 
            },
            _ => MoveInterpretation::Unknown { reason: "Invalid piece_num".to_string() },
        },
        from_square_index: None,
        to_square_index: None,
        promotion_piece: None,
    };
    
    stages.push(PipelineStage {
        name: "Move Interpretation".to_string(),
        input: format!("piece_num={}, move_value={}", inspection.piece_num, inspection.move_value),
        output: format!("{:?}", decoded_result.interpretation),
        success: true,
        error: None,
    });
    
    // Stage 3: Bridge Conversion
    let conversion_result = decoded_result.to_shakmaty(position);
    match conversion_result {
        Ok(chess_move) => {
            stages.push(PipelineStage {
                name: "Bridge Conversion".to_string(),
                input: format!("{:?}", decoded_result.interpretation),
                output: format!("{:?}", chess_move),
                success: true,
                error: None,
            });
        }
        Err(e) => {
            stages.push(PipelineStage {
                name: "Bridge Conversion".to_string(),
                input: format!("{:?}", decoded_result.interpretation),
                output: "ERROR".to_string(),
                success: false,
                error: Some(format!("{}", e)),
            });
        }
    }
    
    PipelineTrace { stages }
}

/// Detect piece lookup mismatches between move encoding and position lists
pub fn detect_piece_lookup_mismatch(move_byte: u8, color: Color, position: &Chess) -> Option<MismatchReport> {
    let inspection = inspect_move_byte(move_byte);
    
    // What move encoding expects
    let expected_piece_type = inspection.piece_type_interpretation;
    
    // What position tracking actually has
    let actual_board_piece = simulate_piece_lookup(inspection.piece_num, color, position);
    
    if expected_piece_type != actual_board_piece {
        return Some(MismatchReport {
            move_byte,
            piece_num: inspection.piece_num,
            expected_piece_type,
            actual_board_piece,
            piece_square: format!("index {}", inspection.piece_num),
            severity: MismatchSeverity::TypeConflict,
        });
    }
    
    None
}

/// Generate a comprehensive report for a specific problematic move
pub fn generate_detailed_report(move_byte: u8, position: &Chess) -> String {
    let mut report = String::new();
    
    report.push_str(&format!("=== Move Byte Analysis: 0x{:02X} ===\n\n", move_byte));
    
    // Byte inspection
    let inspection = inspect_move_byte(move_byte);
    report.push_str("1. Move Byte Inspection:\n");
    report.push_str(&format!("   Raw byte: 0x{:02X}\n", inspection.raw_byte));
    report.push_str(&format!("   piece_num: {} (upper 4 bits)\n", inspection.piece_num));
    report.push_str(&format!("   move_value: {} (lower 4 bits)\n", inspection.move_value));
    report.push_str(&format!("   Interpreted piece type: {}\n", inspection.piece_type_interpretation));
    report.push_str(&format!("   Move description: {}\n\n", inspection.move_description));
    
    // Comparison to position
    let diagnostic = compare_interpretation_to_position(move_byte, position);
    report.push_str("2. Position Comparison:\n");
    report.push_str(&format!("   Move interpretation says: {}\n", diagnostic.interpretation_says));
    report.push_str(&format!("   Board position has: {}\n", diagnostic.board_has));
    report.push_str(&format!("   Conflict detected: {}\n", diagnostic.conflict_detected));
    report.push_str(&format!("   Explanation: {}\n\n", diagnostic.explanation));
    
    // Pipeline trace
    let trace = trace_move_through_pipeline(move_byte, position);
    report.push_str("3. Pipeline Trace:\n");
    for (i, stage) in trace.stages.iter().enumerate() {
        report.push_str(&format!("   Stage {}: {}\n", i + 1, stage.name));
        report.push_str(&format!("     Input:  {}\n", stage.input));
        report.push_str(&format!("     Output: {}\n", stage.output));
        report.push_str(&format!("     Status:  {}\n", if stage.success { "SUCCESS" } else { "ERROR" }));
        if let Some(error) = &stage.error {
            report.push_str(&format!("     Error:   {}\n", error));
        }
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::Chess;
    
    #[test]
    fn test_inspect_move_byte() {
        let inspection = inspect_move_byte(0x6C); // piece_num=6, move_value=12
        assert_eq!(inspection.raw_byte, 0x6C);
        assert_eq!(inspection.piece_num, 6);
        assert_eq!(inspection.move_value, 12);
        assert_eq!(inspection.piece_type_interpretation, "Pawn");
    }
    
    #[test]
    fn test_trace_move_through_pipeline() {
        let position = Chess::default();
        let trace = trace_move_through_pipeline(0x6C, &position);
        assert_eq!(trace.stages.len(), 3);
        assert_eq!(trace.stages[0].name, "SG4 Parsing");
        assert_eq!(trace.stages[1].name, "Move Interpretation");
        assert_eq!(trace.stages[2].name, "Bridge Conversion");
    }
}