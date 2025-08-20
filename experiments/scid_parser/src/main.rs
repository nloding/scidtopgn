// Allow dead code for development and debugging utilities
#![allow(dead_code)]

use std::io;

// CLI modules
mod cli;

// Core SCID parsing modules
mod utils;
mod date;
mod si4;
mod sg4;
mod sn4;

// Position-aware decoding modules
mod position;

// Shakmaty integration modules
mod bridge;
mod error;

fn main() -> io::Result<()> {
    cli::app::run()
}
