//! Memory usage benchmarks
//!
//! These tests measure peak memory usage during parsing.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator to track memory usage
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static PEAK_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            let size = layout.size();
            let old = ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let new = old + size;

            /// Update peak memory usage
            let mut peak = PEAK_ALLOCATED.load(Ordering::Relaxed);
            while new > peak {
                match PEAK_ALLOCATED.compare_exchange_weak(
                    peak,
                    new,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

#[cfg(test)]
mod tests {
    use super::*;
    use scidtopgn_core::ScidReader;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    fn reset_memory_tracking() {
        ALLOCATED.store(0, Ordering::Relaxed);
        PEAK_ALLOCATED.store(0, Ordering::Relaxed);
    }

    fn get_peak_memory() -> usize {
        PEAK_ALLOCATED.load(Ordering::Relaxed)
    }

    #[test]
    fn test_memory_usage_single_game() {
        reset_memory_tracking();

        let db_path = fixture_path("minimal/minimal.si4");
        let reader = ScidReader::open(&db_path).unwrap();

        let game = reader.game(0).unwrap();
        let _pgn = game.to_pgn().unwrap();

        let peak = get_peak_memory();

        println!("Peak memory for single game: {} KB", peak / 1024);

        // Single game should use <100 KB
        assert!(
            peak < 100 * 1024,
            "Single game used {} KB, expected <100 KB",
            peak / 1024
        );
    }

    #[test]
    fn test_memory_usage_iterator() {
        reset_memory_tracking();

        let db_path = fixture_path("minimal/minimal.si4");
        let reader = ScidReader::open(&db_path).unwrap();

        /// Iterate all games (memory should not grow linearly)
        for game in reader.games() {
            let game = game.unwrap();
            let _pgn = game.to_pgn().unwrap();
        }

        let peak = get_peak_memory();

        println!("Peak memory for iterator: {} KB", peak / 1024);

        /// Iterator should not accumulate memory
        /// (If we had 1000 games, peak should still be low)
        assert!(
            peak < 1024 * 1024, // <1 MB
            "Iterator used {} MB, should stay low",
            peak / (1024 * 1024)
        );
    }
}
