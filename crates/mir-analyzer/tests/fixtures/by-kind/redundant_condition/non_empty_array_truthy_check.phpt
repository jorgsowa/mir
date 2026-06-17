===description===
non-empty-array is always truthy; truthy-check on it is a RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string, int> $arr */
function test(array $arr): void {
    if ($arr) {
        $_ = $arr;
    }
}
===expect===
RedundantCondition@4:8-4:12: Condition is always true/false for type 'non-empty-array<string, int>'
