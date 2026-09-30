===description===
UnhandledMatchCondition reports all missing int literal cases, sorted.
===file===
<?php
/** @param 1|2|3|4 $n */
function label(int $n): string {
    return match($n) {
//         ^ +2:5 UnhandledMatchCondition: Unhandled match condition: 1, 3, 4
        2 => "two",
    };
}
===expect===
