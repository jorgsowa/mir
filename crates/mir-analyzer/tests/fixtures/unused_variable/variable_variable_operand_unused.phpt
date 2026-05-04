===description===
variable used only in variable-variable operand

===file===
<?php
function test() {
    $foo = 'bar';
    $$foo = 42;
}
===expect===
UnusedVariable@4:4: Variable $bar is never read
