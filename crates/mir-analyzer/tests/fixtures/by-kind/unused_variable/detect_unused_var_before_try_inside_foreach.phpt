===description===
Detect unused var before try inside foreach
===file===
<?php
function foo() : void {
    $unused = 1;

    while (rand(0, 1)) {
        try {} catch (Exception $e) {}
    }
}
===expect===
UnusedVariable@3:5-3:12: Variable $unused is never read
