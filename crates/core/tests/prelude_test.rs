//! Integration test for prelude module

use scidtopgn_core::prelude::*;

#[test]
fn test_prelude_imports() {
    // Verify error types are available
    let _result: Result<()> = Ok(());
    let _error = ScidError::invalid_format("test");

    // Verify GameResult is available
    let result = GameResult::WhiteWins;
    assert_eq!(result.to_pgn(), "1-0");

    // Verify GameDate is available
    let date = GameDate::new(2022, 12, 19);
    assert_eq!(date.to_pgn_string(), "2022.12.19");

    // Verify shakmaty types are available
    let _square = Square::E4;
    let _color = Color::White;
    let _role = Role::Queen;
}
