===description===
@psalm-param-out (Psalm's tag) is an alias for @param-out and must be
recognized and applied identically.
===config===
suppress=UnusedVariable,UnusedFunction
===file===
<?php
/**
 * @psalm-param-out int $count
 */
function countItems(array $items, mixed &$count): void {
    $count = count($items);
}

$n = null;
countItems([1, 2, 3], $n);
/** @mir-check $n is int */
$_ = $n;
===expect===
