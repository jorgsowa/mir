===description===
Negative control for L3: a CLOSED shape's non-optional key is a real,
provably-present, provably-non-null value (`array{a: int}`, not a generic
`array<K,V>`) — the new `possibly_absent_offset` exemption must not leak
into this case, so the impossibility/redundancy checks still fire exactly
as before the fix.
===file===
<?php
/** @param array{a: int} $shape */
function test(array $shape): void {
    $v = $shape['a'];
    if ($v !== null) {}
}
===expect===
ImpossibleIdenticalComparison@5:8-5:19: '!==' between 'int' and 'null' is always true — these types can never be identical
RedundantCondition@5:8-5:19: Condition is always true/false for type 'bool'
