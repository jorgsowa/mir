===description===
Arrays can never be loosely equal to objects in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr, \stdClass $obj): void {
    if ($arr == $obj) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:20: '==' between 'array<mixed, mixed>' and 'stdClass' is always false — these types can never be loosely equal
