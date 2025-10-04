use crate::core::error::{Result, ScidError};

/// SCID date encoding and decoding functions
/// Based on SCID source code from scidvspc/src/index.cpp and date.h

/// Create a SCID date from year, month, day components
/// SCID encoding: ((year << 9) | (month << 5) | day)
#[allow(dead_code)]
pub fn date_make(year: u32, month: u32, day: u32) -> u32 {
    (year << 9) | (month << 5) | day
}

/// Extract year from SCID date
pub fn date_get_year(date: u32) -> u32 {
    date >> 9
}

/// Extract month from SCID date
pub fn date_get_month(date: u32) -> u32 {
    (date >> 5) & 15
}

/// Extract day from SCID date
pub fn date_get_day(date: u32) -> u32 {
    date & 31
}

/// Set lower 20 bits of a u32 value
#[allow(dead_code)]
pub fn u32_set_low_20(u: u32, x: u32) -> u32 {
    (u & 0xFFF00000) | (x & 0x000FFFFF)
}

/// Set upper 12 bits of a u32 value
#[allow(dead_code)]
pub fn u32_set_high_12(u: u32, x: u32) -> u32 {
    (u & 0x000FFFFF) | ((x & 0xFFF) << 20)
}

/// Set game date in SCID dates field (lower 20 bits)
#[allow(dead_code)]
pub fn scid_set_date(existing_dates: u32, year: u32, month: u32, day: u32) -> u32 {
    let date = date_make(year, month, day);
    u32_set_low_20(existing_dates, date)
}

/// Set event date in SCID dates field (upper 12 bits)
/// Event dates are stored as relative offsets when within ±3 years of game date
#[allow(dead_code)]
pub fn scid_set_event_date(existing_dates: u32, edate: u32) -> u32 {
    let game_date = existing_dates & 0x000FFFFF;
    
    let mut coded_date = date_get_month(edate) << 5;
    coded_date |= date_get_day(edate);
    let eyear = date_get_year(edate);
    let dyear = date_get_year(game_date);
    
    let eyear = if eyear < (dyear - 3) || eyear > (dyear + 3) {
        0
    } else {
        eyear
    };
    
    if eyear == 0 {
        coded_date = 0;
    } else {
        coded_date |= ((eyear + 4 - dyear) & 7) << 9;
    }
    
    u32_set_high_12(existing_dates, coded_date)
}

/// Extract event date from SCID dates field
/// Returns None if event date is not set or invalid
pub fn scid_get_event_date(dates_field: u32) -> Option<(u32, u32, u32)> {
    let game_date = dates_field & 0x000FFFFF;
    let dyear = date_get_year(game_date);
    let edate = (dates_field >> 20) & 0xFFF;
    
    if edate == 0 {
        return None;
    }
    
    let month = date_get_month(edate);
    let day = date_get_day(edate);
    let year_offset = date_get_year(edate) & 7;
    
    if year_offset == 0 {
        return None;
    }
    
    let year = dyear + year_offset - 4;
    Some((year, month, day))
}

/// Validate date components
#[allow(dead_code)]
pub fn validate_date(year: u32, month: u32, day: u32) -> Result<()> {
    if year < 1000 || year > 3000 {
        return Err(ScidError::InvalidDate);
    }
    
    if month < 1 || month > 12 {
        return Err(ScidError::InvalidDate);
    }
    
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => return Err(ScidError::InvalidDate),
    };
    
    if day < 1 || day > max_day {
        return Err(ScidError::InvalidDate);
    }
    
    Ok(())
}

/// Check if year is a leap year
fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Format SCID date as PGN date string (YYYY.MM.DD)
#[allow(dead_code)]
pub fn format_pgn_date(date: u32) -> String {
    let year = date_get_year(date);
    let month = date_get_month(date);
    let day = date_get_day(date);
    
    format!("{}.{:02}.{:02}", year, month, day)
}

