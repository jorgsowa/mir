===description===
str_repeat() with a non-empty-string input and positive count returns non-empty-string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-string $s */
function test_positive_literal(string $s): void {
    $r = str_repeat($s, 3);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param non-empty-string $s
 *  @param positive-int $n */
function test_positive_int(string $s, int $n): void {
    $r = str_repeat($s, $n);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_empty_string_is_not_narrowed(): void {
    $r = str_repeat("", 5);
    /** @mir-check $r is string */
    $_ = $r;
}
===expect===
