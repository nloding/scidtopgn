// SCID-compatible move representation
// Based on scidvspc/src/common.h simpleMoveT

/// SCID-compatible move representation
/// Based on scidvspc/src/common.h simpleMoveT
#[derive(Debug, Clone, PartialEq)]
pub struct ScidMove {
    /// Source square (where piece moves from)
    pub from: Square,

    /// Destination square (where piece moves to)  
    pub to: Square,

    /// Piece that is moving
    pub moving_piece: PieceType,

    /// Piece that is captured (EMPTY if no capture)
    pub captured_piece: PieceType,

    /// Promotion piece (EMPTY if no promotion)
    pub promote: PieceType,

    /// SCID piece number (0-15 for current player)
    pub piece_num: u8,
}

impl ScidMove {
    /// Generate algebraic notation for this move
    /// Examples: "e4", "Nf3", "O-O", "exd5", "e8=Q"
    pub fn to_algebraic(&self, _position: &crate::position::ScidPosition) -> String {
        // Handle castling
        if self.moving_piece == PieceType::King {
            let king_move_distance = (self.to.0 as i8) - (self.from.0 as i8);
            if king_move_distance == 2 {
                return "O-O".to_string(); // Kingside castling
            } else if king_move_distance == -2 {
                return "O-O-O".to_string(); // Queenside castling
            }
        }

        let mut notation = String::new();

        // Add piece letter (except for pawns)
        match self.moving_piece {
            PieceType::King => notation.push('K'),
            PieceType::Queen => notation.push('Q'),
            PieceType::Rook => notation.push('R'),
            PieceType::Bishop => notation.push('B'),
            PieceType::Knight => notation.push('N'),
            PieceType::Pawn => {
                // For pawn captures, add the file
                if self.captured_piece != PieceType::Empty {
                    notation.push(self.from.file_char());
                }
            }
            _ => {}
        }

        // Add capture indicator
        if self.captured_piece != PieceType::Empty {
            notation.push('x');
        }

        // Add destination square
        notation.push_str(&self.to.to_algebraic());

        // Add promotion
        if self.promote != PieceType::Empty {
            notation.push('=');
            match self.promote {
                PieceType::Queen => notation.push('Q'),
                PieceType::Rook => notation.push('R'),
                PieceType::Bishop => notation.push('B'),
                PieceType::Knight => notation.push('N'),
                _ => {}
            }
        }

        notation
    }

    /// Validate move is legal in given position
    pub fn is_legal(&self, position: &crate::position::ScidPosition) -> bool {
        // Basic validation for now
        position.is_move_legal(self)
    }
}

/// Square representation (0-63)
/// SCID square numbering: a1=0, b1=1, c1=2, ..., h8=63
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Square(pub u8);

impl Square {
    /// Convert from algebraic notation (e.g., "e4" -> Square(28))
    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        if s.len() != 2 {
            return Err(format!("Invalid square notation: {}", s));
        }

        let chars: Vec<char> = s.chars().collect();
        let file_char = chars[0];
        let rank_char = chars[1];

        // Parse file (a-h -> 0-7)
        let file = match file_char {
            'a' => 0,
            'b' => 1,
            'c' => 2,
            'd' => 3,
            'e' => 4,
            'f' => 5,
            'g' => 6,
            'h' => 7,
            _ => return Err(format!("Invalid file: {}", file_char)),
        };

        // Parse rank (1-8 -> 0-7)
        let rank = match rank_char {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
            _ => return Err(format!("Invalid rank: {}", rank_char)),
        };

        // Convert to SCID square number: rank * 8 + file
        Ok(Square(rank * 8 + file))
    }

    /// Convert to algebraic notation (e.g., Square(28) -> "e4")
    pub fn to_algebraic(&self) -> String {
        let file = self.file();
        let rank = self.rank();

        let file_char = match file {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '?',
        };

        let rank_char = (rank + 1).to_string();

        format!("{}{}", file_char, rank_char)
    }

    /// Get file (0-7 for a-h)
    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    /// Get rank (0-7 for 1-8)  
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    /// Get file as character
    pub fn file_char(&self) -> char {
        match self.file() {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
            _ => '?',
        }
    }
}

// Constants for piece types
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum PieceType {
    Empty,
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

impl PieceType {
    pub fn to_string(&self) -> &'static str {
        match self {
            PieceType::Empty => "Empty",
            PieceType::King => "King",
            PieceType::Queen => "Queen",
            PieceType::Rook => "Rook",
            PieceType::Bishop => "Bishop",
            PieceType::Knight => "Knight",
            PieceType::Pawn => "Pawn",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match self {
            Color::White => "White",
            Color::Black => "Black",
        }
    }
}

// Conversion implementations to shakmaty types
impl From<Square> for shakmaty::Square {
    fn from(square: Square) -> Self {
        shakmaty::Square::new(square.0 as u32)
    }
}

impl From<Color> for shakmaty::Color {
    fn from(color: Color) -> Self {
        match color {
            Color::White => shakmaty::Color::White,
            Color::Black => shakmaty::Color::Black,
        }
    }
}
