===description===
`pure-callable(...)` and `pure-Closure(...)` parse with the same structural
shape as `callable(...)`/`Closure(...)` (purity qualifier is dropped) instead
of being misparsed as a bogus named class.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/** @param pure-callable(int): string $cb */
function useCallable($cb): string {
    return $cb(1);
}

/** @return pure-Closure(int): string */
function makeClosure() {
    return fn (int $n): string => (string) $n;
}

$closure = makeClosure();
/** @mir-check $closure is Closure(int): string */

===expect===
