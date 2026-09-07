===description===
FALSE POSITIVE reproducer. Valid PHP: `preg_match($re,$s,$m)` defines `$m` by reference when it returns 1.
mir 0.42.0 currently emits (the bug): PossiblyUndefinedVariable@5:13-5:15: $m
Expected: no issue. Remove ===ignore=== to activate once fixed.
===config===
suppress=MixedArgument
php_version=8.4
===file===
<?php
function run(string $s): void {
    // FP expected: PossiblyUndefinedVariable $m (preg_match by-ref out-param + && short-circuit)
    if (str_contains($s, 'x') && preg_match('/(\d+)/', $s, $m) === 1) {
        echo $m[1];
    }
}
===expect===
