//! Performance benchmarks for SCID parsing
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use scidtopgn_core::{PgnOptions, ScidReader};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/data")
        .join(name)
}

/// Benchmark: Opening a database (header + index parsing)
fn bench_open_database(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    c.bench_function("open_database", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            black_box(reader);
        });
    });
}

/// Benchmark: Parsing a single game
fn bench_parse_single_game(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();

    c.bench_function("parse_single_game", |b| {
        b.iter(|| {
            let game = reader.game(black_box(0)).unwrap();
            black_box(game);
        });
    });
}

/// Benchmark: Generating PGN for a single game
fn bench_generate_pgn(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();
    let game = reader.game(0).unwrap();

    c.bench_function("generate_pgn", |b| {
        b.iter(|| {
            let pgn = game.to_pgn().unwrap();
            black_box(pgn);
        });
    });
}

/// Benchmark: Complete workflow (parse + generate PGN)
fn bench_complete_workflow(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    c.bench_function("complete_workflow", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            let game = reader.game(0).unwrap();
            let pgn = game.to_pgn().unwrap();
            black_box(pgn);
        });
    });
}

/// Benchmark: Iterating all games
fn bench_iterate_all_games(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    c.bench_function("iterate_all_games", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();
            let mut count = 0;
            for game in reader.games() {
                count += 1;
                black_box(game.unwrap());
            }
            black_box(count);
        });
    });
}

/// Benchmark: Throughput with different database sizes
fn bench_throughput_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let sizes = vec![("five", 5)];

    for (name, game_count) in sizes {
        let path = fixture_path(&format!("{}.si4", name));

        if !path.exists() {
            eprintln!("Skipping benchmark: {} (fixture not found)", name);
            continue;
        }

        group.throughput(Throughput::Elements(game_count as u64));

        group.bench_with_input(BenchmarkId::from_parameter(name), &path, |b, path| {
            b.iter(|| {
                let reader = ScidReader::open(black_box(path)).unwrap();
                let mut count = 0;
                for game in reader.games() {
                    let game = game.unwrap();
                    let _pgn = game.to_pgn().unwrap();
                    count += 1;
                }
                black_box(count);
            });
        });
    }

    group.finish();
}

/// Benchmark: Memory allocation during parsing
fn bench_memory_allocation(c: &mut Criterion) {
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        return;
    }

    c.bench_function("memory_allocation", |b| {
        b.iter(|| {
            let reader = ScidReader::open(black_box(&db_path)).unwrap();

            for game in reader.games() {
                let game = game.unwrap();
                black_box(game);
            }
        });
    });
}

/// Benchmark: PGN options (compact vs verbose)
fn bench_pgn_formats(c: &mut Criterion) {
    let mut group = c.benchmark_group("pgn_formats");
    let db_path = fixture_path("five.si4");

    if !db_path.exists() {
        eprintln!("Skipping benchmark: five.si4 not found");
        group.finish();
        return;
    }

    let reader = ScidReader::open(&db_path).unwrap();

    group.bench_function("compact", |b| {
        let options = PgnOptions {
            compact: true,
            ..Default::default()
        };

        b.iter(|| {
            let mut output = Vec::new();
            reader.write_pgn(black_box(&mut output), &options).unwrap();
            black_box(output);
        });
    });

    group.bench_function("verbose", |b| {
        let options = PgnOptions {
            verbose: true,
            ..Default::default()
        };

        b.iter(|| {
            let mut output = Vec::new();
            reader.write_pgn(black_box(&mut output), &options).unwrap();
            black_box(output);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_open_database,
    bench_parse_single_game,
    bench_generate_pgn,
    bench_complete_workflow,
    bench_iterate_all_games,
    bench_throughput_by_size,
    bench_memory_allocation,
    bench_pgn_formats,
);

criterion_main!(benches);
