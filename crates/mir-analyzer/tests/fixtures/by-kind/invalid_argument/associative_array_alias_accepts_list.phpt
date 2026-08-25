===description===
`associative-array` follows PHPStan/Psalm semantics here: it is treated as a
semantic alias of `array`, so a plain list satisfies the same declaration.
===config===
suppress=UnusedParam
===file===
<?php
/**
 * @param associative-array<array-key, int> $items
 */
function takesAssociativeArray(array $items): void {}

takesAssociativeArray([1, 2, 3]);
===expect===
