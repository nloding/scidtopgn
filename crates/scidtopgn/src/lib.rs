pub mod common;
pub mod error;
pub mod bytebuf;
pub mod date;
pub mod namebase;
pub mod nfile;
pub mod index;
pub mod position;
pub mod mov;
pub mod game;
pub mod pgn;
pub mod private;
pub mod gfile;
pub mod database;

pub use common::{Piece, Color, Square, GameResult, piece_make, piece_color, piece_type};
pub use error::{Error, Result};
pub use bytebuf::ByteBuffer;
pub use date::{Date, date_make, date_get_year, date_get_month, date_get_day,
               date_decode_to_string, date_encode_from_string, date_valid_string, YEAR_MAX};
pub use namebase::{NameBase, NameType, NameEntry, NAMEBASE_MAGIC, NUM_NAME_TYPES};
pub use nfile::NFile;
pub use index::{Index, IndexHeader, IndexEntry, INDEX_MAGIC, INDEX_SUFFIX,
                INDEX_HEADER_SIZE, INDEX_ENTRY_SIZE, MAX_GAMES, SCID_VERSION};
pub use position::{PieceList, Board, WQ_CASTLE, WK_CASTLE, BQ_CASTLE, BK_CASTLE};
pub use mov::{SimpleMove, make_move_byte, parse_move_byte,
              encode_king, decode_king, encode_knight, decode_knight,
              encode_rook, decode_rook, encode_bishop, decode_bishop,
              encode_queen, decode_queen, encode_pawn, decode_pawn,
              ENCODE_NAG, ENCODE_COMMENT, ENCODE_START_MARKER, 
              ENCODE_END_MARKER, ENCODE_END_GAME, ENCODE_FIRST};
pub use game::{Game, MoveNode, Tag, Marker, MAX_TAGS, MAX_NAGS,
               GAME_DECODE_NONE, GAME_DECODE_TAGS, GAME_DECODE_COMMENTS, GAME_DECODE_ALL,
               nag_to_symbol};
pub use pgn::{PgnParser, parse_pgn, parse_single_game};
pub use gfile::{GFile, FileMode, GFILE_SUFFIX, GF_BLOCKSIZE};
pub use database::{Database, GameIterator};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
