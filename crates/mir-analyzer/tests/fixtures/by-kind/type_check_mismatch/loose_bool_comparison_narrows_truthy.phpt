===description===
`$x == true`/`$x == false` (and negated/swapped forms) narrow like a bare
truthy/falsy check, since PHP defines loose comparison to a bool literal as
`(bool)$x === value`.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param string $x */
function test_equal_true(string $x): void {
    if ($x == true) {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    } else {
        /** @mir-check $x is ''|'0' */
        $_ = $x;
    }
}

/** @param string $x */
function test_equal_false(string $x): void {
    if ($x == false) {
        /** @mir-check $x is ''|'0' */
        $_ = $x;
    } else {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    }
}

/** @param string $x */
function test_not_equal_true(string $x): void {
    if ($x != true) {
        /** @mir-check $x is ''|'0' */
        $_ = $x;
    }
}

/** @param string $x */
function test_not_equal_false(string $x): void {
    if ($x != false) {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    }
}

/** @param string $x */
function test_swapped_operands(string $x): void {
    if (true == $x) {
        /** @mir-check $x is non-empty-string */
        $_ = $x;
    }
}

function test_non_bool_equal_not_narrowed(int $x): void {
    if ($x == 1) {
        /** @mir-check $x is int */
        $_ = $x;
    }
}
===expect===
