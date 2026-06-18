===description===
preg_split with default flags always returns at least one element; result is non-empty-list<string>|false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_default_flags(string $pattern, string $subject): void {
    $parts = preg_split($pattern, $subject);
    /** @mir-check $parts is non-empty-list<string>|false */
    $_ = $parts;
}

function test_explicit_zero_flags(string $pattern, string $subject): void {
    $parts = preg_split($pattern, $subject, -1, 0);
    /** @mir-check $parts is non-empty-list<string>|false */
    $_ = $parts;
}
===expect===
