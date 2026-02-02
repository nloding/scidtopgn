//! Benchmarks for name file parsing

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use scidtopgn_core::database::names::parse_name_database;
use std::path::PathBuf;

fn test_data_path(filename: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(filename)
}

fn bench_parse_complete_name_file(c: &mut Criterion) {
    let path = test_data_path("five.sn4");

    if !path.exists() {
        eprintln!("Skipping benchmark: five.sn4 not found");
        return;
    }

    c.bench_function("parse_five_sn4", |b| {
        b.iter(|| {
            let names = parse_name_database(black_box(&path)).unwrap();
            black_box(names);
        });
    });
}

criterion_group!(benches, bench_parse_complete_name_file);
criterion_main!(benches);
