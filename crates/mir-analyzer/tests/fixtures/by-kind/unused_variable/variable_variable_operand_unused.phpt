===description===
variable used only in variable-variable operand

===config===
suppress=MissingReturnType
===file===
<?php
function test() {
    $foo = 'bar';
    $$foo = 42;
//  ^^^^^ UnusedVariable: Variable $bar is never read
}
===expect===
