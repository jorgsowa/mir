===description===
Negative control for the L30 fix: without an `is_callable()` guard, passing
a bare `object` where `callable` is expected must still be flagged —
narrowing must not leak into the un-guarded case.
===config===
suppress=UnusedParam
===file===
<?php
/** @param callable $fn */
function needsCallable(callable $fn): void {}

/** @param object $x */
function test(object $x): void {
    needsCallable($x);
}
===expect===
InvalidArgument@7:18-7:20: Argument $fn of needsCallable() expects 'callable', got 'object'
