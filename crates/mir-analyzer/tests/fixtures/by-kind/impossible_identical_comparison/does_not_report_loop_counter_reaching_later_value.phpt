===description===
FP: a `for` loop counter compared against a value it can reach on a later
iteration (`$i === 5` inside `for ($i = 0; $i < $n; $i++)`) must not be
flagged as an always-false ImpossibleIdenticalComparison. The loop
fixed-point widening algorithm's first pass sees $i still at its narrow
entry type (e.g. `0` or `0|1`, before widening kicks in); that provisional,
unstabilized-pass type must not leak a diagnostic into the final result.
===config===
suppress=UnusedVariable
===file===
<?php
function find_five(int $n): void {
    for ($i = 0; $i < $n; $i++) {
        if ($i === 5) {
            echo "found";
        }
    }
}
===expect===
