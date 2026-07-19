===description===
`$x > N` (and its `!($x <= N)` negation) proves `$x` isn't null,
independent of N — PHP's null/int ordering-comparison table converts
null to `false` and the literal to `bool(N)`, and `false > bool(N)` can
never be true. `>=`/`<` stay untouched: whether null survives depends on
whether N is 0, so that case is deliberately not covered here.
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

function greaterOrEqualStillAdmitsNull(?int $x): void {
    if ($x >= 5) {
        /** @mir-check $x is int<5, max>|null */
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
