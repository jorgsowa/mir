===description===
UnhandledMatchCondition fires when a match on an int literal union misses a case.
===file===
<?php
/** @param 1|2|3 $n */
function label(int $n): string {
    return match($n) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: 3
        1 => "one",
        2 => "two",
    };
}
===expect===
