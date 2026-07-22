===description===
A dynamic (non-literal) key can't grow a shape in place, so the write falls
straight to the generic array accumulator on the very first write. Without
consulting the parameter's own declared type, the accumulator would narrow
the value type down to just `int` (the one literal assigned so far) even
though the declared type promises `int|string` — the declared type acts as a
floor so the generalized type never ends up narrower than the function's own
contract.
===config===
suppress=UnusedVariable,UnusedParam
===file===
<?php
/** @param array<string, int|string> $counts */
function test(array $counts, string $key): void {
    if ($counts === []) {
        $counts[$key] = 5;
        /** @mir-check $counts is array<string, int|string> */
        $_ = $counts;
    }
}
===expect===
