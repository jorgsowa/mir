===description===
is_numeric($n) on int is always true - fires RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $n): void {
    if (is_numeric($n)) {
        $_ = $n;
    }
}
===expect===
RedundantCondition@3:8-3:22: Condition is always true/false for type 'bool'
