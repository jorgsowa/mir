===description===
UnhandledMatchCondition does NOT fire when a bounded int<min,max> range is fully enumerated by arms.
===file===
<?php
/** @param int<0, 2> $x */
function f(int $x): string {
    return match ($x) {
        0 => 'a',
        1 => 'b',
        2 => 'c',
    };
}
===expect===
