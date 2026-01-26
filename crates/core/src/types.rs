//! Core type definitions for SCID database structures
//!
//! This module defines types used throughout the library. Chess-related types
//! are re-exported from the `shakmaty` library, while SCID-specific types are
//! defined here.

// Re-export shakmaty types for chess operations
// These are used in later phases for move parsing
#[allow(unused_imports)]
pub use shakmaty::{Color, Role, Square};

/// Game date representation
///
/// Dates in SCID are stored as packed binary values. This struct provides
/// a more ergonomic representation.
///
/// # SCID Format Reference
///
/// Game dates are stored in the index file (.si4) at offset 25-28 in each
/// 47-byte game entry. They use 20-bit absolute encoding:
/// - Bits 19-9: Year (0-2047)
/// - Bits 8-5: Month (1-12)
/// - Bits 4-0: Day (1-31)
///
/// See SCID_DATABASE_FORMAT.md lines 225-252 for complete specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GameDate {
    /// Year (e.g., 2022)
    pub year: u16,
    /// Month (1-12)
    pub month: u8,
    /// Day (1-31)
    pub day: u8,
}

impl GameDate {
    /// Create a new GameDate
    pub fn new(year: u16, month: u8, day: u8) -> Self {
        GameDate { year, month, day }
    }

    /// Check if the date is valid
    ///
    /// Returns true if month and day are in valid ranges.
    /// Does not check if the specific day is valid for the month
    /// (e.g., doesn't validate Feb 30).
    pub fn is_valid(&self) -> bool {
        self.month >= 1 && self.month <= 12 && self.day >= 1 && self.day <= 31 && self.year < 2048
    }

    /// Convert to PGN date string format
    ///
    /// PGN standard requires unknown components to use "??" or "????":
    /// - Unknown year: "????.MM.DD"
    /// - Unknown month: "YYYY.??.DD"
    /// - Unknown day: "YYYY.MM.??"
    /// - Fully unknown: "????.??.??"
    ///
    /// SCID uses 0 to indicate unknown date components.
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::GameDate;
    ///
    /// let date = GameDate::new(2022, 12, 25);
    /// assert_eq!(date.to_pgn_string(), "2022.12.25");
    ///
    /// let partial = GameDate { year: 1997, month: 5, day: 0 };
    /// assert_eq!(partial.to_pgn_string(), "1997.05.??");
    ///
    /// let unknown = GameDate { year: 0, month: 0, day: 0 };
    /// assert_eq!(unknown.to_pgn_string(), "????.??.??");
    /// ```
    pub fn to_pgn_string(&self) -> String {
        let year_str = if self.year == 0 {
            "????".to_string()
        } else {
            format!("{:04}", self.year)
        };

        let month_str = if self.month == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.month)
        };

        let day_str = if self.day == 0 {
            "??".to_string()
        } else {
            format!("{:02}", self.day)
        };

        format!("{}.{}.{}", year_str, month_str, day_str)
    }
}

/// Game result
///
/// Represents the outcome of a chess game. This is stored in the index file
/// as part of the variation counts field (bits 15-12).
///
/// # SCID Format Reference
///
/// Results are encoded in the upper 4 bits of a 16-bit field at offset 21-22
/// in each game index entry:
/// - 0 = Unknown/Ongoing (*)
/// - 1 = White wins (1-0)
/// - 2 = Black wins (0-1)
/// - 3 = Draw (1/2-1/2)
///
/// See SCID_DATABASE_FORMAT.md lines 174-187 for complete specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameResult {
    /// White wins (1-0)
    WhiteWins,
    /// Black wins (0-1)
    BlackWins,
    /// Draw (1/2-1/2)
    Draw,
    /// Unknown or ongoing (*)
    #[default]
    Unknown,
}

impl GameResult {
    /// Create from SCID encoding value
    ///
    /// # Arguments
    ///
    /// * `value` - The 4-bit result encoding from SCID (0-3)
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::GameResult;
    ///
    /// assert_eq!(GameResult::from_scid(1), GameResult::WhiteWins);
    /// assert_eq!(GameResult::from_scid(2), GameResult::BlackWins);
    /// assert_eq!(GameResult::from_scid(3), GameResult::Draw);
    /// assert_eq!(GameResult::from_scid(0), GameResult::Unknown);
    /// ```
    pub fn from_scid(value: u8) -> Self {
        match value {
            1 => GameResult::WhiteWins,
            2 => GameResult::BlackWins,
            3 => GameResult::Draw,
            _ => GameResult::Unknown,
        }
    }

    /// Convert to PGN result string
    ///
    /// # Examples
    ///
    /// ```
    /// use scidtopgn_core::GameResult;
    ///
    /// assert_eq!(GameResult::WhiteWins.to_pgn(), "1-0");
    /// assert_eq!(GameResult::BlackWins.to_pgn(), "0-1");
    /// assert_eq!(GameResult::Draw.to_pgn(), "1/2-1/2");
    /// assert_eq!(GameResult::Unknown.to_pgn(), "*");
    /// ```
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
        assert!(!GameDate::new(2022, 13, 1).is_valid());
        assert!(!GameDate::new(2022, 0, 1).is_valid());
        assert!(!GameDate::new(2022, 12, 0).is_valid());
        assert!(!GameDate::new(2022, 12, 32).is_valid());
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
        // Unknown day
        let date = GameDate {
            year: 2022,
            month: 5,
            day: 0,
        };
        assert_eq!(date.to_pgn_string(), "2022.05.??");

        // Unknown month and day
        let date2 = GameDate {
            year: 1997,
            month: 0,
            day: 0,
        };
        assert_eq!(date2.to_pgn_string(), "1997.??.??");

        // Fully unknown
        let date3 = GameDate {
            year: 0,
            month: 0,
            day: 0,
        };
        assert_eq!(date3.to_pgn_string(), "????.??.??");
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
        // Verify we can use re-exported types
        let _square: Square = Square::A1;
        let _color: Color = Color::White;
        let _role: Role = Role::King;
    }
}
