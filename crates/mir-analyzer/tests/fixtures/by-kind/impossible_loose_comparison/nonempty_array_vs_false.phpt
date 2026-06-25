===description===
A non-empty array is always truthy, so it can never be loosely equal to false.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string> $arr */
function test(array $arr): void {
    if ($arr == false) {}
}
===expect===
ImpossibleLooseComparison@4:8-4:21: '==' between 'non-empty-array<int|string, string>' and 'false' is always false — these types can never be loosely equal
