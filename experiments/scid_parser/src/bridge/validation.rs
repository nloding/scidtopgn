use shakmaty::{Chess, Move, Position, Color, Outcome};
use crate::error::{Result, ScidError};

/// Comprehensive validation report for a sequence of moves
#[derive(Debug, Clone)]
pub struct ValidationReport {
    /// Total number of moves in the sequence
    pub total_moves: usize,
    
    /// List of invalid moves with their indices
    pub invalid_moves: Vec<(usize, Move)>,
    
    /// Final position after applying all valid moves
    pub final_position: Chess,
    
    /// Whether the entire sequence is valid
    pub is_valid: bool,
    
    /// Game termination status
    pub termination: GameTermination,
}

/// Possible game termination states
#[derive(Debug, Clone, PartialEq)]
pub enum GameTermination {
    /// Game is still in progress
    InProgress,
    
    /// Checkmate - specified color won
    Checkmate { winner: Color },
    
    /// Stalemate - draw
    Stalemate,
    
    /// Draw by insufficient material
    InsufficientMaterial,
    
    /// Draw by repetition
    Repetition,
    
    /// Draw by 50-move rule
    FiftyMoveRule,
    
    /// Game was abandoned or incomplete
    Incomplete,
}

/// Trait for comprehensive chess move validation
pub trait ChessValidation {
    /// Validate a complete sequence of moves from the starting position
    fn validate_move_sequence(&self, moves: &[Move]) -> Result<ValidationReport>;
    
    /// Check the integrity of a chess position
    fn check_position_integrity(&self, position: &Chess) -> Result<()>;
    
    /// Determine the game termination status for a position
    fn verify_game_termination(&self, position: &Chess) -> GameTermination;
    
    /// Validate that a single move is legal in the given position
    fn validate_single_move(&self, position: &Chess, chess_move: &Move) -> bool {
        position.is_legal(chess_move)
    }
}

/// Default implementation of chess validation
pub struct ChessValidator;

impl ChessValidator {
    /// Create a new chess validator
    pub fn new() -> Self {
        Self
    }
}

impl Default for ChessValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ChessValidation for ChessValidator {
    /// Validate a complete sequence of moves from the starting position
    fn validate_move_sequence(&self, moves: &[Move]) -> Result<ValidationReport> {
        let mut position = Chess::default();
        let mut invalid_moves = Vec::new();
        
        // Apply each move and track invalid ones
        for (idx, chess_move) in moves.iter().enumerate() {
            if !position.is_legal(chess_move) {
                invalid_moves.push((idx, chess_move.clone()));
                // Continue without applying invalid move
            } else {
                // Apply the valid move
                position = position.clone().play(chess_move)
                    .map_err(|e| ScidError::conversion_error(format!(
                        "Failed to apply move {} at index {}: {}", 
                        chess_move, idx, e
                    )))?;
            }
        }
        
        // Check position integrity after all moves
        self.check_position_integrity(&position)?;
        
        // Determine game termination status
        let termination = self.verify_game_termination(&position);
        
        let is_valid = invalid_moves.is_empty();
        
        Ok(ValidationReport {
            total_moves: moves.len(),
            invalid_moves,
            final_position: position,
            is_valid,
            termination,
        })
    }
    
