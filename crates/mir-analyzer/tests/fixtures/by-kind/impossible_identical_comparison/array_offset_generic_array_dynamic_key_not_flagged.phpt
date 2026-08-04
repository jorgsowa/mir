===description===
L3, dynamic-key variant: `$map[$k]` where `$k` isn't a compile-time literal
is at least as uncertain about key presence as the literal-key case — must
not be flagged either.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, string> $map */
function test(array $map, string $k): void {
    $v = $map[$k];
    if ($v === null) {}
}
===expect===
