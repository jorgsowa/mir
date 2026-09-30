===description===
A placeholder occupying the slot ahead of a variadic parameter's own
arguments — must not desync the variadic-binding logic or crash.
===config===
suppress=UnusedVariable
===file===
<?php

function sum(int ...$nums): int {
    return array_sum($nums);
}

$partial = sum(?, 2, 3);
//             ^ ParseError: Parse error: 'partial function application' requires PHP 8.6 or higher
===expect===
