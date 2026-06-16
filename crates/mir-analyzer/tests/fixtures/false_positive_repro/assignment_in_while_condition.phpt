===description===
FALSE POSITIVE reproducer. Valid PHP: The `&&`-guarded assignment in the `while` condition defines `$line` in the body.
mir 0.42.0 currently emits (the bug): PossiblyUndefinedVariable@5:20-5:25: $line
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
function run($resource): void {
    while (!feof($resource) && ($line = fgets($resource))) {
        // expect: PossiblyUndefinedVariable $line (&&-guarded assignment in cond)
        echo strlen($line);
    }
}
===expect===
