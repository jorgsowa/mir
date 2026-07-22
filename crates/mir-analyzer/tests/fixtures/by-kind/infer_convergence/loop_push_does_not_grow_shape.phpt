===description===
Regression guard: a `[]` push inside a loop body must keep generalizing to
`list<int>` on every pass, exactly like before shape-preserving writes were
introduced for straight-line code — growing a property per iteration would
never let the fixed-point loop analysis converge.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
function test(): void {
    $result = [];
    for ($i = 0; $i < 10; $i++) {
        $result[] = $i;
    }
    /** @mir-check $result is list<int> */
    $_ = $result;
}
===expect===
