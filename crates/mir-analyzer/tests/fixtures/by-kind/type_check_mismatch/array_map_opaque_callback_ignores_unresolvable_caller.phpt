===description===
A caller that forwards its own opaque callback (itself unresolvable) must not
poison the result to the generic `array` fallback — only resolvable callers
contribute to the union.
===config===
suppress=MixedAssignment
===file===
<?php
function apply(callable $cb, array $nums): array {
    $result = array_map($cb, $nums);
    /** @mir-check $result is array<array-key, string> */
    return $result;
}

function resolvableCaller(array $nums): void {
    apply(fn(int $x): string => (string) $x, $nums);
}

function unresolvableCaller(callable $mystery, array $nums): void {
    apply($mystery, $nums);
}
===expect===
