// Variation testing command  
use crate::sg4::*;

pub fn execute(base_path: &str) -> std::io::Result<()> {
    let sg4_path = format!("{}.sg4", base_path);
    
    println!("🌳 TESTING VARIATION TREE PARSING");
    println!("📂 Reading: {}", sg4_path);
    
    // Read the SG4 file
    match std::fs::read(&sg4_path) {
        Ok(file_data) => {
            // Parse game boundaries first
            let games = find_game_boundaries(&file_data);
            if !games.is_empty() {
                println!("📊 Found {} games", games.len());
                
                // Test on first game with variation support
                if let Some((start_offset, end_offset)) = games.first() {
                    let game_data = &file_data[*start_offset..*end_offset];
                    println!("\n🎮 Testing Game 1 with Variation Trees ({} bytes)", game_data.len());
                    
                    match parse_game_with_position_tracking(game_data, 1) {
                        Ok((moves, notation)) => {
                            // TODO: Re-add variation tree when function is updated
                            let variation_tree = crate::sg4::VariationTree::new();
                            println!("\n🌳 VARIATION TREE RESULTS:");
                            println!("✅ Successfully parsed {} main line moves", moves.len());
                            println!("🌿 Variation tree depth: {}", variation_tree.current_depth);
                            println!("📝 Total elements in tree: {}", variation_tree.main_line.len());
                            
                            // Show variation structure
                            let variations_count = variation_tree.main_line.iter()
                                .map(|node| node.variations.len())
                                .sum::<usize>();
                            if variations_count > 0 {
                                println!("🌿 Found {} variations in the game", variations_count);
                            }
                            
                            // Show first few moves with variations
                            println!("\n📝 Generated notation with variations:");
                            for (i, note) in notation.iter().take(15).enumerate() {
                                println!("  {}. {}", i + 1, note);
                            }
                            if notation.len() > 15 {
                                println!("  ... and {} more moves", notation.len() - 15);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Variation tree parsing failed: {}", e);
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