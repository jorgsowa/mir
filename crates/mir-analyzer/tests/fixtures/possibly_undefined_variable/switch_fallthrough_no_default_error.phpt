===source===
<?php
function foo(int $x): string {
    switch ($x) {
        case 1:
            $y = "hello";
            break;
        case 2:
            $y = "world";
            // no break — falls through to end of switch
    }
    return $y;
}
===expect===
PossiblyUndefinedVariable: $y
