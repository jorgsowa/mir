===description===
mixed is open — no ImpossibleIdenticalComparison should fire.
===config===
suppress=UnusedVariable,UnusedParam,MixedArgument
===file===
<?php
function test(mixed $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
    if ($x === null) {}
    if ($x === false) {}
}
===expect===
