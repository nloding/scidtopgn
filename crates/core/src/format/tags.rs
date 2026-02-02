use crate::database::{eco_to_string, GameIndexEntry, NameDatabase, RatingType};
use crate::types::{GameDate, GameResult};
use std::collections::HashMap;

/// Format Seven Tag Roster according to PGN specification
///
/// The Seven Tag Roster (STR) consists of the required tags
/// that must appear in every PGN game in a specific order.
pub struct SevenTagRoster {
    pub event: String,
    pub site: String,
    pub date: String,
    pub round: String,
    pub white: String,
    pub black: String,
    pub result: String,
}

impl SevenTagRoster {
    /// Create Seven Tag Roster from SCID index and name data
    pub fn from_scid(index_entry: &GameIndexEntry, names: &NameDatabase) -> Self {
        SevenTagRoster {
            event: Self::get_name(&names.events, index_entry.event_id),
            site: Self::get_name(&names.sites, index_entry.site_id),
            date: Self::format_date(&index_entry.game_date),
            round: Self::get_name(&names.rounds, index_entry.round_id),
            white: Self::get_name(&names.players, index_entry.white_id),
            black: Self::get_name(&names.players, index_entry.black_id),
            result: Self::format_result(&index_entry.result),
        }
    }

    /// Get name from name database by ID, with "?" fallback
    fn get_name(names: &[String], id: u32) -> String {
        names
            .get(id as usize)
            .filter(|s| !s.is_empty())
            .cloned()
            .unwrap_or_else(|| "?".to_string())
    }

    /// Format game date as YYYY.MM.DD
    ///
    /// Unknown components use "?" placeholder:
    /// - Full unknown: "????.??.??"
    /// - Partial unknown: "2023.??.??" or "2023.12.??"
    fn format_date(date: &GameDate) -> String {
        let year = if date.year > 0 {
            format!("{:04}", date.year)
        } else {
            "????".to_string()
        };

        let month = if date.month > 0 && date.month <= 12 {
            format!("{:02}", date.month)
        } else {
            "??".to_string()
        };

        let day = if date.day > 0 && date.day <= 31 {
            format!("{:02}", date.day)
        } else {
            "??".to_string()
        };

        format!("{}.{}.{}", year, month, day)
    }

    /// Format game result
    fn format_result(result: &GameResult) -> String {
        result.to_string().to_string()
    }

    /// Format as PGN tag lines
    ///
    /// Returns the seven tags in correct order, one per line.
    /// Does NOT include trailing blank line.
    pub fn to_pgn(&self) -> String {
        let mut pgn = String::new();

        pgn.push_str(&Self::format_tag("Event", &self.event));
        pgn.push_str(&Self::format_tag("Site", &self.site));
        pgn.push_str(&Self::format_tag("Date", &self.date));
        pgn.push_str(&Self::format_tag("Round", &self.round));
        pgn.push_str(&Self::format_tag("White", &self.white));
        pgn.push_str(&Self::format_tag("Black", &self.black));
        pgn.push_str(&Self::format_tag("Result", &self.result));

        pgn
    }

    /// Format a single PGN tag
    ///
    /// Format: `[<TagName> "<TagValue>"]\n`
    ///
    /// Escapes double quotes in tag value: `"` → `\"`
    fn format_tag(name: &str, value: &str) -> String {
        let escaped_value = value.replace('"', "\\\"");
        format!("[{} \"{}\"]\n", name, escaped_value)
    }
}

