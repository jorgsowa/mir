===description===
InvalidArrayAssignment fires for float-typed variables.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(float $a): void {
    $a[0] = 5;
}
===expect===
InvalidArrayAssignment@3:4-3:13: Cannot use [] assignment on non-array type 'float'
