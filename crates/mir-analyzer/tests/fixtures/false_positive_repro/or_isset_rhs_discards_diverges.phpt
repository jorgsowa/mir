===description===
`!isset($x) || RHS` merges two paths ($x unset; $x set and RHS true) rather
than discarding RHS's `diverges` — a contradiction found while analyzing RHS
alone (the "$x set" path) must not make the whole condition look
unreachable, since the "$x unset" path is still live.
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

// The RHS's contradiction (int is never instanceof Marker) proves the
// "$x set" path is dead, so the only surviving path is "$x unset" — $x is
// definitely (not just possibly) null here.
function definitely_null(?string $x, int $y): void {
    if (!isset($x) || $y instanceof Marker) {
        echo strlen($x);
    }
}
===expect===
NullArgument@17:20-17:22: Argument $string of strlen() cannot be null
