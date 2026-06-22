===description===
preg_split with default flags (0) always returns at least one element and the false
case only fires on an invalid regex, so the result is non-empty-list<string>.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php

function test_default_flags(string $pattern, string $subject): void {
    $parts = preg_split($pattern, $subject);
    /** @mir-check $parts is non-empty-list<string> */
    $_ = $parts;
}

function test_explicit_zero_flags(string $pattern, string $subject): void {
    $parts = preg_split($pattern, $subject, -1, 0);
    /** @mir-check $parts is non-empty-list<string> */
    $_ = $parts;
}
===expect===
