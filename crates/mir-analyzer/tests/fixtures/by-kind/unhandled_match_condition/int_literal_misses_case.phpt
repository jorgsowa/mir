===description===
UnhandledMatchCondition fires when a match on an int literal union misses a case.
===file===
<?php
/** @param 1|2|3 $n */
function label(int $n): string {
    return match($n) {
        1 => "one",
        2 => "two",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: 3
