===description===
FALSE POSITIVE reproducer. Valid PHP: `int<1,255>` is a subtype of `int<0,255>`.
mir 0.42.0 currently emits (the bug): InvalidArgument@8:12-8:20: expected int<0,255>, actual int<1,255>
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
/** @param int<0,255> $c */
function channel(int $c): void {}
/** @return int<1,255> */
function bright(): int { return 200; }
function run(): void {
    // FP expected: InvalidArgument int<1,255> vs int<0,255>
    channel(bright());
}
===expect===
