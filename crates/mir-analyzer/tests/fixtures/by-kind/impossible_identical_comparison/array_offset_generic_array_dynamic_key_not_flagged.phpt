===description===
Null comparisons on generic array reads with dynamic keys are allowed.
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
