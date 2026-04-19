===source===
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
UndefinedVariable: Variable $y is not defined
