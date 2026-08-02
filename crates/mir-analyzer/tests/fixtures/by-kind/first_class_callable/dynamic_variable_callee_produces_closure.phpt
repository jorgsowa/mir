===description===
`$c(...)` on a variable holding a `callable` (not a bare function name)
always produces a `Closure`, same as the static-name form — the callee
isn't statically resolvable to a known signature, but the expression's
result type must still be `Closure`, not the callee's own `callable` type.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function wrap(callable $c): void {
    $x = $c(...);
    /** @mir-check $x is Closure */
    $_ = $x;
}
===expect===
