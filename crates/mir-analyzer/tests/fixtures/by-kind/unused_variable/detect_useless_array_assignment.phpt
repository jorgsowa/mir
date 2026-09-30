===description===
Detect useless array assignment
===file===
<?php
function foo() : void {
    $a = [];
//  ^^ UnusedVariable: Variable $a is never read
    $a[0] = 1;
}
===expect===
