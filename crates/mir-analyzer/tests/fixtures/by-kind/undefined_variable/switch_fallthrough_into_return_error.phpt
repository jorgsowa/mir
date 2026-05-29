===description===
A variable assigned only inside a case that falls through into a return
is not defined on the path where no case matches, so it reports UndefinedVariable
after the switch.
===file===
<?php
// case 1 has no break and chains into case 2 which returns. Both cases
// effectively diverge, so $y is only reachable when no case matches (no
// default) — i.e., $y is always undefined at the echo.
function foo(int $x): void {
    switch ($x) {
        case 1:
            $y = "a";
            // no break — falls through into case 2
        case 2:
            return;
    }
    echo $y;
}
===expect===
UndefinedVariable@13:10-13:12: Variable $y is not defined
