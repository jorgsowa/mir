===source===
<?php
// Bug: elseif branches were discarded from the post-if merge — even with the bug
// fixed, a variable assigned only in if/elseif but not else must still be
// possibly-undefined after the chain.
function foo(int $x): string {
    if ($x > 0) {
        $result = 'positive';
    } elseif ($x < 0) {
        $result = 'negative';
    }
    return $result;
}
===expect===
PossiblyUndefinedVariable: $result
