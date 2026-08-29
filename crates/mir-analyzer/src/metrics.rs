//! Opt-in performance counters. Activated by `MIR_TIMING=1`.
//!
//! Captures data we need to decide whether the pull-based refactor
//! (Phases 3–5 of `sequential-popping-parasol.md`) is justified: how often
//! `FileAnalyzer`'s post-body-analysis retry loop iterates, how much time each
//! iteration costs, and how many lazy loads it triggers.
//!
//! When `MIR_TIMING` is unset the counters are no-ops (an `AtomicBool` check
//! plus a branch). Safe to leave compiled in.

use std::collections::BTreeMap;
use std::mem::size_of;
#[cfg(test)]
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

static ENABLED: OnceLock<bool> = OnceLock::new();
#[cfg(test)]
static TEST_ENABLED: AtomicBool = AtomicBool::new(false);

fn enabled() -> bool {
    #[cfg(test)]
    if TEST_ENABLED.load(Ordering::Relaxed) {
        return true;
    }
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
    /// Number of body-analysis invocations summed across all analyses.
    pub body_analysis_runs: AtomicU64,
    /// Lazy loads attempted (one per unresolved FQCN passed to `load_class`).
    pub lazy_loads_attempted: AtomicU64,
    /// Lazy loads that resolved to a class (the call returned `Some`).
    pub lazy_loads_resolved: AtomicU64,
    /// Total body-analysis wall time in microseconds.
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
    /// FQCN normalization mismatch, definition-collection collection gap, or
    /// resolver-points-at-wrong-file.
    pub ll_fail_ingest_then_missing: AtomicU64,

    /// Number of times `lookup_function_node_for_decl` fell through to the
    /// O(N) short-name scan over all workspace functions. Non-zero means we
    /// should build a short-name → FQN index.
    pub fn_short_name_scans: AtomicU64,

    // FlowState::branch() clone profiling — upper bound on COW savings.
    // "upper bound" because COW only helps branches that never write the field;
    // branches that do write still pay the clone (just deferred to make_mut).
    /// Total FlowState::branch() calls.
    pub flow_branch_calls: AtomicU64,
    /// Sum of read_vars.len() at each branch() call.
    pub flow_branch_read_vars_entries: AtomicU64,
    /// Sum of var_locations.len() at each branch() call.
    pub flow_branch_var_locs_entries: AtomicU64,
    /// Sum of last_write_locs.len() at each branch() call.
    pub flow_branch_last_write_entries: AtomicU64,

    /// Number of whole-file body walks (`BodyAnalyzer::analyze_bodies`).
    pub whole_file_body_walks: AtomicU64,
    /// Number of per-scope salsa executions (`infer_scope`).
    pub scopes_analyzed: AtomicU64,
    /// Total `ResolvedSymbol` entries produced across all analysis paths.
    pub symbols_allocated: AtomicU64,
    /// Number of targeted cursor-resolution calls (`resolve_at`).
    pub name_at_calls: AtomicU64,
    /// Total wall time spent in `name_at`, in microseconds.
    pub name_at_micros: AtomicU64,
    /// `name_at` queries answered by compact facts.
    pub name_at_compact_hits: AtomicU64,
    /// `name_at` queries that still required a fallback symbol walk.
    pub name_at_fallback_walks: AtomicU64,
    /// Number of targeted cursor-resolution calls (`resolve_at`).
    pub resolve_at_calls: AtomicU64,
    /// Total wall time spent in `resolve_at`, in microseconds.
    pub resolve_at_micros: AtomicU64,
    /// `resolve_at` queries answered by compact typed facts.
    pub resolve_at_compact_hits: AtomicU64,
    /// `resolve_at` queries that still required a fallback symbol walk.
    pub resolve_at_fallback_walks: AtomicU64,
    /// Number of cursor hover lookups (`hover_at`).
    pub hover_at_calls: AtomicU64,
    /// Total wall time spent in `hover_at`, in microseconds.
    pub hover_at_micros: AtomicU64,
    /// Number of cursor definition lookups (`definition_at`).
    pub definition_at_calls: AtomicU64,
    /// Total wall time spent in `definition_at`, in microseconds.
    pub definition_at_micros: AtomicU64,

    /// Lower-bound retained bytes for `analyze_file` memos.
    pub analyze_file_retained_bytes: AtomicU64,
    /// Number of `analyze_file` memo payloads measured.
    pub analyze_file_retained_samples: AtomicU64,
    /// Lower-bound retained bytes for `infer_scope` memos.
    pub infer_scope_retained_bytes: AtomicU64,
    /// Number of `infer_scope` memo payloads measured.
    pub infer_scope_retained_samples: AtomicU64,
    /// Lower-bound retained bytes for `infer_function` memos.
    pub infer_function_retained_bytes: AtomicU64,
    /// Number of `infer_function` memo payloads measured.
    pub infer_function_retained_samples: AtomicU64,
}

