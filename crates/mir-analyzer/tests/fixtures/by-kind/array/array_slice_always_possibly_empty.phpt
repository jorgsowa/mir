===description===
array_slice always returns a possibly-empty result because the offset and length
cannot be evaluated statically. When the source is a list and preserve_keys is
false (the default), the result is still a list (re-indexed), just never non-empty.
When preserve_keys is true, the result is a plain array preserving key types.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<int> $ne
 */
function test(array $ne, int $offset, int $length): void {
    // non-empty-list<int> with default preserve_keys=false → list<int> (not non-empty)
    $sliced = array_slice($ne, $offset, $length);
    /** @mir-check $sliced is list<int> */
    $_ = $sliced;

    // preserve_keys=true → array<int, int> (keys preserved, not re-indexed)
    $sliced_keys = array_slice($ne, $offset, $length, true);
    /** @mir-check $sliced_keys is array<int, int> */
    $_ = $sliced_keys;
}
===expect===
