===description===
array_map over multiple arrays re-indexes with integer keys
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param array<string, int> $a
 * @param array<string, int> $b
 */
function test(array $a, array $b): void {
    $r = array_map(fn(int $x, int $y): int => $x + $y, $a, $b);
    /** @mir-check $r is array<int, int> */
    $_ = $r;
}
===expect===
