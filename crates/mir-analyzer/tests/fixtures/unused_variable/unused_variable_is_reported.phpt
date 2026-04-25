===file===
<?php
function foo(): int {
    $unused = 1;
    return 42;
}
===expect===
UnusedVariable: Variable $unused is never read
