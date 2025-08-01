use std::fs::File;
use std::io::{self, Write, BufWriter};
use std::path::Path;

use crate::scid::{ScidDatabase, GameIndex};
use crate::scid::moves::parse_scid_moves;

/// PGN exporter for SCID databases
pub struct PgnExporter {
    include_variations: bool,
    include_comments: bool,
    max_games: Option<usize>,
}

impl PgnExporter {
    pub fn new() -> Self {
        PgnExporter {
            include_variations: false,
            include_comments: false,
            max_games: None,
        }
    }
    
    pub fn with_variations(mut self, include: bool) -> Self {
        self.include_variations = include;
        self
    }
    
    pub fn with_comments(mut self, include: bool) -> Self {
        self.include_comments = include;
        self
    }
    
    pub fn with_max_games(mut self, max: usize) -> Self {
        self.max_games = Some(max);
        self
    }
    
    /// Export SCID database to PGN file
    pub fn export(&mut self, database: &mut ScidDatabase, output_path: &Path) -> io::Result<usize> {
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);
        
        // Clone the game indices to avoid borrowing issues
        let games: Vec<_> = database.game_indices().to_vec();
        let total_games = games.len();
        let export_count = self.max_games.map(|max| max.min(total_games)).unwrap_or(total_games);
        
        let mut exported = 0;
        
        for (game_num, game_index) in games.iter().enumerate() {
            if exported >= export_count {
                break;
            }
            
            // Skip deleted games
            if game_index.is_deleted() {
                continue;
            }
            
            // Export game
            self.export_game(&mut writer, database, game_index, game_num)?;
            writer.write_all(b"\n")?; // Empty line between games
            
            exported += 1;
            
            // Progress indicator for large exports
            if exported % 1000 == 0 {
                eprintln!("Exported {} games...", exported);
            }
        }
        
        writer.flush()?;
        Ok(exported)
    }
    
    fn export_game<W: Write>(&mut self, writer: &mut W, database: &mut ScidDatabase, 
                           game_index: &GameIndex, game_num: usize) -> io::Result<()> {
        // Write PGN headers
        self.write_headers(writer, database, game_index, game_num)?;
        
        // Write moves
        self.write_moves(writer, database, game_index)?;
        
        // Write game result
        writeln!(writer, "{}", game_index.result_string())?;
        
        Ok(())
    }
    
    fn write_headers<W: Write>(&self, writer: &mut W, database: &ScidDatabase, 
                             game_index: &GameIndex, game_num: usize) -> io::Result<()> {
        // Event
        let event = database.event_name(game_index.event_id)
            .unwrap_or("Unknown Event");
        writeln!(writer, "[Event \"{}\"]", event)?;
        
        // Site
        let site = database.site_name(game_index.site_id)
            .unwrap_or("Unknown Site");
        writeln!(writer, "[Site \"{}\"]", site)?;
        
        // Date (game date)
        writeln!(writer, "[Date \"{}\"]", game_index.game_date_string())?;
        
        // EventDate (if different from game date)
        if let Some(event_date_str) = game_index.event_date_string() {
            writeln!(writer, "[EventDate \"{}\"]", event_date_str)?;
        }
        
        // Round
        let round = database.round_name(game_index.round_id)
            .unwrap_or("?");
        writeln!(writer, "[Round \"{}\"]", round)?;
        
        // White player
        let white = database.player_name(game_index.white_id)
            .unwrap_or("Unknown Player");
        writeln!(writer, "[White \"{}\"]", white)?;
        
        // Black player
        let black = database.player_name(game_index.black_id)
            .unwrap_or("Unknown Player");
        writeln!(writer, "[Black \"{}\"]", black)?;
        
        // Result
        writeln!(writer, "[Result \"{}\"]", game_index.result_string())?;
        
        // Optional headers
        if game_index.white_elo > 0 {
            writeln!(writer, "[WhiteElo \"{}\"]", game_index.white_elo)?;
        }
        
        if game_index.black_elo > 0 {
            writeln!(writer, "[BlackElo \"{}\"]", game_index.black_elo)?;
        }
        
        if game_index.eco > 0 {
            writeln!(writer, "[ECO \"{}\"]", self.eco_to_string(game_index.eco))?;
        }
        
        // Add some metadata
        writeln!(writer, "[PlyCount \"{}\"]", game_index.num_half_moves)?;
        
        writeln!(writer)?; // Empty line after headers
        
        Ok(())
    }
    
    fn write_moves<W: Write>(&mut self, writer: &mut W, database: &mut ScidDatabase, 
                           game_index: &GameIndex) -> io::Result<()> {
        // Get raw game data
        let game_data = database.game_data(game_index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, 
                                       format!("Failed to read game data: {}", e)))?;
        
        // Parse moves from SCID format
        let moves = parse_scid_moves(&game_data);
        
        if moves.is_empty() {
            // If we can't parse the moves yet, output a placeholder
            writeln!(writer, "{{ Unable to parse SCID moves - feature not yet implemented }}")?;
        } else {
            // Output moves in PGN format
            let mut move_number = 1;
            for (i, mv) in moves.iter().enumerate() {
                if i % 2 == 0 {
                    write!(writer, "{}. ", move_number)?;
                }
                
                write!(writer, "{} ", mv.to_algebraic())?;
                
                if i % 2 == 1 {
                    move_number += 1;
                    if i % 20 == 19 {
                        writeln!(writer)?; // Line break every 10 moves
                    }
                }
            }
            
            if !moves.is_empty() {
                writeln!(writer)?;
            }
        }
        
        Ok(())
    }
    
    fn eco_to_string(&self, eco: u16) -> String {
        // Convert ECO code to string format
        // This is a simplified implementation
        if eco == 0 {
            "?".to_string()
        } else {
            format!("ECO{:03}", eco)
        }
    }
}

impl Default for PgnExporter {
    fn default() -> Self {
        Self::new()
    }
}
