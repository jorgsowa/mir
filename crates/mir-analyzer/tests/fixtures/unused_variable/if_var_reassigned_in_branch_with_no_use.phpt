===description===
ifVarReassignedInBranchWithNoUse
===file===
<?php
$a = true;

if (rand(0, 1)) {
    $a = false;
}
===expect===
UnusedVariable
===ignore===
TODO
