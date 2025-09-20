
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub struct Square(pub u8);

impl Square {
    pub fn from_algebraic(s: &str) -> Result<Self, String> {
        if s.len() != 2 {
            return Err(format!("Invalid square notation: {}", s));
        }
        
        let chars: Vec<char> = s.chars().collect();
        let file_char = chars[0];
        let rank_char = chars[1];
        
        let file = match file_char {
            'a'..='h' => file_char as u8 - b'a',
            _ => return Err(format!("Invalid file: {}", file_char)),
        };
        
        let rank = match rank_char {
            '1'..='8' => rank_char as u8 - b'1',
            _ => return Err(format!("Invalid rank: {}", rank_char)),
        };
        
        Ok(Square(rank * 8 + file))
    }
    
    pub fn to_algebraic(&self) -> String {
        let file = self.0 % 8;
        let rank = self.0 / 8;
        
        let file_char = (b'a' + file) as char;
        let rank_char = (b'1' + rank) as char;
        
        format!("{}{}", file_char, rank_char)
    }
}
