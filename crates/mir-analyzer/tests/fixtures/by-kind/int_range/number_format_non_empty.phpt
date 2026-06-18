===description===
number_format() always returns a non-empty string (even for 0).
===config===
suppress=UnusedVariable
===file===
<?php
function test_number_format_zero(): void {
    $r = number_format(0);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_number_format_decimals(): void {
    $r = number_format(1234.56, 2);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_number_format_float(): void {
    $r = number_format(0.0, 2, '.', ',');
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}
===expect===
