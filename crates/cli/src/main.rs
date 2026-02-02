mod args;
mod error;
mod filter;
mod progress;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use scidtopgn_core::prelude::*;
use std::fs::File;
use std::io::{self, Write};

fn main() -> Result<()> {
    let args = Args::parse();

    args.validate().context("Invalid arguments")?;

    if !args.quiet {
        eprintln!("Opening database: {}", args.input.display());
    }

    let open_options = if args.mmap {
        OpenOptions::memory_mapped()
    } else {
        OpenOptions::auto_detect().mmap_threshold(parse_size_string(&args.mmap_threshold)?)
    };

    let reader = ScidReader::open_with_options(&args.input, open_options)
        .with_context(|| format!("Failed to open database: {}", args.input.display()))?;

    if args.info {
        return show_database_info(&reader);
    }

    if !args.quiet {
        eprintln!("Database: {}", reader.description());
        eprintln!("Total games: {}", reader.game_count());
    }

    let pgn_options = PgnOptions {
        include_comments: !args.no_comments,
        include_variations: !args.no_variations,
        compact: args.compact,
        line_width: if args.compact { usize::MAX } else { 80 },
        include_supplemental_tags: true,
    };

    let output: Box<dyn Write> = if args.is_stdout() {
        Box::new(io::stdout())
    } else {
        let path = args.output.as_ref().unwrap();
        let file = File::create(path)
            .with_context(|| format!("Failed to create output file: {}", path.display()))?;

        if !args.quiet {
            eprintln!("Writing to: {}", path.display());
        }

        Box::new(file)
    };

    convert_database(&reader, output, &args, &pgn_options)?;

    if !args.quiet {
        eprintln!("Conversion complete!");
    }

    Ok(())
}

fn show_database_info(reader: &ScidReader) -> Result<()> {
    println!("Database Information");
    println!("====================");
    println!();
    println!("Description: {}", reader.description());
    println!("Version:     {}", reader.version());
    println!("Total games: {}", reader.game_count());
    println!();
    println!("Names:");
    println!("  Players:   {}", reader.names().players.len());
    println!("  Events:    {}", reader.names().events.len());
    println!("  Sites:     {}", reader.names().sites.len());
    println!("  Rounds:    {}", reader.names().rounds.len());

    Ok(())
}

fn parse_size_string(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();
    let (num_str, multiplier) = if s.ends_with('G') {
        (&s[..s.len() - 1], 1024 * 1024 * 1024)
    } else if s.ends_with('M') {
        (&s[..s.len() - 1], 1024 * 1024)
    } else if s.ends_with('K') {
        (&s[..s.len() - 1], 1024)
    } else {
        (s.as_str(), 1)
    };

    let num: u64 = num_str
        .parse()
        .with_context(|| format!("Invalid size: {}", s))?;

    Ok(num * multiplier)
}

fn parse_error_mode(s: &str) -> ErrorMode {
    match s.to_lowercase().as_str() {
        "lenient" => ErrorMode::Lenient,
        "best-effort" => ErrorMode::BestEffort,
        _ => ErrorMode::Strict,
    }
}

fn convert_database(
    reader: &ScidReader,
    mut output: Box<dyn Write>,
    args: &Args,
    options: &PgnOptions,
) -> Result<()> {
    let total_games = reader.game_count();
    let (start_idx, end_idx) = if let Some(ref range) = args.range {
        let (start, end) = Args::parse_range(range)?;
        let start = start.unwrap_or(1).saturating_sub(1);
        let end = end.unwrap_or(total_games);
        (start, end.min(total_games))
    } else {
        (0, total_games)
    };

    let games_to_process = end_idx - start_idx;

    if !args.quiet && games_to_process == 0 {
        eprintln!("Warning: No games selected");
        return Ok(());
    }

    let progress = if args.verbose && games_to_process > 100 && !args.is_stdout() {
        Some(progress::create_progress_bar(games_to_process))
    } else {
        None
    };

    let error_mode = parse_error_mode(&args.error_mode);
    let max_errors = if args.max_errors == 0 {
        usize::MAX
    } else {
        args.max_errors
    };

    let mut processed = 0;
    let mut skipped = 0;
    let mut errors = 0;
    let mut partial = 0;

    for (idx, game_result) in reader
        .games()
        .enumerate()
        .skip(start_idx)
        .take(games_to_process)
    {
        let game = match game_result {
            Ok(g) => g,
            Err(e) => {
                errors += 1;

                match error_mode {
                    ErrorMode::Strict => {
                        return Err(e).with_context(|| format!("Game {} failed", idx + 1));
                    }
                    ErrorMode::Lenient => {
                        if !args.quiet {
                            eprintln!("Warning: Skipping game {}: {}", idx + 1, e);
                        }
                        skipped += 1;
                        if errors >= max_errors {
                            return Err(anyhow::anyhow!(
                                "Maximum errors ({}) reached, stopping",
                                max_errors
                            ));
                        }
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        continue;
                    }
                    ErrorMode::BestEffort => {
                        if args.include_partial {
                            if let Some(partial_game) = e.partial_game() {
                                if !args.quiet {
                                    eprintln!("Warning: Partial game {}: {}", idx + 1, e);
                                }
                                partial += 1;
                                let pgn = partial_game
                                    .to_pgn_with_comment(&format!("{{ Parsing stopped: {} }}", e));
                                output
                                    .write_all(pgn.as_bytes())
                                    .context("Failed to write output")?;
                                if let Some(ref pb) = progress {
                                    pb.inc(1);
                                }
                                continue;
                            }
                        }
                        if !args.quiet {
                            eprintln!("Warning: Skipping game {}: {}", idx + 1, e);
                        }
                        skipped += 1;
                        if errors >= max_errors {
                            return Err(anyhow::anyhow!(
                                "Maximum errors ({}) reached, stopping",
                                max_errors
                            ));
                        }
                        if let Some(ref pb) = progress {
                            pb.inc(1);
                        }
                        continue;
                    }
                }
            }
        };

        if !filter::should_include_game(&game, args) {
            skipped += 1;
            if let Some(ref pb) = progress {
                pb.inc(1);
            }
            continue;
        }

        let pgn = game
            .to_pgn_with_options(options)
            .with_context(|| format!("Failed to convert game {}", idx + 1))?;

        output
            .write_all(pgn.as_bytes())
            .context("Failed to write output")?;

        processed += 1;

        if let Some(ref pb) = progress {
            pb.inc(1);
            pb.set_message(format!(
                "OK: {} | Skip: {} | Err: {}",
                processed, skipped, errors
            ));
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message(format!(
            "Complete! Converted: {} | Skipped: {} | Errors: {} | Partial: {}",
            processed, skipped, errors, partial
        ));
    }

    if !args.quiet {
        eprintln!("Processed: {} games", processed);
        if skipped > 0 {
            eprintln!("Skipped:   {} games (filtered)", skipped);
        }
        if errors > 0 {
            eprintln!("Errors:    {} games", errors);
        }
        if partial > 0 {
            eprintln!("Partial:   {} games", partial);
        }
    }

    Ok(())
}