    /// Check the integrity of a chess position
    fn check_position_integrity(&self, position: &Chess) -> Result<()> {
        // Verify basic position constraints
        let board = position.board();
        
        // Count kings by iterating through the board
        let mut white_king_count = 0;
        let mut black_king_count = 0;
        let mut pawn_on_end_rank = false;
        
        for square in shakmaty::Square::ALL {
            if let Some(piece) = board.piece_at(square) {
                match piece.role {
                    shakmaty::Role::King => {
                        match piece.color {
                            Color::White => white_king_count += 1,
                            Color::Black => black_king_count += 1,
                        }
                    }
                    shakmaty::Role::Pawn => {
                        // Check if pawn is on first or last rank
                        if square.rank() == shakmaty::Rank::First || square.rank() == shakmaty::Rank::Eighth {
                            pawn_on_end_rank = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Verify exactly one king per side
        if white_king_count != 1 {
            return Err(ScidError::conversion_error(format!(
                "Invalid position: found {} white kings, expected 1", 
                white_king_count
            )));
        }
        
        if black_king_count != 1 {
            return Err(ScidError::conversion_error(format!(
                "Invalid position: found {} black kings, expected 1", 
                black_king_count
            )));
        }
        
        // Check for pawns on end ranks
        if pawn_on_end_rank {
            return Err(ScidError::conversion_error(
                "Invalid position: pawns found on first or eighth rank".to_string()
            ));
        }
        
        // Basic check: ensure the position has some structure
        // More complex validation (like checking for impossible positions) could be added here
        
        Ok(())
    }
    
    /// Determine the game termination status for a position
    fn verify_game_termination(&self, position: &Chess) -> GameTermination {
        // Check if the game has ended
        match position.outcome() {
            Some(Outcome::Decisive { winner }) => {
                // This is checkmate
                GameTermination::Checkmate { winner }
            }
            Some(Outcome::Draw) => {
                // Determine the type of draw
                if position.is_stalemate() {
                    GameTermination::Stalemate
                } else if position.is_insufficient_material() {
                    GameTermination::InsufficientMaterial
                } else {
                    // Could be repetition or 50-move rule, but we need more context
                    // For now, we'll classify as general draw
                    GameTermination::Repetition
                }
            }
            None => {
                // Game is still in progress
                GameTermination::InProgress
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{Square, Role};
    
    #[test]
    fn test_validator_creation() {
        let validator = ChessValidator::new();
        
        // Test that we can create a validation report for an empty sequence
        let moves = vec![];
        let report = validator.validate_move_sequence(&moves).unwrap();
        
        assert_eq!(report.total_moves, 0);
        assert!(report.is_valid);
        assert_eq!(report.termination, GameTermination::InProgress);
    }
    
    #[test]
    fn test_valid_move_sequence() {
        let validator = ChessValidator::new();
        
        // Create a simple valid move sequence: 1. e4 e5
        let moves = vec![
            Move::Normal {
                role: Role::Pawn,
                from: Square::E2,
                to: Square::E4,
                capture: None,
                promotion: None,
            },
            Move::Normal {
                role: Role::Pawn,
                from: Square::E7,
                to: Square::E5,
                capture: None,
                promotion: None,
            },
        ];
        
        let report = validator.validate_move_sequence(&moves).unwrap();
        
        assert_eq!(report.total_moves, 2);
        assert!(report.is_valid);
        assert!(report.invalid_moves.is_empty());
        assert_eq!(report.termination, GameTermination::InProgress);
    }
    
    #[test]
    fn test_invalid_move_detection() {
        let validator = ChessValidator::new();
        
        // Create an invalid move sequence: try to move pawn backwards
        let moves = vec![
            Move::Normal {
                role: Role::Pawn,
                from: Square::E2,
                to: Square::E1, // Invalid: pawn moving backwards
                capture: None,
                promotion: None,
            },
        ];
        
        let report = validator.validate_move_sequence(&moves).unwrap();
        
        assert_eq!(report.total_moves, 1);
        assert!(!report.is_valid);
        assert_eq!(report.invalid_moves.len(), 1);
        assert_eq!(report.invalid_moves[0].0, 0); // First move is invalid
    }
    
    #[test]
    fn test_position_integrity_valid() {
        let validator = ChessValidator::new();
        let position = Chess::default(); // Starting position
        
        let result = validator.check_position_integrity(&position);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_game_termination_in_progress() {
        let validator = ChessValidator::new();
        let position = Chess::default(); // Starting position
        
        let termination = validator.verify_game_termination(&position);
        assert_eq!(termination, GameTermination::InProgress);
    }
}