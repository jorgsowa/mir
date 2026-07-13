===description===
UnhandledMatchCondition still fires when a bounded int<min,max> range leaves a value uncovered.
===file===
<?php
/** @param int<0, 2> $x */
function f(int $x): string {
    return match ($x) {
        0 => 'a',
        1 => 'b',
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: 2
