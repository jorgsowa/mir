===description===
The parser explicitly allows the bare `...` rest placeholder after a named
argument (`f(a: 1, ...)`), unlike an ordinary positional argument which is
forbidden in that position. Must not crash either way.
===config===
suppress=UnusedVariable
===file===
<?php

function add3(int $a, int $b, int $c): int {
    return $a + $b + $c;
}

$partial = add3(a: 1, ...);
===expect===
ParseError@7:22-7:25: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
