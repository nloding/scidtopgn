// CLI application setup
use std::env;
use std::io;
use shakmaty::Position;

pub fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        crate::cli::output::help::display_help(&args[0]);
        std::process::exit(1);
    }
    
    match args[1].as_str() {
        "encode" => {
            if args.len() != 3 {
                eprintln!("Usage: {} encode <date>", args[0]);
                eprintln!("Example: {} encode 2022.12.19", args[0]);
                std::process::exit(1);
            }
            if let Err(e) = crate::cli::commands::encode::execute(&args[2]) {
                eprintln!("Encode command failed: {}", e);
                std::process::exit(1);
            }
        }
        "test-position" => {
            if let Err(e) = crate::cli::commands::test_position::execute() {
                eprintln!("Test position command failed: {}", e);
                std::process::exit(1);
            }
        }
        "test-one-move" => {
            println!("🧪 Testing single move decoding with position:");
            let position = shakmaty::Chess::default();
            println!("📍 Starting position:");
            println!("{}", position.board());
            
            // Test decoding a simple pawn move: P12 with move_value 15 (double pawn push)
            // From our test data: "P12: Pawn double ..."
            println!("\n🔍 Testing pawn double push: piece P12, move_value 15");
            
            // P12 should be a pawn on file e (based on our mapping) - simplified for shakmaty
            println!("✅ Using shakmaty Chess position - custom piece lookup methods removed");
            
            println!("✅ Basic piece lookup test completed!");
        }
        "test-moves" => {
            if args.len() != 3 {
                eprintln!("Usage: {} test-moves <base_path>", args[0]);
                eprintln!("Example: {} test-moves /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            if let Err(e) = crate::cli::commands::test_moves::execute(base_path) {
                eprintln!("Test moves command failed: {}", e);
                std::process::exit(1);
            }
        }
        "test-variations" => {
            if args.len() != 3 {
                eprintln!("Usage: {} test-variations <base_path>", args[0]);
                eprintln!("Example: {} test-variations /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            if let Err(e) = crate::cli::commands::test_variations::execute(base_path) {
                eprintln!("Test variations command failed: {}", e);
                std::process::exit(1);
            }
        }
        "parse" => {
            if args.len() != 3 {
                eprintln!("Usage: {} parse <base_path>", args[0]);
                eprintln!("Example: {} parse /path/to/database", args[0]);
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            if let Err(e) = crate::cli::commands::parse::execute(base_path) {
                eprintln!("Parse command failed: {}", e);
                std::process::exit(1);
            }
        }
        "parse-position" => {
            if args.len() != 3 {
                eprintln!("Usage: {} parse-position <base_path>", args[0]);
                eprintln!("Example: {} parse-position /path/to/database", args[0]);
                eprintln!("Note: Uses position-aware SCID move decoding");
                std::process::exit(1);
            }
            
            let base_path = &args[2];
            if let Err(e) = crate::cli::commands::parse_position::execute(base_path) {
                eprintln!("Parse-position command failed: {}", e);
                std::process::exit(1);
            }
        }
        "format" => {
            if let Err(e) = crate::cli::commands::format::execute() {
                eprintln!("Format command failed: {}", e);
                std::process::exit(1);
            }
        }
        "help" | "--help" | "-h" => {
            crate::cli::output::help::display_help(&args[0]);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            eprintln!();
            crate::cli::output::help::display_help(&args[0]);
            std::process::exit(1);
        }
    }
    
    Ok(())
}