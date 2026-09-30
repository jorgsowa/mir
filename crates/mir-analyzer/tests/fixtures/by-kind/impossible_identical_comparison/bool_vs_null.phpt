===description===
A bool-typed variable can never be === to null.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(bool $b): void {
    if ($b === null) {}
//      ^^^^^^^^^^^ ImpossibleIdenticalComparison: '===' between 'bool' and 'null' is always false — these types can never be identical
}
===expect===
