===description===
FN: `@var` annotations were never checked against the variable's already-
known type before overwriting it — asserting a disjoint, impossible type
(here `string` on a variable known to be the literal int `1`) was silently
accepted instead of flagged as a contradiction.
===config===
suppress=UnusedVariable
===file===
<?php
function f(): void {
    $x = 1;
    /** @var string $x */
    echo strlen($x);
//  ^^^^^^^^^^^^^^^^ DocblockTypeContradiction: Type '1' makes '@var string $x' impossible — this can never hold
}
===expect===
