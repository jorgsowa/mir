===description===
InvalidArrayAssignment fires for bool-typed variables.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(bool $a): void {
    $a[0] = 5;
}
===expect===
InvalidArrayAssignment@3:4-3:13: Cannot use [] assignment on non-array type 'bool'
