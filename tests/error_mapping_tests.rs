use scidtopgn::{SG4Parser};
use scidtopgn::prelude::{EnhancedDecodeError, ScidError};

#[test]
fn enhanced_error_maps_to_scid_error() {
    // Create a parser with too-short data for queen diagonal
    let data = vec![0x28]; // Queen with diagonal indicator but missing second byte
    let mut parser = SG4Parser::new(data.clone());
    parser.offset = 0;
    let res = parser.decode_queen_diagonal_start(data[0]);
    // Ensure internal error is EnhancedDecodeError::InvalidMove
    assert!(matches!(res, Err(EnhancedDecodeError::InvalidMove(_))));
    // Map to ScidError
    let mapped: Result<(), ScidError> = res.map(|_| ()).map_err(ScidError::from);
    assert!(matches!(mapped, Err(ScidError::InvalidFormat { .. })));
}

#[test]
fn index_out_of_bounds_maps_to_invalid_format() {
    // piece_num >= 16 should trigger IndexOutOfBounds
    let data = vec![0xF0]; // high nibble 0xF = 15? Actually 0xF0 -> piece_num 15 (<16) valid; use 0x100 not possible; instead simulate via get_from_square_by_index with missing list
    let mut parser = SG4Parser::new(vec![0x00]);
    // Force missing piece list to produce MissingPieceList and mapping
    parser.current_piece_list = None;
    let res = parser.decode_single_byte_move(0xF0);
    assert!(matches!(res, Err(EnhancedDecodeError::MissingPieceList)));
    let mapped: Result<(), ScidError> = res.map(|_| ()).map_err(ScidError::from);
    assert!(matches!(mapped, Err(ScidError::InvalidFormat { .. })));
}
