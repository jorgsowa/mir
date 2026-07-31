===description===
Every argument can be a `?` placeholder at once (full currying spelled out
explicitly, rather than via the bare `...` rest marker) — each occupies its
own positionally-aligned slot.
===config===
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$curried = add(?, ?);
===expect===
ParseError@7:15-7:16: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
ParseError@7:18-7:19: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
