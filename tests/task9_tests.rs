use scidtopgn::SG4Parser;
use scidtopgn::prelude::{EnhancedDecodeError, ScidError};
use scidtopgn::formats::sg4::MoveInterpretation;

#[test]
fn index_based_routing_uses_board_derived_role() {
    // Data with a single byte: high nibble selects index 0 (white king at e1), move_value=0
    let data = vec![0x00];
    let mut parser = SG4Parser::new(data.clone());
    parser.offset = 0;
    // decode via role-aware single-byte path
    let dm = parser.decode_single_byte_move(data[0]).expect("decode should succeed");
    assert_eq!(dm.from_square_index, Some(4)); // e1 index
    // Interpretation should be King move for move_value 0
    assert!(matches!(dm.interpretation, MoveInterpretation::King { .. }));
    assert!(matches!(dm.piece_type, Some(_)));
}

#[test]
fn queen_diagonal_multi_byte_consumes_two_bytes_and_sets_target() {
    // First byte: piece index 1 (white queen at d1 index 3), move_value 8 (diagonal group)
    // Second byte: distance encoded in low nibble, use 0x02 => distance 2
    let data = vec![0x18, 0x02];
    let mut parser = SG4Parser::new(data.clone());
    parser.offset = 0;
    let dm = parser.decode_queen_diagonal_start(data[0]).expect("queen diagonal should decode");
    assert_eq!(dm.from_square_index, Some(3)); // d1 index
    // dir_code 0 => up-right, distance 2 => from (file=3, rank=0) to (5,2) => index 21
    assert_eq!(dm.to_square_index, Some(21));
    assert_eq!(parser.offset, 2, "offset should advance by two bytes");
}

#[test]
fn piece_list_synchronization_updates_indices() {
    let data = vec![0x18, 0x02];
    let mut parser = SG4Parser::new(data.clone());
    parser.offset = 0;
    let dm = parser.decode_queen_diagonal_start(data[0]).expect("queen diagonal should decode");
    parser.update_position(&dm).map_err(ScidError::from).expect("position update should succeed");
    let list = parser.current_piece_list.expect("piece list present");
    // piece_num is high nibble (1), should now point to to_square_index 21
    assert_eq!(list[dm.piece_num as usize], dm.to_square_index.expect("to index"));
}
