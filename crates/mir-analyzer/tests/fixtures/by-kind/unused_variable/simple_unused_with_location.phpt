===description===
Verify UnusedVariable is reported at the correct line and column.
===config===
suppress=MissingReturnType
===file===
<?php
function example() {
    $unused = 42;
    return 10;
}
===expect===
UnusedVariable@3:5-3:12: Variable $unused is never read
