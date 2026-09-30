===description===
A non-empty array is always truthy, so it can never be loosely equal to null
either — null converts to an empty array for the comparison.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string> $arr */
function test(array $arr): void {
    if ($arr == null) {}
//      ^^^^^^^^^^^^ ImpossibleLooseComparison: '==' between 'non-empty-array<int|string, string>' and 'null' is always false — these types can never be loosely equal
//      ^^^^^^^^^^^^ RedundantCondition: Condition is always true/false for type 'bool'
}
===expect===
