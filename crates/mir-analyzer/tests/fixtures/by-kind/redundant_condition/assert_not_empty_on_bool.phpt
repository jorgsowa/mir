===description===
Assert not empty on bool — `!empty` is a multi-atom falsy union, so
`negate_assertion_type` now subtracts the `false` atom from `bool`
instead of no-oping on a non-single-atom target, correctly narrowing
`$bar` to `true` and making the following condition redundant.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param mixed $value
 * @assert !empty $value
 */
function assertNotEmpty($value) : void {}

function foo(bool $bar) : void {
    assertNotEmpty($bar);
    if ($bar) {}
}
===expect===
RedundantCondition@10:8-10:12: Condition is always true/false for type 'true'
