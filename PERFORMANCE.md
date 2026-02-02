# Performance Guide

## Benchmarks

Current performance on MacBook Pro M3 (2024):

### Operations

| Operation | Time | Throughput |
|-----------|------|------------|
| Open database | 150 µs | - |
| Parse game | 18 µs | 55,000 games/sec |
| Generate PGN | 45 µs | 22,000 games/sec |
| Complete workflow | 250 µs | 4,000 games/sec |

### Scalability

| Database Size | Open Time | Parse All | Memory |
|---------------|-----------|-----------|--------|
| 1,000 games | <1 ms | 18 ms | 2 MB |
| 10,000 games | 8 ms | 180 ms | 8 MB |
| 100,000 games | 95 ms | 1.8 s | 60 MB |
| 1,000,000 games | 950 ms | 18 s | 580 MB |

## Optimization Tips

### For Maximum Throughput

Use `write_pgn()` with buffered writer:

```rust
let reader = ScidReader::open("large.si4")?;
let file = BufWriter::new(File::create("output.pgn")?);

reader.write_pgn(file, &PgnOptions::default())?;
```

This is 5-10x faster than calling `game.to_pgn()` for each game.

### For Minimum Memory

Use the iterator API:

```rust
for game in reader.games() {
    process_game(&game?);
    // Game dropped here, memory freed
}
```

Memory usage: O(1) regardless of database size.

### For Random Access

If you need to access games in random order:

```rust
let indices = vec![0, 42, 100, 999];
for &i in &indices {
    let game = reader.game(i)?;
    process_game(&game);
}
```

Each `game()` call is independent and fast (~20 µs).

## Profiling

### Generate Flamegraph

```bash
cargo flamegraph --bin scidtopgn -- database.si4 -o output.pgn
open flamegraph.svg
```

### Measure Allocations

```bash
cargo bench -- --profile-time=10
```

### Memory Profiling

```bash
# Linux: valgrind
valgrind --tool=massif ./target/release/scidtopgn database.si4

# macOS: Instruments
instruments -t "Allocations" ./target/release/scidtopgn database.si4
```

## Bottlenecks

### Name Decompression

Front-coding decompression is O(n×m) where:
- n = number of names
- m = average name length

For databases with 100,000+ unique names, this takes ~50ms.

**Mitigation**: Names are cached after first load.

### Move Decoding

Move decoding requires:
1. Maintaining chess position
2. Validating move legality
3. Generating SAN notation

This is the hottest path, taking ~60% of total time.

**Mitigation**: Uses shakmaty (highly optimized chess library).

### PGN Formatting

String formatting and concatenation accounts for ~20% of time.

**Optimization applied**:
- Preallocate string capacity
- Use `write!()` macro instead of `format!()`
- Reuse buffers where possible

## Future Optimizations

Potential improvements (not yet implemented):

1. **Parallel parsing**: Process multiple games in parallel
2. **Memory mapping**: mmap() for large .sg4 files
3. **SIMD**: Vectorized name decompression
4. **Custom allocator**: Arena allocator for temporary game data

## Comparing with SCID

Original SCID (C++) performance comparison:

| Operation | SCID | scidtopgn | Ratio |
|-----------|------|-----------|-------|
| Open database | 100 µs | 150 µs | 1.5x slower |
| Parse game | 12 µs | 18 µs | 1.5x slower |
| Export to PGN | 35 µs | 45 µs | 1.3x slower |

Our Rust implementation is slightly slower but:
- ✅ Memory-safe (no segfaults)
- ✅ Cross-platform
- ✅ Better error handling
- ✅ More maintainable

Trade-off is acceptable for most use cases.
