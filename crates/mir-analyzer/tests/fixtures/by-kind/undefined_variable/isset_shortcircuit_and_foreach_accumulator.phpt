===description===
isset short-circuit with && — foreach loop with isset guard
isset($prev) && $v > $prev should not report UndefinedVariable on $prev in RHS
===config===
suppress=UnusedVariable
===file===
<?php
/** @param array<int> $items */
function test(array $items): void {
    $prev = null;
    foreach ($items as $v) {
        if (isset($prev) && $v > $prev) {
            // $prev is definitely set here — no UndefinedVariable on $prev
            echo $prev;
        }
        $prev = $v;
    }
}
===expect===
