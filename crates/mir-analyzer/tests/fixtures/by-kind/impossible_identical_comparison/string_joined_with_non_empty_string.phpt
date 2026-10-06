===description===
Joining `string` with `non-empty-string` at a branch merge collapses to
`string`, in either order, so the comparison message names a single type.
===config===
<mir>
  <issueHandlers>
    <UnusedVariable errorLevel="suppress"/>
  </issueHandlers>
</mir>
===file===
<?php
/** @param non-empty-string $strict */
function ternary(string $plain, string $strict, bool $c): void {
    $x = $c ? $plain : $strict;
    /** @mir-check $x is string */
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}

/** @param non-empty-string $strict */
function reversed(string $plain, string $strict, bool $c): void {
    $x = $c ? $strict : $plain;
    /** @mir-check $x is string */
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}

/** @param non-empty-string $strict */
function ifElse(string $plain, string $strict, bool $c): void {
    if ($c) { $x = $plain; } else { $x = $strict; }
    /** @mir-check $x is string */
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'string' and 'null' is always false — these types can never be identical
}

/** @param non-empty-string $a */
function onlyNonEmpty(string $a, bool $c): void {
    $x = $c ? $a : $a;
    /** @mir-check $x is non-empty-string */
    if ($x === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'non-empty-string' and 'null' is always false — these types can never be identical
}
===expect===