/// Parse PGN date string (YYYY.MM.DD) to SCID date
#[allow(dead_code)]
pub fn parse_pgn_date(pgn_date: &str) -> Result<u32> {
    let parts: Vec<&str> = pgn_date.split('.').collect();
    if parts.len() != 3 {
        return Err(ScidError::InvalidDate);
    }
    
    let year = parts[0].parse::<u32>()
        .map_err(|_| ScidError::InvalidDate)?;
    let month = parts[1].parse::<u32>()
        .map_err(|_| ScidError::InvalidDate)?;
    let day = parts[2].parse::<u32>()
        .map_err(|_| ScidError::InvalidDate)?;
    
    validate_date(year, month, day)?;
    
    Ok(date_make(year, month, day))
}

/// Extract game date from SCID dates field (lower 20 bits)
pub fn extract_game_date(dates_field: u32) -> (u32, u32, u32) {
    let game_date = dates_field & 0x000FFFFF;
    (
        date_get_year(game_date),
        date_get_month(game_date),
        date_get_day(game_date),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date_make_and_extract() {
        let date = date_make(2022, 12, 19);
        assert_eq!(date_get_year(date), 2022);
        assert_eq!(date_get_month(date), 12);
        assert_eq!(date_get_day(date), 19);
    }

    #[test]
    fn test_validate_date() {
        assert!(validate_date(2022, 12, 19).is_ok());
        assert!(validate_date(2020, 2, 29).is_ok()); // Leap year
        assert!(validate_date(2021, 2, 29).is_err()); // Not leap year
        assert!(validate_date(2022, 13, 1).is_err()); // Invalid month
        assert!(validate_date(2022, 4, 31).is_err()); // Invalid day
    }

    #[test]
    fn test_leap_year() {
        assert!(is_leap_year(2020));
        assert!(is_leap_year(2000));
        assert!(!is_leap_year(2021));
        assert!(!is_leap_year(1900));
    }

    #[test]
    fn test_pgn_date_formatting() {
        let date = date_make(2022, 12, 19);
        assert_eq!(format_pgn_date(date), "2022.12.19");
    }

    #[test]
    fn test_pgn_date_parsing() {
        let result = parse_pgn_date("2022.12.19");
        assert!(result.is_ok());
        let date = result.unwrap();
        assert_eq!(date_get_year(date), 2022);
        assert_eq!(date_get_month(date), 12);
        assert_eq!(date_get_day(date), 19);
    }

    #[test]
    fn test_pgn_date_parsing_invalid() {
        assert!(parse_pgn_date("invalid").is_err());
        assert!(parse_pgn_date("2022.13.01").is_err());
        assert!(parse_pgn_date("2022.12.32").is_err());
    }

    #[test]
    fn test_scid_date_functions() {
        let dates_field = scid_set_date(0, 2022, 12, 19);
        let (year, month, day) = extract_game_date(dates_field);
        assert_eq!(year, 2022);
        assert_eq!(month, 12);
        assert_eq!(day, 19);
    }

    #[test]
    fn test_event_date_encoding() {
        let _game_date = date_make(2022, 12, 19);
        let event_date = date_make(2022, 12, 15);
        let dates_field = scid_set_date(0, 2022, 12, 19);
        let final_dates = scid_set_event_date(dates_field, event_date);
        
        let decoded = scid_get_event_date(final_dates);
        assert!(decoded.is_some());
        let (year, month, day) = decoded.unwrap();
        assert_eq!(year, 2022);
        assert_eq!(month, 12);
        assert_eq!(day, 15);
    }

    #[test]
    fn test_event_date_out_of_range() {
        let _game_date = date_make(2022, 12, 19);
        let event_date = date_make(2030, 12, 15); // Too far in future
        let dates_field = scid_set_date(0, 2022, 12, 19);
        let final_dates = scid_set_event_date(dates_field, event_date);
        
        let decoded = scid_get_event_date(final_dates);
        assert!(decoded.is_none()); // Should be None due to year range
    }
}
