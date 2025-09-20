use assert_cmd::Command;

#[test]
fn test_cli_parse_command() {
    let mut cmd = Command::cargo_bin("scidtopgn").unwrap();
    cmd.args(&["parse", "tests/data/five"]).assert().success();
}
