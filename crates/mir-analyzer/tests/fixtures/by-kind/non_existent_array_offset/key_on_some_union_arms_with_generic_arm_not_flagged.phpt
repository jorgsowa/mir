===description===
A key missing from one closed shape of a mixed union (shapes plus a generic array) is not flagged when another arm may hold it.
===file===
<?php
/** @param array{a: int}|array{b: string}|array<string, int> $x */
function mixed_union(array $x): void {
    echo $x['a'];
}

/** @param array{a: int}|array{b: string}|array<string, int> $x */
function guarded(array $x): void {
    if (isset($x['a'])) {
        echo $x['a'];
    }
}
