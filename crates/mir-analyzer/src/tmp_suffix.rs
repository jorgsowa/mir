//! A per-writer suffix for the temp files the caches rename into place.
//!
//! Both call sites used `std::process::id()`, which is not available on every
//! target this crate builds for: wasm has no pids and the call panics. That
//! target is not hypothetical — `crates/mir-wasm` builds for
//! `wasm32-unknown-unknown` and the docs playground ships it. It survives only
//! because analysing one source string never reaches either cache; an embedder
//! that opens a project does, on the analysis cache's periodic flush, so the
//! panic arrives seconds in rather than at start-up where it would be obvious.
//!
//! The suffix only has to keep two writers racing on the same entry from
//! choosing the same name; the rename that follows is what makes the write
//! atomic. A pid gives that between processes and a counter gives it within
//! one, so take the pid where there is one — where there is not, one process is
//! all there is.

use std::sync::atomic::{AtomicU32, Ordering};

/// A suffix unique among concurrent writers of the same cache entry.
pub(crate) fn next() -> u32 {
    process_id().unwrap_or_else(counter)
}

/// This process's id, or `None` on a target that has no processes.
fn process_id() -> Option<u32> {
    #[cfg(not(target_family = "wasm"))]
    {
        Some(std::process::id())
    }
    #[cfg(target_family = "wasm")]
    {
        None
    }
}

/// A suffix that separates writers within one process.
fn counter() -> u32 {
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// The one thing the suffix has to guarantee.
    #[test]
    fn the_counter_separates_concurrent_writers() {
        let threads: Vec<_> = (0..8)
            .map(|_| std::thread::spawn(|| (0..200).map(|_| counter()).collect::<Vec<_>>()))
            .collect();
        let all: Vec<u32> = threads
            .into_iter()
            .flat_map(|t| t.join().unwrap())
            .collect();
        assert_eq!(
            all.iter().collect::<HashSet<_>>().len(),
            all.len(),
            "two writers chose the same temp suffix"
        );
    }

    /// Whatever the target, asking for a suffix answers instead of panicking.
    #[test]
    fn next_answers_without_a_pid() {
        assert_ne!(counter(), counter());
        let _ = next();
    }
}