/// Get the appropriate PGN tag name for a rating based on its type
///
/// Different rating systems use different tag names in PGN:
/// - Standard Elo: WhiteElo / BlackElo
/// - USCF: WhiteUSCF / BlackUSCF
/// - Rapid: WhiteRapidElo / BlackRapidElo
/// - ICCF: WhiteICCF / BlackICCF
/// - DWZ: WhiteDWZ / BlackDWZ
/// - ECF: WhiteECF / BlackECF
///
/// # Arguments
///
/// * `color` - "White" or "Black"
/// * `rating_type` - The rating system type
///
/// # Returns
///
/// The appropriate PGN tag name (e.g., "WhiteElo", "BlackUSCF")
fn rating_tag_name(color: &str, rating_type: RatingType) -> &'static str {
    match (color, rating_type) {
        ("White", RatingType::None) | ("White", RatingType::Elo) => "WhiteElo",
        ("White", RatingType::Rapid) => "WhiteRapidElo",
        ("White", RatingType::Iccf) => "WhiteICCF",
        ("White", RatingType::Uscf) => "WhiteUSCF",
        ("White", RatingType::Dwz) => "WhiteDWZ",
        ("White", RatingType::Ecf) => "WhiteECF",
        ("White", RatingType::Unknown) => "WhiteElo",
        ("Black", RatingType::None) | ("Black", RatingType::Elo) => "BlackElo",
        ("Black", RatingType::Rapid) => "BlackRapidElo",
        ("Black", RatingType::Iccf) => "BlackICCF",
        ("Black", RatingType::Uscf) => "BlackUSCF",
        ("Black", RatingType::Dwz) => "BlackDWZ",
        ("Black", RatingType::Ecf) => "BlackECF",
        ("Black", RatingType::Unknown) => "BlackElo",
        _ => "WhiteElo", // Fallback (shouldn't happen)
    }
}

/// Supplemental PGN tags beyond the Seven Tag Roster
#[derive(Debug, Clone, Default)]
pub struct SupplementalTags {
    pub white_elo: Option<u16>,
    pub black_elo: Option<u16>,
    /// Rating type for white (Gap 15: enables proper tag names like WhiteUSCF)
    pub white_rating_type: RatingType,
    /// Rating type for black (Gap 15: enables proper tag names like BlackUSCF)
    pub black_rating_type: RatingType,
    pub eco_code: Option<String>,
    pub opening: Option<String>,
    pub variation: Option<String>,
    pub event_date: Option<String>,
    pub fen: Option<String>,
    pub ply_count: Option<u16>,
    pub time_control: Option<String>,
    pub termination: Option<String>,
    pub annotator: Option<String>,
    pub custom_tags: HashMap<String, String>,
}

impl SupplementalTags {
    /// Create supplemental tags from SCID index and game data
    pub fn from_scid(
        index_entry: &GameIndexEntry,
        game_tags: &HashMap<String, String>,
        fen: Option<&str>,
    ) -> Self {
        let mut tags = SupplementalTags::default();

        // Ratings with type information (Gap 15)
        // Use helper methods to get both value and type
        if index_entry.has_white_rating() {
            tags.white_elo = Some(index_entry.white_elo);
            tags.white_rating_type = index_entry.white_rating_type();
        }
        if index_entry.has_black_rating() {
            tags.black_elo = Some(index_entry.black_elo);
            tags.black_rating_type = index_entry.black_rating_type();
        }

        // ECO code using shared eco_to_string() from Phase 2 (Gap 4)
        // Returns None if eco_code is 0 or invalid
        tags.eco_code = eco_to_string(index_entry.eco_code);

        // FEN for non-standard starts
        if let Some(fen_str) = fen {
            tags.fen = Some(fen_str.to_string());
        }

        // Ply count (half-moves)
        if index_entry.half_moves > 0 {
            tags.ply_count = Some(index_entry.half_moves);
        }

        // Copy custom tags from game file
        tags.custom_tags = game_tags.clone();

        tags
    }

    // NOTE: format_eco_code() removed - use eco_to_string() from database module (Gap 4)

