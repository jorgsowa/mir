===description===
non-empty-list is always truthy; truthy-check on it is a RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param non-empty-list<int> $arr */
function test(array $arr): void {
    if ($arr) {
//      ^^^^ RedundantCondition: Condition of type 'non-empty-list<int>' always evaluates the same way, so one branch is unreachable
        $_ = $arr;
    }
}
===expect===
