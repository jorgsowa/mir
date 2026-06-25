===description===
Objects can never be loosely equal to integers in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj, int $n): void {
    if ($obj == $n) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:18: '==' between 'stdClass' and 'int' is always false — these types can never be loosely equal
