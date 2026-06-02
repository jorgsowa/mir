// Integration tests for cross-file UNANNOTATED inferred generic method returns (E3).
//
// `$b = new Box(5); $b->get();` where `Box` has `@template T`,
// a constructor that types `$value` as `T` (`@param T $value`), and a method
// `get()` with NO `@return` whose body is `return $this->value;` — with `Box`
// defined in a DIFFERENT file from the call site — must resolve to `int`, not
// `mixed`.
//
// Two layers cooperate to make this work:
//   1. `analyze_new` infers the class-level type params from the constructor
//      args, so `new Box(5)` is typed `Box<int>` (not bare `Box`).
//   2. The promoted-property collector types `$value` as the template param `T`
//      from the constructor's `@param T $value`, so the unannotated `get()` body
//      `return $this->value;` infers `T`, which the call site substitutes to int.

mod common;

use mir_analyzer::symbol::ReferenceKind;
use mir_analyzer::{AnalysisSession, BatchOptions, PhpVersion};
use mir_types::Atomic;

use self::common::{create_temp_dir, path_to_str, write_file};

/// Locate the resolved type of the named method call recorded in `caller_file`.
fn resolved_method_type(
    result: &mir_analyzer::AnalysisResult,
    caller_file: &str,
    method: &str,
) -> mir_types::Type {
    result
        .symbols
        .iter()
        .find(|s| {
            s.file.as_ref() == caller_file
                && matches!(&s.kind, ReferenceKind::MethodCall { method: m, .. } if m.as_ref() == method)
        })
        .map(|s| s.resolved_type.clone())
        .unwrap_or_else(|| panic!("MethodCall({method}) must be recorded in the caller file"))
}

/// Locate the resolved receiver type recorded for the `new ClassName(...)`
/// expression (the `ClassReference` symbol) in `caller_file`.
fn resolved_new_receiver_type(
    result: &mir_analyzer::AnalysisResult,
    caller_file: &str,
    class: &str,
) -> mir_types::Type {
    result
        .symbols
        .iter()
        .find(|s| {
            s.file.as_ref() == caller_file
                && matches!(&s.kind, ReferenceKind::ClassReference(n) if n.as_ref() == class)
        })
        .map(|s| s.resolved_type.clone())
        .unwrap_or_else(|| panic!("ClassReference({class}) must be recorded in the caller file"))
}

/// True when the receiver type is a bare `TNamedObject` (no generic type params)
/// for `class`.
fn is_bare_named_object(ty: &mir_types::Type, class: &str) -> bool {
    ty.types.iter().any(|t| {
        matches!(
            t,
            Atomic::TNamedObject { fqcn, type_params }
                if fqcn.as_ref() == class && type_params.is_empty()
        )
    })
}

/// Count diagnostics recorded against `caller_file` matching `pred`.
fn count_issues(
    result: &mir_analyzer::AnalysisResult,
    caller_file: &str,
    pred: impl Fn(&mir_issues::IssueKind) -> bool,
) -> usize {
    result
        .issues
        .iter()
        .filter(|i| i.location.file.as_ref() == caller_file && pred(&i.kind))
        .count()
}

/// True when any diagnostic of Error severity was recorded against `caller_file`.
fn has_error(result: &mir_analyzer::AnalysisResult, caller_file: &str) -> bool {
    result.issues.iter().any(|i| {
        i.location.file.as_ref() == caller_file && i.severity == mir_issues::Severity::Error
    })
}

/// Generic `Box` whose promoted `$value` is typed `T` via the constructor's
/// `@param T $value`, with an UNANNOTATED `get()` returning `$this->value`.
const BOX_SRC: &str = "<?php\n\
/**\n\
 * @template T\n\
 */\n\
class Box {\n\
    /** @param T $value */\n\
    public function __construct(public $value) {}\n\
    public function get() { return $this->value; }\n\
}\n";

#[test]
fn cross_file_unannotated_generic_return_resolves_int() {
    let dir = create_temp_dir("cross_file_unannotated_generic_return_int");
    let box_file = write_file(&dir, "box.php", BOX_SRC);

    // Call site lives in a different file.
    let app_src = "<?php\nfunction app(): void { $b = new Box(5); $b->get(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[box_file.clone(), app_file.clone()], &BatchOptions::new());

    // The `new Box(5)` receiver carries a WIDENED `int` type param (not the
    // literal `5`), so the unannotated `get()` resolves to `int`.
    let ty = resolved_method_type(&result, app_str, "get");
    assert!(
        ty.contains(|t| matches!(t, Atomic::TInt)),
        "expected $b->get() to resolve to widened int, got {ty}"
    );
    assert!(
        !ty.contains(|t| matches!(t, Atomic::TLiteralInt(_))),
        "expected the literal 5 to be widened to int, got {ty}"
    );
    assert!(
        !ty.is_mixed(),
        "expected a concrete int, not mixed, got {ty}"
    );
}

