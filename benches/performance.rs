#![allow(dead_code)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use scidtopgn_core::{PgnOptions, ScidReader};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

fn open_database(path: &str) -> ScidReader {
    let start = Instant::now();
    let reader = ScidReader::open(path).expect("Failed to open database");
    let elapsed = start.elapsed();
    println!("Open database: {:?}", elapsed);
    reader
}

fn parse_game(reader: &ScidReader, index: usize) {
    let game = reader.game(index).expect("Failed to parse game");
    let _pgn = game.to_pgn().expect("Failed to generate PGN");
}

fn write_pgn_file(reader: &ScidReader, output_path: &str) {
    let start = Instant::now();
    let file = File::create(output_path).expect("Failed to create output file");
    let writer = BufWriter::new(file);
    reader
        .write_pgn(writer, &PgnOptions::default())
        .expect("Failed to write PGN");
    let elapsed = start.elapsed();
    println!("Write PGN file: {:?}", elapsed);
}

fn read_pgn_from_memory(reader: &ScidReader) {
    let start = Instant::now();
    let _pgn = reader
        .to_pgn(&PgnOptions::default())
        .expect("Failed to generate PGN");
    let elapsed = start.elapsed();
    println!("Read PGN from memory: {:?}", elapsed);
}

fn iterate_games(reader: &ScidReader) {
    let start = Instant::now();
    let mut count = 0;
    for result in reader.games() {
        let game = result.expect("Failed to parse game");
        count += 1;
    }
    let elapsed = start.elapsed();
    println!("Iterate games: {} games in {:?}", count, elapsed);
}

fn benchmark_open_database(c: &mut Criterion) {
    let mut group = c.benchmark_group("open_database");

    for size in [100, 1000, 10000, 100000] {
        group.bench_with_input(BenchmarkId::new("open", size), &size, |b, &size| {
            let path = format!("tests/data/five.si4");
            b.iter(|| {
                let _reader = open_database(&path);
            });
        });
    }

    group.finish();
}

fn benchmark_parse_game(c: &mut Criterion) {
    let reader = open_database("tests/data/five.si4");

    let mut group = c.benchmark_group("parse_game");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("parse", size), &size, |b, &size| {
            b.iter(|| {
                for i in 0..size.min(reader.game_count()) {
                    parse_game(&reader, i);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_write_pgn(c: &mut Criterion) {
    let reader = open_database("tests/data/five.si4");
    let output_path = "/tmp/test_output.pgn";

    let mut group = c.benchmark_group("write_pgn");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("write", size), &size, |b, &size| {
            b.iter(|| {
                write_pgn_file(&reader, output_path);
            });
        });
    }

    group.finish();
}

fn benchmark_read_pgn(c: &mut Criterion) {
    let reader = open_database("tests/data/five.si4");

    let mut group = c.benchmark_group("read_pgn");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("read", size), &size, |b, &size| {
            b.iter(|| {
                read_pgn_from_memory(&reader);
            });
        });
    }

    group.finish();
}

fn benchmark_iterate_games(c: &mut Criterion) {
    let reader = open_database("tests/data/five.si4");

    let mut group = c.benchmark_group("iterate_games");

    for size in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("iterate", size), &size, |b, &size| {
            b.iter(|| {
                iterate_games(&reader);
            });
        });
    }

    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");

    group.bench_function("memory_streaming", |b| {
        b.iter(|| {
            let reader = open_database("tests/data/five.si4");
            let mut count = 0;
            for result in reader.games() {
                let _game = result.expect("Failed to parse game");
                count += 1;
            }
            count
        });
    });

    group.bench_function("memory_all_pgn", |b| {
        b.iter(|| {
            let reader = open_database("tests/data/five.si4");
            let _pgn = reader
                .to_pgn(&PgnOptions::default())
                .expect("Failed to generate PGN");
        });
    });

    group.finish();
}

fn benchmark_pipeline(c: &mut Criterion) {
    let reader = open_database("tests/data/five.si4");

    let mut group = c.benchmark_group("pipeline");

    group.bench_function("full_conversion", |b| {
        b.iter(|| {
            let output_path = "/tmp/test_pipeline.pgn";
            write_pgn_file(&reader, output_path);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_open_database,
    benchmark_parse_game,
    benchmark_write_pgn,
    benchmark_read_pgn,
    benchmark_iterate_games,
    benchmark_memory_usage,
    benchmark_pipeline
);

criterion_main!(benches);
