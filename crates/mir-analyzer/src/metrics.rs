//! Opt-in performance counters. Activated by `MIR_TIMING=1`.
//!
//! Captures data we need to decide whether the pull-based refactor
//! (Phases 3–5 of `sequential-popping-parasol.md`) is justified: how often
//! `FileAnalyzer`'s post-Pass-2 retry loop iterates, how much time each
//! iteration costs, and how many lazy loads it triggers.
//!
//! When `MIR_TIMING` is unset the counters are no-ops (an `AtomicBool` check
//! plus a branch). Safe to leave compiled in.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();

fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var("MIR_TIMING")
            .map(|v| v != "0" && !v.is_empty())
            .unwrap_or(false)
    })
}

/// Globally-aggregated counters. Logged on `dump()` (e.g. CLI end-of-run).
#[derive(Default)]
pub struct Counters {
    /// Number of `FileAnalyzer::analyze` invocations.
    pub file_analyses: AtomicU64,
    /// Number of Pass-2 invocations summed across all analyses. Equals
    /// `file_analyses` if the retry loop never iterated.
    pub body_analysis_runs: AtomicU64,
    /// Iterations of the retry loop (`file_analyzer.rs`'s
    /// `MAX_LAZY_LOAD_ITERATIONS` block) that actually executed.
    pub retry_iterations: AtomicU64,
    /// Lazy loads attempted (one per unresolved FQCN passed to
    /// `load_class_transitive`).
    pub lazy_loads_attempted: AtomicU64,
    /// Lazy loads that resolved to a class (the call returned `Some`).
    pub lazy_loads_resolved: AtomicU64,
    /// Total Pass-2 wall time in microseconds.
    pub body_analysis_micros: AtomicU64,

    /// `collect_and_ingest_file` calls that hit the on-disk stub cache.
    pub stub_cache_hits: AtomicU64,
    /// `collect_and_ingest_file` calls that missed and had to parse.
    pub stub_cache_misses: AtomicU64,

    // Failure-bucket counts for `AnalysisSession::load_class`. Sum of
    // these three == `lazy_loads_attempted - lazy_loads_resolved` (the
    // total failure count). Diagnoses *why* lazy-load fails on real
    // workloads — drives the Phase 3 decision in `docs/perf-baseline.md`.
    /// No resolver configured (`with_psr4` / `with_class_resolver` never
    /// called).
    pub ll_fail_no_resolver: AtomicU64,
    /// Resolver returned `None` for the FQCN (PSR-4 prefix / classmap
    /// didn't match).
    pub ll_fail_resolver_none: AtomicU64,
    /// Resolver mapped the FQCN to a path, but `SourceProvider::read`
    /// returned `None` (file unreadable / missing).
    pub ll_fail_source_unreadable: AtomicU64,
    /// Resolver mapped, source read, but after `ingest_file` the class is
    /// still not present in the index. Most interesting bucket — points at
    /// FQCN normalization mismatch, Pass-1 collection gap, or
    /// resolver-points-at-wrong-file.
    pub ll_fail_ingest_then_missing: AtomicU64,
}

static COUNTERS: Counters = Counters {
    file_analyses: AtomicU64::new(0),
    body_analysis_runs: AtomicU64::new(0),
    retry_iterations: AtomicU64::new(0),
    lazy_loads_attempted: AtomicU64::new(0),
    lazy_loads_resolved: AtomicU64::new(0),
    body_analysis_micros: AtomicU64::new(0),
    stub_cache_hits: AtomicU64::new(0),
    stub_cache_misses: AtomicU64::new(0),
    ll_fail_no_resolver: AtomicU64::new(0),
    ll_fail_resolver_none: AtomicU64::new(0),
    ll_fail_source_unreadable: AtomicU64::new(0),
    ll_fail_ingest_then_missing: AtomicU64::new(0),
};

