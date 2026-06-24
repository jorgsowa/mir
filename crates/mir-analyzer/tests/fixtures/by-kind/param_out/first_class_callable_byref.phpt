===description===
First-class callable syntax `fn(...)` preserves by-ref params in the TClosure,
so calling through the stored callable correctly pre-marks and writes back.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @param-out int $n
 */
function nextId(mixed &$n): void {
    static $counter = 0;
    $n = ++$counter;
}

$gen = nextId(...);
$gen($id);
// $id is defined — no UndefinedVariable, no PossiblyUndefinedVariable.
echo $id;
===expect===
