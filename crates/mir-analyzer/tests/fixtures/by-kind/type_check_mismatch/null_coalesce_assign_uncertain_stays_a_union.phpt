===description===
Regression guard: `??=` must NOT claim certainty it doesn't have. A
non-optional property whose type still admits `null` (set in one branch,
left null in another) could genuinely go either way at runtime, so the
result stays the safe union of "kept the old value" and "ran the
right-hand side" — same for a property that's optional (present in only
one merged branch).
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function nullableNonOptional(): void {
    $arr = ['a' => null];
    if (rand(0, 1)) {
        $arr['a'] = 5;
    }
    $arr['a'] ??= 99;
    /** @mir-check $arr is array{'a': 5|99} */
    $_ = $arr;
}

function optionalKey(): void {
    $arr = [];
    if (rand(0, 1)) {
        $arr['a'] = 5;
    }
    $arr['a'] ??= 99;
    /** @mir-check $arr is array<string, 5|99> */
    $_ = $arr;
}
===expect===
