===description===
variable used only in variable-variable operand

===config===
suppress=MissingReturnType
===file===
<?php
function test() {
    $foo = 'bar';
    $$foo = 42;
}
===expect===
UnusedVariable@4:4-4:9: Variable $bar is never read
