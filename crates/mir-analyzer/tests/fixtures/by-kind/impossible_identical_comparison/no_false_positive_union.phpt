===description===
A union that includes the literal's family does not fire.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string|int $x): void {
    if ($x === "foo") {}
    if ($x === 42) {}
}
===expect===
