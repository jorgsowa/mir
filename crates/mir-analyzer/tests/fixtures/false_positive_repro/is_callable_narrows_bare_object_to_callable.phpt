===description===
FP-L30: `is_callable($x)` narrowing kept a bare `object`-typed value typed
as plain `object` — `narrow_to_callable` only filtered atoms out, never
transformed one into something an actual `callable`-typed target accepts.
Passing the narrowed value to a `callable`-typed param then flagged
InvalidArgument ('callable' vs 'object'), even though the runtime check
just proved it callable.
===config===
suppress=UnusedParam
===file===
<?php
/** @param callable $fn */
function needsCallable(callable $fn): void {}

/** @param object $x */
function test(object $x): void {
    if (is_callable($x)) {
        needsCallable($x);
        $x();
    }
}
===expect===
