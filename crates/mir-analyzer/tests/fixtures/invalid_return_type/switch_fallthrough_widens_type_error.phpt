===source===
<?php
// Before the fallthrough-context fix, the case-2 assignment was silently dropped,
// so $y appeared to be `int` post-switch and no error was reported.
function foo(int $x): int {
    $y = 0;
    switch ($x) {
        case 2:
            $y = "not an int";
            // no break — falls through to end of switch
    }
    return $y;
}
===expect===
InvalidReturnType: Return type '"not an int"|0' is not compatible with declared 'int'
