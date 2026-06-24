===description===
A string-typed variable can never be === to a boolean.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(string $s): void {
    if ($s === false) {}
    if ($s === true) {}
}
===expect===
ImpossibleIdenticalComparison@3:8-3:20: '===' between 'string' and 'false' is always false — these types can never be identical
ImpossibleIdenticalComparison@4:8-4:19: '===' between 'string' and 'true' is always false — these types can never be identical
