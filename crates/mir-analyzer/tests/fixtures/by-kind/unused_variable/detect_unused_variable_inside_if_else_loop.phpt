===description===
Detect unused variable inside if else loop
===file===
<?php
function foo() : void {
    $a = 1;

    if (rand(0, 1)) {
    } else {
        while (rand(0, 1)) {
            $a = 2;
        }
    }
}
===expect===
UnusedVariable@3:5-3:7: Variable $a is never read
