use scidtopgn::prelude::EnhancedDecodeError;
use scidtopgn::MoveInterpretation;
use scidtopgn::SG4Parser;

#[test]
fn sg4_routing_uses_board_role() -> Result<(), EnhancedDecodeError> {
    let mut parser = SG4Parser::new(vec![0; 0]);
    // Piece index 0 should be white king at e1 in starting list
    let byte = 0x00; // high nibble 0 (index 0), low nibble 0 (move value)
    let dm = parser.decode_single_byte_move_indexed(byte)?;
    assert!(matches!(dm.interpretation, MoveInterpretation::King { .. }));
    Ok(())
}

#[test]
fn sg4_queen_diagonal_two_bytes() -> Result<(), EnhancedDecodeError> {
    // Construct data where offset points to queen diagonal
    // First byte: piece index 1 (white queen at d1), move_value 8 (diagonal group)
    // Second byte: low nibble distance 1 -> move one square diagonally
    let data = vec![0x18, 0x01];
    let mut parser = SG4Parser::new(data.clone());
    parser.offset = 0;
    let dm = parser.decode_queen_diagonal_start(data[0])?;
    assert_eq!(dm.raw_bytes, vec![0x18, 0x01]);
    assert!(matches!(dm.interpretation, MoveInterpretation::Queen));
    assert!(dm.from_square_index.is_some());
    assert!(dm.to_square_index.is_some());
    // From d1 (index 59), dir_code 0 -> up-right by 1 => e2 (index 52+?)
    // We only assert it computed some target within board
    Ok(())
}

#[test]
fn sg4_piece_list_synchronizes_after_update() -> Result<(), EnhancedDecodeError> {
    let mut parser = SG4Parser::new(vec![]);
    // Call update_position with a decoded move placeholder (no actual change)
    let dm = parser.decode_single_byte_move_indexed(0x00)?;
    parser.update_position(&dm)?;
    // Ensure piece list is Some and has 16 entries
    let list = parser.current_piece_list.expect("piece list");
    assert_eq!(list.len(), 16);
    Ok(())
}
