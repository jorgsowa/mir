===description===
L20 (remaining form): a fully-qualified GLOBAL class name (`\Glob`, no
namespace) used as a receiver's type, referenced from inside a namespaced
file via first-class-callable syntax (`$x->method(...)`), broke method
lookup. `expr/mod.rs`'s `CallableCreateKind::Method` arm called
`db::resolve_name` (raw-source-text rules) directly on the already-canonical
fqcn `named_object_fqcn()` returns, instead of `db::resolve_receiver_fqcn` —
the same already-fixed anti-pattern from `call/method.rs`'s intersection
loop, by-ref premarking, etc., recurring at this one remaining call site.
Since the mis-resolved fqcn ("App\Glob") doesn't exist,
`emit_undefined_method_for_callable`'s own unresolvable-receiver suppression
(L25 policy) silently swallows the failure instead of raising
`UndefinedMethod` — so the bug surfaces as a silently-degraded closure type
(bare `Closure`, losing the method's real signature) rather than a false
diagnostic, hence the `@mir-check` assertion below rather than an empty
expected-issues list.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
namespace {
    class Glob {
        public function run(): int { return 1; }
    }
}

namespace App {
    /**
     * @param \Glob $x
     */
    function callsMethod($x): void {
        $fn = $x->run(...);
        /** @mir-check $fn is Closure(): int */
        $y = $fn;
    }
}
===expect===
