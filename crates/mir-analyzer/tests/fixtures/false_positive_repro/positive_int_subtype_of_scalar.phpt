===description===
FALSE POSITIVE reproducer. Valid PHP: `positive-int` is a subtype of `scalar`.
mir 0.42.0 currently emits (the bug): InvalidArgument@8:12-8:20: expected scalar, actual positive-int
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
/** @param scalar $value */
function equalTo($value): void {}
/** @return positive-int */
function count_(): int { return 1; }
function run(): void {
    // FP expected: InvalidArgument positive-int vs scalar
    equalTo(count_());
}
===expect===
