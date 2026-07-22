===description===
array_pad on a pure-list source (sequential int keys) always renumbers to a
fresh list regardless of pad direction, merging the fill value's type into
the element union. A literal non-zero $length guarantees the result is
non-empty even when the source itself isn't provably non-empty.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/**
 * @param non-empty-list<int> $nums
 * @param list<string> $maybe_empty
 */
function test(array $nums, array $maybe_empty, int $len): void {
    $padded = array_pad($nums, 5, 'x');
    /** @mir-check $padded is non-empty-list<int|"x"> */
    $_ = $padded;

    $padded_negative = array_pad($nums, -3, true);
    /** @mir-check $padded_negative is non-empty-list<int|true> */
    $_ = $padded_negative;

    $maybe = array_pad($maybe_empty, $len, 0);
    /** @mir-check $maybe is list<string|0> */
    $_ = $maybe;
}
===expect===