#[test]
fn cross_file_unannotated_generic_return_resolves_string() {
    let dir = create_temp_dir("cross_file_unannotated_generic_return_string");
    let box_file = write_file(&dir, "box.php", BOX_SRC);

    let app_src = "<?php\nfunction app(): void { $b = new Box(\"hello\"); $b->get(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[box_file.clone(), app_file.clone()], &BatchOptions::new());

    // The `new Box("hello")` receiver carries a WIDENED `string` type param (not
    // the literal `"hello"`), so the unannotated `get()` resolves to `string`.
    let ty = resolved_method_type(&result, app_str, "get");
    assert!(
        ty.contains(|t| matches!(t, Atomic::TString)),
        "expected $b->get() to resolve to widened string, got {ty}"
    );
    assert!(
        !ty.contains(|t| matches!(t, Atomic::TLiteralString(_))),
        "expected the literal \"hello\" to be widened to string, got {ty}"
    );
    assert!(
        !ty.is_mixed(),
        "expected a concrete string, not mixed, got {ty}"
    );
}

#[test]
fn cross_file_unannotated_generic_return_explicit_var_property() {
    // The same outcome via an explicit `@var T` property + a non-promoted
    // constructor that assigns it — exercises `resolve_property_type` reading the
    // declared template type directly.
    let dir = create_temp_dir("cross_file_unannotated_generic_return_explicit_var");
    let box_src = "<?php\n\
/**\n\
 * @template T\n\
 */\n\
class Holder {\n\
    /** @var T */\n\
    public $value;\n\
    /** @param T $v */\n\
    public function __construct($v) { $this->value = $v; }\n\
    public function get() { return $this->value; }\n\
}\n";
    let box_file = write_file(&dir, "holder.php", box_src);

    let app_src = "<?php\nfunction app(): void { $h = new Holder(42); $h->get(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[box_file.clone(), app_file.clone()], &BatchOptions::new());

    let ty = resolved_method_type(&result, app_str, "get");
    assert!(
        ty.contains(|t| matches!(t, Atomic::TInt | Atomic::TLiteralInt(_))),
        "expected $h->get() to resolve to int, got {ty}"
    );
}

#[test]
fn cross_file_unannotated_non_generic_return_still_works() {
    // Regression guard: an unannotated return on a NON-generic class must keep
    // inferring its concrete type from the body (here: a literal int), and the
    // `analyze_new` type-param inference must NOT spuriously parameterise it.
    let dir = create_temp_dir("cross_file_unannotated_non_generic_return");

    let lib_src = "<?php\n\
class Counter {\n\
    public function answer() { return 42; }\n\
}\n";
    let lib_file = write_file(&dir, "counter.php", lib_src);

    let app_src = "<?php\nfunction app(): void { $c = new Counter(); $c->answer(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    let ty = resolved_method_type(&result, app_str, "answer");
    assert!(
        ty.contains(|t| matches!(t, Atomic::TInt | Atomic::TLiteralInt(_))),
        "expected answer() to resolve to int, got {ty}"
    );
}

#[test]
fn cross_file_self_referential_unannotated_return_falls_back_without_hanging() {
    // A method whose unannotated body returns the result of calling itself must
    // fall back to a safe type via the INFER_IN_PROGRESS cycle guard, and must
    // NOT hang. The key assertion is that analysis terminates and records the
    // call; the resolved type is allowed to be a safe fallback.
    let dir = create_temp_dir("cross_file_self_referential_unannotated_return");

    let rec_src = "<?php\n\
/**\n\
 * @template T\n\
 */\n\
class Rec {\n\
    /** @param T $value */\n\
    public function __construct(public $value) {}\n\
    public function loop() { return $this->loop(); }\n\
}\n";
    let rec_file = write_file(&dir, "rec.php", rec_src);

    let app_src = "<?php\nfunction app(): void { $r = new Rec(1); $r->loop(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[rec_file.clone(), app_file.clone()], &BatchOptions::new());

    // Must terminate and record the call (type may be a safe fallback).
    let _ty = resolved_method_type(&result, app_str, "loop");
}

// ---------------------------------------------------------------------------
// Regression tests for the bound-fallback false positive (review finding E3).
//
// `infer_new_type_params` must parameterise the `new` receiver ONLY from
// template bindings derived from constructor ARGUMENTS. A bounded template the
// constructor never binds must stay `mixed` (bare type), so a later `T`-typed
// method call does NOT falsely substitute the param to the bound and reject
// otherwise-valid arguments.
// ---------------------------------------------------------------------------

