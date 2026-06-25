===description===
UnhandledMatchCondition fires for a single int literal subject with no matching arm.
===config===
suppress=TypeDoesNotContainType
===file===
<?php
/** @param 42 $n */
function check(int $n): string {
    return match($n) {
        0 => "zero",
    };
}
===expect===
UnhandledMatchCondition@4:11-6:12: Unhandled match condition: 42
