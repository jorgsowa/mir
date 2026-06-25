===description===
Objects can never be loosely equal to strings in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj, string $s): void {
    if ($obj == $s) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:18: '==' between 'stdClass' and 'string' is always false — these types can never be loosely equal