    /// Format as PGN tag lines
    ///
    /// Returns supplemental tags in alphabetical order (PGN spec recommendation).
    /// Does NOT include trailing blank line.
    pub fn to_pgn(&self) -> String {
        let mut tags = Vec::new();

        // Annotator
        if let Some(ref annotator) = self.annotator {
            tags.push(("Annotator", annotator.clone()));
        }

        // Black rating (Gap 15: use appropriate tag name based on rating type)
        if let Some(elo) = self.black_elo {
            let tag_name = rating_tag_name("Black", self.black_rating_type);
            tags.push((tag_name, elo.to_string()));
        }

        // ECO
        if let Some(ref eco) = self.eco_code {
            tags.push(("ECO", eco.clone()));
        }

        // EventDate
        if let Some(ref date) = self.event_date {
            tags.push(("EventDate", date.clone()));
        }

        // FEN
        if let Some(ref fen) = self.fen {
            tags.push(("FEN", fen.clone()));
            tags.push(("SetUp", "1".to_string())); // Required with FEN
        }

        // Opening
        if let Some(ref opening) = self.opening {
            tags.push(("Opening", opening.clone()));
        }

        // PlyCount
        if let Some(count) = self.ply_count {
            tags.push(("PlyCount", count.to_string()));
        }

        // Termination
        if let Some(ref term) = self.termination {
            tags.push(("Termination", term.clone()));
        }

        // TimeControl
        if let Some(ref tc) = self.time_control {
            tags.push(("TimeControl", tc.clone()));
        }

        // Variation
        if let Some(ref var) = self.variation {
            tags.push(("Variation", var.clone()));
        }

        // White rating (Gap 15: use appropriate tag name based on rating type)
        if let Some(elo) = self.white_elo {
            let tag_name = rating_tag_name("White", self.white_rating_type);
            tags.push((tag_name, elo.to_string()));
        }

        // Custom tags (alphabetically sorted)
        let mut custom: Vec<_> = self.custom_tags.iter().collect();
        custom.sort_by_key(|(k, _)| k.as_str());
        for (name, value) in custom {
            tags.push((name.as_str(), value.clone()));
        }

        // Format all tags
        tags.iter()
            .map(|(name, value)| SevenTagRoster::format_tag(name, value))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seven_tag_roster_complete() {
        let str = SevenTagRoster {
            event: "World Championship".to_string(),
            site: "New York, NY USA".to_string(),
            date: "1886.01.11".to_string(),
            round: "1".to_string(),
            white: "Steinitz, Wilhelm".to_string(),
            black: "Zukertort, Johannes".to_string(),
            result: "1-0".to_string(),
        };

        let pgn = str.to_pgn();

        assert!(pgn.contains("[Event \"World Championship\"]"));
        assert!(pgn.contains("[Site \"New York, NY USA\"]"));
        assert!(pgn.contains("[Date \"1886.01.11\"]"));
        assert!(pgn.contains("[Round \"1\"]"));
        assert!(pgn.contains("[White \"Steinitz, Wilhelm\"]"));
        assert!(pgn.contains("[Black \"Zukertort, Johannes\"]"));
        assert!(pgn.contains("[Result \"1-0\"]"));

        let event_pos = pgn.find("[Event").unwrap();
        let site_pos = pgn.find("[Site").unwrap();
        let date_pos = pgn.find("[Date").unwrap();
        let result_pos = pgn.find("[Result").unwrap();

        assert!(event_pos < site_pos);
        assert!(site_pos < date_pos);
        assert!(date_pos < result_pos);
    }

    #[test]
    fn test_date_formatting() {
        let date = GameDate {
            year: 2023,
            month: 12,
            day: 25,
        };
        assert_eq!(SevenTagRoster::format_date(&date), "2023.12.25");

        let date = GameDate {
            year: 2023,
            month: 12,
            day: 0,
        };
        assert_eq!(SevenTagRoster::format_date(&date), "2023.12.??");

        let date = GameDate {
            year: 0,
            month: 0,
            day: 0,
        };
        assert_eq!(SevenTagRoster::format_date(&date), "????.??.??");
    }

    #[test]
    fn test_quote_escaping() {
        let str = SevenTagRoster {
            event: "Tournament \"The Best\"".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "O'Brien, James".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let pgn = str.to_pgn();
        assert!(pgn.contains("[Event \"Tournament \\\"The Best\\\"\"]"));
    }

    #[test]
    fn test_unknown_values() {
        let str = SevenTagRoster {
            event: "?".to_string(),
            site: "?".to_string(),
            date: "????.??.??".to_string(),
            round: "?".to_string(),
            white: "?".to_string(),
            black: "?".to_string(),
            result: "*".to_string(),
        };

        let pgn = str.to_pgn();
        assert!(pgn.contains("[Event \"?\"]"));
        assert!(pgn.contains("[Date \"????.??.??\"]"));
        assert!(pgn.contains("[Result \"*\"]"));
    }
}

#[cfg(test)]
mod supplemental_tests {
    use super::*;

    #[test]
    fn test_elo_ratings() {
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2850);
        tags.black_elo = Some(2810);
        // Default rating type is None, which outputs as "WhiteElo"/"BlackElo"

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[BlackElo \"2810\"]"));
        assert!(pgn.contains("[WhiteElo \"2850\"]"));
    }

