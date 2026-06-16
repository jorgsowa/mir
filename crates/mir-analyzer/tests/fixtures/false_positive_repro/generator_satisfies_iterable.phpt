===description===
FALSE POSITIVE reproducer. Valid PHP: A `Generator` is `Traversable`, which satisfies an `iterable` parameter.
mir 0.42.0 currently emits (the bug): InvalidArgument@6:17-6:22 (expected array<mixed,int>, actual Generator) + cascade InvalidReturnType@6:4-6:24
Expected: no issue. Remove ===ignore=== to activate once fixed.
===ignore===
===config===
php_version=8.4
===file===
<?php
/** @param iterable<int> $items */
function total(iterable $items): int { return iterator_count($items); }
function gen(): Generator { yield 1; }
function run(): int {
    return total(gen());   // Generator into iterable
}
===expect===
