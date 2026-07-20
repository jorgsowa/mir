===description===
`$x === true`/`false` on `scalar`/`mixed` narrows to the specific bool
literal on a match (mirrors literal_comparison_collapses_wide_atoms.phpt's
string/int treatment) instead of dropping the atom (scalar, unsound —
could produce an empty type) or keeping it unnarrowed (mixed, imprecise).
The non-match branch must keep the wide type unchanged.
===config===
suppress=UnusedVariable,UnusedParam,MixedAssignment
===file===
<?php
/** @param scalar $x */
function scalarEqTrue($x): void {
    if ($x === true) {
        /** @mir-check $x is true */
        $_ = $x;
    } else {
        /** @mir-check $x is scalar */
        $_ = $x;
    }
}

/** @param scalar $x */
function scalarEqFalse($x): void {
    if ($x === false) {
        /** @mir-check $x is false */
        $_ = $x;
    } else {
        /** @mir-check $x is scalar */
        $_ = $x;
    }
}

function mixedEqTrue(mixed $x): void {
    if ($x === true) {
        /** @mir-check $x is true */
        $_ = $x;
    } else {
        /** @mir-check $x is mixed */
        $_ = $x;
    }
}

function mixedNeqFalse(mixed $x): void {
    if ($x !== false) {
        /** @mir-check $x is mixed */
        $_ = $x;
    } else {
        /** @mir-check $x is false */
        $_ = $x;
    }
}
===expect===
