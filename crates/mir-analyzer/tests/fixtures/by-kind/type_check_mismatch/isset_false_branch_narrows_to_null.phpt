===description===
`isset($x)` false branch narrows a plain, always-assigned variable to
null — the only other reason isset() can be false. Mirrors the existing
property/static-property counterparts, which already handle both
branches. A non-nullable always-assigned parameter makes the false
branch provably unreachable.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function narrowsParamToNull(?string $x): void {
    if (!isset($x)) {
        /** @mir-check $x is null */
        $_ = $x;
    } else {
        /** @mir-check $x is string */
        $_ = $x;
    }
}

function nonNullableParamUnreachable(string $x): void {
    if (!isset($x)) {
        /** @mir-check $_ is never */
        $_ = 1;
    }
}
===expect===
RedundantCondition@13:8-13:18: Condition is always true/false for type 'bool'
