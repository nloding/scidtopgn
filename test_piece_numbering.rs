use shakmaty::{Board, Chess, Color, Move, Square};
use std::collections::HashMap;

pub struct PieceNumberMapping {
    white_pieces: HashMap<u8, Square>,
    black_pieces: HashMap<u8, Square>,
    white_count: u8,
    black_count: u8,
}

impl PieceNumberMapping {
    pub fn standard_start() -> Self {
        let mut mapping = Self {
            white_pieces: HashMap::with_capacity(16),
            black_pieces: HashMap::with_capacity(16),
            white_count: 16,
            black_count: 16,
        };

        mapping.white_pieces.insert(0, Square::E1);
        mapping.white_pieces.insert(1, Square::A1);
        mapping.white_pieces.insert(2, Square::B1);
        mapping.white_pieces.insert(3, Square::C1);
        mapping.white_pieces.insert(4, Square::D1);
        mapping.white_pieces.insert(5, Square::F1);
        mapping.white_pieces.insert(6, Square::G1);
        mapping.white_pieces.insert(7, Square::H1);

        for file in 0..8 {
            mapping.white_pieces.insert(
                8 + file,
                Square::from_coords(shakmaty::File::new(file as u32), shakmaty::Rank::Second),
            );
        }

        mapping.black_pieces.insert(0, Square::E8);
        mapping.black_pieces.insert(1, Square::A8);
        mapping.black_pieces.insert(2, Square::B8);
        mapping.black_pieces.insert(3, Square::C8);
        mapping.black_pieces.insert(4, Square::D8);
        mapping.black_pieces.insert(5, Square::F8);
        mapping.black_pieces.insert(6, Square::G8);
        mapping.black_pieces.insert(7, Square::H8);

        for file in 0..8 {
            mapping.black_pieces.insert(
                8 + file,
                Square::from_coords(shakmaty::File::new(file as u32), shakmaty::Rank::Seventh),
            );
        }

        mapping
    }

    pub fn from_position(chess: &Chess) -> Self {
        let mut mapping = Self {
            white_pieces: HashMap::new(),
            black_pieces: HashMap::new(),
            white_count: 0,
            black_count: 0,
        };

        for square in chess.board().by_role(shakmaty::Role::King) {
            if let Some(piece) = chess.board().piece_at(square) {
                if piece.color == Color::White {
                    mapping.white_pieces.insert(0, square);
                } else {
                    mapping.black_pieces.insert(0, square);
                }
            }
        }

        let mut white_num = 1u8;
        let mut black_num = 1u8;

        for rank in (0..8u32).rev() {
            for file in 0..8u32 {
                let square =
                    Square::from_coords(shakmaty::File::new(file), shakmaty::Rank::new(rank));

                if let Some(piece) = chess.board().piece_at(square) {
                    if piece.role == shakmaty::Role::King {
                        continue;
                    }

                    if piece.color == Color::White {
                        mapping.white_pieces.insert(white_num, square);
                        white_num += 1;
                    } else {
                        mapping.black_pieces.insert(black_num, square);
                        black_num += 1;
                    }
                }
            }
        }

        mapping.white_count = white_num;
        mapping.black_count = black_num;

        mapping
    }

    pub fn get_square(&self, piece_num: u8, color: Color) -> Option<Square> {
        match color {
            Color::White => self.white_pieces.get(&piece_num).copied(),
            Color::Black => self.black_pieces.get(&piece_num).copied(),
        }
    }

    pub fn update_after_move(&mut self, chess_move: &Move, color: Color) {
        if chess_move.is_capture() {
            let capture_square = if chess_move.is_en_passant() {
                let target = chess_move.to();
                let ep_rank = match color {
                    Color::White => shakmaty::Rank::Fifth,
                    Color::Black => shakmaty::Rank::Fourth,
                };
                Square::from_coords(target.file(), ep_rank)
            } else {
                chess_move.to()
            };

            let (enemy_pieces, enemy_count) = match color {
                Color::White => (&mut self.black_pieces, &mut self.black_count),
                Color::Black => (&mut self.white_pieces, &mut self.white_count),
            };

            let captured_slot: Option<u8> = enemy_pieces
                .iter()
                .find(|(_, &sq)| sq == capture_square)
                .map(|(&num, _)| num);

            if let Some(captured_num) = captured_slot {
                *enemy_count -= 1;
                let last_slot = *enemy_count;

                if captured_num != last_slot {
                    if let Some(&last_square) = enemy_pieces.get(&last_slot) {
                        enemy_pieces.insert(captured_num, last_square);
                    }
                }

                enemy_pieces.remove(&last_slot);
            }
        }

        let own_pieces = match color {
            Color::White => &mut self.white_pieces,
            Color::Black => &mut self.black_pieces,
        };

        match chess_move {
            Move::Normal { from, to, .. } => {
                let piece_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
            }
            Move::Castle { king, rook } => {
                let king_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::G, king.rank())
                } else {
                    Square::from_coords(shakmaty::File::C, king.rank())
                };

                let rook_to = if rook.file() > king.file() {
                    Square::from_coords(shakmaty::File::F, king.rank())
                } else {
                    Square::from_coords(shakmaty::File::D, king.rank())
                };

                own_pieces.insert(0, king_to);

                let rook_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *rook)
                    .map(|(&num, _)| num);

                if let Some(num) = rook_num {
                    own_pieces.insert(num, rook_to);
                }
            }
            Move::EnPassant { from, to } => {
                let piece_num = own_pieces
                    .iter()
                    .find(|(_, &sq)| sq == *from)
                    .map(|(&num, _)| num);

                if let Some(num) = piece_num {
                    own_pieces.insert(num, *to);
                }
            }
            Move::Put { .. } => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_piece_numbering() {
        let mapping = PieceNumberMapping::standard_start();

        assert_eq!(mapping.get_square(0, Color::White), Some(Square::E1));
        assert_eq!(mapping.get_square(1, Color::White), Some(Square::A1));
        assert_eq!(mapping.get_square(2, Color::White), Some(Square::B1));
        assert_eq!(mapping.get_square(3, Color::White), Some(Square::C1));
        assert_eq!(mapping.get_square(4, Color::White), Some(Square::D1));
        assert_eq!(mapping.get_square(5, Color::White), Some(Square::F1));
        assert_eq!(mapping.get_square(6, Color::White), Some(Square::G1));
        assert_eq!(mapping.get_square(7, Color::White), Some(Square::H1));
        assert_eq!(mapping.get_square(8, Color::White), Some(Square::A2));
        assert_eq!(mapping.get_square(15, Color::White), Some(Square::H2));

        assert_eq!(mapping.get_square(0, Color::Black), Some(Square::E8));
        assert_eq!(mapping.get_square(1, Color::Black), Some(Square::A8));
        assert_eq!(mapping.get_square(4, Color::Black), Some(Square::D8));
    }

    #[test]
    fn test_piece_mapping_after_move() {
        let mut mapping = PieceNumberMapping::standard_start();

        let chess_move = Move::Normal {
            role: shakmaty::Role::Pawn,
            from: Square::E2,
            capture: None,
            to: Square::E4,
            promotion: None,
        };

        mapping.update_after_move(&chess_move, Color::White);

        assert_eq!(mapping.get_square(12, Color::White), Some(Square::E4));
    }
}
