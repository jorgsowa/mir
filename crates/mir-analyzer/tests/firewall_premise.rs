//! Feasibility check for the body-aware signature firewall.
//!
//! The firewall skips re-analyzing a file's *dependents* when only the bodies
//! of its **declared-return** callables changed. That is sound only if a
//! dependent's diagnostics never depend on such a body. This suite mutates the
//! body of a declared-return callable in `provider.php` and asserts the issues
//! reported in the *consumer* file are byte-identical across the mutation.
//!
//! These run against the current analyzer (no firewall) so they document the
//! premise the firewall relies on, and would fail loudly if a body-derived
//! cross-file channel ever appeared.

mod common;

use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};

use self::common::{create_temp_dir, write_file};

/// Analyze `provider` + `consumer` from a fresh session and return the issue
/// kinds reported *in the consumer file*, sorted for comparison.
fn consumer_issue_kinds(provider_src: &str, consumer_src: &str) -> Vec<String> {
    let dir = create_temp_dir("firewall_premise");
    let provider = write_file(&dir, "provider.php", provider_src);
    let consumer = write_file(&dir, "consumer.php", consumer_src);
    // No cache: we are probing the analyzer's intrinsic behavior, not caching.
    let session = AnalysisSession::new(PhpVersion::LATEST);
    let result = session.analyze_paths(&[provider, consumer.clone()], &BatchOptions::new());
    let consumer_str = consumer.to_string_lossy().to_string();
    let mut kinds: Vec<String> = result
        .issues
        .iter()
        .filter(|i| i.location.file.as_ref() == consumer_str.as_str())
        .map(|i| format!("{}@{}", i.kind.name(), i.location.line))
        .collect();
    kinds.sort();
    kinds
}

fn assert_consumer_stable(provider_a: &str, provider_b: &str, consumer: &str, case: &str) {
    let before = consumer_issue_kinds(provider_a, consumer);
    let after = consumer_issue_kinds(provider_b, consumer);
    assert_eq!(
        before, after,
        "[{case}] consumer diagnostics changed when only a declared-return body changed:\n  \
         before={before:?}\n  after={after:?}"
    );
}

#[test]
fn declared_return_subtype_swap_does_not_affect_consumer() {
    // get(): Dog is declared. Body returns Dog in v1, Cat in v2. A consumer
    // calling ->bark() must see the *declared* Dog either way (Cat has no bark).
    let v1 = "<?php\nclass Dog { public function bark(): void {} }\nclass Cat {}\n\
        class Provider { public function get(): Dog { return new Dog(); } }\n";
    let v2 = "<?php\nclass Dog { public function bark(): void {} }\nclass Cat {}\n\
        class Provider { public function get(): Dog { $x = 1; return new Cat(); } }\n";
    let consumer = "<?php\nfunction use_it(Provider $p): void {\n    $p->get()->bark();\n}\n";
    assert_consumer_stable(v1, v2, consumer, "subtype_swap");
}

#[test]
fn declared_return_body_throws_change_does_not_affect_consumer() {
    // risky(): void declares @throws RuntimeException. Changing what the body
    // actually throws must not change the consumer's @throws obligations, which
    // derive from the declared docblock.
    let v1 = "<?php\nclass Provider {\n    /** @throws RuntimeException */\n    \
        public function risky(): void { throw new RuntimeException('x'); }\n}\n";
    let v2 = "<?php\nclass Provider {\n    /** @throws RuntimeException */\n    \
        public function risky(): void { if (true) { throw new LogicException('y'); } }\n}\n";
    let consumer = "<?php\nclass Consumer {\n    /** @throws RuntimeException */\n    \
        public function call(Provider $p): void { $p->risky(); }\n}\n";
    assert_consumer_stable(v1, v2, consumer, "throws_change");
}

#[test]
fn declared_return_free_function_body_does_not_affect_consumer() {
    // A free function with a declared return type; the consumer chains a method
    // off its result. The declared return governs the consumer regardless of body.
    let v1 = "<?php\nclass Widget { public function render(): void {} }\n\
        function make(): Widget { return new Widget(); }\n";
    let v2 = "<?php\nclass Widget { public function render(): void {} }\n\
        function make(): Widget { error_log('changed'); return new Widget(); }\n";
    let consumer = "<?php\nfunction draw(): void {\n    make()->render();\n}\n";
    assert_consumer_stable(v1, v2, consumer, "free_function_body");
}

#[test]
fn untyped_return_body_does_affect_consumer_negative_control() {
    // Negative control: NO declared return type. The consumer relies on the
    // *inferred* return (body-derived), so changing the body MUST change the
    // consumer's diagnostics. This is exactly the callable the firewall must
    // NOT strip — and it proves this harness can detect a real body→dependent
    // leak, so the four positive cases above are meaningful.
    let v1 = "<?php\nclass Dog { public function bark(): void {} }\nclass Cat {}\n\
        class Provider { public function get() { return new Dog(); } }\n";
    let v2 = "<?php\nclass Dog { public function bark(): void {} }\nclass Cat {}\n\
        class Provider { public function get() { return new Cat(); } }\n";
    let consumer = "<?php\nfunction use_it(Provider $p): void {\n    $p->get()->bark();\n}\n";
    let before = consumer_issue_kinds(v1, consumer);
    let after = consumer_issue_kinds(v2, consumer);
    assert_ne!(
        before, after,
        "negative control failed: an inferred-return body change did not reach the \
         consumer, so the positive cases prove nothing about this harness's sensitivity"
    );
}

#[test]
fn declared_return_added_statements_do_not_affect_consumer() {
    // Pure line-count change inside a declared-return body (the common edit).
    let v1 = "<?php\nclass Provider { public function value(): int { return 1; } }\n";
    let v2 = "<?php\nclass Provider { public function value(): int {\n        \
        $a = 1;\n        $b = 2;\n        $c = $a + $b;\n        return $c;\n    } }\n";
    let consumer = "<?php\nfunction consume(Provider $p): int {\n    return $p->value();\n}\n";
    assert_consumer_stable(v1, v2, consumer, "added_statements");
}
