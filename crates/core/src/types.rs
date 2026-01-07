//! Core type definitions for SCID database structures

// Re-export shakmaty types for chess operations
pub use shakmaty::{Chess, Color, Move, Piece, Position, Role, Square};

/// Game date representation
///
/// Dates in SCID are stored as packed binary values at offset 25-28 in each
/// 47-byte game entry. They use 20-bit absolute encoding:
/// - Bits 19-9: Year (0-2047)
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GameDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl GameDate {
    /// Create a new GameDate
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        debug_assert!(month <= 12, "Invalid month: {}", month);
        debug_assert!(day <= 31, "Invalid day: {}", day);
        debug_assert!(year < 2048, "Year too large: {}", year);

        GameDate { year, month, day }
    }

    /// Check if the date is valid
    pub fn is_valid(&self) -> bool {
        self.month >= 1 && self.month <= 12 && self.day >= 1 && self.day <= 31 && self.year < 2048
    }

    /// Format as PGN date string (YYYY.MM.DD)
    pub fn to_pgn_string(&self) -> String {
        format!("{}.{:02}.{:02}", self.year, self.month, self.day)
    }

    /// Format with unknown components as "??"
    pub fn to_pgn_string_with_unknowns(&self) -> String {
        let year = if self.year == 0 {
            "????".to_string()
        } else {
            self.year.to_string()
        };

        let month = if self.month == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.month)
        };

        let day = if self.day == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.day)
        };

        format!("{}.{}.{}", year, month, day)
    }
}

/// Game result
///
/// Results are encoded in the upper 4 bits of a 16-bit field at offset 21-22:
/// - 0 = Unknown/Ongoing (*)
/// - 1 = White wins (1-0)
/// - 2 = Black wins (0-1)
/// - 3 = Draw (1/2-1/2)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
    #[default]
    Unknown,
}

impl GameResult {
    /// Create from SCID encoding value
    pub fn from_scid(value: u8) -> Self {
        match value {
            1 => GameResult::WhiteWins,
            2 => GameResult::BlackWins,
            3 => GameResult::Draw,
            _ => GameResult::Unknown,
        }
    }

    /// Convert to PGN result string
    pub fn to_pgn(&self) -> &'static str {
        match self {
            GameResult::WhiteWins => "1-0",
            GameResult::BlackWins => "0-1",
            GameResult::Draw => "1/2-1/2",
            GameResult::Unknown => "*",
        }
    }
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_pgn())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_date_creation() {
        let date = GameDate::new(2022, 12, 19);
        assert_eq!(date.year, 2022);
        assert_eq!(date.month, 12);
        assert_eq!(date.day, 19);
    }

    #[test]
    fn test_game_date_validation() {
        assert!(GameDate::new(2022, 12, 19).is_valid());
        assert!(GameDate::new(2047, 1, 1).is_valid());
        // Invalid dates constructed directly (not via new() which has debug_assert)
        assert!(!GameDate {
            year: 2022,
            month: 13,
            day: 1
        }
        .is_valid());
        assert!(!GameDate {
            year: 2022,
            month: 0,
            day: 1
        }
        .is_valid());
        assert!(!GameDate {
            year: 2022,
            month: 12,
            day: 0
        }
        .is_valid());
        assert!(!GameDate {
            year: 2022,
            month: 12,
            day: 32
        }
        .is_valid());
    }

    #[test]
    fn test_game_date_pgn_string() {
        let date = GameDate::new(2022, 12, 19);
        assert_eq!(date.to_pgn_string(), "2022.12.19");

        let date2 = GameDate::new(2000, 1, 5);
        assert_eq!(date2.to_pgn_string(), "2000.01.05");
    }

    #[test]
    fn test_game_date_unknown_components() {
        let date = GameDate {
            year: 2022,
            month: 0,
            day: 15,
        };
        assert_eq!(date.to_pgn_string_with_unknowns(), "2022.??.15");

        let date2 = GameDate {
            year: 0,
            month: 0,
            day: 0,
        };
        assert_eq!(date2.to_pgn_string_with_unknowns(), "????.??.??");
    }

    #[test]
    fn test_game_date_default() {
        let date = GameDate::default();
        assert_eq!(date.year, 0);
        assert_eq!(date.month, 0);
        assert_eq!(date.day, 0);
    }

    #[test]
    fn test_game_result_from_scid() {
        assert_eq!(GameResult::from_scid(0), GameResult::Unknown);
        assert_eq!(GameResult::from_scid(1), GameResult::WhiteWins);
        assert_eq!(GameResult::from_scid(2), GameResult::BlackWins);
        assert_eq!(GameResult::from_scid(3), GameResult::Draw);
        assert_eq!(GameResult::from_scid(99), GameResult::Unknown);
    }

    #[test]
    fn test_game_result_to_pgn() {
        assert_eq!(GameResult::WhiteWins.to_pgn(), "1-0");
        assert_eq!(GameResult::BlackWins.to_pgn(), "0-1");
        assert_eq!(GameResult::Draw.to_pgn(), "1/2-1/2");
        assert_eq!(GameResult::Unknown.to_pgn(), "*");
    }

    #[test]
    fn test_game_result_display() {
        assert_eq!(GameResult::WhiteWins.to_string(), "1-0");
        assert_eq!(GameResult::Draw.to_string(), "1/2-1/2");
    }

    #[test]
    fn test_game_result_default() {
        assert_eq!(GameResult::default(), GameResult::Unknown);
    }

    #[test]
    fn test_shakmaty_reexports() {
        let _square: Square = Square::A1;
        let _color: Color = Color::White;
        let _role: Role = Role::King;
    }
}