#[test]
fn bounded_unbound_template_does_not_fabricate_receiver_param() {
    // The exact repro from the review: `@template T of Base` whose ctor does NOT
    // bind T (`__construct(int $id)`), with a `@param T $item` method. Before the
    // fix, `new Repo(5)` was fabricated as `Repo<Base>` (T fell back to its bound
    // `Base`), and `$r->add(new Other())` was rejected with a false
    // `InvalidArgument`. The receiver must instead be bare `Repo`, and the call
    // must emit NO diagnostic.
    let dir = create_temp_dir("bounded_unbound_template_no_false_positive");

    let lib_src = "<?php\n\
class Base {}\n\
class Other {}\n\
/**\n\
 * @template T of Base\n\
 */\n\
class Repo {\n\
    public function __construct(int $id) {}\n\
    /** @param T $item */\n\
    public function add($item): void {}\n\
}\n";
    let lib_file = write_file(&dir, "repo.php", lib_src);

    let app_src = "<?php\nfunction app(): void { $r = new Repo(5); $r->add(new Other()); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    // The receiver of `new Repo(5)` must be bare `Repo`, NOT `Repo<Base>`.
    let recv = resolved_new_receiver_type(&result, app_str, "Repo");
    assert!(
        is_bare_named_object(&recv, "Repo"),
        "expected `new Repo(5)` to be bare `Repo`, got {recv}"
    );

    // No false InvalidArgument (or any error) on the later `T`-typed method call.
    assert_eq!(
        count_issues(&result, app_str, |k| matches!(
            k,
            mir_issues::IssueKind::InvalidArgument { .. }
                | mir_issues::IssueKind::PossiblyInvalidArgument { .. }
        )),
        0,
        "expected NO InvalidArgument on $r->add(new Other()); issues: {:?}",
        result.issues
    );
    assert!(
        !has_error(&result, app_str),
        "expected NO error-severity diagnostic; issues: {:?}",
        result.issues
    );
}

