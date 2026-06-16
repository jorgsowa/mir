===description===
FALSE POSITIVE reproducer. Valid PHP: A `do { } while` body always executes at least once, so `$id` is always defined.
mir 0.42.0 currently emits (the bug): PossiblyUndefinedVariable@7:11-7:14 ($id) + cascade InvalidReturnType@7:4-7:15
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
function run(): int {
    do {
        $id = random_int(1, 10);
    } while ($id > 5);
    // FP expected: PossiblyUndefinedVariable $id (do-body is unconditional)
    return $id;
}
===expect===
