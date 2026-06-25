===description===
Arrays can never be loosely equal to strings in PHP.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(array $arr, string $s): void {
    if ($arr == $s) {}
}
===expect===
ImpossibleLooseComparison@3:8-3:18: '==' between 'array<mixed, mixed>' and 'string' is always false — these types can never be loosely equal
