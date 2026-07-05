===description===
`yield from $anotherGenerator` is *meant* to contribute the delegated
generator's own key/value type params, propagating through generator
delegation chains — but when the delegate is an unannotated function in the
same file, resolving its inferred type from within the caller's own
inference pass hits `inferred_function_return_type_demand`'s per-file (not
per-function) reentrancy guard and degrades to `mixed`. This is a
pre-existing limitation of the inference engine (not introduced by generator
inference specifically — the same guard affects any same-file call chain
between two unannotated functions), pinned here rather than asserting the
ideal-but-currently-unreachable `Generator<string, int, mixed, void>`.
===config===
suppress=UnusedVariable,MissingReturnType
===file===
<?php
function inner() {
    yield 'k' => 1;
}

function outer() {
    yield from inner();
}

$g = outer();
/** @mir-check $g is Generator<mixed, mixed, mixed, void> */
$_ = 1;
===expect===
