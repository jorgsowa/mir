===description===
implode() with a non-empty array of non-empty strings returns non-empty-string.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-list<non-empty-string> $parts */
function test_non_empty_parts(array $parts): void {
    $r = implode(',', $parts);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

/** @param list<string> $parts */
function test_possibly_empty_parts(array $parts): void {
    $r = implode(',', $parts);
    /** @mir-check $r is string */
    $_ = $r;
}
===expect===
