use shakmaty::{Chess, Position};
use crate::formats::DecodedMove;
use crate::core::error::{Result, ScidError};
use crate::bridge::moves::ScidToShakmaty;

/// Position tracker for converting SCID moves to algebraic notation
/// 
/// This struct maintains a chess position and provides methods to apply
/// SCID moves while tracking the resulting positions and converting to
/// standard algebraic notation.
#[derive(Debug, Clone)]
pub struct PositionTracker {
    /// Current chess position using shakmaty library
    position: Chess,
    /// Number of moves applied to track progress
    move_count: usize,
}

impl PositionTracker {
    /// Create a new position tracker at the starting position
    /// 
    /// Returns a PositionTracker initialized with the standard chess
    /// starting position and a move count of 0.
    pub fn new() -> Self {
        Self {
            position: Chess::default(),
            move_count: 0,
        }
    }
    
    /// Apply a decoded SCID move to the position
    /// 
    /// Converts the SCID move to a shakmaty move, applies it to the
    /// position, and returns the algebraic notation.
    /// 
    /// # Arguments
    /// * `decoded_move` - The SCID move to apply
    /// 
    /// # Returns
    /// * `Ok(String)` - The algebraic notation of the move
    /// * `Err(ScidError)` - If move conversion or application fails
    /// 
    /// # Examples
    /// ```
    /// use scidtopgn::position::move_converter::PositionTracker;
    /// 
    /// let mut tracker = PositionTracker::new();
    /// // Apply a move and get its notation
    /// let notation = tracker.apply_move(&some_decoded_move)?;
    /// println!("Move: {}", notation);
    /// ```
    pub fn apply_move(&mut self, decoded_move: &DecodedMove) -> Result<String> {
        // Convert SCID move to shakmaty move
        let shakmaty_move = decoded_move.to_shakmaty(&self.position)?;
        
        // Convert to algebraic notation before applying
        let san = shakmaty::san::San::from_move(&self.position, &shakmaty_move);
        let notation = san.to_string();
        
        // Apply the move to update position
        self.position = self.position.clone().play(&shakmaty_move)
            .map_err(|e| ScidError::conversion_error(format!("Failed to apply move: {}", e)))?;
        
        // Increment move count
        self.move_count += 1;
        
        Ok(notation)
    }
    
    /// Get the current move count
    /// 
    /// Returns the number of moves that have been applied to this tracker.
    pub fn move_count(&self) -> usize {
        self.move_count
    }
    
    /// Get the current position
    /// 
    /// Returns a reference to the current chess position.
    pub fn position(&self) -> &Chess {
        &self.position
    }
    
    /// Reset the tracker to the starting position
    /// 
    /// Resets the position to the standard chess starting position
    /// and sets the move count back to 0.
    pub fn reset(&mut self) {
        self.position = Chess::default();
        self.move_count = 0;
    }
}

impl Default for PositionTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{DecodedMove, MoveInterpretation};
    
    #[test]
    fn test_position_tracker_initialization() {
        let tracker = PositionTracker::new();
        
        assert_eq!(tracker.move_count(), 0);
        // Position should be at starting position
        let fen = tracker.position().to_string();
        assert!(fen.contains("rnbqkbnr")); // White pieces
        assert!(fen.contains("RNBQKBNR")); // Black pieces
    }
    
    #[test]
    fn test_default_implementation() {
        let tracker = PositionTracker::default();
        
        assert_eq!(tracker.move_count(), 0);
        // Should be same as new()
        let _ = PositionTracker::new();
    }
    
    #[test]
    fn test_reset_functionality() {
        let mut tracker = PositionTracker::new();
        
        // We can't easily apply moves without a proper DecodedMove,
        // but we can test reset functionality
        tracker.reset();
        assert_eq!(tracker.move_count(), 0);
    }
    
    #[test]
    fn test_position_access() {
        let tracker = PositionTracker::new();
        let position = tracker.position();
        
        // Should be able to access the position
        let fen = position.to_string();
        assert!(!fen.is_empty());
    }
    
    #[test]
    fn test_apply_move_error_handling() {
        let mut tracker = PositionTracker::new();
        
        // Create a move that will cause an error
        let invalid_move = DecodedMove {
            raw_bytes: vec![0xFF],
            piece_num: 31,
            move_value: 15,
            interpretation: MoveInterpretation::Unknown {
                reason: "Invalid move for testing".to_string(),
            },
            from_square_index: None,
            to_square_index: None,
            promotion_piece: None,
        };
        
        // Should return an error
        let result = tracker.apply_move(&invalid_move);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_simple_move_structure() {
        let simple_move = DecodedMove {
            raw_bytes: vec![0x00],
            piece_num: 0,
            move_value: 0,
            interpretation: MoveInterpretation::King {
                direction_code: 0,
                is_castle: false,
            },
            from_square_index: Some(4),
            to_square_index: Some(12),
            promotion_piece: None,
        };
        
        // Verify the move structure is accessible
        assert_eq!(simple_move.piece_num, 0);
        assert_eq!(simple_move.move_value, 0);
        
        match &simple_move.interpretation {
            MoveInterpretation::King { direction_code, is_castle } => {
                assert_eq!(*direction_code, 0);
                assert!(!is_castle);
            }
            _ => panic!("Expected King interpretation"),
        }
    }
}