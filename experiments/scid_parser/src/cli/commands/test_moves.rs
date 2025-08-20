// Move testing command
use crate::sg4::*;

pub fn execute(base_path: &str) -> std::io::Result<()> {
    let sg4_path = format!("{}.sg4", base_path);
    
    println!("🔥 TESTING POSITION-AWARE MOVE PARSING");
    println!("📂 Reading: {}", sg4_path);
    
    // Read the SG4 file
    match std::fs::read(&sg4_path) {
        Ok(file_data) => {
            // Parse game boundaries first
            let games = find_game_boundaries(&file_data);
            if !games.is_empty() {
                    println!("📊 Found {} games", games.len());
                    
                    // Test on first game only for now
                    if let Some((start_offset, end_offset)) = games.first() {
                        let game_data = &file_data[*start_offset..*end_offset];
                        println!("\n🎮 Testing Game 1 ({} bytes)", game_data.len());
                        
                        match parse_game_with_position_tracking(game_data, 1) {
                            Ok((moves, notation)) => {
                                println!("\n🎯 RESULTS:");
                                println!("✅ Successfully parsed {} moves", moves.len());
                                println!("📝 Generated notation:");
                                for (i, note) in notation.iter().take(10).enumerate() {
                                    println!("  {}. {}", i + 1, note);
                                }
                                if notation.len() > 10 {
                                    println!("  ... and {} more moves", notation.len() - 10);
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Position-aware parsing failed: {}", e);
                            }
                        }
                    } else {
                        eprintln!("❌ No games found in file");
                    }
            } else {
                eprintln!("❌ No games found in file");
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to read SG4 file: {}", e);
        }
    }
    Ok(())
}