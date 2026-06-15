===description===
Detect unused variable inside if loop with echo inside
===file===
<?php
function foo() : void {
    $a = 1;

    if (rand(0, 1)) {
        while (rand(0, 1)) {
            $a = 2;
            echo $a;
        }
    }
}
===expect===
UnusedVariable@3:4-3:6: Variable $a is never read
