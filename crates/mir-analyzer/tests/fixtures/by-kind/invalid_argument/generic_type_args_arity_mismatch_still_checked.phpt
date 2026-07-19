===description===
A type-args arity mismatch (`TypedMap<int>` against an expected
`TypedMap<string, int>`) no longer short-circuits as compatible — the
overlapping type-arg positions are still compared, so a real mismatch there
is caught, while a genuine positional match stays silent.
===config===
suppress=UnusedParam,UnusedVariable
===file===
<?php
/**
 * @template K
 * @template V
 */
class TypedMap {}

/** @param TypedMap<string, int> $m */
function needsStringIntMap($m): void {}

function test_mismatched_first_type_arg_is_flagged(): void {
    /** @var TypedMap<int> $m */
    $m = new TypedMap();
    needsStringIntMap($m);
}

function test_matching_partial_type_arg_stays_silent(): void {
    /** @var TypedMap<string> $m */
    $m = new TypedMap();
    needsStringIntMap($m);
}
===expect===
InvalidArgument@14:22-14:24: Argument $m of needsStringIntMap() expects 'TypedMap<string, int>', got 'TypedMap<int>'
