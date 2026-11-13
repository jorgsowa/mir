===description===
`array{}` joined with a non-empty list or array may be empty, so it is the plain list or
array and a truthiness check on it isn't redundant.
===file===
<?php
/** @param non-empty-list<int> $one */
function f(array $one, bool $c): ?int {
    $keys = [];
    if ($c) {
        $keys = $one;
    }
    /** @mir-check $keys is list<int> */
    if ($keys) {
        return 1;
    }
    return null;
}

/** @param array{}|non-empty-array<string, int> $b */
function g(array $b): int {
    /** @mir-check $b is array<string, int> */
    return count($b);
}
