use crate::args::Args;
use scidtopgn_core::Game;

pub fn should_include_game(game: &Game, args: &Args) -> bool {
    true
}
