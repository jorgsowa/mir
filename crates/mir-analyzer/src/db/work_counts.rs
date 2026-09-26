//! Per-database counts of analysis work, so tests can assert which paths ran.

use std::sync::atomic::{AtomicU64, Ordering};

/// A unit of analysis work counted by [`WorkCounts`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// `BodyAnalyzer::analyze_bodies` walked a whole file.
    WholeFileWalk,
    /// `infer_scope` executed (not served from its memo).
    ScopeAnalysis,
    /// `ResolvedSymbol`s produced by a whole-file walk.
    SymbolAllocated,
    /// `name_at` answered from compact facts.
    NameAtCompact,
    /// `name_at` fell back to a symbol walk.
    NameAtFallback,
    /// `symbol_at` answered from compact facts.
    SymbolAtCompact,
    /// `symbol_at` fell back to a symbol walk.
    SymbolAtFallback,
}

const WORK_KINDS: usize = Work::SymbolAtFallback as usize + 1;

/// Always-on counters, shared by every clone of one database. Unlike the
/// process-global [`crate::metrics`], concurrent sessions never see each
/// other's counts.
#[derive(Default, Debug)]
pub struct WorkCounts([AtomicU64; WORK_KINDS]);

impl WorkCounts {
    pub(crate) fn add(&self, work: Work, n: u64) {
        self.0[work as usize].fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self, work: Work) -> u64 {
        self.0[work as usize].load(Ordering::Relaxed)
    }
}
