/// Find a piece of given type for current player and return both square and piece number
/// Used when interpretation piece type is available to fix dual numbering systems
/// FIXED: Improved piece list synchronization to maintain consistency
fn find_piece_by_type(position: &ScidPosition, piece_type: PieceType, color: Color) -> Result<(Square, u8), String> {
    // Get the actual piece list for current player to maintain synchronization
    let piece_list = position.piece_list(color);
    
    // Find pieces by checking piece list entries against actual board
    // This ensures piece numbers and board stay synchronized
    let mut matching_pieces = Vec::new();
    
    for (piece_num, &square) in piece_list.iter().enumerate() {
        if let Some(board_piece_type) = position.piece_at(square) {
            if board_piece_type == piece_type && square != Square(255) { // 255 = empty/off-board
                matching_pieces.push((square, piece_num as u8));
            }
        }
    }
    
    if matching_pieces.is_empty() {
        return Err(format!("No {:?} found in piece list for {:?}", piece_type, color));
    }
    
    // Use the first matching piece (heuristic - could be improved with better selection)
    let (square, piece_num) = matching_pieces[0];
    
    Ok((square, piece_num))
}