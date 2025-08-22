use crate::position::ScidPosition;

/// Manages position state for variation tree exploration
#[derive(Debug, Clone)]
pub struct PositionState {
    /// Complete board state
    position: ScidPosition,
    
    /// Move number
    move_number: usize,
    
    /// Whose turn it is
    white_to_move: bool,
}

/// Stack-based position state manager for variations
#[derive(Debug)]
pub struct PositionStateManager {
    /// Stack of saved states
    state_stack: Vec<PositionState>,
}

impl PositionStateManager {
    pub fn new() -> Self {
        Self {
            state_stack: Vec::new(),
        }
    }
    
    /// Save current position state before exploring variation
    pub fn save_state(&mut self, position: &ScidPosition, move_number: usize, white_to_move: bool) {
        let state = PositionState {
            position: position.clone(),
            move_number,
            white_to_move,
        };
        self.state_stack.push(state);
    }
    
    /// Restore previous position state after variation exploration
    pub fn restore_state(&mut self) -> Option<(ScidPosition, usize, bool)> {
        self.state_stack.pop().map(|state| {
            (state.position, state.move_number, state.white_to_move)
        })
    }
    
    /// Peek at the most recent state without removing it
    pub fn peek_state(&self) -> Option<(&ScidPosition, usize, bool)> {
        self.state_stack.last().map(|state| {
            (&state.position, state.move_number, state.white_to_move)
        })
    }
    
    /// Get the current depth (number of saved states)
    pub fn depth(&self) -> usize {
        self.state_stack.len()
    }
    
    /// Clear all saved states
    pub fn clear(&mut self) {
        self.state_stack.clear();
    }
}

impl Default for PositionStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_manager_save_restore() {
        let mut manager = PositionStateManager::new();
        let position = ScidPosition::new_starting_position();
        
        // Initially empty
        assert_eq!(manager.depth(), 0);
        assert!(manager.restore_state().is_none());
        
        // Save a state
        manager.save_state(&position, 1, true);
        assert_eq!(manager.depth(), 1);
        
        // Peek at state
        let (_, move_num, white_turn) = manager.peek_state().unwrap();
        assert_eq!(move_num, 1);
        assert_eq!(white_turn, true);
        
        // Restore state
        let (restored_pos, restored_move, restored_turn) = manager.restore_state().unwrap();
        assert_eq!(restored_move, 1);
        assert_eq!(restored_turn, true);
        assert_eq!(manager.depth(), 0);
    }
    
    #[test]
    fn test_state_manager_multiple_saves() {
        let mut manager = PositionStateManager::new();
        let position = ScidPosition::new_starting_position();
        
        // Save multiple states
        manager.save_state(&position, 1, true);
        manager.save_state(&position, 2, false);
        manager.save_state(&position, 3, true);
        
        assert_eq!(manager.depth(), 3);
        
        // Restore in LIFO order
        let (_, move3, turn3) = manager.restore_state().unwrap();
        assert_eq!(move3, 3);
        assert_eq!(turn3, true);
        
        let (_, move2, turn2) = manager.restore_state().unwrap();
        assert_eq!(move2, 2);
        assert_eq!(turn2, false);
        
        let (_, move1, turn1) = manager.restore_state().unwrap();
        assert_eq!(move1, 1);
        assert_eq!(turn1, true);
        
        assert_eq!(manager.depth(), 0);
    }
}