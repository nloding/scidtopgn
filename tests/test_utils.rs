use std::path::{Path, PathBuf};
use anyhow::Result;

pub fn five_test_data() -> PathBuf {
    PathBuf::from("tests/data/five")
}

pub fn create_test_database() -> Result<scidtopgn::api::ScidDatabase> {
    let base = five_test_data();
    Ok(scidtopgn::api::ScidDatabase::open(&base)?)
}

pub fn create_test_database_with_path(base: &Path) -> Result<scidtopgn::api::ScidDatabase> {
    Ok(scidtopgn::api::ScidDatabase::open(base)?)
}

pub fn test_files_exist() -> bool {
    let base = five_test_data();
    base.with_extension("si4").exists()
        && base.with_extension("sn4").exists()
        && base.with_extension("sg4").exists()
}

pub fn load_reference_pgn() -> Result<String> {
    let p = PathBuf::from("tests/data/five.pgn");
    Ok(std::fs::read_to_string(p)?)
}
