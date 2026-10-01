===description===
Targeting PHP 8.6 lets the parser accept partial-application placeholders, so
no version-gate ParseError is raised and the placeholder argument is not
type-checked.
===config===
php_version=8.6
suppress=UnusedVariable
===file===
<?php

function add(int $a, int $b): int {
    return $a + $b;
}

$partial = add(?, 5);
===expect===
