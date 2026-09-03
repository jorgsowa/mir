===description===
FALSE POSITIVE reproducer. Valid PHP: `positive-int` is a subtype of `int<0,max>`.
mir 0.42.0 currently emits (the bug): InvalidArgument@8:13-8:19: expected int<0,max>, actual positive-int
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
php_version=8.4
===file===
<?php
/** @param int<0,max> $n */
function sleepish(int $n): void { echo $n; }
/** @return positive-int */
function size(): int { return 3; }
function run(): void {
    // FP expected: InvalidArgument positive-int vs int<0,max> (subtype rejected)
    sleepish(size());
}
===expect===
