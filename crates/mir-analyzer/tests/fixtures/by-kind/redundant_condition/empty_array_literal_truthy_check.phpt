===description===
Empty array literal is always falsy - truthy check should fire RedundantCondition
===config===
suppress=UnusedVariable
===file===
<?php
function test(): void {
    $a = [];
    if ($a) {
//      ^^ RedundantCondition: Condition is always true/false for type 'array{}'
        $_ = $a;
    }
}
===expect===
