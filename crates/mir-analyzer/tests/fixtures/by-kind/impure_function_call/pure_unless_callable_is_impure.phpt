===description===
A function tagged `@pure-unless-callable-is-impure $f` is pure when the
callable passed as `$f` is, and impure otherwise.
===file===
<?php
/**
 * @param callable(int): int $f
 * @param list<int> $values
 * @return list<int>
 * @pure-unless-callable-is-impure $f
 */
function mapInts(callable $f, array $values): array {
    $result = [];
    foreach ($values as $value) {
        $result[] = $f($value);
    }
    return $result;
}

function logged(int $n): int {
    echo $n;
    return $n;
}

/** @pure */
function increment(int $n): int {
    return $n + 1;
}

/**
 * @pure
 * @param callable(int): int $unknown
 * @param pure-callable(int): int $known
 */
function caller(callable $unknown, callable $known): int {
    $a = mapInts(static fn (int $n): int => $n + 1, [1]);
    $b = mapInts(increment(...), [1]);
    $c = mapInts(values: [1], f: $known);
    mapInts(logged(...), [1]);
//  ^^^^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mapInts() in a @pure function
    mapInts($unknown, [1]);
//  ^^^^^^^^^^^^^^^^^^^^^^ ImpureFunctionCall: Calling impure function mapInts() in a @pure function
    return $a[0] + $b[0] + $c[0];
}