static COUNTERS: Counters = Counters {
    file_analyses: AtomicU64::new(0),
    body_analysis_runs: AtomicU64::new(0),
    lazy_loads_attempted: AtomicU64::new(0),
    lazy_loads_resolved: AtomicU64::new(0),
    body_analysis_micros: AtomicU64::new(0),
    stub_cache_hits: AtomicU64::new(0),
    stub_cache_misses: AtomicU64::new(0),
    ll_fail_no_resolver: AtomicU64::new(0),
    ll_fail_resolver_none: AtomicU64::new(0),
    ll_fail_source_unreadable: AtomicU64::new(0),
    ll_fail_ingest_then_missing: AtomicU64::new(0),
    fn_short_name_scans: AtomicU64::new(0),
    flow_branch_calls: AtomicU64::new(0),
    flow_branch_read_vars_entries: AtomicU64::new(0),
    flow_branch_var_locs_entries: AtomicU64::new(0),
    flow_branch_last_write_entries: AtomicU64::new(0),
    whole_file_body_walks: AtomicU64::new(0),
    scopes_analyzed: AtomicU64::new(0),
    symbols_allocated: AtomicU64::new(0),
    name_at_calls: AtomicU64::new(0),
    name_at_micros: AtomicU64::new(0),
    name_at_compact_hits: AtomicU64::new(0),
    name_at_fallback_walks: AtomicU64::new(0),
    resolve_at_calls: AtomicU64::new(0),
    resolve_at_micros: AtomicU64::new(0),
    resolve_at_compact_hits: AtomicU64::new(0),
    resolve_at_fallback_walks: AtomicU64::new(0),
    hover_at_calls: AtomicU64::new(0),
    hover_at_micros: AtomicU64::new(0),
    definition_at_calls: AtomicU64::new(0),
    definition_at_micros: AtomicU64::new(0),
    analyze_file_retained_bytes: AtomicU64::new(0),
    analyze_file_retained_samples: AtomicU64::new(0),
    infer_scope_retained_bytes: AtomicU64::new(0),
    infer_scope_retained_samples: AtomicU64::new(0),
    infer_function_retained_bytes: AtomicU64::new(0),
    infer_function_retained_samples: AtomicU64::new(0),
};

static BODY_WALKS_BY_FILE: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());
static SCOPES_BY_FILE: Mutex<BTreeMap<String, u64>> = Mutex::new(BTreeMap::new());

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

pub fn record_fn_short_name_scan() {
    if enabled() {
        COUNTERS.fn_short_name_scans.fetch_add(1, Ordering::Relaxed);
    }
}

pub fn record_flow_branch(read_vars: usize, var_locs: usize, last_write: usize) {
    if enabled() {
        COUNTERS.flow_branch_calls.fetch_add(1, Ordering::Relaxed);
        COUNTERS
            .flow_branch_read_vars_entries
            .fetch_add(read_vars as u64, Ordering::Relaxed);
        COUNTERS
            .flow_branch_var_locs_entries
            .fetch_add(var_locs as u64, Ordering::Relaxed);
        COUNTERS
            .flow_branch_last_write_entries
            .fetch_add(last_write as u64, Ordering::Relaxed);
    }
}

pub fn record_whole_file_body_walk(file: &str, symbols_allocated: usize) {
    if !enabled() {
        return;
    }
    COUNTERS
        .whole_file_body_walks
        .fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .symbols_allocated
        .fetch_add(symbols_allocated as u64, Ordering::Relaxed);
    let mut walks = BODY_WALKS_BY_FILE.lock().unwrap();
    *walks.entry(file.to_string()).or_insert(0) += 1;
}

pub fn record_scope_analysis(file: &str, symbols_allocated: usize) {
    if !enabled() {
        return;
    }
    COUNTERS.scopes_analyzed.fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .symbols_allocated
        .fetch_add(symbols_allocated as u64, Ordering::Relaxed);
    let mut scopes = SCOPES_BY_FILE.lock().unwrap();
    *scopes.entry(file.to_string()).or_insert(0) += 1;
}

