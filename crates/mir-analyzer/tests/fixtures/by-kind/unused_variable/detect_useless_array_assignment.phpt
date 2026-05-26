===description===
Detect useless array assignment
===file===
<?php
function foo() : void {
    $a = [];
    $a[0] = 1;
}
===expect===
UnusedVariable
===ignore===
TODO
