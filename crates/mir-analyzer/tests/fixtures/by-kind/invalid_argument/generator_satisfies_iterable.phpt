===description===
A `Generator` is `Traversable`, which satisfies an `iterable` parameter.
===config===
<mir>
  <phpVersion>8.4</phpVersion>
</mir>
===file===
<?php
/** @param iterable<int> $items */
function total(iterable $items): int { return iterator_count($items); }
function gen(): Generator { yield 1; }
function run(): int {
    return total(gen());   // Generator into iterable
}
===expect===
