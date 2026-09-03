===description===
FP: a dynamically-invoked callable variable that happens to share a name
with a narrowing builtin (`$is_null(...)`) must not be treated as if that
builtin were actually called. The function-call dispatch used to match
`ExprKind::Variable` callees too, resolving `$is_null` to the string
"is_null" (the variable's own identifier, not its runtime value) and
narrowing `$x` to `null` even though the callable bound to `$is_null` has
nothing to do with the real `is_null()`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_dynamic_call_variable_named_like_builtin(?int $x): void {
    $is_null = fn(mixed $v): bool => true;
    if ($is_null($x)) {
        /** @mir-check $x is ?int */
        $_ = $x;
    }
}
===expect===
