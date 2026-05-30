===description===
Detect unused variable inside if elseif loop
===file===
<?php
function foo() : void {
    $a = 1;

    if (rand(0, 1)) {
    } elseif (rand(0, 1)) {
        while (rand(0, 1)) {
            $a = 2;
        }
    }
}
===expect===
UnusedVariable@3:5-3:7: Variable $a is never read
