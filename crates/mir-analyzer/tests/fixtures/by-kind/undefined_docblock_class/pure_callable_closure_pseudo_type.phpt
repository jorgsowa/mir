===description===
`pure-callable(...)` and `pure-Closure(...)` parse with the same structural
shape as `callable(...)`/`Closure(...)`, keeping their purity, instead of
being misparsed as a bogus named class.
===config===
<mir>
  <issueHandlers>
    <UnusedParam errorLevel="suppress"/>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
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
/** @mir-check $closure is pure-Closure(int): string */
$_ = 1;
