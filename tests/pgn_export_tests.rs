#[cfg(test)]
mod tests {
    use scidtopgn::api::{PgnExporter, ExportOptions, GameState, GameMetadata};
    use scidtopgn::sg4::StreamingGameParseState;

    fn build_game_state() -> GameState {
        let mut state = GameState::new();
        state.set_metadata(GameMetadata {
            white: "Player White".to_string(),
            black: "Player Black".to_string(),
            event: "Test Tournament".to_string(),
            site: "Test City".to_string(),
            date: "2023.12.25".to_string(),
            result: "1-0".to_string(),
            white_elo: Some(1800),
            black_elo: Some(1750),
            round: Some("1".to_string()),
            eco: Some("B13".to_string()),
        });
        state
    }

    fn empty_parsed_game() -> StreamingGameParseState {
        StreamingGameParseState { elements: vec![] }
    }

    #[test]
    fn test_pgn_standards_compliance() {
        let game_state = build_game_state();
        let parsed = empty_parsed_game();
        let exporter = PgnExporter::new(&game_state, &parsed).unwrap();

        let pgn_output = exporter.export().expect("Should export game successfully");

        assert!(pgn_output.contains("[Event "));
        assert!(pgn_output.contains("[White "));
        assert!(pgn_output.contains("[Black "));
        assert!(pgn_output.contains("[Result "));
    }

    #[test]
    fn test_export_without_optional_headers() {
        let options = ExportOptions {
            include_optional_tags: false,
            ..Default::default()
        };

        let game_state = build_game_state();
        let parsed = empty_parsed_game();
        let exporter = PgnExporter::with_options(&game_state, &parsed, options).unwrap();

        let pgn_output = exporter.export().expect("Should export without optional headers");

        assert!(pgn_output.contains("[Event "));
        assert!(pgn_output.contains("[Site "));
        assert!(pgn_output.contains("[Date "));
        assert!(pgn_output.contains("[Round "));
        assert!(pgn_output.contains("[White "));
        assert!(pgn_output.contains("[Black "));
        assert!(pgn_output.contains("[Result "));

        assert!(!pgn_output.contains("[WhiteElo "));
        assert!(!pgn_output.contains("[BlackElo "));
        assert!(!pgn_output.contains("[ECO "));
    }

    #[test]
    fn test_header_validation() {
        let options = ExportOptions {
            validate_moves: true,
            ..Default::default()
        };

        let game_state = build_game_state();
        let parsed = empty_parsed_game();
        let exporter = PgnExporter::with_options(&game_state, &parsed, options).unwrap();

        let pgn_output = exporter.export().expect("Should export with valid headers");

        assert!(pgn_output.contains("[Event \"Test Tournament\"]"));
        assert!(pgn_output.contains("[Site \"Test City\"]"));
        assert!(pgn_output.contains("[Date \"2023.12.25\"]"));
        assert!(pgn_output.contains("[Round \"1\"]"));
        assert!(pgn_output.contains("[White \"Player White\"]"));
        assert!(pgn_output.contains("[Black \"Player Black\"]"));
        assert!(pgn_output.contains("[Result \"1-0\"]"));
    }
}