pub fn record_file_analysis() {
    if enabled() {
        COUNTERS.file_analyses.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_body_analysis(duration_micros: u64) {
    if enabled() {
        COUNTERS.body_analysis_runs.fetch_add(1, Ordering::Relaxed);
        COUNTERS
            .body_analysis_micros
            .fetch_add(duration_micros, Ordering::Relaxed);
    }
}

pub fn record_retry_iteration() {
    if enabled() {
        COUNTERS.retry_iterations.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_lazy_load_attempt(resolved: bool) {
    if enabled() {
        COUNTERS
            .lazy_loads_attempted
            .fetch_add(1, Ordering::Relaxed);
        if resolved {
            COUNTERS.lazy_loads_resolved.fetch_add(1, Ordering::Relaxed);
        }
    }
}

pub fn record_stub_cache_hit() {
    if enabled() {
        COUNTERS.stub_cache_hits.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_stub_cache_miss() {
    if enabled() {
        COUNTERS.stub_cache_misses.fetch_add(1, Ordering::Relaxed);
    }
}

/// Reason for a `load_class` failure. Variants align 1:1 with the
/// `ll_fail_*` counters; see [`Counters`] for semantics.
#[derive(Copy, Clone, Debug)]
pub enum LazyLoadFailure {
    NoResolver,
    ResolverNone,
    SourceUnreadable,
    IngestThenMissing,
}

/// Up to 40 sampled FQCNs per failure bucket. Diagnostic only — printed by
/// `dump()` when `MIR_TIMING=1`. Behind a `Mutex` because failures may
/// happen concurrently in parallel analysis paths.
static FAILURE_SAMPLES: std::sync::Mutex<FailureSamples> =
    std::sync::Mutex::new(FailureSamples::new());

struct FailureSamples {
    no_resolver: Vec<String>,
    resolver_none: Vec<String>,
    source_unreadable: Vec<String>,
    ingest_then_missing: Vec<String>,
}

impl FailureSamples {
    const fn new() -> Self {
        Self {
            no_resolver: Vec::new(),
            resolver_none: Vec::new(),
            source_unreadable: Vec::new(),
            ingest_then_missing: Vec::new(),
        }
    }

    fn push(bucket: &mut Vec<String>, fqcn: &str) {
        if bucket.len() < 40 && !bucket.iter().any(|s| s == fqcn) {
            bucket.push(fqcn.to_string());
        }
    }
}

pub fn record_lazy_load_failure(reason: LazyLoadFailure, fqcn: &str) {
    if !enabled() {
        return;
    }
    let counter = match reason {
        LazyLoadFailure::NoResolver => &COUNTERS.ll_fail_no_resolver,
        LazyLoadFailure::ResolverNone => &COUNTERS.ll_fail_resolver_none,
        LazyLoadFailure::SourceUnreadable => &COUNTERS.ll_fail_source_unreadable,
        LazyLoadFailure::IngestThenMissing => &COUNTERS.ll_fail_ingest_then_missing,
    };
    counter.fetch_add(1, Ordering::Relaxed);
    let mut samples = FAILURE_SAMPLES.lock().unwrap();
    let bucket = match reason {
        LazyLoadFailure::NoResolver => &mut samples.no_resolver,
        LazyLoadFailure::ResolverNone => &mut samples.resolver_none,
        LazyLoadFailure::SourceUnreadable => &mut samples.source_unreadable,
        LazyLoadFailure::IngestThenMissing => &mut samples.ingest_then_missing,
    };
    FailureSamples::push(bucket, fqcn);
}

fn render_samples() -> String {
    let s = FAILURE_SAMPLES.lock().unwrap();
    let mut out = String::new();
    let buckets = [
        ("no_resolver", &s.no_resolver),
        ("resolver_none", &s.resolver_none),
        ("source_unreadable", &s.source_unreadable),
        ("ingest_then_missing", &s.ingest_then_missing),
    ];
    for (name, b) in buckets {
        if b.is_empty() {
            continue;
        }
        out.push_str(&format!("\n  sample {} ({}):", name, b.len()));
        for fqcn in b.iter().take(20) {
            out.push_str(&format!("\n    {fqcn}"));
        }
    }
    out
}

/// RAII scope guard for measuring Pass-2 wall time. Drop emits the record.
pub struct BodyAnalysisScope {
    start: Option<Instant>,
}

impl BodyAnalysisScope {
    pub fn new() -> Self {
        Self {
            start: if enabled() {
                Some(Instant::now())
            } else {
                None
            },
        }
    }
}

impl Default for BodyAnalysisScope {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for BodyAnalysisScope {
    fn drop(&mut self) {
        if let Some(t0) = self.start {
            record_body_analysis(t0.elapsed().as_micros() as u64);
        }
    }
}

/// Render counters as a human-readable block. Returns `None` if metrics are
/// disabled. Intended to be printed at end of batch (`ProjectAnalyzer`) or
/// session shutdown.
pub fn dump() -> Option<String> {
    if !enabled() {
        return None;
    }
    let analyses = COUNTERS.file_analyses.load(Ordering::Relaxed);
    let body_analysis_runs = COUNTERS.body_analysis_runs.load(Ordering::Relaxed);
    let retries = COUNTERS.retry_iterations.load(Ordering::Relaxed);
    let attempts = COUNTERS.lazy_loads_attempted.load(Ordering::Relaxed);
    let resolved = COUNTERS.lazy_loads_resolved.load(Ordering::Relaxed);
    let body_analysis_micros = COUNTERS.body_analysis_micros.load(Ordering::Relaxed);
    let cache_hits = COUNTERS.stub_cache_hits.load(Ordering::Relaxed);
    let cache_misses = COUNTERS.stub_cache_misses.load(Ordering::Relaxed);
    let ll_no_resolver = COUNTERS.ll_fail_no_resolver.load(Ordering::Relaxed);
    let ll_resolver_none = COUNTERS.ll_fail_resolver_none.load(Ordering::Relaxed);
    let ll_source_unreadable = COUNTERS.ll_fail_source_unreadable.load(Ordering::Relaxed);
    let ll_ingest_missing = COUNTERS.ll_fail_ingest_then_missing.load(Ordering::Relaxed);

    let avg_iterations = if analyses == 0 {
        0.0
    } else {
        body_analysis_runs as f64 / analyses as f64
    };
    let avg_pass2_us = body_analysis_micros
        .checked_div(body_analysis_runs)
        .unwrap_or(0);

    let samples = render_samples();
    Some(format!(
        "mir metrics:\n  \
         file analyses        : {analyses}\n  \
         pass-2 runs          : {body_analysis_runs}  (avg per analysis: {avg_iterations:.3})\n  \
         retry iterations     : {retries}\n  \
         pass-2 wall time     : {body_analysis_micros} us  (avg/run: {avg_pass2_us} us)\n  \
         lazy load attempts   : {attempts}  resolved: {resolved}\n  \
         lazy load failures   : no_resolver={ll_no_resolver}  resolver_none={ll_resolver_none}  \
         source_unreadable={ll_source_unreadable}  ingest_then_missing={ll_ingest_missing}\n  \
         stub cache           : hits {cache_hits}  misses {cache_misses}{samples}"
    ))
}
