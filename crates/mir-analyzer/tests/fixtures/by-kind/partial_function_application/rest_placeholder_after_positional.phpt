===description===
A bare trailing `...` (distinct from `...$arr` unpack and from the `f(...)`
first-class-callable marker — this one comes after a real positional
argument) curries every remaining parameter at once. Parsed as
`Arg { value: None, unpack: true }`.
===config===
suppress=UnusedVariable
===file===
<?php

function add3(int $a, int $b, int $c): int {
    return $a + $b + $c;
}

$partial = add3(1, ...);
===expect===
ParseError@7:19-7:22: Parse error: 'partial function application' requires PHP 8.6 or higher (targeting PHP 8.5)
