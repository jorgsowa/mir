===description===
`is_closed_precise` only treated a union as "closed" (exhaustively
excludable) when it had a range atom or 2+ members — a single literal
atom left after a prior `!==` exclusion shrinks a 2-member docblock union
down to one was never recognized as closed, so a further exclusion that
empties it entirely wasn't flagged unreachable. Gated on `from_docblock`
so loop-widening's own single-literal under-approximation (unrelated,
never docblock-sourced) isn't affected.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param 'a'|'b' $s */
function stringSequentialExclusionUnreachable($s): void {
    if ($s !== 'a') {
        if ($s !== 'b') {
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}

/** @param 1|2 $n */
function intSequentialExclusionUnreachable($n): void {
    if ($n !== 1) {
        if ($n !== 2) {
            /** @mir-check $_ is never */
            $_ = 1;
        }
    }
}

/** @param 'a'|'b'|'c' $s */
function threeMemberUnionStillReachableAfterOneExclusion($s): void {
    if ($s !== 'a') {
        if ($s !== 'b') {
            /** @mir-check $s is 'c' */
            $_ = 1;
        }
    }
}
===expect===
RedundantCondition@5:12-5:22: Condition is always true/false for type 'bool'
RedundantCondition@15:12-15:20: Condition is always true/false for type 'bool'
