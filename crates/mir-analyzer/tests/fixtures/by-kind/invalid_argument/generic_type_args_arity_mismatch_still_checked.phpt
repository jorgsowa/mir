===description===
A type-args arity mismatch (`TypedMap<int>` against an expected
`TypedMap<string, int>`) no longer short-circuits as compatible — the
overlapping type-arg positions are still compared, so a real mismatch there
is caught. Both partial-arity `@var` docblocks are now also flagged
directly (InvalidDocblock) regardless of whether the supplied positions
happen to match.
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
//  ^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: TypedMap expects 2 template argument(s), got 1
    needsStringIntMap($m);
//                    ^^ InvalidArgument: Argument $m of needsStringIntMap() expects 'TypedMap<string, int>', got 'TypedMap<int>'
}

function test_matching_partial_type_arg_now_flagged_for_arity(): void {
    /** @var TypedMap<string> $m */
    $m = new TypedMap();
//  ^^^^^^^^^^^^^^^^^^^^ InvalidDocblock: Invalid docblock: TypedMap expects 2 template argument(s), got 1
    needsStringIntMap($m);
}
===expect===
