===description===
A bare trailing `...` (distinct from `...$arr` unpack and from the `f(...)`
first-class-callable marker — this one comes after a real positional
argument) curries every remaining parameter at once. Parsed as
`Arg { value: None, unpack: true }`.
===config===
php_version=8.5
suppress=UnusedVariable
===file===
<?php

function add3(int $a, int $b, int $c): int {
    return $a + $b + $c;
}

$partial = add3(1, ...);
//                 ^^^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
===expect===
