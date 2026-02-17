use clap::Parser;

#[derive(Parser)]
#[command(name = "scidtopgn")]
#[command(about = "Convert SCID databases to PGN format", long_about = None)]
pub struct Args {
    /// Database path (without extension)
    pub database: String,

    /// Output file (default: stdout)
    #[arg(short, long)]
    pub output: Option<String>,

    /// Show game count only
    #[arg(long)]
    pub count: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}
