===description===
array_map's element type through an opaque `callable $cb` parameter is the
union of every caller's concrete callback return type, not just the first
one found.
===config===
suppress=MixedAssignment
===file===
<?php
function apply(callable $cb, array $nums): array {
    $result = array_map($cb, $nums);
    /** @mir-check $result is array<array-key, string|int> */
    return $result;
}

function callerA(array $nums): void {
    apply(fn(int $x): string => (string) $x, $nums);
}

function callerB(array $nums): void {
    apply(fn(int $x): int => $x, $nums);
}
===expect===