    #[test]
    fn test_rating_types_uscf() {
        // Gap 15: Test USCF rating type outputs correct tag name
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2100);
        tags.white_rating_type = RatingType::Uscf;
        tags.black_elo = Some(1950);
        tags.black_rating_type = RatingType::Uscf;

        let pgn = tags.to_pgn();
        assert!(
            pgn.contains("[WhiteUSCF \"2100\"]"),
            "Should use WhiteUSCF tag"
        );
        assert!(
            pgn.contains("[BlackUSCF \"1950\"]"),
            "Should use BlackUSCF tag"
        );
        assert!(!pgn.contains("WhiteElo"), "Should NOT use WhiteElo");
    }

    #[test]
    fn test_rating_types_mixed() {
        // Gap 15: Test mixed rating types (one player Elo, one USCF)
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2500);
        tags.white_rating_type = RatingType::Elo;
        tags.black_elo = Some(2100);
        tags.black_rating_type = RatingType::Uscf;

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[WhiteElo \"2500\"]"));
        assert!(pgn.contains("[BlackUSCF \"2100\"]"));
    }

    #[test]
    fn test_rating_tag_name_function() {
        // Gap 15: Test the rating_tag_name helper function
        assert_eq!(rating_tag_name("White", RatingType::Elo), "WhiteElo");
        assert_eq!(rating_tag_name("Black", RatingType::Elo), "BlackElo");
        assert_eq!(rating_tag_name("White", RatingType::Uscf), "WhiteUSCF");
        assert_eq!(rating_tag_name("Black", RatingType::Uscf), "BlackUSCF");
        assert_eq!(rating_tag_name("White", RatingType::Rapid), "WhiteRapidElo");
        assert_eq!(rating_tag_name("Black", RatingType::Dwz), "BlackDWZ");
        assert_eq!(rating_tag_name("White", RatingType::Ecf), "WhiteECF");
        assert_eq!(rating_tag_name("Black", RatingType::Iccf), "BlackICCF");
        assert_eq!(rating_tag_name("White", RatingType::None), "WhiteElo"); // Default
    }

    #[test]
    fn test_eco_code_formatting() {
        // Using eco_to_string() from database module (Gap 4)
        use crate::database::eco_to_string;

        // eco_to_string returns None for 0 (no ECO assigned)
        assert_eq!(eco_to_string(0), None);

        // Valid ECO codes
        assert_eq!(eco_to_string(1), Some("A01".to_string()));
        assert_eq!(eco_to_string(99), Some("A99".to_string()));
        assert_eq!(eco_to_string(100), Some("B00".to_string()));
        assert_eq!(eco_to_string(497), Some("E97".to_string()));

        // Invalid (beyond E99)
        assert_eq!(eco_to_string(500), None);
    }

    #[test]
    fn test_fen_with_setup() {
        let mut tags = SupplementalTags::default();
        tags.fen = Some("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1".to_string());

        let pgn = tags.to_pgn();
        assert!(
            pgn.contains("[FEN \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1\"]")
        );
        assert!(pgn.contains("[SetUp \"1\"]"));
    }

    #[test]
    fn test_alphabetical_ordering() {
        let mut tags = SupplementalTags::default();
        tags.white_elo = Some(2500);
        tags.eco_code = Some("C84".to_string());
        tags.annotator = Some("Smith, J.".to_string());

        let pgn = tags.to_pgn();

        // Verify alphabetical order
        let ann_pos = pgn.find("[Annotator").unwrap();
        let eco_pos = pgn.find("[ECO").unwrap();
        let white_pos = pgn.find("[WhiteElo").unwrap();

        assert!(ann_pos < eco_pos);
        assert!(eco_pos < white_pos);
    }

    #[test]
    fn test_custom_tags() {
        let mut tags = SupplementalTags::default();
        tags.custom_tags
            .insert("WhiteTitle".to_string(), "GM".to_string());
        tags.custom_tags
            .insert("BlackTitle".to_string(), "IM".to_string());

        let pgn = tags.to_pgn();
        assert!(pgn.contains("[BlackTitle \"IM\"]"));
        assert!(pgn.contains("[WhiteTitle \"GM\"]"));
    }

    #[test]
    fn test_skip_empty_values() {
        let tags = SupplementalTags::default();
        let pgn = tags.to_pgn();

        // Should be empty (no tags with values)
        assert_eq!(pgn, "");
    }
}
