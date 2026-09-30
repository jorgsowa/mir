===description===
Detect unused var before try inside foreach
===file===
<?php
function foo() : void {
    $unused = 1;
//  ^^^^^^^ UnusedVariable: Variable $unused is never read

    while (rand(0, 1)) {
        try {} catch (Exception $e) {}
    }
}
===expect===
