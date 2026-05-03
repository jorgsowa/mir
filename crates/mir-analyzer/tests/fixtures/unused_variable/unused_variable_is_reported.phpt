===description===
A variable that is assigned but never read in its scope reports UnusedVariable.
===file===
<?php
function foo(): int {
    $unused = 1;
    return 42;
}
===expect===
UnusedVariable: Variable $unused is never read
===ignore===
TODO
