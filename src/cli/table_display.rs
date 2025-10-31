//! Table formatting and display functions for SCID database parsing
//! 
//! This module provides formatted table display functions for various SCID database components
//! including headers, names, statistics, and game information.

use crate::formats::{ScidDatabase, DatabaseFileType};
use crate::core::error::Result;
use crate::api::NameType;

/// Truncate string to max_len or pad with spaces
/// If too long, shows "…" at end
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    } else {
        format!("{:<width$}", s, width = max_len)
    }
}

/// Display SI4 header information in formatted table
/// 
/// Shows magic number, version, base type, game count, auto-load setting,
/// and description in a Unicode box-drawing table.
pub fn display_si4_header(db: &ScidDatabase) -> Result<()> {
    let header = db.si4_header();
    
    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
    println!("│ Field                   │ Value                                           │");
    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
    println!("│ Magic                   │ {:<47} │", header.magic);
    println!("│ Version                 │ {:<47} │", header.version);
    println!("│ Base Type               │ {:<47} │", header.base_type);
    println!("│ Number of Games         │ {:<47} │", header.num_games);
    println!("│ Auto Load Game          │ {:<47} │", header.auto_load);
    println!("│ Description             │ {:<47} │", truncate_string(&header.description, 43));
    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
    
    Ok(())
}

/// Display SN4 header information in formatted table
/// 
/// Shows magic number, timestamp, name counts, and maximum frequencies
/// for players, events, sites, and rounds in a Unicode table.
pub fn display_sn4_header(db: &ScidDatabase) -> Result<()> {
    let header = db.sn4_header();
    
    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
    println!("│ Field                   │ Value                                           │");
    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
    println!("│ Magic                   │ {:<47} │", header.magic);
    println!("│ Timestamp               │ {:<47} │", header.timestamp);
    println!("│ Num Names Player        │ {:<47} │", header.num_names_player);
    println!("│ Num Names Event         │ {:<47} │", header.num_names_event);
    println!("│ Num Names Site          │ {:<47} │", header.num_names_site);
    println!("│ Num Names Round         │ {:<47} │", header.num_names_round);
    println!("│ Max Frequency Player    │ {:<47} │", header.max_frequency_player);
    println!("│ Max Frequency Event     │ {:<47} │", header.max_frequency_event);
    println!("│ Max Frequency Site      │ {:<47} │", header.max_frequency_site);
    println!("│ Max Frequency Round     │ {:<47} │", header.max_frequency_round);
    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
    
    Ok(())
}

/// Display names table for a specific name type
/// 
/// Shows all names of the given type (Player/Event/Site/Round) with
/// their IDs and frequencies in a formatted table.
pub fn display_names_table(db: &ScidDatabase, name_type: NameType) -> Result<()> {
    let type_str = match name_type {
        NameType::Player => "Player",
        NameType::Event => "Event",
        NameType::Site => "Site",
        NameType::Round => "Round",
    };
    
    println!("┌────────────┬────────┬──────────┬─────────────────────────────────────────────┐");
    println!("│ Type       │ ID     │ Frequency│ Name                                        │");
    println!("├────────────┼────────┼──────────┼─────────────────────────────────────────────┤");
    
    for result in db.iter_names(name_type) {
        match result {
            Ok(record) => {
                println!("│ {:<10} │ {:>6} │ {:>8} │ {:<43} │", 
                    type_str, record.id, record.frequency, truncate_string(&record.name, 43));
            }
            Err(e) => {
                eprintln!("Error reading name record: {}", e);
                break;
            }
        }
    }
    
    println!("└────────────┴────────┴──────────┴─────────────────────────────────────────────┘");
    
    Ok(())
}

/// Display SG4 file statistics in formatted table
/// 
/// Shows file size, game count, and average game size calculated
/// from the actual SG4 file on disk.
pub fn display_sg4_stats(db: &ScidDatabase) -> Result<()> {
    let stats = db.statistics();
    let sg4_size = db.file_size(DatabaseFileType::Sg4)?;
    
    println!("┌─────────────────────────┬─────────────────────────────────────────────────┐");
    println!("│ Field                   │ Value                                           │");
    println!("├─────────────────────────┼─────────────────────────────────────────────────┤");
    println!("│ File Size               │ {} bytes{:<18} │", sg4_size, "");
    println!("│ Games Found             │ {:<47} │", stats.num_games);
    if stats.num_games > 0 {
        let avg_size = sg4_size / (stats.num_games as usize);
        println!("│ Average Game Size       │ {} bytes{:<18} │", avg_size, "");
    }
    println!("└─────────────────────────┴─────────────────────────────────────────────────┘");
    
    Ok(())
}

