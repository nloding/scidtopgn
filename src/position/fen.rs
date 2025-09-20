use crate::position::board::ScidPosition;

pub struct Fen;

impl Fen {
    pub fn from_position(_position: &ScidPosition) -> String {
        // Placeholder
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
    }
}
