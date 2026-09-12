===description===
Null comparisons on generic array reads with literal keys are allowed.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<int, string> $map */
function notIdenticalDirection(array $map): void {
    $v = $map[5];
    if ($v !== null) {}
}

/** @param array<int, string> $map */
function identicalDirection(array $map): void {
    $v = $map[5];
    if ($v === null) {}
}
===expect===