/// Display game moves with algebraic notation
/// 
/// Converts SCID moves to standard algebraic chess notation using PositionTracker
/// and displays them in a formatted PGN-style sequence.
fn display_game_moves(db: &ScidDatabase, game_index: u32) -> Result<()> {
    use crate::position::PositionTracker;
    
    let game = db.get_game(game_index)?;
    
    if game.moves.is_empty() {
        println!("  📋 Moves: [No moves available]");
        return Ok(());
    }
    
    let mut tracker = PositionTracker::new();
    let mut notations = Vec::new();
    let mut error_count = 0;
    
    for (i, decoded_move) in game.moves.iter().enumerate() {
        match tracker.apply_move(decoded_move) {
            Ok(notation) => notations.push(notation),
            Err(e) => {
                eprintln!("    ⚠️  Error decoding move {}: {}", i + 1, e);
                error_count += 1;
            }
        }
    }
    
    let moves_pgn = format_moves_as_pgn(&notations);
    println!("  📋 Moves ({}): {}", notations.len(), moves_pgn);
    
    if error_count > 0 {
        eprintln!("    ⚠️  {} move(s) failed to decode", error_count);
    }
    
    Ok(())
}

/// Format move notations as PGN-style text
/// 
/// Takes a vector of algebraic move notations and formats them
/// in standard PGN game notation with move numbers.
pub fn format_moves_as_pgn(notations: &[String]) -> String {
    if notations.is_empty() {
        return "[No moves]".to_string();
    }
    
    let mut result = String::new();
    let mut move_num = 1;
    
    // Process moves in pairs (white + black)
    for chunk in notations.chunks(2) {
        if !result.is_empty() {
            result.push(' ');
        }
        
        match chunk {
            [white_move] => {
                result.push_str(&format!("{}. {}", move_num, white_move));
                move_num += 1;
            }
            [white_move, black_move] => {
                result.push_str(&format!("{}. {} {}", move_num, white_move, black_move));
                move_num += 1;
            }
            _ => {} // Shouldn't happen with chunks(2)
        }
    }
    
    result.trim().to_string()
}

/// Display individual game information with metadata
/// 
/// Shows comprehensive game metadata including resolved player names,
/// event/site information, date, result, ECO code, ELO ratings,
/// and move count. Optionally includes move display if available.
pub fn display_game(
    db: &ScidDatabase, 
    game_index: u32,
    include_moves: bool,
) -> Result<()> {
    let game = db.get_game(game_index)?;
    let idx = &game.index;
    
    println!("GAME {} DETAILS", game_index + 1);
    println!("┌─────────────────────────┬──────────────────────────────────────────────────┐");
    println!("│ Field                   │ Value                                            │");
    println!("├─────────────────────────┼──────────────────────────────────────────────────┤");
    
    // Resolve names
    let white_name = db.get_player_name(idx.white_id)?
        .unwrap_or_else(|| format!("Player {}", idx.white_id));
    let black_name = db.get_player_name(idx.black_id)?
        .unwrap_or_else(|| format!("Player {}", idx.black_id));
    let event_name = db.get_event_name(idx.event_id)?
        .unwrap_or_else(|| format!("Event {}", idx.event_id));
    let site_name = db.get_site_name(idx.site_id)?
        .unwrap_or_else(|| format!("Site {}", idx.site_id));
    
    // Display fields
    println!("│ Game Number             │ {:<48} │", game_index + 1);
    println!("│ Offset                  │ {:<48} │", idx.offset);
    println!("│ Length                  │ {} bytes{:<38} │", idx.length, "");
    println!("│ White Player            │ {} (ID: {:<38}) │", truncate_string(&white_name, 28), idx.white_id);
    println!("│ Black Player            │ {} (ID: {:<38}) │", truncate_string(&black_name, 28), idx.black_id);
    println!("│ Event                   │ {} (ID: {:<40}) │", truncate_string(&event_name, 28), idx.event_id);
    println!("│ Site                    │ {} (ID: {:<40}) │", truncate_string(&site_name, 28), idx.site_id);
    println!("│ Date                    │ {}.{:02}.{:02}{:<42} │", idx.year, idx.month, idx.day, "");
    
    let result_str = match idx.result {
        0 => "*".to_string(),
        1 => "1-0".to_string(),
        2 => "0-1".to_string(),
        3 => "1/2-1/2".to_string(),
        _ => "Unknown".to_string(),
    };
    println!("│ Result                  │ {:<48} │", result_str);
    println!("│ ECO Code                │ {:<48} │", idx.eco);
    println!("│ White ELO               │ {:<48} │", idx.white_elo);
    println!("│ Black ELO               │ {:<48} │", idx.black_elo);
    println!("│ Half Moves              │ {:<48} │", idx.num_half_moves);
    
    println!("└─────────────────────────┴──────────────────────────────────────────────────┘");
    
    if include_moves {
        display_game_moves(db, game_index)?;
    }
    
    println!();
    Ok(())
}