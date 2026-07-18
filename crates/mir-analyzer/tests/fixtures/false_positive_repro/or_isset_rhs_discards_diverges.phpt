===description===
`!isset($x) || RHS` must discard RHS's `diverges` along with its narrowed
vars — a contradiction found while analyzing RHS alone must not make the
whole condition look unreachable.
===config===
===file===
<?php
interface Marker {}

// Positive: the RHS instanceof can never hold for an int, but the $x-unset
// path is still reachable, so the condition as a whole must not be flagged.
function reachable(?string $x, int $y): void {
    if (!isset($x) || $y instanceof Marker) {
        echo "reached";
    }
}

// Negative control: $x must still be treated as nullable inside the body —
// the fix must not over-correct into skipping narrowing checks entirely.
function still_nullable(?string $x, int $y): void {
    if (!isset($x) || $y instanceof Marker) {
        echo strlen($x);
    }
}
===expect===
PossiblyNullArgument@16:20-16:22: Argument $string of strlen() might be null
