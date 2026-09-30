===description===
UnhandledMatchCondition handles negative int literals in the union.
===file===
<?php
/** @param -1|0|1 $n */
function sign(int $n): string {
    return match($n) {
//         ^ +3:5 UnhandledMatchCondition: Unhandled match condition: 1
        -1 => "negative",
        0  => "zero",
    };
}
===expect===
