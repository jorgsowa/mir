===description===
`$x > N` (and its `!($x <= N)` negation) proves `$x` isn't null,
independent of N — PHP's null/int ordering-comparison table converts
null to `false` and the literal to `bool(N)`, and `false > bool(N)` can
never be true. `>=`/`<` are N-dependent (see
`int_comparison_n_dependent_null_stripping.phpt` for the full truth
table); `$x >= 5` here correctly excludes null since N != 0.
===config===
suppress=UnusedVariable,UnusedParam,PossiblyNullPropertyAccess
===file===
<?php
function greaterThanTrueBranch(?int $x): void {
    if ($x > 5) {
        /** @mir-check $x is int<6, max> */
        $_ = 1;
    }
}

function lessOrEqualFalseBranch(?int $x): void {
    if ($x <= 5) {
        return;
    }
    /** @mir-check $x is int<6, max> */
    $_ = 1;
}

function greaterOrEqualNonzeroExcludesNull(?int $x): void {
    if ($x >= 5) {
        /** @mir-check $x is int<5, max> */
        $_ = 1;
    }
}

class Box {
    public ?int $n = null;
}

function propGreaterThan(Box $x): void {
    if ($x->n > 5) {
        /** @mir-check $x->n is int<6, max> */
        $_ = 1;
    }
}
===expect===
