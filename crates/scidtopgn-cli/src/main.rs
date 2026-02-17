mod args;

use std::fs::File;
use std::io::{BufWriter, Write};

use anyhow::Result;
use clap::Parser;

use args::Args;
use scidtopgn::Database;

fn main() -> Result<()> {
    let args = Args::parse();

    let mut db = Database::open(&args.database)?;

    if args.count {
        println!("{} games", db.num_games());
        return Ok(());
    }

    let mut writer: Box<dyn Write> = match &args.output {
        Some(path) => {
            let file = File::create(path)?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(std::io::stdout()),
    };

    let total = db.num_games();
    for i in 0..total {
        let game = db.get_game(i)?;
        let pgn = game.to_pgn();
        writeln!(writer, "{}", pgn)?;

        if args.verbose && (i + 1) % 100 == 0 {
            eprintln!("Processed {}/{} games", i + 1, total);
        }
    }

    if args.verbose {
        eprintln!("Done. {} games processed.", total);
    }

    Ok(())
}
