//! Bump allocator arena management with adaptive pre-sizing.
//!
//! The Bump allocator grows as needed, but pre-sizing reduces allocation cycles
//! and improves performance for predictable file sizes. This module provides
//! heuristic capacity estimates based on source size.

/// Create a pre-sized Bump arena optimized for the given source size.
///
/// Arena capacity is estimated based on typical PHP code ratios:
/// - 1 KB source → ~4 KB arena (dense definitions)
/// - 10 KB source → ~32 KB arena (typical file)
/// - 100 KB source → ~256 KB arena (large file)
/// - 1 MB source → ~1 MB arena (huge file)
///
/// Estimates assume typical AST expansion ratios; oversizing avoids allocation
/// overhead while keeping memory waste reasonable.
pub fn create_parse_arena(source_len: usize) -> bumpalo::Bump {
    let capacity = match source_len {
        0..=1_000 => 4_096,
        1_001..=10_000 => 32_768,
        10_001..=100_000 => 262_144,
        100_001..=1_000_000 => 1_048_576,
        _ => 4_194_304, // 4 MB for very large files
    };
    bumpalo::Bump::with_capacity(capacity)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_capacities_are_reasonable() {
        let tiny = create_parse_arena(100);
        let small = create_parse_arena(5_000);
        let medium = create_parse_arena(50_000);
        let large = create_parse_arena(500_000);

        // Arena is created successfully (capacity doesn't fail)
        tiny.alloc(0u8);
        small.alloc(0u8);
        medium.alloc(0u8);
        large.alloc(0u8);
    }
}
