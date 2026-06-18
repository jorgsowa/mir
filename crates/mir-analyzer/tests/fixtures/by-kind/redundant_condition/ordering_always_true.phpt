===description===
Ordering comparison that is always true for a bounded int range fires RedundantCondition.
`$a < 10` where `$a: int<min, 5>` is always true — the false branch is unreachable.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param int<min, 5> $a */
function test(int $a): void {
    if ($a < 10) {
        // always taken
    }
}
===expect===
RedundantCondition@4:8-4:15: Condition is always true/false for type 'bool'
