//! `metrics::dump` rendering. Alone in its binary: counters are process-global,
//! so concurrent analysis tests would skew the exact totals asserted here.

use mir_analyzer::metrics::*;

#[test]
fn dump_includes_new_navigation_and_retained_metrics() {
    std::env::set_var("MIR_TIMING", "1");
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

    let dump = dump().expect("MIR_TIMING=1 enables metrics");
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
