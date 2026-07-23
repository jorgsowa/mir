===description===
Spreading a single, closed, string-keyed shape into a new array literal
(`[...$a]`) preserves each key as its own literal instead of widening the
key domain to a generic `string`.
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $a = ['x' => 1, 'y' => 2];
    $merged = [...$a];
    /** @mir-check $merged is array{x: 1, y: 2} */
    $_ = $merged;
}
===expect===
