===description===
Negative control: without the `is_string()` guard, the same shape key's
`string|int` union still correctly flags a `string`-only call — proving
the previous fixture's clean result comes from real narrowing, not from
the check being skipped/loosened for array offsets in general.
===file===
<?php
/** @param array{v: string|int} $arr */
function test(array $arr): string {
    return strtoupper($arr['v']);
}
===expect===
PossiblyInvalidArgument@4:22-4:31: Argument $string of strtoupper() expects 'string', possibly different type 'string|int' provided
