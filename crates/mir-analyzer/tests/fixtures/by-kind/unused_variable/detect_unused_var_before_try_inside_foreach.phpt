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
UnusedVariable@3:4-3:11: Variable $unused is never read
UnusedVariable@6:21-6:38: Variable $e is never read
