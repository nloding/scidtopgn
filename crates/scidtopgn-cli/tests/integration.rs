use std::process::Command;

use tempfile::TempDir;

fn get_workspace_root() -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    if manifest_dir.contains("/crates/") {
        let parent = std::path::Path::new(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .unwrap();
        parent.to_string_lossy().to_string()
    } else {
        manifest_dir
    }
}

#[test]
fn test_cli_count() {
    let workspace = get_workspace_root();

    let output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "tests/data/one", "--count"])
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 games"), "Expected '1 games' in output, got: {}", stdout);
}

#[test]
fn test_cli_output() {
    let workspace = get_workspace_root();

    let output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "tests/data/one"])
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[Event \"47th ch-Bangahbandhu 2022\"]"));
    assert!(stdout.contains("[White \"Hossain, Enam\"]"));
    assert!(stdout.contains("1. e4 c5"));
    assert!(stdout.contains("1/2-1/2"));
}

#[test]
fn test_cli_five_games() {
    let workspace = get_workspace_root();

    let output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "tests/data/five"])
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");

    let stdout = String::from_utf8_lossy(&output.stdout);

    let game_count = stdout.matches("[Event \"").count();
    assert_eq!(game_count, 5, "Expected 5 games, found {}", game_count);
}

#[test]
fn test_cli_output_to_file() {
    let workspace = get_workspace_root();
    let temp = TempDir::new().expect("Failed to create temp dir");
    let output_path = temp.path().join("output.pgn");

    let output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "tests/data/one", "-o"])
        .arg(&output_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "CLI should succeed");
    assert!(output_path.exists(), "Output file should exist");

    let contents = std::fs::read_to_string(&output_path).expect("Failed to read output");
    assert!(contents.contains("[Event \"47th ch-Bangahbandhu 2022\"]"));
}

#[test]
fn test_cli_import() {
    let workspace = get_workspace_root();
    let temp = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp.path().join("imported");

    let output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");

    assert!(output.status.success(), "Import should succeed: {:?}", String::from_utf8_lossy(&output.stderr));

    let mut db = scidtopgn::Database::open(&db_path).expect("Failed to open imported database");
    assert_eq!(db.num_games(), 1, "Should have 1 game");

    let game = db.get_game(0).expect("Failed to get game");
    assert_eq!(game.white, "Hossain, Enam");
    assert_eq!(game.black, "Murshed, N");
    assert!(game.eco.is_some(), "ECO should be preserved");

    let pgn = game.to_pgn();
    assert!(pgn.contains("1. e4 c5"), "Moves should be preserved: {}", pgn);
}

#[test]
fn test_cli_import_append() {
    let workspace = get_workspace_root();
    let temp = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp.path().join("append_test");

    let output1 = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");
    assert!(output1.status.success());

    let output2 = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");
    assert!(output2.status.success());

    let db = scidtopgn::Database::open(&db_path).expect("Failed to open database");
    assert_eq!(db.num_games(), 2, "Should have 2 games after append");
}

#[test]
fn test_cli_import_overwrite() {
    let workspace = get_workspace_root();
    let temp = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp.path().join("overwrite_test");

    let output1 = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");
    assert!(output1.status.success());

    let output2 = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "--overwrite", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run CLI");
    assert!(output2.status.success(), "Overwrite should succeed: {:?}", String::from_utf8_lossy(&output2.stderr));

    let db = scidtopgn::Database::open(&db_path).expect("Failed to open database");
    assert_eq!(db.num_games(), 1, "Should have 1 game after overwrite");
}

#[test]
fn test_cli_pgn_roundtrip() {
    let workspace = get_workspace_root();
    let temp = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp.path().join("roundtrip");
    let pgn_path = temp.path().join("exported.pgn");

    let import_output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "import", "tests/data/one.pgn", "-o"])
        .arg(&db_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run import");
    assert!(import_output.status.success());

    let export_output = Command::new("cargo")
        .args(["run", "-p", "scidtopgn-cli", "--release", "--", "export"])
        .arg(&db_path)
        .arg("-o")
        .arg(&pgn_path)
        .current_dir(&workspace)
        .output()
        .expect("Failed to run export");
    assert!(export_output.status.success());

    let exported = std::fs::read_to_string(&pgn_path).expect("Failed to read exported");

    assert!(exported.contains("[Event \"47th ch-Bangahbandhu 2022\"]"));
    assert!(exported.contains("[White \"Hossain, Enam\"]"));
    assert!(exported.contains("[ECO \"B36f\"]"));
    assert!(exported.contains("1. e4 c5"));
    assert!(exported.contains("19. Nxe6"));
}
