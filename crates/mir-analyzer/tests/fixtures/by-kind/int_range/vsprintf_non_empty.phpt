===description===
vsprintf() gets the same non-empty-string narrowing as sprintf() when the
format string guarantees it — the return-type check only ever consults the
format-string argument, which vsprintf shares with sprintf.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test_literal_prefix(int $n): void {
    $r = vsprintf("id=%d", [$n]);
    /** @mir-check $r is non-empty-string */
    $_ = $r;
}

function test_pure_string_format_is_not_narrowed(string $s): void {
    $r = vsprintf("%s", [$s]);
    /** @mir-check $r is string */
    $_ = $r;
}
===expect===
