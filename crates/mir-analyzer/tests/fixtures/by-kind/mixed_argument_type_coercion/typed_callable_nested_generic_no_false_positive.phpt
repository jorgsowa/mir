===description===
A template parameter nested inside a container (`array<TKey, TValue>`) used as
a typed-callable's own parameter type must not be flagged. The unresolvable-
named-type guard originally only inspected top-level atomics, missing
templates nested inside array/list/keyed-array/intersection/generic type
arguments — this checks that recursion reaches them.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @template TKey
 * @template TValue
 * @param array<TKey, TValue> $arr
 * @param callable(array<TKey,TValue>): bool $callback
 */
function useMap(array $arr, callable $callback): bool {
    return $callback($arr);
}

useMap(['a' => 1], function (int $x): bool { return $x > 0; });
===expect===
