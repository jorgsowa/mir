===description===
A loop body that is only break consumes the iterable without using the value
===file===
<?php
/** @return Generator<int, int> */
function generate(): Generator { yield 1; yield 2; }

foreach (generate() as $first) { break; }

foreach (generate() as $second) {
    break;
}
