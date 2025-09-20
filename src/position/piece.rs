use crate::position::moves::{PieceType, Square};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: crate::position::moves::Color,
}
