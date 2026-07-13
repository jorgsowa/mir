===description===
A bounded int<min,max> range too large to enumerate is still treated as possibly-unmatched, not silently accepted.
===file===
<?php
/** @param int<0, 100000> $x */
function f(int $x): string {
    return match ($x) {
        0 => 'a',
        1 => 'b',
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: possibly-unmatched value of type 'int<0, 100000>'
