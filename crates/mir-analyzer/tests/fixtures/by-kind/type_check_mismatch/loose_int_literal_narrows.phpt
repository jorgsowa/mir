===description===
`$x == 42` / `!= 42` narrows an int-only-typed variable the same way
the strict `===`/`!==` sibling does (loose comparison agrees with
strict when every atom is already int-like). A mixed-category value
(e.g. int|string) is left unnarrowed — a string like "42" could loosely
equal the same value in a way strict comparison wouldn't.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function eqNarrowsToLiteral(int $x): void {
    if ($x == 42) {
        /** @mir-check $x is 42 */
        $_ = $x;
    }
}

function neqExcludesLiteral(int $x): void {
    if ($x != 42) {
        /** @mir-check $x is int */
        $_ = $x;
    } else {
        /** @mir-check $x is 42 */
        $_ = $x;
    }
}

/** @param int|string $x */
function mixedCategoryNotNarrowed($x): void {
    if ($x == 42) {
        /** @mir-check $x is int|string */
        $_ = $x;
    }
}
===expect===
