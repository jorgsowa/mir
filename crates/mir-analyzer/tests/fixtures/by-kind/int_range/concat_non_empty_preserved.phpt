===description===
Concatenation with a non-empty operand (int, non-empty-string, positive-int)
yields non-empty-string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param positive-int $n */
function test_int_concat(int $n): void {
    $r = "prefix_" . $n;
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_nonempty_right(string $s): void {
    $r = "value_" . $s;
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s */
function test_nonempty_left(string $s): void {
    $r = $s . "_suffix";
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-negative-int $n */
function test_non_negative_int_concat(int $n): void {
    $r = "count=" . $n;
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_plain_string_is_not_narrowed(string $s): void {
    $r = $s . $s;
    /** @mir-check $r is string */
    $_ = $r;
}

/** @param positive-int $n */
function test_concat_assign(int $n): void {
    $r = "id=";
    $r .= $n;
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}
===expect===
