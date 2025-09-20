use crate::position::state_manager::PositionStateManager;
use crate::position::{PieceType, ScidMove, ScidPosition, Square};
use crate::variation::{Variation, VariationGameElement, VariationMove, VariationTree};

/// Builder for constructing variation trees from parsed game elements
/// with position state management for accurate algebraic notation
#[derive(Debug)]
pub struct VariationTreeBuilder {
    main_line: Vec<VariationMove>,
    variations: Vec<Variation>,
    current_variation_stack: Vec<VariationInProgress>,
    move_counter: usize,
    is_white_turn: bool,

    // Position tracking for accurate move decoding
    current_position: ScidPosition,
    position_state_manager: PositionStateManager,
}

#[derive(Debug)]
struct VariationInProgress {
    start_move_index: usize,
    moves: Vec<VariationMove>,
    sub_variations: Vec<Variation>,
    depth: usize,
}

impl VariationTreeBuilder {
    pub fn new() -> Self {
        Self {
            main_line: Vec::new(),
            variations: Vec::new(),
            current_variation_stack: Vec::new(),
            move_counter: 1,
            is_white_turn: true,
            current_position: ScidPosition::new_starting_position(),
            position_state_manager: PositionStateManager::new(),
        }
    }

    pub fn build_tree(
        &mut self,
        elements: &[VariationGameElement],
    ) -> Result<VariationTree, String> {
        for element in elements {
            match element {
                VariationGameElement::Move { piece_num, .. } => {
                    // Use position-aware move decoding to get proper ScidMove
                    // For now, create a placeholder until we integrate with the decoder
                    let scid_move = ScidMove {
                        from: Square(0),                  // Will be filled by position decoder
                        to: Square(8),                    // Will be filled by position decoder
                        moving_piece: PieceType::Pawn,    // Will be filled by position decoder
                        captured_piece: PieceType::Empty, // Will be filled by position decoder
                        promote: PieceType::Empty,        // Will be filled by position decoder
                        piece_num: *piece_num,
                    };

                    // Generate algebraic notation using current position
                    let algebraic = scid_move.to_algebraic(&self.current_position);

                    let variation_move = VariationMove {
                        chess_move: scid_move.clone(),
                        move_number: if self.is_white_turn {
                            self.move_counter
                        } else {
                            self.move_counter
                        },
                        is_white_move: self.is_white_turn,
                        comments: Vec::new(),
                        nags: Vec::new(),
                        algebraic,
                    };

                    // Apply move to position for accurate tracking
                    if let Err(e) = self.current_position.do_move(&scid_move) {
                        eprintln!("Warning: Failed to apply move: {}", e);
                    }

                    if self.current_variation_stack.is_empty() {
                        // Main line move
                        self.main_line.push(variation_move);
                    } else {
                        // Variation move
                        if let Some(current_var) = self.current_variation_stack.last_mut() {
                            current_var.moves.push(variation_move);
                        }
                    }

                    // Update move counters
                    if !self.is_white_turn {
                        self.move_counter += 1;
                    }
                    self.is_white_turn = !self.is_white_turn;
                }

                VariationGameElement::VariationStart { depth, .. } => {
                    // Save current position state before starting variation
                    self.position_state_manager.save_state(
                        &self.current_position,
                        self.move_counter,
                        self.is_white_turn,
                    );

                    let start_index = if self.current_variation_stack.is_empty() {
                        self.main_line.len().saturating_sub(1)
                    } else {
                        self.current_variation_stack
                            .last()
                            .map(|v| v.moves.len().saturating_sub(1))
                            .unwrap_or(0)
                    };

                    let new_variation = VariationInProgress {
                        start_move_index: start_index,
                        moves: Vec::new(),
                        sub_variations: Vec::new(),
                        depth: *depth,
                    };

                    self.current_variation_stack.push(new_variation);
                }

                VariationGameElement::VariationEnd { .. } => {
                    if let Some(completed_variation) = self.current_variation_stack.pop() {
                        let variation = Variation {
                            start_move_index: completed_variation.start_move_index,
                            moves: completed_variation.moves,
                            sub_variations: completed_variation.sub_variations,
                            depth: completed_variation.depth,
                        };

                        if let Some(parent_variation) = self.current_variation_stack.last_mut() {
                            parent_variation.sub_variations.push(variation);
                        } else {
                            self.variations.push(variation);
                        }
                    }

                    // Restore position state after finishing variation
                    if let Some((restored_position, restored_move, restored_turn)) =
                        self.position_state_manager.restore_state()
                    {
                        self.current_position = restored_position;
                        self.move_counter = restored_move;
                        self.is_white_turn = restored_turn;
                    }
                }

                VariationGameElement::Comment { text, .. } => {
                    if let Some(last_move) = self.get_last_move_mut() {
                        last_move.comments.push(text.clone());
                    }
                }

                VariationGameElement::Nag { nag_value, .. } => {
                    if let Some(last_move) = self.get_last_move_mut() {
                        last_move.nags.push(*nag_value);
                    }
                }

                VariationGameElement::GameEnd { .. } => {
                    // Game ended - finalize any remaining variations
                    while !self.current_variation_stack.is_empty() {
                        if let Some(completed_variation) = self.current_variation_stack.pop() {
                            let variation = Variation {
                                start_move_index: completed_variation.start_move_index,
                                moves: completed_variation.moves,
                                sub_variations: completed_variation.sub_variations,
                                depth: completed_variation.depth,
                            };
                            self.variations.push(variation);
                        }
                    }
                    break;
                }
            }
        }

        Ok(VariationTree {
            main_line: self.main_line.clone(),
            variations: self.variations.clone(),
        })
    }

    fn get_last_move_mut(&mut self) -> Option<&mut VariationMove> {
        if let Some(current_var) = self.current_variation_stack.last_mut() {
            current_var.moves.last_mut()
        } else {
            self.main_line.last_mut()
        }
    }
}
