===description===
FP: `if ($first) { $first = false; } else { ... }` inside a loop body must
not be flagged as a RedundantCondition. The loop's fixed-point widening
algorithm re-runs the body up to 3 times to converge on a stable variable
type; only its FIRST pass sees $first as the literal `true` it starts as —
every later pass (and every iteration after the first at runtime) sees
`bool`. Diagnostics from an unstabilized pass must not leak into the final
result just because that pass happened to see an overly-narrow type.
===config===
suppress=UnusedVariable
===file===
<?php
/** @param list<string> $items */
function join_with_commas(array $items): string {
    $out = "";
    $first = true;
    foreach ($items as $item) {
        if ($first) {
            $first = false;
        } else {
            $out .= ", ";
        }
        $out .= $item;
    }
    return $out;
}
===expect===
