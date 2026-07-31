===description===
Regression guard: `f(...)` alone (nothing before the ellipsis) is the
pre-existing PHP 8.1 first-class-callable syntax, NOT the PHP 8.6 rest
placeholder — the parser disambiguates by checking for `...)` as the very
first token. This must keep working exactly as before 0.19 (no spurious
"requires PHP 8.6" version-gate diagnostic).
===config===
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$fn = add(...);
$result = $fn(1, 2);
===expect===
