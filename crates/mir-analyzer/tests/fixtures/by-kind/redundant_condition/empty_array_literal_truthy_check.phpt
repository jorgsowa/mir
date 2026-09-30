===description===
Empty array literal is always falsy - truthy check should fire RedundantCondition
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $a = [];
    if ($a) {
//      ^^ RedundantCondition: Condition of type 'array{}' always evaluates the same way, so one branch is unreachable
        $_ = $a;
    }
}
===expect===