pub fn record_resolve_at(duration_micros: u64) {
    if !enabled() {
        return;
    }
    COUNTERS.resolve_at_calls.fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .resolve_at_micros
        .fetch_add(duration_micros, Ordering::Relaxed);
}

pub fn record_resolve_at_compact_hit() {
    if !enabled() {
        return;
    }
    COUNTERS
        .resolve_at_compact_hits
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_resolve_at_fallback_walk() {
    if !enabled() {
        return;
    }
    COUNTERS
        .resolve_at_fallback_walks
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_name_at(duration_micros: u64) {
    if !enabled() {
        return;
    }
    COUNTERS.name_at_calls.fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .name_at_micros
        .fetch_add(duration_micros, Ordering::Relaxed);
}

pub fn record_name_at_compact_hit() {
    if !enabled() {
        return;
    }
    COUNTERS
        .name_at_compact_hits
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_name_at_fallback_walk() {
    if !enabled() {
        return;
    }
    COUNTERS
        .name_at_fallback_walks
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_hover_at(duration_micros: u64) {
    if !enabled() {
        return;
    }
    COUNTERS.hover_at_calls.fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .hover_at_micros
        .fetch_add(duration_micros, Ordering::Relaxed);
}

pub fn record_definition_at(duration_micros: u64) {
    if !enabled() {
        return;
    }
    COUNTERS.definition_at_calls.fetch_add(1, Ordering::Relaxed);
    COUNTERS
        .definition_at_micros
        .fetch_add(duration_micros, Ordering::Relaxed);
}

pub fn record_analyze_file_retained(issues: usize, ref_locs: usize) {
    if !enabled() {
        return;
    }
    let bytes = (issues * size_of::<mir_issues::Issue>()
        + ref_locs * size_of::<crate::db::RefLoc>()) as u64;
    COUNTERS
        .analyze_file_retained_bytes
        .fetch_add(bytes, Ordering::Relaxed);
    COUNTERS
        .analyze_file_retained_samples
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_infer_scope_retained(
    issues: usize,
    ref_locs: usize,
    inferred_functions: usize,
    inferred_methods: usize,
    inferred_properties: usize,
) {
    if !enabled() {
        return;
    }
    let bytes = (issues * size_of::<mir_issues::Issue>()
        + ref_locs * size_of::<crate::db::RefLoc>()
        + inferred_functions * (size_of::<Arc<str>>() + size_of::<mir_types::Type>())
        + inferred_methods
            * (size_of::<Arc<str>>() + size_of::<Arc<str>>() + size_of::<mir_types::Type>())
        + inferred_properties
            * (size_of::<Arc<str>>() + size_of::<Arc<str>>() + size_of::<mir_types::Type>()))
        as u64;
    COUNTERS
        .infer_scope_retained_bytes
        .fetch_add(bytes, Ordering::Relaxed);
    COUNTERS
        .infer_scope_retained_samples
        .fetch_add(1, Ordering::Relaxed);
}

pub fn record_infer_function_retained(issues: usize, ref_locs: usize, has_return_type: bool) {
    if !enabled() {
        return;
    }
    let bytes = (issues * size_of::<mir_issues::Issue>()
        + ref_locs * size_of::<crate::db::RefLoc>()
        + usize::from(has_return_type) * size_of::<mir_types::Type>()) as u64;
    COUNTERS
        .infer_function_retained_bytes
        .fetch_add(bytes, Ordering::Relaxed);
    COUNTERS
        .infer_function_retained_samples
        .fetch_add(1, Ordering::Relaxed);
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

fn render_top_counts(label: &str, counts: &Mutex<BTreeMap<String, u64>>) -> String {
    let counts = counts.lock().unwrap();
    if counts.is_empty() {
        return String::new();
    }
    let mut top: Vec<(&str, u64)> = counts.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    top.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let mut out = String::new();
    out.push_str(&format!("\n  top {label}:"));
    for (file, n) in top.into_iter().take(10) {
        out.push_str(&format!("\n    {n:>4}  {file}"));
    }
    out
}

/// RAII scope guard for measuring body-analysis wall time. Drop emits the record.
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
    let attempts = COUNTERS.lazy_loads_attempted.load(Ordering::Relaxed);
    let resolved = COUNTERS.lazy_loads_resolved.load(Ordering::Relaxed);
    let body_analysis_micros = COUNTERS.body_analysis_micros.load(Ordering::Relaxed);
    let cache_hits = COUNTERS.stub_cache_hits.load(Ordering::Relaxed);
    let cache_misses = COUNTERS.stub_cache_misses.load(Ordering::Relaxed);
    let ll_no_resolver = COUNTERS.ll_fail_no_resolver.load(Ordering::Relaxed);
    let ll_resolver_none = COUNTERS.ll_fail_resolver_none.load(Ordering::Relaxed);
    let ll_source_unreadable = COUNTERS.ll_fail_source_unreadable.load(Ordering::Relaxed);
    let ll_ingest_missing = COUNTERS.ll_fail_ingest_then_missing.load(Ordering::Relaxed);
    let fn_short_scans = COUNTERS.fn_short_name_scans.load(Ordering::Relaxed);
    let branch_calls = COUNTERS.flow_branch_calls.load(Ordering::Relaxed);
    let branch_read_vars = COUNTERS
        .flow_branch_read_vars_entries
        .load(Ordering::Relaxed);
    let branch_var_locs = COUNTERS
        .flow_branch_var_locs_entries
        .load(Ordering::Relaxed);
    let branch_last_write = COUNTERS
        .flow_branch_last_write_entries
        .load(Ordering::Relaxed);
    let whole_file_body_walks = COUNTERS.whole_file_body_walks.load(Ordering::Relaxed);
    let scopes_analyzed = COUNTERS.scopes_analyzed.load(Ordering::Relaxed);
    let symbols_allocated = COUNTERS.symbols_allocated.load(Ordering::Relaxed);
    let name_at_calls = COUNTERS.name_at_calls.load(Ordering::Relaxed);
    let name_at_micros = COUNTERS.name_at_micros.load(Ordering::Relaxed);
    let name_at_compact_hits = COUNTERS.name_at_compact_hits.load(Ordering::Relaxed);
    let name_at_fallback_walks = COUNTERS.name_at_fallback_walks.load(Ordering::Relaxed);
    let resolve_at_calls = COUNTERS.resolve_at_calls.load(Ordering::Relaxed);
    let resolve_at_micros = COUNTERS.resolve_at_micros.load(Ordering::Relaxed);
    let resolve_at_compact_hits = COUNTERS.resolve_at_compact_hits.load(Ordering::Relaxed);
    let resolve_at_fallback_walks = COUNTERS.resolve_at_fallback_walks.load(Ordering::Relaxed);
    let hover_at_calls = COUNTERS.hover_at_calls.load(Ordering::Relaxed);
    let hover_at_micros = COUNTERS.hover_at_micros.load(Ordering::Relaxed);
    let definition_at_calls = COUNTERS.definition_at_calls.load(Ordering::Relaxed);
    let definition_at_micros = COUNTERS.definition_at_micros.load(Ordering::Relaxed);
    let analyze_file_retained_bytes = COUNTERS.analyze_file_retained_bytes.load(Ordering::Relaxed);
    let analyze_file_retained_samples = COUNTERS
        .analyze_file_retained_samples
        .load(Ordering::Relaxed);
    let infer_scope_retained_bytes = COUNTERS.infer_scope_retained_bytes.load(Ordering::Relaxed);
    let infer_scope_retained_samples = COUNTERS
        .infer_scope_retained_samples
        .load(Ordering::Relaxed);
    let infer_function_retained_bytes = COUNTERS
        .infer_function_retained_bytes
        .load(Ordering::Relaxed);
    let infer_function_retained_samples = COUNTERS
        .infer_function_retained_samples
        .load(Ordering::Relaxed);

    let avg_pass2_us = body_analysis_micros
        .checked_div(body_analysis_runs)
        .unwrap_or(0);
    let avg_name_at_us = name_at_micros.checked_div(name_at_calls).unwrap_or(0);
    let avg_resolve_at_us = resolve_at_micros.checked_div(resolve_at_calls).unwrap_or(0);
    let avg_hover_at_us = hover_at_micros.checked_div(hover_at_calls).unwrap_or(0);
    let avg_definition_at_us = definition_at_micros
        .checked_div(definition_at_calls)
        .unwrap_or(0);
    let avg_analyze_file_retained = analyze_file_retained_bytes
        .checked_div(analyze_file_retained_samples)
        .unwrap_or(0);
    let avg_infer_scope_retained = infer_scope_retained_bytes
        .checked_div(infer_scope_retained_samples)
        .unwrap_or(0);
    let avg_infer_function_retained = infer_function_retained_bytes
        .checked_div(infer_function_retained_samples)
        .unwrap_or(0);

    // Compute upper-bound clone bytes per run: entry counts × per-entry bytes.
    // Name = 4B (interned u32), map entry overhead ~50B for FxHash open-addressing.
    // These are UPPER bounds — COW only helps write-free branches.
    let entry_bytes: u64 = 54; // Name(4) + value(16 max) + ~34B hashmap overhead per slot
    let read_vars_ub_mb = branch_read_vars * entry_bytes / 1_000_000;
    let var_locs_ub_mb = branch_var_locs * entry_bytes * 2 / 1_000_000; // value=(u32,u16,u32,u16)=12B
    let last_write_ub_mb = branch_last_write * entry_bytes * 2 / 1_000_000;

    let samples = render_samples();
    let body_walks = render_top_counts("body walks/file", &BODY_WALKS_BY_FILE);
    let scopes = render_top_counts("scopes analyzed/file", &SCOPES_BY_FILE);
    Some(format!(
        "mir metrics:\n  \
         file analyses        : {analyses}\n  \
         body analysis runs   : {body_analysis_runs}\n  \
         body analysis time   : {body_analysis_micros} us  (avg/run: {avg_pass2_us} us)\n  \
         whole-file walks     : {whole_file_body_walks}\n  \
         scopes analyzed      : {scopes_analyzed}\n  \
         symbols allocated    : {symbols_allocated}\n  \
         name_at              : {name_at_calls} calls  {name_at_micros} us total  (avg {avg_name_at_us} us)\n  \
         name_at path         : compact {name_at_compact_hits}  fallback {name_at_fallback_walks}\n  \
         resolve_at           : {resolve_at_calls} calls  {resolve_at_micros} us total  (avg {avg_resolve_at_us} us)\n  \
         resolve_at path      : compact {resolve_at_compact_hits}  fallback {resolve_at_fallback_walks}\n  \
         hover_at             : {hover_at_calls} calls  {hover_at_micros} us total  (avg {avg_hover_at_us} us)\n  \
         definition_at        : {definition_at_calls} calls  {definition_at_micros} us total  (avg {avg_definition_at_us} us)\n  \
         lazy load attempts   : {attempts}  resolved: {resolved}\n  \
         lazy load failures   : no_resolver={ll_no_resolver}  resolver_none={ll_resolver_none}  \
         source_unreadable={ll_source_unreadable}  ingest_then_missing={ll_ingest_missing}\n  \
         stub cache           : hits {cache_hits}  misses {cache_misses}\n  \
         fn short-name scans  : {fn_short_scans}\n  \
         retained/analyze_file: {analyze_file_retained_bytes} B  ({analyze_file_retained_samples} samples, avg {avg_analyze_file_retained} B)\n  \
         retained/infer_scope : {infer_scope_retained_bytes} B  ({infer_scope_retained_samples} samples, avg {avg_infer_scope_retained} B)\n  \
         retained/infer_fn    : {infer_function_retained_bytes} B  ({infer_function_retained_samples} samples, avg {avg_infer_function_retained} B)\n  \
         flow branch()        : {branch_calls} calls\n  \
           read_vars          : {branch_read_vars} total entries  (~{read_vars_ub_mb} MB upper bound)\n  \
           var_locations      : {branch_var_locs} total entries  (~{var_locs_ub_mb} MB upper bound)\n  \
           last_write_locs    : {branch_last_write} total entries  (~{last_write_ub_mb} MB upper bound){body_walks}{scopes}{samples}"
    ))
}

#[cfg(test)]
pub(crate) fn test_reset() {
    TEST_ENABLED.store(true, Ordering::Relaxed);
    COUNTERS.file_analyses.store(0, Ordering::Relaxed);
    COUNTERS.body_analysis_runs.store(0, Ordering::Relaxed);
    COUNTERS.lazy_loads_attempted.store(0, Ordering::Relaxed);
    COUNTERS.lazy_loads_resolved.store(0, Ordering::Relaxed);
    COUNTERS.body_analysis_micros.store(0, Ordering::Relaxed);
    COUNTERS.stub_cache_hits.store(0, Ordering::Relaxed);
    COUNTERS.stub_cache_misses.store(0, Ordering::Relaxed);
    COUNTERS.ll_fail_no_resolver.store(0, Ordering::Relaxed);
    COUNTERS.ll_fail_resolver_none.store(0, Ordering::Relaxed);
    COUNTERS
        .ll_fail_source_unreadable
        .store(0, Ordering::Relaxed);
    COUNTERS
        .ll_fail_ingest_then_missing
        .store(0, Ordering::Relaxed);
    COUNTERS.fn_short_name_scans.store(0, Ordering::Relaxed);
    COUNTERS.flow_branch_calls.store(0, Ordering::Relaxed);
    COUNTERS
        .flow_branch_read_vars_entries
        .store(0, Ordering::Relaxed);
    COUNTERS
        .flow_branch_var_locs_entries
        .store(0, Ordering::Relaxed);
    COUNTERS
        .flow_branch_last_write_entries
        .store(0, Ordering::Relaxed);
    COUNTERS.whole_file_body_walks.store(0, Ordering::Relaxed);
    COUNTERS.scopes_analyzed.store(0, Ordering::Relaxed);
    COUNTERS.symbols_allocated.store(0, Ordering::Relaxed);
    COUNTERS.name_at_calls.store(0, Ordering::Relaxed);
    COUNTERS.name_at_micros.store(0, Ordering::Relaxed);
    COUNTERS.name_at_compact_hits.store(0, Ordering::Relaxed);
    COUNTERS.name_at_fallback_walks.store(0, Ordering::Relaxed);
    COUNTERS.resolve_at_calls.store(0, Ordering::Relaxed);
    COUNTERS.resolve_at_micros.store(0, Ordering::Relaxed);
    COUNTERS.resolve_at_compact_hits.store(0, Ordering::Relaxed);
    COUNTERS
        .resolve_at_fallback_walks
        .store(0, Ordering::Relaxed);
    COUNTERS.hover_at_calls.store(0, Ordering::Relaxed);
    COUNTERS.hover_at_micros.store(0, Ordering::Relaxed);
    COUNTERS.definition_at_calls.store(0, Ordering::Relaxed);
    COUNTERS.definition_at_micros.store(0, Ordering::Relaxed);
    COUNTERS
        .analyze_file_retained_bytes
        .store(0, Ordering::Relaxed);
    COUNTERS
        .analyze_file_retained_samples
        .store(0, Ordering::Relaxed);
    COUNTERS
        .infer_scope_retained_bytes
        .store(0, Ordering::Relaxed);
    COUNTERS
        .infer_scope_retained_samples
        .store(0, Ordering::Relaxed);
    COUNTERS
        .infer_function_retained_bytes
        .store(0, Ordering::Relaxed);
    COUNTERS
        .infer_function_retained_samples
        .store(0, Ordering::Relaxed);
    BODY_WALKS_BY_FILE.lock().unwrap().clear();
    SCOPES_BY_FILE.lock().unwrap().clear();
    let mut samples = FAILURE_SAMPLES.lock().unwrap();
    samples.no_resolver.clear();
    samples.resolver_none.clear();
    samples.source_unreadable.clear();
    samples.ingest_then_missing.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dump_includes_new_navigation_and_retained_metrics() {
        test_reset();
        record_whole_file_body_walk("/tmp/a.php", 3);
        record_whole_file_body_walk("/tmp/a.php", 2);
        record_scope_analysis("/tmp/a.php", 4);
        record_scope_analysis("/tmp/b.php", 1);
        record_name_at(13);
        record_name_at_compact_hit();
        record_resolve_at(17);
        record_resolve_at_fallback_walk();
        record_hover_at(23);
        record_definition_at(11);
        record_analyze_file_retained(2, 3);
        record_infer_scope_retained(1, 2, 1, 2, 3);
        record_infer_function_retained(1, 1, true);

        let dump = dump().expect("metrics enabled in tests");
        assert!(dump.contains("whole-file walks     : 2"));
        assert!(dump.contains("scopes analyzed      : 2"));
        assert!(dump.contains("symbols allocated    : 10"));
        assert!(dump.contains("name_at              : 1 calls"));
        assert!(dump.contains("name_at path         : compact 1  fallback 0"));
        assert!(dump.contains("resolve_at           : 1 calls"));
        assert!(dump.contains("resolve_at path      : compact 0  fallback 1"));
        assert!(dump.contains("hover_at             : 1 calls"));
        assert!(dump.contains("definition_at        : 1 calls"));
        assert!(dump.contains("retained/analyze_file:"));
        assert!(dump.contains("retained/infer_scope :"));
        assert!(dump.contains("retained/infer_fn    :"));
        assert!(dump.contains("top body walks/file:"));
        assert!(dump.contains("/tmp/a.php"));
        assert!(dump.contains("top scopes analyzed/file:"));
        assert!(dump.contains("/tmp/b.php"));
    }
}
