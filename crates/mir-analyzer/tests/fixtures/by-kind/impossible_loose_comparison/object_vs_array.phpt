===description===
Objects can never be loosely equal to arrays in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(\stdClass $obj, array $arr): void {
    if ($obj == $arr) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:20: '==' between 'stdClass' and 'array' is always false — these types can never be loosely equal
