===description===
is_numeric($n) on int is always true - fires RedundantCondition
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(int $n): void {
    if (is_numeric($n)) {
//      ^^^^^^^^^^^^^^ RedundantCondition: Condition of type 'bool' always evaluates the same way, so one branch is unreachable
        $_ = $n;
    }
}
===expect===
