===description===
Empty array literal is always falsy - truthy check should fire RedundantCondition
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $a = [];
    if ($a) {
        $_ = $a;
    }
}
===expect===
RedundantCondition@4:8-4:10: Condition is always true/false for type 'array{}'
