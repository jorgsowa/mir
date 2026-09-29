===description===
A `Generator` is `Traversable`, which satisfies an `iterable` parameter.
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
