===description===
Detect useless array assignment
===file===
<?php
function foo() : void {
    $a = [];
    $a[0] = 1;
}
===expect===
UnusedVariable@3:4-3:6: Variable $a is never read
