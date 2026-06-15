===description===
A variable that is assigned but never read in its scope reports UnusedVariable.
===file===
<?php
function foo(): int {
    $unused = 1;
    return 42;
}
===expect===
UnusedVariable@3:4-3:11: Variable $unused is never read
