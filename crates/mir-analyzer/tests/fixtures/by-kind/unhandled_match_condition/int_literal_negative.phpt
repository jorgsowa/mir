===description===
UnhandledMatchCondition handles negative int literals in the union.
===file===
<?php
/** @param -1|0|1 $n */
function sign(int $n): string {
    return match($n) {
        -1 => "negative",
        0  => "zero",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: 1
