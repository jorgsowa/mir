===description===
UnhandledMatchCondition fires when a match on an int-mask type misses values.
int-mask<1, 2> expands to 0|1|2|3; arms for 0 and 1 leave 2 and 3 uncovered.
===config===
suppress=UnusedParam
===file===
<?php
/** @param int-mask<1, 2> $flags */
function describe(int $flags): string {
    return match($flags) {
        0 => "none",
        1 => "first only",
    };
}
===expect===
UnhandledMatchCondition@4:11-7:5: Unhandled match condition: 2, 3
