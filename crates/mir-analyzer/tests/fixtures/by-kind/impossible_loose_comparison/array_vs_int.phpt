===description===
Arrays can never be loosely equal to integers in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr, int $n): void {
    if ($arr == $n) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:18: '==' between 'array' and 'int' is always false — these types can never be loosely equal
