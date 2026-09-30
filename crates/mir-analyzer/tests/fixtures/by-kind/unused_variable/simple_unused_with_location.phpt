===description===
Verify UnusedVariable is reported at the correct line and column.
===config===
suppress=MissingReturnType
===file===
<?php
function example() {
    $unused = 42;
//  ^^^^^^^ UnusedVariable: Variable $unused is never read
    return 10;
}
===expect===
