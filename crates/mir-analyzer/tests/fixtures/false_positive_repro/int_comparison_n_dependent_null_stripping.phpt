===description===
`$x >= N` / `$x < N` strip null on a nullable receiver only when N makes
it possible — PHP compares null to an int by converting null to
bool(false) and the literal to bool(N), so whether null survives depends
on whether N == 0, unlike the already-fixed N-independent `>`/`<=`
directions. Covers both var and property receivers.
===config===
suppress=UnusedVariable,UnusedParam,MissingConstructor
===file===
<?php
function greaterOrEqualZeroAdmitsNull(?int $x): void {
    if ($x >= 0) {
        /** @mir-check $x is int<0, max>|null */
        $_ = 1;
    }
}

function greaterOrEqualNonzeroExcludesNull(?int $x): void {
    if ($x >= 5) {
        /** @mir-check $x is int<5, max> */
        $_ = 1;
    }
}

function lessZeroExcludesNull(?int $x): void {
    if ($x < 0) {
        /** @mir-check $x is int<min, -1> */
        $_ = 1;
    }
}

function lessNonzeroAdmitsNull(?int $x): void {
    if ($x < 5) {
        /** @mir-check $x is int<min, 4>|null */
        $_ = 1;
    }
}

class Box {
    public ?int $n = null;
}

function propGreaterOrEqualNonzeroExcludesNull(Box $x): void {
    if ($x->n >= 5) {
        /** @mir-check $x->n is int<5, max> */
        $_ = 1;
    }
}

function propLessZeroExcludesNull(Box $x): void {
    if ($x->n < 0) {
        /** @mir-check $x->n is int<min, -1> */
        $_ = 1;
    }
}
===expect===
