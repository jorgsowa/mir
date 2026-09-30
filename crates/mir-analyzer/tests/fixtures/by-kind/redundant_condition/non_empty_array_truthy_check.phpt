===description===
non-empty-array is always truthy; truthy-check on it is a RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-array<string, int> $arr */
function test(array $arr): void {
    if ($arr) {
//      ^^^^ RedundantCondition: Condition of type 'non-empty-array<string, int>' always evaluates the same way, so one branch is unreachable
        $_ = $arr;
    }
}
===expect===