#[test]
fn partial_inference_leaves_unbound_template_mixed() {
    // Two templates where the ctor binds only one (`K` from a `T`-typed first
    // arg) and never references the bounded second (`V of Base`). The unbound `V`
    // must stay `mixed` — not fabricated to `Base` — so a `V`-typed method still
    // accepts an unrelated argument with no false positive.
    let dir = create_temp_dir("partial_inference_unbound_template_mixed");

    let lib_src = "<?php\n\
class Base {}\n\
class Other {}\n\
/**\n\
 * @template K\n\
 * @template V of Base\n\
 */\n\
class Map {\n\
    /** @param K $key */\n\
    public function __construct($key) {}\n\
    /** @param V $value */\n\
    public function put($value): void {}\n\
}\n";
    let lib_file = write_file(&dir, "map.php", lib_src);

    let app_src = "<?php\nfunction app(): void { $m = new Map(\"k\"); $m->put(new Other()); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    // K bound from the literal arg (widened to string); V must NOT be `Base`.
    let recv = resolved_new_receiver_type(&result, app_str, "Map");
    let v_is_base = recv.types.iter().any(|t| {
        matches!(t, Atomic::TNamedObject { fqcn, type_params }
            if fqcn.as_ref() == "Map"
                && type_params.get(1).is_some_and(|p| {
                    p.types
                        .iter()
                        .any(|a| matches!(a, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Base"))
                }))
    });
    assert!(
        !v_is_base,
        "expected unbound V to be mixed, not fabricated to Base, got {recv}"
    );

    assert_eq!(
        count_issues(&result, app_str, |k| matches!(
            k,
            mir_issues::IssueKind::InvalidArgument { .. }
                | mir_issues::IssueKind::PossiblyInvalidArgument { .. }
        )),
        0,
        "expected NO InvalidArgument on $m->put(new Other()); issues: {:?}",
        result.issues
    );
}

#[test]
fn generic_class_without_constructor_is_bare_and_does_not_panic() {
    // A generic class with templates but NO constructor must yield a bare type
    // (early-return path) and must not panic.
    let dir = create_temp_dir("generic_class_no_constructor_bare");

    let lib_src = "<?php\n\
/**\n\
 * @template T of \\Stringable\n\
 */\n\
class Bag {\n\
    /** @param T $item */\n\
    public function add($item): void {}\n\
}\n";
    let lib_file = write_file(&dir, "bag.php", lib_src);

    let app_src = "<?php\nfunction app(): void { $b = new Bag(); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    let recv = resolved_new_receiver_type(&result, app_str, "Bag");
    assert!(
        is_bare_named_object(&recv, "Bag"),
        "expected `new Bag()` to be bare `Bag`, got {recv}"
    );
    assert!(
        !has_error(&result, app_str),
        "expected NO error-severity diagnostic; issues: {:?}",
        result.issues
    );
}

#[test]
fn variadic_union_nullable_ctor_args_do_not_panic_or_false_positive() {
    // A constructor mixing a variadic `T`-typed parameter, plus a bounded second
    // template `U of Base` that is taken via a nullable/union-typed parameter
    // that does NOT reference U. U must stay mixed; binding `T` from the variadic
    // args must not panic, and a later `U`-typed method must accept an unrelated
    // argument with no false positive.
    let dir = create_temp_dir("variadic_union_nullable_ctor_args");

    let lib_src = "<?php\n\
class Base {}\n\
class Other {}\n\
/**\n\
 * @template T\n\
 * @template U of Base\n\
 */\n\
class Coll {\n\
    /**\n\
     * @param T ...$items\n\
     */\n\
    public function __construct(?string $label, int|string $tag, ...$items) {}\n\
    /** @param U $value */\n\
    public function setValue($value): void {}\n\
}\n";
    let lib_file = write_file(&dir, "coll.php", lib_src);

    let app_src = "<?php\nfunction app(): void { \
$c = new Coll(null, 7, 1, 2, 3); $c->setValue(new Other()); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    // The receiver must exist and the bounded, ctor-unbound `U` must not be Base.
    let recv = resolved_new_receiver_type(&result, app_str, "Coll");
    let u_is_base = recv.types.iter().any(|t| {
        matches!(t, Atomic::TNamedObject { fqcn, type_params }
            if fqcn.as_ref() == "Coll"
                && type_params.get(1).is_some_and(|p| {
                    p.types
                        .iter()
                        .any(|a| matches!(a, Atomic::TNamedObject { fqcn, .. } if fqcn.as_ref() == "Base"))
                }))
    });
    assert!(
        !u_is_base,
        "expected unbound U to be mixed, not fabricated to Base, got {recv}"
    );

    assert_eq!(
        count_issues(&result, app_str, |k| matches!(
            k,
            mir_issues::IssueKind::InvalidArgument { .. }
                | mir_issues::IssueKind::PossiblyInvalidArgument { .. }
        )),
        0,
        "expected NO InvalidArgument; issues: {:?}",
        result.issues
    );
}

#[test]
fn literal_int_type_param_is_widened_so_setter_accepts_other_int() {
    // `new Box(5)` must carry `Box<int>` (widened), NOT `Box<5>`, so a later
    // `$b->set(6)` for `set(T $v)` is accepted (T = int, not the literal 5).
    let dir = create_temp_dir("literal_int_type_param_widened");

    let lib_src = "<?php\n\
/**\n\
 * @template T\n\
 */\n\
class Box {\n\
    /** @param T $value */\n\
    public function __construct(public $value) {}\n\
    /** @param T $value */\n\
    public function set($value): void { $this->value = $value; }\n\
}\n";
    let lib_file = write_file(&dir, "box.php", lib_src);

    let app_src = "<?php\nfunction app(): void { $b = new Box(5); $b->set(6); }\n";
    let app_file = write_file(&dir, "app.php", app_src);
    let app_str = path_to_str(&app_file);

    let analyzer = AnalysisSession::new(PhpVersion::LATEST);
    let result =
        analyzer.analyze_paths(&[lib_file.clone(), app_file.clone()], &BatchOptions::new());

    // Receiver type param must be widened int, not the literal 5.
    let recv = resolved_new_receiver_type(&result, app_str, "Box");
    let param_is_widened_int = recv.types.iter().any(|t| {
        matches!(t, Atomic::TNamedObject { fqcn, type_params }
        if fqcn.as_ref() == "Box"
            && type_params.first().is_some_and(|p| {
                p.contains(|a| matches!(a, Atomic::TInt))
                    && !p.contains(|a| matches!(a, Atomic::TLiteralInt(_)))
            }))
    });
    assert!(
        param_is_widened_int,
        "expected `new Box(5)` receiver to be `Box<int>` (widened), got {recv}"
    );

    assert_eq!(
        count_issues(&result, app_str, |k| matches!(
            k,
            mir_issues::IssueKind::InvalidArgument { .. }
                | mir_issues::IssueKind::PossiblyInvalidArgument { .. }
        )),
        0,
        "expected NO InvalidArgument on $b->set(6) after widening; issues: {:?}",
        result.issues
    );
}
