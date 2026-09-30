===description===
Detect unused variable inside if loop
===file===
<?php
function foo() : void {
    $a = 1;
//  ^^ UnusedVariable: Variable $a is never read

    if (rand(0, 1)) {
        while (rand(0, 1)) {
            $a = 2;
        }
    }
}
===expect===
