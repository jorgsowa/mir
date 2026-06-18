===description===
is_scalar() narrowing handles string/int subtypes correctly — no false RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string|array $x */
function test_non_empty_string(mixed $x): void {
    if (is_scalar($x)) {
        // true branch: only non-empty-string (array is not scalar)
        $_ = $x;
    }
    // no RedundantCondition — array can bypass is_scalar
}

/** @param positive-int|array $x */
function test_positive_int(mixed $x): void {
    if (is_scalar($x)) {
        // true branch: only positive-int
        $_ = $x;
    }
    // no RedundantCondition — array can bypass is_scalar
}
===expect===
