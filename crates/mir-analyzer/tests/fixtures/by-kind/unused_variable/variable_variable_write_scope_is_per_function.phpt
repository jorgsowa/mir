===description===
A variable-variable write only exempts its own scope; another function's unused variable is still reported.
===file===
<?php
function dynamic(string $k): void {
    ${$k} = 1;
}

function plain(): void {
    $x = 1;
//  ^^ UnusedVariable: Variable $x is never read
}
