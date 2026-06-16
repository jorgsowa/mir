===description===
FALSE POSITIVE reproducer. Valid PHP: `${"$key"} = ...` defines `$a`/`$b` via variable-variables.
mir 0.42.0 currently emits (the bug): UndefinedVariable@7:9-7:11: $a
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
function run(array $opts): void {
    foreach (['a', 'b'] as $key) {
        ${"$key"} = $opts[$key] ?? null;
    }
    // FP expected: UndefinedVariable $a (variable-variables not tracked)
    echo $a;
}
===expect===
